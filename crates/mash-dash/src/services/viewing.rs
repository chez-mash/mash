//! Reading another machine's own dashboard, for the viewing switcher.
//!
//! A machine can be viewed "live" when the services inventory names its
//! dashboard: an address and, usually, the services token that dashboard
//! accepts. The browser asks this dashboard, which asks the other one and adds
//! that token itself, so the token never reaches the browser. Only the history
//! reads a scoped page makes are forwarded, only by GET, and only to machines
//! the inventory names; anything else is refused here. A machine that does not
//! answer is reported as such, and the page falls back to its published copy.
use std::sync::Arc;
use std::time::Duration;

use aide::axum::ApiRouter;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::super::Dash;

/// The history reads a scoped page makes. A peer presenting this machine's
/// services token may make them too, which is what lets another dashboard
/// show this machine live.
const READS: &[&str] = &[
    "/api/chats",
    "/api/search",
    "/api/search/live",
    "/api/records",
    "/api/record",
    "/api/view",
    "/api/chat",
];

/// The catalogue of a few thousand chats is a few megabytes; more than this
/// is not a dashboard answering.
const LIMIT: usize = 64 * 1024 * 1024;

/// A machine that is away should not hold a page for the services client's
/// longer wait; the page shows the published copy instead.
#[cfg(not(test))]
const WAIT: Duration = Duration::from_secs(20);
#[cfg(test)]
const WAIT: Duration = Duration::from_millis(400);

/// Whether a request comes from another dashboard presenting this machine's
/// services token, for a service route or one of the history reads above.
pub(crate) fn is_peer(
    services: &super::Services,
    service_path: bool,
    request: &axum::extract::Request,
) -> bool {
    let reads = request.method() == Method::GET && READS.contains(&request.uri().path());
    (service_path || reads) && services.has_token() && services.authorized(request.headers())
}

pub(crate) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::super::contract::reads;
    use aide::axum::routing::get_with;
    router
        .api_route(
            "/api/viewing",
            get_with(
                viewing,
                reads::<Viewing>(
                    "/api/viewing",
                    "The other machines whose own dashboards this one can read live.",
                ),
            ),
        )
        // One configured machine's answer to one of `READS`, with its query.
        // Documented here rather than in the contract: it answers whatever the
        // forwarded route does.
        .route("/api/peer/{machine}/{*rest}", axum::routing::get(forward))
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub(crate) struct Viewing {
    /// Machines with a configured dashboard, by machine ID. Whether one
    /// answers is known only when it is asked.
    machines: Vec<String>,
}

async fn viewing(State(state): State<Arc<Dash>>) -> axum::Json<Viewing> {
    let local = state.services.local_id().to_owned();
    axum::Json(Viewing {
        machines: state
            .services
            .peers
            .iter()
            .filter(|peer| peer.id != local)
            .map(|peer| peer.id.clone())
            .collect(),
    })
}

fn refuse(status: StatusCode, message: String) -> Response {
    (status, axum::Json(crate::AppError::configuration(message))).into_response()
}

async fn forward(
    State(state): State<Arc<Dash>>,
    Path((machine, rest)): Path<(String, String)>,
    RawQuery(query): RawQuery,
) -> Response {
    let path = format!("/{rest}");
    if !READS.contains(&path.as_str()) {
        return refuse(
            StatusCode::NOT_FOUND,
            "Only history reads can be made on another machine".into(),
        );
    }
    let Some(peer) = state.services.peers.iter().find(|peer| peer.id == machine) else {
        return refuse(
            StatusCode::NOT_FOUND,
            format!("{machine} has no dashboard configured here"),
        );
    };
    let url = match query {
        Some(query) => format!("{}{path}?{query}", peer.url),
        None => format!("{}{path}", peer.url),
    };
    let mut request = state
        .services
        .client
        .get(url)
        .timeout(WAIT)
        .header("x-mash-service-forwarded", "1");
    if let Some(token) = &peer.token {
        request = request.bearer_auth(token);
    }
    let mut response = match request.send().await {
        Ok(response) => response,
        Err(error) if error.is_timeout() => {
            return refuse(
                StatusCode::GATEWAY_TIMEOUT,
                format!("{machine}'s dashboard didn't answer in time"),
            );
        }
        Err(_) => {
            return refuse(
                StatusCode::BAD_GATEWAY,
                format!("{machine}'s dashboard could not be reached"),
            );
        }
    };
    let status = response.status();
    if !status.is_success() {
        let message = if status == reqwest::StatusCode::UNAUTHORIZED {
            format!("{machine}'s dashboard refused this dashboard's services token")
        } else {
            format!("{machine}'s dashboard answered HTTP {}", status.as_u16())
        };
        return refuse(StatusCode::BAD_GATEWAY, message);
    }
    let mut body = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) if body.len() + chunk.len() <= LIMIT => body.extend_from_slice(&chunk),
            Ok(Some(_)) => {
                return refuse(
                    StatusCode::BAD_GATEWAY,
                    format!("{machine}'s dashboard sent more than a dashboard answers"),
                );
            }
            Ok(None) => break,
            Err(error) if error.is_timeout() => {
                return refuse(
                    StatusCode::GATEWAY_TIMEOUT,
                    format!("{machine}'s dashboard didn't answer in time"),
                );
            }
            Err(_) => {
                return refuse(
                    StatusCode::BAD_GATEWAY,
                    format!("{machine}'s dashboard stopped answering"),
                );
            }
        }
    }
    // Only the body is passed on: the other dashboard's headers say nothing
    // the browser needs, and this answer carries this dashboard's own.
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

#[cfg(test)]
mod tests;
