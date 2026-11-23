//! Recording the dashboard's tool use in the usage log: searches, reads,
//! Tool explorer runs and every action, once per request, with its query or
//! body as arguments. Page plumbing (the chat catalogue, health, live
//! events, service status) is not tool use and is not recorded. Bodies that
//! carry a credential (a join string, an admin key, the assistant's key) are
//! never stored; the rest are masked like any recorded argument.
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use mash_app::core;
use mash_app::usage::{self, Outcome, Started, Surface};

use crate::Dash;

/// The largest body kept as arguments; a bigger one is recorded by size.
const BODY_LIMIT: usize = 64 * 1024;

/// Reads of history and tool runs, recorded on GET.
const READS: [&str; 8] = [
    "/api/search",
    "/api/search/live",
    "/api/search/stream",
    "/api/view",
    "/api/chat",
    "/api/record",
    "/api/records",
    "/api/console/call",
];

/// Actions whose body is a credential, recorded without it.
const SECRET_BODIES: [&str; 3] = [
    "/api/shared/join",
    "/api/shared/admin/key",
    "/api/services/assistant",
];

fn recorded(method: &Method, path: &str) -> bool {
    if method == Method::GET {
        READS.contains(&path)
    } else {
        // Every change the dashboard makes is a call; sign-in is not.
        path.starts_with("/api/") && !path.starts_with("/api/session")
    }
}

pub(crate) async fn record(
    State(state): State<Arc<Dash>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();
    if !recorded(request.method(), &path) {
        return next.run(request).await;
    }
    let started = Started::now();
    let client = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(browser);
    let mut arguments = query(request.uri());
    let request = if request.method() == Method::GET {
        request
    } else {
        let (parts, body) = request.into_parts();
        let Ok(bytes) = axum::body::to_bytes(body, BODY_LIMIT * 16).await else {
            return (
                axum::http::StatusCode::PAYLOAD_TOO_LARGE,
                "request body too large",
            )
                .into_response();
        };
        arguments.insert("body".to_owned(), body_argument(&path, &bytes));
        Request::from_parts(parts, axum::body::Body::from(bytes))
    };
    let response = next.run(request).await;
    let mut call = started.call(
        Surface::Dashboard,
        &path,
        serde_json::Value::Object(arguments),
    );
    call.client = client;
    call.response_bytes = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        call.outcome = Outcome::Error;
        call.error_message = Some(format!("HTTP {}", status.as_u16()));
    }
    let database = state
        .paths
        .db
        .clone()
        .unwrap_or_else(core::default_database_path);
    usage::record(&database, call);
    response
}

fn query(uri: &axum::http::Uri) -> serde_json::Map<String, serde_json::Value> {
    let pairs = axum::extract::Query::<Vec<(String, String)>>::try_from_uri(uri)
        .map_or_else(|_| Vec::new(), |query| query.0);
    pairs
        .into_iter()
        .map(|(key, value)| (key, serde_json::Value::String(value)))
        .collect()
}

fn body_argument(path: &str, bytes: &[u8]) -> serde_json::Value {
    if SECRET_BODIES.contains(&path) {
        return serde_json::Value::String("[not recorded: carries a credential]".to_owned());
    }
    if bytes.len() > BODY_LIMIT {
        return serde_json::Value::String(format!("[{} bytes]", bytes.len()));
    }
    serde_json::from_slice(bytes)
        .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(bytes).into_owned()))
}

/// The browser family, not its full user-agent string.
fn browser(agent: &str) -> String {
    let family = if agent.contains("Edg/") {
        "Edge"
    } else if agent.contains("Firefox/") {
        "Firefox"
    } else if agent.contains("Chrome/") {
        "Chrome"
    } else if agent.contains("Safari/") {
        "Safari"
    } else if agent.contains("AppleWebKit/") {
        // The Mash desktop app's WebKit view reports no browser name.
        "WebKit"
    } else {
        agent.split(['/', ' ']).next().unwrap_or("unknown")
    };
    family.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_tool_use_is_recorded() {
        let cases = [
            (Method::GET, "/api/search", true),
            (Method::GET, "/api/console/call", true),
            (Method::GET, "/api/chats", false),
            (Method::GET, "/api/health", false),
            (Method::GET, "/api/events", false),
            (Method::GET, "/api/services", false),
            (Method::POST, "/api/services/action", true),
            (Method::POST, "/api/session", false),
            (Method::POST, "/api/assistant", true),
        ];
        for (method, path, expected) in cases {
            assert_eq!(recorded(&method, path), expected, "{method} {path}");
        }
    }

    #[test]
    fn credential_bodies_are_never_kept() {
        for path in SECRET_BODIES {
            assert_eq!(
                body_argument(path, br#"{"join":"mash-join.secret"}"#),
                serde_json::Value::String("[not recorded: carries a credential]".to_owned())
            );
        }
        assert_eq!(
            body_argument("/api/services/action", br#"{"action":"restart"}"#),
            serde_json::json!({"action": "restart"})
        );
    }

    #[test]
    fn browsers_are_named_by_family() {
        let cases = [
            (
                "Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
                "Safari",
            ),
            (
                "Mozilla/5.0 (X11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36",
                "Chrome",
            ),
            (
                "Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15 (KHTML, like Gecko)",
                "WebKit",
            ),
            ("curl/8.5.0", "curl"),
        ];
        for (agent, family) in cases {
            assert_eq!(browser(agent), family, "{agent}");
        }
    }
}
