//! `mash dash`: read-only history with optional authenticated service control.
//! It serves the front end built
//! from `dashboard/` (embedded at compile time) and a small JSON API over the
//! same typed core as the CLI and MCP. It binds to loopback by default and
//! answers only requests addressed to the bound host, so other websites cannot
//! reach it through DNS rebinding; it sends no CORS headers.
// The search futures this crate drives nest deeper than the default limit
// when the compiler proves them `Send`, as they do in `mash-app`.
#![recursion_limit = "256"]

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use mash_app::core::{self, Operation, Paths, Request};
use mash_app::{
    BUILD_REVISION, chat_catalog, config, observability, openapi, records, runtime, search_filter,
    search_results, shared, storage, views, watch, write_json,
};
use mash_mcp as mcp;
use mash_model::{AppError, ErrorBody, ErrorKind, progress, provider, redaction};
use std::net::SocketAddr;
use std::sync::Arc;

mod access;
mod assets;
mod assistant;
mod contract;
mod live;
mod search;
mod services;
mod setup;

pub use assets::has_web_dashboard;

struct Dash {
    paths: Paths,
    hosts: Vec<String>,
    /// Whether only this machine can reach the dashboard.
    loopback: bool,
    access: access::Access,
    /// Read only the shared index: this machine has no agent history itself.
    shared_only: bool,
    model_work: tokio::sync::Semaphore,
    services: services::Services,
    console: crate::mcp::console::Console,
    live: live::Live,
    /// Questions the assistant may be answering at once.
    questions: Arc<tokio::sync::Semaphore>,
}

/// How other devices reach this dashboard.
pub struct Reach {
    pub listen: SocketAddr,
    /// Names a reverse proxy or private network gives the dashboard.
    pub hosts: Vec<String>,
    /// Environment variable holding the token every visitor signs in with.
    pub access_token_env: Option<String>,
    /// Serve only the shared index.
    pub shared_only: bool,
}

pub async fn run(
    paths: Paths,
    reach: Reach,
    services_config: Option<std::path::PathBuf>,
    fleet_config: Option<std::path::PathBuf>,
    fleet_node: Option<String>,
) -> Result<(), AppError> {
    let Reach {
        listen,
        hosts: names,
        access_token_env,
        shared_only,
    } = reach;
    let access = access::Access::load(access_token_env.as_deref())?;
    if !names.is_empty() && !access.required() {
        return Err(AppError::configuration(
            "--allow-host makes the dashboard reachable from other devices; set --access-token-env as well",
        ));
    }
    let services = tokio::task::spawn_blocking(move || {
        if let Some(path) = fleet_config {
            services::Services::load_fleet(&path, fleet_node.as_deref())
        } else {
            services::Services::load(services_config.as_deref())
        }
    })
    .await
    .map_err(|_| AppError::internal("load services configuration"))??;
    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(|_| AppError::configuration("could not bind the dashboard address"))?;
    let address = listener
        .local_addr()
        .map_err(|_| AppError::internal("get dashboard address"))?;
    let loopback = address.ip().is_loopback() && names.is_empty();
    let mut hosts = allowed_hosts(address);
    hosts.extend(names);
    let mut started = serde_json::json!({ "dashboard": format!("http://{address}/") });
    if !loopback && !access.required() {
        started["warning"] = "anyone who can reach this address can read your chat history; set --access-token-env to require sign-in".into();
    }
    // Live updates watch the local index and, when one is configured, listen
    // to the shared one. A configuration that cannot be read leaves that out;
    // the history routes report what is wrong with it.
    let database =
        (!shared_only).then(|| paths.db.clone().unwrap_or_else(core::default_database_path));
    let config = paths.config.clone();
    let upstream = tokio::task::spawn_blocking(move || crate::config::load(config.as_deref()))
        .await
        .ok()
        .and_then(Result::ok)
        .and_then(|config| config.shared)
        .map(|config| live::Upstream {
            config,
            own: shared_only,
        });
    let state = Arc::new(Dash {
        console: crate::mcp::console::Console::new(paths.clone()),
        live: live::Live::new(database, upstream),
        questions: Arc::new(tokio::sync::Semaphore::new(assistant::MAX_QUESTIONS)),
        paths,
        hosts,
        loopback,
        access,
        shared_only,
        model_work: tokio::sync::Semaphore::new(1),
        services,
    });
    crate::write_json(&mut std::io::stderr(), &started)?;
    let stopping = state.clone();
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            // Open event streams would otherwise hold the server open.
            stopping.live.close();
        })
        .await
        .map_err(|_| AppError::internal("dashboard server failed"))
}

/// Every API route with its description, and the contract document they add
/// up to (see `contract`).
fn api() -> (Router<Arc<Dash>>, aide::openapi::OpenApi) {
    let mut document = aide::openapi::OpenApi {
        info: aide::openapi::Info {
            title: "mash dash".into(),
            description: Some(
                "The local dashboard's API. Generated from the server's routes; do not edit."
                    .into(),
            ),
            version: "1".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let router = history_routes(setup::routes(assistant::routes(live::routes(
        services::routes(aide::axum::ApiRouter::new()),
    ))))
    .finish_api(&mut document);
    (router, document)
}

/// The routes that read history, status and settings.
fn history_routes(router: aide::axum::ApiRouter<Arc<Dash>>) -> aide::axum::ApiRouter<Arc<Dash>> {
    use aide::axum::routing::get_with;
    use contract::{changes, reads};
    router
        .api_route(
            "/api/health",
            get_with(
                health,
                reads::<Health>("/api/health", "The build this dashboard is running."),
            ),
        )
        .api_route(
            "/api/services/settings",
            get_with(
                settings,
                reads::<Settings>(
                    "/api/services/settings",
                    "The settings in this machine's configuration file.",
                ),
            )
            .post_with(
                change_settings,
                changes::<Settings>("/api/services/settings", "Rewrite the configuration file."),
            ),
        )
        .api_route(
            "/api/status",
            get_with(
                status,
                reads::<core::StatusResponse>(
                    "/api/status",
                    "Local readiness and the resolved configuration.",
                ),
            ),
        )
        .api_route(
            "/api/chats",
            get_with(
                list_chats,
                reads::<core::DashboardCatalog>(
                    "/api/chats",
                    "Every chat's metadata, newest first.",
                ),
            ),
        )
        .api_route(
            "/api/search",
            get_with(
                search::search,
                reads::<core::SearchResponse>("/api/search", "Chats that match a question."),
            ),
        )
        .api_route(
            "/api/view",
            get_with(
                view,
                reads::<core::ViewResponse>("/api/view", "The messages around one message."),
            ),
        )
        .api_route(
            "/api/chat",
            get_with(
                chat,
                reads::<core::ViewResponse>("/api/chat", "One page of a chat's messages."),
            ),
        )
        .api_route(
            "/api/records",
            get_with(
                list_records,
                reads::<crate::records::Page>("/api/records", "Saved memories, newest first."),
            ),
        )
        .api_route(
            "/api/record",
            get_with(
                record,
                reads::<crate::records::Record>("/api/record", "One memory with its history."),
            ),
        )
        .api_route(
            "/api/console/tools",
            get_with(
                console_tools,
                reads::<crate::mcp::console::Catalogue>(
                    "/api/console/tools",
                    "The tools the MCP server offers, as an agent sees them.",
                ),
            ),
        )
        .api_route(
            "/api/console/call",
            get_with(
                console_call,
                reads::<crate::mcp::console::Answer>(
                    "/api/console/call",
                    "Run one read-only tool.",
                ),
            ),
        )
}

/// The contract every route above keeps, as an `OpenAPI` document.
async fn openapi() -> Response {
    static DOCUMENT: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    axum::Json(DOCUMENT.get_or_init(|| contract::document(&api().1))).into_response()
}

fn router(state: Arc<Dash>) -> Router {
    api()
        .0
        .route("/api/openapi.json", axum::routing::get(openapi))
        .route("/api/session", axum::routing::get(session).post(sign_in))
        .route("/api/session/end", axum::routing::post(sign_out))
        .route("/api/session/code", axum::routing::post(sign_in_code))
        .fallback(axum::routing::get(asset))
        .layer(axum::middleware::from_fn_with_state(state.clone(), guard))
        // The chat catalogue is over a megabyte of JSON for a few thousand
        // chats; a phone on a slow link should not have to carry all of it.
        .layer(
            tower_http::compression::CompressionLayer::new()
                .quality(tower_http::CompressionLevel::Fastest),
        )
        .layer(axum::middleware::from_fn(trace))
        .with_state(state)
}

/// The names a browser uses for the bound address; anything else is refused.
fn allowed_hosts(address: SocketAddr) -> Vec<String> {
    let port = address.port();
    let mut hosts = vec![address.to_string()];
    if address.ip().is_loopback() {
        hosts.extend([
            format!("localhost:{port}"),
            format!("127.0.0.1:{port}"),
            format!("[::1]:{port}"),
        ]);
    }
    hosts
}

/// One span per request when tracing is on (`MASH_TRACE=1`). It opens when
/// the request arrives and closes when the response is ready or the browser
/// goes away, so a span that closes without a status is a request the client
/// abandoned. Only the path is recorded: queries can hold search text.
#[tracing::instrument(name = "http_request", skip_all, fields(request_id = crate::observability::request_id(), process_id = std::process::id(), method = %request.method(), path = request.uri().path(), status = tracing::field::Empty, elapsed_ms = tracing::field::Empty))]
async fn trace(request: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    let span = tracing::Span::current();
    span.record("status", response.status().as_u16());
    span.record("elapsed_ms", crate::progress::milliseconds(started));
    response
}

/// How long a browser may keep a response.
#[derive(Clone, Copy)]
enum Cache {
    Api,
    Hashed,
    Image,
    Page,
}

async fn guard(
    State(state): State<Arc<Dash>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !state
        .hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(host))
    {
        return (StatusCode::MISDIRECTED_REQUEST, "unexpected Host header").into_response();
    }
    let path = request.uri().path();
    let service_path = path == "/api/services" || path.starts_with("/api/services/");
    let session_path = matches!(path, "/api/session" | "/api/session/end");
    // The page and its assets say nothing about history; everything under
    // /api does, except the build stamp and signing in. Another dashboard
    // presents the services token instead of a session.
    let peer =
        service_path && state.services.has_token() && state.services.authorized(request.headers());
    if path.starts_with("/api/")
        && path != "/api/health"
        && !session_path
        && !peer
        && !state.access.signed_in(request.headers())
    {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(AppError::configuration(
                "Sign in with the dashboard access token",
            )),
        )
            .into_response();
    }
    if service_path
        && request.uri().path() != "/api/services/access"
        && !state.services.authorized(request.headers())
    {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(AppError::configuration(
                "Unlock services with the dashboard access token",
            )),
        )
            .into_response();
    }
    let service_action = (session_path
        || path == "/api/session/code"
        || matches!(
            request.uri().path(),
            "/api/services/action"
                | "/api/services/fleet/jobs"
                | "/api/services/settings"
                | "/api/services/setup"
                | "/api/assistant"
        ))
        && request.method() == axum::http::Method::POST;
    if service_action && let Some(origin) = request.headers().get(header::ORIGIN) {
        let valid = origin
            .to_str()
            .ok()
            .and_then(|value| reqwest::Url::parse(value).ok())
            .is_some_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.authority().eq_ignore_ascii_case(host)
            });
        if !valid {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    if !service_action
        && request.method() != axum::http::Method::GET
        && request.method() != axum::http::Method::HEAD
    {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let cache = Cache::of(request.uri().path());
    harden(next.run(request).await, cache)
}

impl Cache {
    fn of(path: &str) -> Self {
        if path.starts_with("/api/") {
            Self::Api
        } else if path.starts_with("/assets/") {
            Self::Hashed
        } else if std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("svg") || extension.eq_ignore_ascii_case("png")
            })
        {
            Self::Image
        } else {
            Self::Page
        }
    }
}

/// The caching and browser-safety headers every response carries.
fn harden(mut response: Response, cache: Cache) -> Response {
    let headers = response.headers_mut();
    // Built assets carry a content hash in their name, so a browser may keep
    // them for good. Logos change rarely. The page itself is always rechecked.
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(match cache {
            Cache::Api => "no-store",
            Cache::Hashed => "public, max-age=31536000, immutable",
            Cache::Image => "public, max-age=86400",
            Cache::Page => "no-cache",
        }),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'",
        ),
    );
    response
}

/// Whether this dashboard asks visitors to sign in, and whether this one has.
async fn session(State(state): State<Arc<Dash>>, headers: axum::http::HeaderMap) -> Response {
    axum::Json(serde_json::json!({
        "required": state.access.required(),
        "signed_in": state.access.signed_in(&headers),
    }))
    .into_response()
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SignIn {
    token: String,
}

/// A reverse proxy that ends TLS says so; the cookie is then kept to HTTPS.
fn forwarded_https(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("https"))
}

async fn sign_in(
    State(state): State<Arc<Dash>>,
    headers: axum::http::HeaderMap,
    input: Result<axum::Json<SignIn>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Ok(axum::Json(input)) = input else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match state
        .access
        .sign_in(&input.token, forwarded_https(&headers))
    {
        Ok(cookie) => (
            [(header::SET_COOKIE, cookie)],
            axum::Json(serde_json::json!({ "required": true, "signed_in": true })),
        )
            .into_response(),
        Err(access::Refusal::Wrong) => (
            StatusCode::UNAUTHORIZED,
            axum::Json(AppError::configuration("That access token is not right")),
        )
            .into_response(),
        Err(access::Refusal::TooMany) => (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(AppError::configuration(
                "Too many wrong tokens; wait a minute and try again",
            )),
        )
            .into_response(),
    }
}

/// A short code for signing one more browser in; the guard has already
/// required the caller to be signed in.
async fn sign_in_code(State(state): State<Arc<Dash>>) -> Response {
    if !state.access.required() {
        return (
            StatusCode::CONFLICT,
            axum::Json(AppError::configuration(
                "this dashboard has no access token, so there is nothing to sign in to",
            )),
        )
            .into_response();
    }
    match state.access.issue_code() {
        Ok(code) => axum::Json(serde_json::json!({ "code": code, "minutes": 10 })).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(error)).into_response(),
    }
}

async fn sign_out(State(state): State<Arc<Dash>>, headers: axum::http::HeaderMap) -> Response {
    (
        [(
            header::SET_COOKIE,
            access::Access::sign_out(forwarded_https(&headers)),
        )],
        axum::Json(serde_json::json!({
            "required": state.access.required(),
            "signed_in": !state.access.required(),
        })),
    )
        .into_response()
}

#[derive(serde::Serialize, schemars::JsonSchema)]
struct Health {
    build: &'static str,
}

async fn health() -> Response {
    axum::Json(Health {
        build: crate::BUILD_REVISION,
    })
    .into_response()
}

/// Local readiness and the resolved configuration, as `mash status` reports
/// them. Read-only: it names credential variables but never their values.
async fn status(State(state): State<Arc<Dash>>) -> Response {
    let request = Request {
        paths: state.paths.clone(),
        operation: Operation::Status {
            shared: false,
            coverage: None,
        },
    };
    json_result(core::execute(request).await)
}

/// The configuration file this dashboard's Mash reads.
fn configuration_path(paths: &Paths) -> Result<std::path::PathBuf, AppError> {
    match &paths.config {
        Some(path) => Ok(path.clone()),
        None => crate::config::default_path(),
    }
}

async fn settings(State(state): State<Arc<Dash>>) -> Response {
    let paths = state.paths.clone();
    let view = tokio::task::spawn_blocking(move || {
        crate::config::settings::view(&configuration_path(&paths)?)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("read settings")));
    settings_result(view, state.loopback || state.services.managed())
}

/// Rewrites the configuration file. A dashboard other machines can reach
/// accepts this only when it was started with a services configuration, the
/// same opt-in that lets it manage services; like every service route it is
/// then subject to that configuration's access token.
async fn change_settings(
    State(state): State<Arc<Dash>>,
    input: Result<
        axum::Json<crate::config::settings::Change>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Response {
    if !state.loopback && !state.services.managed() {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(AppError::configuration(
                "settings can be changed on the machine itself, or from a dashboard started with a services configuration",
            )),
        )
            .into_response();
    }
    let Ok(axum::Json(change)) = input else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(AppError::configuration("invalid settings change")),
        )
            .into_response();
    };
    let paths = state.paths.clone();
    let view = tokio::task::spawn_blocking(move || {
        crate::config::settings::apply(&configuration_path(&paths)?, &change)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("write settings")));
    settings_result(view, true)
}

/// The settings, with whether this dashboard accepts changes to them.
#[derive(serde::Serialize, schemars::JsonSchema)]
struct Settings {
    #[serde(flatten)]
    view: crate::config::settings::View,
    editable: bool,
}

fn settings_result(
    view: Result<crate::config::settings::View, AppError>,
    editable: bool,
) -> Response {
    match view {
        Ok(view) => axum::Json(Settings { view, editable }).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.error })),
        )
            .into_response(),
    }
}

async fn list_chats(State(state): State<Arc<Dash>>) -> Response {
    let request = Request {
        paths: state.paths.clone(),
        operation: Operation::DashboardCatalog {
            shared_only: state.shared_only,
        },
    };
    json_result(core::execute(request).await)
}

#[derive(serde::Deserialize, Default, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct RecordParams {
    q: Option<String>,
    kind: Option<crate::records::Kind>,
    state: Option<crate::records::Lifecycle>,
    review: Option<crate::records::Review>,
    chat: Option<String>,
    shared: bool,
    node: Option<String>,
    offset: usize,
    cursor_node: Option<String>,
    cursor_offset: usize,
}

async fn list_records(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<RecordParams>,
) -> Response {
    let operation = Operation::Records(crate::records::List {
        query: params.q.filter(|query| !query.trim().is_empty()),
        kind: params.kind,
        state: params.state,
        review: params.review,
        chat: params.chat,
        shared: params.shared || state.shared_only,
        node_id: params.node,
        cursor: params.cursor_node.map(|node_id| crate::records::Cursor {
            node_id,
            offset: params.cursor_offset,
        }),
        offset: params.offset,
        include_inactive: true,
        limit: Some(20),
        ..Default::default()
    });
    json_result(
        core::execute(Request {
            paths: state.paths.clone(),
            operation,
        })
        .await,
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RecordDetailParams {
    id: String,
    node: Option<String>,
    before_revision: Option<i64>,
    #[serde(default)]
    link_offset: usize,
}

async fn record(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<RecordDetailParams>,
) -> Response {
    let operation = Operation::Record(crate::records::Get {
        id: params.id,
        node_id: params.node,
        history: true,
        before_revision: params.before_revision,
        link_offset: params.link_offset,
        ..Default::default()
    });
    json_result(
        core::execute(Request {
            paths: state.paths.clone(),
            operation,
        })
        .await,
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ViewParams {
    id: String,
    #[serde(default)]
    context: Option<u32>,
    node: Option<String>,
    #[serde(default)]
    shared: bool,
}

/// Messages on each side of the anchor; chunked chat reads are tracked in mash-twg6.
const MAX_VIEW_CONTEXT: u32 = 500;

async fn view(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<ViewParams>,
) -> Response {
    let operation = if params.shared {
        if params.node.as_ref().is_none_or(String::is_empty) {
            return json_result(Err(AppError::usage(
                "shared chat reads require a source node",
            )));
        }
        Operation::View {
            id: params.id,
            context: params.context.unwrap_or(50).min(MAX_VIEW_CONTEXT),
            shared: params.node,
        }
    } else {
        Operation::ChatContext {
            id: params.id,
            context: params.context.unwrap_or(50).min(MAX_VIEW_CONTEXT),
            node: params.node,
        }
    };
    json_result(
        core::execute(Request {
            paths: state.paths.clone(),
            operation,
        })
        .await,
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ChatParams {
    id: String,
    #[serde(default)]
    offset: u32,
    #[serde(default)]
    limit: Option<u32>,
    node: Option<String>,
    #[serde(default)]
    shared: bool,
}

/// Messages per page of a chat; whole small chats fit in one.
const MAX_CHAT_PAGE: u32 = 500;

async fn chat(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<ChatParams>,
) -> Response {
    let operation = Operation::ChatMessages {
        chat_id: params.id,
        offset: params.offset,
        node: params.node,
        shared: params.shared,
        limit: params
            .limit
            .unwrap_or(MAX_CHAT_PAGE)
            .clamp(1, MAX_CHAT_PAGE),
    };
    json_result(
        core::execute(Request {
            paths: state.paths.clone(),
            operation,
        })
        .await,
    )
}

async fn console_tools() -> Response {
    match crate::mcp::console::Console::catalogue() {
        Ok(catalogue) => axum::Json(catalogue).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(error)).into_response(),
    }
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct ConsoleCall {
    name: String,
    /// The tool's arguments as a JSON object; absent means none.
    arguments: Option<String>,
}

/// Runs one read-only tool. A GET like every other history route: the console
/// refuses tools that change data, so nothing here writes.
async fn console_call(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(call): axum::extract::Query<ConsoleCall>,
) -> Response {
    let arguments = match call.arguments.as_deref().filter(|text| !text.is_empty()) {
        None => serde_json::Map::new(),
        Some(text) => match serde_json::from_str(text) {
            Ok(serde_json::Value::Object(arguments)) => arguments,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(AppError::usage("arguments must be a JSON object")),
                )
                    .into_response();
            }
        },
    };
    match state.console.call(&call.name, arguments).await {
        Ok(reply) => axum::Json(reply).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(error)).into_response(),
    }
}

fn json_result(result: Result<core::Response, AppError>) -> Response {
    match result {
        Ok(response) => axum::Json(response).into_response(),
        Err(error) => {
            let status = if error.error.retryable {
                StatusCode::SERVICE_UNAVAILABLE
            } else if error.error.kind == crate::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (
                status,
                axum::Json(serde_json::json!({ "error": error.error })),
            )
                .into_response()
        }
    }
}

async fn asset(uri: Uri) -> Response {
    let path = match uri.path() {
        "/" => "/index.html",
        other => other,
    };
    let Some((_, body)) = assets::ASSETS.iter().find(|(route, _)| *route == path) else {
        if !has_web_dashboard() {
            return (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                "<!doctype html><title>Mash</title><h1>The dashboard has not been built into this copy of Mash</h1><p>Use <a href=\"https://getmash.io/search/#your-first-search\">terminal search</a> or <a href=\"https://getmash.io/install-methods/#include-the-web-dashboard\">build with the dashboard</a>.</p>",
            )
                .into_response();
        }
        return StatusCode::NOT_FOUND.into_response();
    };
    ([(header::CONTENT_TYPE, content_type(path))], *body).into_response()
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests;
