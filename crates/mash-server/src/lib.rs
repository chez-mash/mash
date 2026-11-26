// The search routes hold the library's typed async search futures.
#![recursion_limit = "256"]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Extension, Request, State, rejection::JsonRejection};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
#[cfg(test)]
use axum::{Router, middleware};

mod admin;
mod admission;
mod attachments;
mod chats;
mod contexts;
mod events;
mod group_status;
mod join;
mod machines;
mod presence;
mod roster;
mod routes;
#[cfg(test)]
use routes::document;
use routes::router;
pub mod setup;
mod sync;

use mash_app::shared::server::{Identity, configuration, store};
use mash_app::shared::{self, protocol};
use mash_app::{
    blocking, chat_catalog, config, observability, records, remote, silver_query, storage,
    write_json,
};
#[cfg(test)]
use mash_app::{core, openapi, search_filter, search_results, semantic, views};
use mash_model::{AppError, attachments as files, platform, progress, redaction};
#[cfg(test)]
use mash_model::{message_context, provider, role, tools};
use store::Store;

use mash_model::ErrorKind;
#[cfg(test)]
use mash_model::test_support;
#[cfg(test)]
use mash_silver as silver;

#[derive(Clone)]
struct App {
    activity: shared::activity::Registry,
    mutation_capacity: usize,
    attachments_dir: Option<std::path::PathBuf>,
    catalog: sync::CatalogHandle,
    store: Store,
    /// The tokens accepted now; reread while the server runs.
    nodes: roster::Nodes,
    /// The added-machines file this server follows, when it has one.
    live: Option<Arc<roster::Live>>,
    joins: Arc<tokio::sync::Semaphore>,
    uploads: Arc<tokio::sync::Semaphore>,
    waiters: Arc<tokio::sync::Semaphore>,
    /// When each machine last checked in.
    presence: presence::Presence,
}

#[derive(serde::Serialize)]
struct Listening {
    listening: std::net::SocketAddr,
}

struct ApiError(StatusCode, AppError);
/// A failure answers with the error body, whatever the route.
impl aide::OperationOutput for ApiError {
    type Inner = crate::AppError;
}
impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        let status = match error.error.kind {
            crate::ErrorKind::Usage | crate::ErrorKind::Configuration => StatusCode::BAD_REQUEST,
            crate::ErrorKind::Conflict => StatusCode::CONFLICT,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };
        Self(status, error)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(self.1)).into_response()
    }
}
fn json<T>(input: Result<Json<T>, JsonRejection>) -> Result<T, ApiError> {
    input
        .map(|Json(input)| input)
        .map_err(|error| ApiError(error.status(), AppError::usage("invalid JSON request")))
}
#[tracing::instrument(name = "http_request", skip_all, fields(request_id = crate::observability::request_id(), process_id = std::process::id(), trace_id = tracing::field::Empty, status = tracing::field::Empty, elapsed_ms = tracing::field::Empty))]
async fn authorize(State(app): State<App>, request: Request, next: Next) -> Response {
    let span = tracing::Span::current();
    if !span.is_disabled() {
        use crate::observability::http::{HEADER, TraceId};
        let mut values = request.headers().get_all(HEADER).iter();
        let id = values
            .next()
            .and_then(|value| {
                values
                    .next()
                    .is_none()
                    .then(|| TraceId::parse(value.as_bytes()))
                    .flatten()
            })
            .or_else(TraceId::generate);
        if let Some(id) = id {
            span.record("trace_id", id.as_str());
        }
    }
    let started = std::time::Instant::now();
    let response = authorize_inner(app, request, next).await;
    tracing::Span::current().record("status", response.status().as_u16());
    tracing::Span::current().record("elapsed_ms", crate::progress::milliseconds(started));
    response
}

async fn authorize_inner(app: App, mut request: Request, next: Next) -> Response {
    // A join string's secret is the credential of the two join routes.
    if matches!(request.uri().path(), "/v1/join" | "/v1/join/check") {
        return tokio::time::timeout(Duration::from_secs(30), next.run(request))
            .await
            .unwrap_or_else(|_| {
                ApiError(
                    StatusCode::REQUEST_TIMEOUT,
                    AppError::shared("request timed out"),
                )
                .into_response()
            });
    }
    // Management takes an admin key, never a machine's token.
    if request.uri().path().starts_with("/v1/admin/") {
        return admin::gate(app, request, next).await;
    }
    let token = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    let roster = app.nodes.current();
    let hash = token.map(|t| blake3::hash(t.as_bytes()));
    let identity = hash.and_then(|hash| roster.tokens.get(&hash));
    let Some(identity) = identity else {
        // A credential the operator removed or replaced is told so, so the
        // machine can say why rather than report a broken token.
        if hash.is_some_and(|hash| roster.revoked.contains(&hash)) {
            let error = AppError::credential_revoked(
                "this server no longer accepts this machine's credential",
            );
            return ApiError(StatusCode::UNAUTHORIZED, error).into_response();
        }
        let mut error = AppError::shared("invalid node authorization");
        error.error.retryable = false;
        return ApiError(StatusCode::UNAUTHORIZED, error).into_response();
    };
    for (header, expected) in [
        ("x-mash-group", &identity.group_id),
        ("x-mash-node", &identity.node_id),
    ] {
        if request
            .headers()
            .get(header)
            .is_some_and(|value| value.to_str().ok() != Some(expected.as_str()))
        {
            let mut error = AppError::shared("token does not match configured node identity");
            error.error.retryable = false;
            return ApiError(StatusCode::FORBIDDEN, error).into_response();
        }
    }
    request.extensions_mut().insert(identity.clone());
    // A WebSocket owns its own durable operation lifetime; the HTTP bulk
    // deadline must not terminate a resumable synchronization connection.
    if request.uri().path() == "/v1/sync" {
        return next.run(request).await;
    }
    let timeout = Duration::from_secs(600);
    tokio::time::timeout(timeout, next.run(request))
        .await
        .unwrap_or_else(|_| {
            ApiError(
                StatusCode::REQUEST_TIMEOUT,
                AppError::shared("request timed out"),
            )
            .into_response()
        })
}
async fn fingerprints(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
) -> Result<Json<shared::FingerprintReport>, ApiError> {
    Ok(Json(
        app.store
            .fingerprint_report(&id.group_id, id.dimensions)
            .await?,
    ))
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SharedQuery {
    /// Member of the caller's group; defaults to the caller's own node.
    #[serde(default)]
    node: Option<String>,
    request: crate::silver_query::Request,
}

/// Typed, read-only query over one node's shared silver. Scope is derived from
/// the token, never the body: `node` may only name a member of that group, and
/// there is no SQL field to accept.
async fn query(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<SharedQuery>, JsonRejection>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let SharedQuery { node, request } = json(input)?;
    let target = match node {
        None => id,
        Some(node) => app
            .nodes
            .current()
            .tokens
            .values()
            .find(|candidate| candidate.group_id == id.group_id && candidate.node_id == node)
            .cloned()
            .ok_or_else(|| {
                ApiError(
                    StatusCode::NOT_FOUND,
                    AppError::usage("node is not in this group"),
                )
            })?,
    };
    Ok(Json(
        app.store
            .query(&target, request, Duration::from_secs(30))
            .await?,
    ))
}

async fn freshness(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
) -> Result<Json<shared::Freshness>, ApiError> {
    let mut freshness = app.store.freshness(&id).await?;
    freshness.activity = Some(app.activity.snapshot(
        &id.group_id,
        app.mutation_capacity,
        app.uploads.available_permits(),
    ));
    freshness.unpublished_nodes = Some(unpublished_nodes(
        &app.nodes.current().tokens,
        &id.group_id,
        &freshness,
    ));
    let group = id.group_id.clone();
    let node = id.node_id.clone();
    // The server's one catalog connection: a second one opened per request
    // would contend with the transfers this reports on.
    freshness.baseline = sync::catalog(
        &app.catalog,
        move |catalog| -> Result<Option<crate::shared::BaselineCoverage>, AppError> {
            Ok(catalog.latest_backfill(&group, &node)?.map(|backfill| {
                crate::shared::BaselineCoverage {
                    id: backfill.backfill_id,
                    state: backfill.state,
                    covered: backfill.covered,
                    total: backfill.total_chats,
                }
            }))
        },
    )
    .await
    .ok()
    .flatten();
    Ok(Json(freshness))
}

fn unpublished_nodes(
    identities: &HashMap<blake3::Hash, Identity>,
    group: &str,
    freshness: &shared::Freshness,
) -> Vec<String> {
    identities
        .values()
        .filter(|id| {
            id.group_id == group
                && !freshness
                    .nodes
                    .iter()
                    .any(|node| node.node_id == id.node_id)
        })
        .map(|id| id.node_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
async fn search(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<protocol::Search>, JsonRejection>,
) -> Result<Json<protocol::Candidates>, ApiError> {
    let mut candidates = app.store.search(&id, json(input)?).await?;
    shared::prepare_candidates(&mut candidates, app.store.responses());
    Ok(Json(candidates))
}

async fn list_records(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<crate::records::List>, JsonRejection>,
) -> Result<Json<crate::records::Page>, ApiError> {
    let mut page = app.store.list_records(&id, json(input)?).await?;
    for record in &mut page.records {
        record.redact(app.store.responses());
    }
    Ok(Json(page))
}

async fn get_record(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<crate::records::Get>, JsonRejection>,
) -> Result<Json<crate::records::Record>, ApiError> {
    let mut record = app.store.get_record(&id, json(input)?).await?;
    record.redact(app.store.responses());
    Ok(Json(record))
}

async fn view(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<protocol::View>, JsonRejection>,
) -> Result<Json<Vec<crate::storage::Message>>, ApiError> {
    let input = json(input)?;
    if input.context > 1000 || input.id.is_empty() || input.id.len() > 4096 {
        return Err(AppError::usage("invalid view request").into());
    }
    let mut messages = app.store.view(&id, input).await?;
    let redaction = app.store.responses();
    for message in &mut messages {
        message.mask_text(|text| redaction.text(text));
        redaction.attachments(&mut message.attachments);
        redaction.tools(&mut message.tools);
    }
    Ok(Json(messages))
}

async fn chat_page(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    input: Result<Json<protocol::ChatPage>, JsonRejection>,
) -> Result<Json<Vec<crate::storage::Message>>, ApiError> {
    let input = json(input)?;
    if input.limit == 0
        || input.limit > 500
        || input.chat_id.is_empty()
        || input.chat_id.len() > 4096
    {
        return Err(AppError::usage("invalid chat page request").into());
    }
    let mut messages = app.store.chat_page(&id, input).await?;
    let redaction = app.store.responses();
    for message in &mut messages {
        message.mask_text(|text| redaction.text(text));
        redaction.attachments(&mut message.attachments);
        redaction.tools(&mut message.tools);
    }
    Ok(Json(messages))
}
pub use machines::{add_admin, add_node, retire, revoke_admin};
pub(crate) use machines::{node_name, retire_node};

mod serve;
pub use serve::{Activation, run};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod contract {
    /// `docs/internal/shared-server-openapi.json` is a reference copy of this
    /// server's API. Rewrite it with `MASH_WRITE_CONTRACT=1` after changing a
    /// route or a type it reads or answers with.
    #[test]
    fn committed_server_contract_matches_the_routes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/internal/shared-server-openapi.json");
        let mut current = serde_json::to_string_pretty(&super::document()).unwrap();
        current.push('\n');
        if std::env::var_os("MASH_WRITE_CONTRACT").is_some() {
            std::fs::write(&path, &current).unwrap();
        }
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            committed == current,
            "docs/internal/shared-server-openapi.json is out of date: rerun this test with MASH_WRITE_CONTRACT=1"
        );
    }
}
