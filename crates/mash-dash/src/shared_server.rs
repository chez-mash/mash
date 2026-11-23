//! The shared server as this dashboard sees it: its state, read with this
//! machine's token; managing its machines with the admin key saved here; and
//! joining a shared index from a join string.
//!
//! Reading the state needs nothing more than the dashboard does. Joining and
//! management change this machine or the server, so they are offered only
//! where settings may be changed: on this machine itself, or on a dashboard
//! that every visitor must sign in to. A dashboard open to its network with no
//! sign-in never uses the admin key saved here. The key and join strings
//! travel only in request bodies, never in an address, and no answer repeats
//! them except a new join string, shown once to the admin who made it.
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_app::core::{self, InitRequest, Operation, Request};
use mash_app::shared::join::{self, CheckAnswer, Join};
use mash_app::shared::manage::{self, Change, Changed};
use mash_app::shared::server::report::{GroupStatus, JoinAnswer, MachineRequest, Overview};
use mash_app::shared::{admin_key, server::enrollment::JoinStatus};
use mash_model::AppError;
use serde::{Deserialize, Serialize};

use super::Dash;

pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::{changes, reads};
    use aide::axum::routing::{get_with, post_with};
    router
        .api_route(
            "/api/shared/server",
            get_with(
                server,
                reads::<ServerView>(
                    "/api/shared/server",
                    "The shared server this machine publishes to, as it reports this machine's group.",
                ),
            ),
        )
        .api_route(
            "/api/shared/admin",
            get_with(
                overview,
                reads::<Overview>(
                    "/api/shared/admin",
                    "Every machine of the shared server, read with the admin key saved here.",
                ),
            ),
        )
        .api_route(
            "/api/shared/admin/key",
            post_with(
                set_key,
                changes::<KeyState>(
                    "/api/shared/admin/key",
                    "Save an admin key on this machine after the server accepts it, or forget it.",
                ),
            ),
        )
        .api_route(
            "/api/shared/admin/change",
            post_with(
                change,
                changes::<ChangeAnswer>(
                    "/api/shared/admin/change",
                    "Add, replace, remove or delete one machine with the admin key saved here.",
                ),
            ),
        )
        .api_route(
            "/api/shared/join",
            post_with(
                join_shared,
                changes::<JoinAnswerView>(
                    "/api/shared/join",
                    "Check a join string without saving anything, or join with it.",
                ),
            ),
        )
}

/// Whether this dashboard may change this machine or the server: it is on
/// this machine, or every visitor signs in to it.
fn may_change(state: &Dash) -> bool {
    state.loopback || state.access.required()
}

fn failed(status: StatusCode, error: &AppError) -> Response {
    (
        status,
        axum::Json(serde_json::json!({ "error": error.error })),
    )
        .into_response()
}

fn refused() -> Response {
    failed(
        StatusCode::FORBIDDEN,
        &AppError::configuration(
            "this can be done on the machine itself, or from a dashboard that every visitor signs in to",
        ),
    )
}

async fn shared_config(state: &Dash) -> Result<Option<mash_app::config::SharedConfig>, AppError> {
    let path = state.paths.config.clone();
    tokio::task::spawn_blocking(move || mash_app::config::load(path.as_deref()))
        .await
        .map_err(|_| AppError::internal("read the configuration"))?
        .map(|config| config.shared)
}

/// The shared server as this machine sees it.
#[derive(Serialize, schemars::JsonSchema)]
struct ServerView {
    /// This machine's shared settings, when it joined one.
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<GroupStatus>,
    /// Why the status could not be read: the server did not answer, or no
    /// longer accepts this machine's credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    problem: Option<mash_model::ErrorBody>,
    /// An admin key is saved on this machine.
    admin_key: bool,
    /// This dashboard may join a server or manage its machines.
    may_change: bool,
}

async fn server(State(state): State<Arc<Dash>>) -> Response {
    let shared = match shared_config(&state).await {
        Ok(shared) => shared,
        Err(error) => return failed(StatusCode::BAD_REQUEST, &error),
    };
    let mut view = ServerView {
        server: None,
        group_id: None,
        node_id: None,
        status: None,
        problem: None,
        admin_key: admin_key::load().is_some(),
        may_change: may_change(&state),
    };
    if let Some(config) = shared {
        view.server = Some(config.api_url.clone());
        view.group_id = Some(config.group_id.clone());
        view.node_id = Some(config.node_id.clone());
        match manage::status(&config).await {
            Ok(status) => view.status = Some(status),
            Err(error) => view.problem = Some(error.error),
        }
    }
    axum::Json(view).into_response()
}

/// The shared settings and the saved admin key, or why management is not
/// possible here.
async fn management(
    state: &Dash,
) -> Result<(mash_app::config::SharedConfig, String), Box<Response>> {
    if !may_change(state) {
        return Err(Box::new(refused()));
    }
    let Some(config) = shared_config(state)
        .await
        .map_err(|error| Box::new(failed(StatusCode::BAD_REQUEST, &error)))?
    else {
        return Err(Box::new(failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("this machine has not joined a shared index"),
        )));
    };
    let key = admin_key::load().ok_or_else(|| {
        Box::new(failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("no admin key is saved on this machine; add one in Settings"),
        ))
    })?;
    Ok((config, key))
}

fn server_error(error: &AppError) -> Response {
    let status = match error.error.kind {
        // Not 401: that would ask the visitor to sign in to this dashboard again.
        mash_model::ErrorKind::NotAdmin => StatusCode::FORBIDDEN,
        mash_model::ErrorKind::Usage | mash_model::ErrorKind::Conflict => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_GATEWAY,
    };
    failed(status, error)
}

async fn overview(State(state): State<Arc<Dash>>) -> Response {
    let (config, key) = match management(&state).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    match manage::overview(&config, &key).await {
        Ok(overview) => axum::Json(overview).into_response(),
        Err(error) => server_error(&error),
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct KeyInput {
    /// The admin key to save; absent to forget the saved one.
    #[serde(default)]
    key: Option<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
struct KeyState {
    admin_key: bool,
}

/// Save a key only once the server accepts it, so a mistyped key is never
/// kept; forgetting changes nothing on the server.
async fn set_key(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<KeyInput>, JsonRejection>,
) -> Response {
    if !may_change(&state) {
        return refused();
    }
    let Ok(axum::Json(input)) = input else {
        return failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("invalid admin key request"),
        );
    };
    let Some(key) = input.key.map(|key| key.trim().to_owned()) else {
        return match admin_key::forget() {
            Ok(()) => axum::Json(KeyState { admin_key: false }).into_response(),
            Err(error) => failed(StatusCode::INTERNAL_SERVER_ERROR, &error),
        };
    };
    if !mash_app::shared::server::admin::well_formed(&key) {
        return failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("that is not a Mash admin key; copy the whole `mash-admin.…` value"),
        );
    }
    let config = match shared_config(&state).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            return failed(
                StatusCode::BAD_REQUEST,
                &AppError::usage("join a shared index before adding its admin key"),
            );
        }
        Err(error) => return failed(StatusCode::BAD_REQUEST, &error),
    };
    if let Err(error) = manage::overview(&config, &key).await {
        return server_error(&error);
    }
    match admin_key::save(&key) {
        Ok(()) => axum::Json(KeyState { admin_key: true }).into_response(),
        Err(error) => failed(StatusCode::INTERNAL_SERVER_ERROR, &error),
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum ChangeKind {
    Add,
    Replace,
    Remove,
    Delete,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ChangeInput {
    change: ChangeKind,
    #[serde(flatten)]
    machine: MachineRequest,
}

#[derive(Serialize, schemars::JsonSchema)]
struct ChangeAnswer {
    /// A new join string, for an addition or a replaced credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    join: Option<JoinAnswer>,
}

async fn change(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<ChangeInput>, JsonRejection>,
) -> Response {
    let (config, key) = match management(&state).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    let Ok(axum::Json(input)) = input else {
        return failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("invalid machine request"),
        );
    };
    let kind = match input.change {
        ChangeKind::Add => Change::Add,
        ChangeKind::Replace => Change::Replace,
        ChangeKind::Remove => Change::Remove,
        ChangeKind::Delete => Change::Delete,
    };
    match manage::change(&config, &key, kind, &input.machine).await {
        Ok(Changed::Join(join)) => axum::Json(ChangeAnswer { join: Some(join) }).into_response(),
        Ok(Changed::Done(_)) => axum::Json(ChangeAnswer { join: None }).into_response(),
        Err(error) => server_error(&error),
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct JoinInput {
    join: String,
    /// Join now; otherwise only check, saving nothing.
    #[serde(default)]
    apply: bool,
}

/// What a join string would do, or did.
#[derive(Serialize, schemars::JsonSchema)]
struct JoinAnswerView {
    server: String,
    group_id: String,
    node_id: String,
    check: CheckAnswer,
    /// This machine joined: its configuration and token are saved.
    joined: bool,
}

async fn join_shared(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<JoinInput>, JsonRejection>,
) -> Response {
    if !may_change(&state) {
        return refused();
    }
    let Ok(axum::Json(input)) = input else {
        return failed(
            StatusCode::BAD_REQUEST,
            &AppError::usage("invalid join request"),
        );
    };
    let parsed = match Join::decode(&input.join) {
        Ok(parsed) => parsed,
        Err(error) => return failed(StatusCode::BAD_REQUEST, &error),
    };
    let check = match join::check(&parsed).await {
        Ok(check) => check,
        Err(error) => return failed(StatusCode::BAD_GATEWAY, &error),
    };
    let mut answer = JoinAnswerView {
        server: parsed.api_url.clone(),
        group_id: parsed.group_id.clone(),
        node_id: parsed.node_id.clone(),
        check,
        joined: false,
    };
    if !input.apply || answer.check.status != JoinStatus::Ready {
        return axum::Json(answer).into_response();
    }
    match apply(&state, parsed).await {
        Ok(()) => {
            answer.joined = true;
            axum::Json(answer).into_response()
        }
        Err(error) => failed(StatusCode::BAD_REQUEST, &error),
    }
}

/// Join as `mash init --join … --no-index --no-agents` does: save the shared
/// settings and the token, and leave indexing, publishing and agents alone.
async fn apply(state: &Dash, parsed: Join) -> Result<(), AppError> {
    let paths = state.paths.clone();
    let for_plan = parsed.clone();
    let plan = tokio::task::spawn_blocking(move || core::join_plan(&paths, &for_plan))
        .await
        .map_err(|_| AppError::internal("plan the join"))??;
    let request = InitRequest {
        force: false,
        agents: false,
        index: false,
        marketplace: core::MARKETPLACE.to_owned(),
        plan: Some(plan),
        join: Some(parsed),
        selected_agents: None,
        backfill_memory: None,
        agent_path: None,
    };
    core::execute(Request {
        paths: state.paths.clone(),
        operation: Operation::Init(request),
    })
    .await
    .map(|_| ())
}
