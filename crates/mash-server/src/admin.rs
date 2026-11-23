//! Managing machines from a dashboard on any machine, with an admin key.
//!
//! Every `/v1/admin/` route passes through [`gate`], which accepts only an
//! admin key (`mash-admin.…`, kept by the server as a hash beside its
//! machines). A machine's token is never one, so a token that can publish and
//! search can never add, remove or delete anything. Wrong keys are counted
//! across the whole server: after [`MAX_FAILURES`] in a minute every admin
//! request is refused until the minute is over, so a key cannot be guessed by
//! trying, and a server under attack fails closed rather than open.
//!
//! Each change goes through the same locked file as `mash-server --add-node`
//! and is applied at once. It is recorded with the key's name and the
//! machine the request says it came from.
use std::sync::Mutex;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use mash_app::shared::server::{admin, configuration, enrollment};
use mash_model::AppError;
use serde::{Deserialize, Serialize};

use super::group_status::{self, MachineStatus};
use super::{ApiError, App, json};

/// Wrong admin keys allowed per minute, across the server.
pub(crate) const MAX_FAILURES: u32 = 10;
const WINDOW_SECONDS: u64 = 60;

/// Failed admin attempts in the current window: its start and its count.
static FAILURES: Mutex<(u64, u32)> = Mutex::new((0, 0));

/// The admin key a request was made with, by name.
#[derive(Clone)]
pub(crate) struct Admin {
    key: String,
    /// The machine the request says it came from (`x-mash-from`).
    from: Option<String>,
}

fn refused(status: StatusCode, message: &str) -> Response {
    let mut error = AppError::new(11, mash_model::ErrorKind::NotAdmin, message, false);
    error.error.retryable = status == StatusCode::TOO_MANY_REQUESTS;
    ApiError(status, error).into_response()
}

/// Whether too many wrong keys arrived in this window; counts one more when
/// `failed` is set.
fn locked(now: u64, failed: bool) -> bool {
    let mut window = FAILURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if now.saturating_sub(window.0) >= WINDOW_SECONDS {
        *window = (now, 0);
    }
    if failed {
        window.1 = window.1.saturating_add(1);
    }
    window.1 >= MAX_FAILURES
}

/// Admit an admin request only with an admin key the server holds now.
pub(crate) async fn gate(app: App, mut request: Request, next: Next) -> Response {
    let now = enrollment::now();
    if locked(now, false) {
        return refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many wrong admin keys; admin requests are refused for a minute",
        );
    }
    let key = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|key| admin::well_formed(key));
    let name = key.and_then(|key| {
        app.nodes
            .current()
            .admins
            .get(&blake3::hash(key.as_bytes()))
            .cloned()
    });
    let Some(name) = name else {
        locked(now, true);
        tracing::warn!("refused an admin request without a valid admin key");
        return refused(
            StatusCode::UNAUTHORIZED,
            "this is not an admin key for this server",
        );
    };
    let from = request
        .headers()
        .get("x-mash-from")
        .and_then(|value| value.to_str().ok())
        .filter(|value| mash_app::config::validate_identifier(value, "machine").is_ok())
        .map(str::to_owned);
    request.extensions_mut().insert(Admin { key: name, from });
    next.run(request).await
}

/// Every machine in every group, the actions recorded, and the admin keys.
#[derive(Serialize, schemars::JsonSchema)]
pub(crate) struct Overview {
    build: String,
    started_at: u64,
    now: u64,
    machines: Vec<MachineStatus>,
    /// Newest first.
    actions: Vec<admin::Action>,
    keys: Vec<KeyName>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub(crate) struct KeyName {
    name: String,
    created_at: u64,
}

pub(crate) async fn overview(
    State(app): State<App>,
    Extension(_admin): Extension<Admin>,
) -> Result<Json<Overview>, ApiError> {
    let (machines, now) = group_status::machines(&app, |_| true).await?;
    let live = app.live.as_ref().ok_or_else(not_live)?;
    let path = live.configuration.clone();
    let added = mash_app::blocking(move || enrollment::read(&path)).await?;
    Ok(Json(Overview {
        build: mash_app::BUILD_REVISION.to_owned(),
        started_at: live.started_at,
        now,
        machines,
        actions: added.audit.iter().rev().cloned().collect(),
        keys: added
            .admins
            .iter()
            .map(|key| KeyName {
                name: key.name.clone(),
                created_at: key.created_at,
            })
            .collect(),
    }))
}

fn not_live() -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        AppError::usage("this server's machines are set in its configuration file only"),
    )
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct MachineRequest {
    /// Defaults to the server's only group.
    #[serde(default)]
    group_id: Option<String>,
    node_id: String,
    /// For a deletion: the machine's name typed again.
    #[serde(default)]
    confirm: Option<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub(crate) struct JoinAnswer {
    /// Shown once; it holds the machine's one-time secret.
    join: String,
    expires_in_seconds: u64,
    /// The server must restart before it accepts machines in this group.
    restart_needed: bool,
}

#[derive(Serialize, schemars::JsonSchema)]
pub(crate) struct Done {
    done: bool,
}

/// The group a request names, or the server's only one.
async fn group_of(app: &App, request: &MachineRequest) -> Result<String, ApiError> {
    if let Some(group) = &request.group_id {
        mash_app::config::validate_identifier(group, "group_id")?;
        return Ok(group.clone());
    }
    let live = app.live.as_ref().ok_or_else(not_live)?;
    let config = configuration::load(&live.configuration).await?;
    let mut groups = configuration::members(&config)
        .into_iter()
        .map(|member| member.group_id)
        .collect::<Vec<_>>();
    groups.sort();
    groups.dedup();
    match groups.as_slice() {
        [group] => Ok(group.clone()),
        _ => Err(AppError::usage("this server serves several groups; name the group").into()),
    }
}

/// Apply one change to the machines file, record it, and take it into effect.
async fn change<T: Send + 'static>(
    app: &App,
    admin: &Admin,
    action: &str,
    group: &str,
    node: &str,
    apply: impl FnOnce(&std::path::Path, &mut enrollment::Added) -> Result<T, AppError> + Send + 'static,
) -> Result<T, ApiError> {
    mash_app::config::validate_identifier(node, "node_id")?;
    let live = app.live.as_ref().ok_or_else(not_live)?;
    let path = live.configuration.clone();
    let record = admin::Action {
        at: enrollment::now(),
        key: admin.key.clone(),
        from: admin.from.clone(),
        action: action.to_owned(),
        target: format!("{group}/{node}"),
    };
    let result = mash_app::blocking(move || {
        let applied = enrollment::update(&path, |added| apply(&path, added))?;
        enrollment::update(&path, |added| {
            admin::record(added, record);
            Ok(())
        })?;
        Ok(applied)
    })
    .await?;
    // The change is saved; take it in now. Work it starts, such as deleting
    // a machine's history, continues in the server's own loop if it cannot
    // finish here, so its failure does not undo or misreport the change.
    if let Err(error) = super::roster::refresh(app, live, true).await {
        tracing::warn!(message = %error.error.message, "an admin change is saved but not applied yet");
    }
    tracing::info!(action, group, node, "admin changed a machine");
    Ok(result)
}

pub(crate) async fn add(
    State(app): State<App>,
    Extension(admin): Extension<Admin>,
    input: Result<Json<MachineRequest>, JsonRejection>,
) -> Result<Json<JoinAnswer>, ApiError> {
    let request = json(input)?;
    let group = group_of(&app, &request).await?;
    let live = app.live.as_ref().ok_or_else(not_live)?;
    let config = configuration::load(&live.configuration).await?;
    let configured = configuration::configured(&config);
    let (g, n) = (group.clone(), request.node_id.clone());
    let join = change(
        &app,
        &admin,
        "add",
        &group,
        &request.node_id,
        move |_, added| {
            let secret = enrollment::add(added, &configured, &g, &n, enrollment::now())?;
            enrollment::join_string_for(added, &g, &n, &secret)
        },
    )
    .await?;
    Ok(Json(JoinAnswer {
        join,
        expires_in_seconds: enrollment::JOIN_LIFETIME,
        restart_needed: !app.store.serves(&group),
    }))
}

pub(crate) async fn replace(
    State(app): State<App>,
    Extension(admin): Extension<Admin>,
    input: Result<Json<MachineRequest>, JsonRejection>,
) -> Result<Json<JoinAnswer>, ApiError> {
    let request = json(input)?;
    let group = group_of(&app, &request).await?;
    let (g, n) = (group.clone(), request.node_id.clone());
    let join = change(
        &app,
        &admin,
        "replace",
        &group,
        &request.node_id,
        move |_, added| {
            let secret = enrollment::replace(added, &g, &n, enrollment::now())?;
            enrollment::join_string_for(added, &g, &n, &secret)
        },
    )
    .await?;
    Ok(Json(JoinAnswer {
        join,
        expires_in_seconds: enrollment::JOIN_LIFETIME,
        restart_needed: !app.store.serves(&group),
    }))
}

pub(crate) async fn remove(
    State(app): State<App>,
    Extension(admin): Extension<Admin>,
    input: Result<Json<MachineRequest>, JsonRejection>,
) -> Result<Json<Done>, ApiError> {
    let request = json(input)?;
    let group = group_of(&app, &request).await?;
    let (g, n) = (group.clone(), request.node_id.clone());
    change(
        &app,
        &admin,
        "remove",
        &group,
        &request.node_id,
        move |_, added| enrollment::remove(added, &g, &n, enrollment::now()),
    )
    .await?;
    Ok(Json(Done { done: true }))
}

/// Delete everything a removed machine published. The name must be typed
/// again, and the machine must already be removed, so no credential can
/// publish while the deletion runs.
pub(crate) async fn delete(
    State(app): State<App>,
    Extension(admin): Extension<Admin>,
    input: Result<Json<MachineRequest>, JsonRejection>,
) -> Result<Json<Done>, ApiError> {
    let request = json(input)?;
    if request.confirm.as_deref() != Some(request.node_id.as_str()) {
        return Err(AppError::usage(
            "type the machine's name to confirm deleting what it published",
        )
        .into());
    }
    let group = group_of(&app, &request).await?;
    let (g, n) = (group.clone(), request.node_id.clone());
    change(
        &app,
        &admin,
        "delete",
        &group,
        &request.node_id,
        move |_, added| enrollment::request_delete(added, &g, &n, enrollment::now()),
    )
    .await?;
    Ok(Json(Done { done: true }))
}

#[cfg(test)]
pub(crate) fn reset_failures() {
    *FAILURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = (0, 0);
}
