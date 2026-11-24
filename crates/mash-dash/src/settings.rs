//! The configuration file's settings, and setting Mashy up, from the browser.
//!
//! These are service routes, so the router's guard applies the services token
//! and origin check first. A dashboard other machines can reach accepts a
//! change, or Mashy's model service and API key, only when it was started with
//! a services configuration; the saved key is never sent back.
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_app::config;
use mash_model::AppError;

use super::Dash;
use mash_app::core::Paths;

/// The configuration file's settings, and setting Mashy up.
pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::{changes, reads};
    use aide::axum::routing::{get_with, post_with};
    router
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
            "/api/services/assistant",
            post_with(
                set_up_assistant,
                changes::<Settings>(
                    "/api/services/assistant",
                    "Point Mashy at a model service and save its API key on this machine.",
                ),
            ),
        )
}

/// The configuration file this dashboard's Mash reads.
pub(super) fn configuration_path(paths: &Paths) -> Result<std::path::PathBuf, AppError> {
    match &paths.config {
        Some(path) => Ok(path.clone()),
        None => config::default_path(),
    }
}

async fn settings(State(state): State<Arc<Dash>>) -> Response {
    let paths = state.paths.clone();
    let view =
        tokio::task::spawn_blocking(move || config::settings::view(&configuration_path(&paths)?))
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
    input: Result<axum::Json<config::settings::Change>, axum::extract::rejection::JsonRejection>,
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
        config::settings::apply(&configuration_path(&paths)?, &change)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("write settings")));
    settings_result(view, true)
}

/// Sets Mashy up: points the configuration at a model service and saves its
/// API key with this machine's other saved credentials. The same machines
/// that may change settings may do this; the key is never sent back.
async fn set_up_assistant(
    State(state): State<Arc<Dash>>,
    input: Result<
        axum::Json<config::settings::AssistantSetup>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Response {
    if !state.loopback && !state.services.managed() {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(AppError::configuration(
                "Mashy can be set up on the machine itself, or from a dashboard started with a services configuration",
            )),
        )
            .into_response();
    }
    let Ok(axum::Json(setup)) = input else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(AppError::configuration("invalid assistant setup")),
        )
            .into_response();
    };
    let paths = state.paths.clone();
    let view = tokio::task::spawn_blocking(move || {
        config::settings::set_up_assistant(&configuration_path(&paths)?, &setup)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("set up the assistant")));
    settings_result(view, true)
}

/// The settings, with whether this dashboard accepts changes to them.
#[derive(serde::Serialize, schemars::JsonSchema)]
struct Settings {
    #[serde(flatten)]
    view: config::settings::View,
    editable: bool,
}

fn settings_result(view: Result<config::settings::View, AppError>, editable: bool) -> Response {
    match view {
        Ok(view) => axum::Json(Settings { view, editable }).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.error })),
        )
            .into_response(),
    }
}
