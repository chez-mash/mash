//! Links that open a chat in its agent's own desktop app on this machine.
//!
//! A link is offered only where it was checked against the app itself, and
//! only when nobody but this machine can reach the dashboard, since the link
//! opens an app on the computer that follows it. Each app's route, and which
//! sessions it opens, is described where the link is made (`links`).
//!
//! Apps and their session lists are read from their macOS and Windows
//! locations; elsewhere nothing is offered.
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use mash_model::provider::Provider;

use super::Dash;

#[cfg(any(target_os = "macos", windows, test))]
mod links;
#[cfg(any(windows, test))]
mod registry;
#[cfg(test)]
mod tests;

pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::reads;
    use aide::axum::routing::get_with;
    router.api_route(
        "/api/open-in",
        get_with(
            open_in,
            reads::<OpenIn>(
                "/api/open-in",
                "The link that opens a chat of this machine in its agent's desktop app, if any.",
            ),
        ),
    )
}

// On a system without these apps the parameters are read by nobody and no
// link is ever made, but the route and its schema are the same everywhere.
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct OpenInParams {
    agent: Provider,
    /// The chat's ID in Mash.
    id: String,
    /// The folder the chat ran in, as Mash recorded it.
    cwd: Option<String>,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
struct OpenIn {
    /// Absent when no app on this machine can open the chat.
    link: Option<AppLink>,
}

#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
#[derive(Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AppLink {
    /// The app's name, as a button names it.
    app: &'static str,
    url: String,
    /// Whether the app does not have the session yet, so following the link
    /// adds it to the app as a new session.
    adds_session: bool,
}

async fn open_in(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<OpenInParams>,
) -> Response {
    let link = if state.loopback && !state.shared_only {
        find(params).await
    } else {
        None
    };
    axum::Json(OpenIn { link }).into_response()
}

#[cfg(any(target_os = "macos", windows))]
async fn find(params: OpenInParams) -> Option<AppLink> {
    let machine = links::Machine::this()?;
    tokio::task::spawn_blocking(move || {
        links::app_link(params.agent, &params.id, params.cwd.as_deref(), &machine)
    })
    .await
    .ok()
    .flatten()
}

#[cfg(not(any(target_os = "macos", windows)))]
#[allow(clippy::unused_async)]
async fn find(_params: OpenInParams) -> Option<AppLink> {
    None
}
