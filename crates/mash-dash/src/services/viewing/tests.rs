use super::super::Peer;
use super::super::tests::{TOKEN, app_signed};
use std::sync::{Arc, Mutex};

/// The token this dashboard presents to the other machine's dashboard.
const PEER_TOKEN: &str = "peer-services-token-abcdefghijklmnopqrstuvwxyz";

/// A stand-in for another machine's dashboard: it records each request's path,
/// query and Authorization header, answers JSON, and can be made to hang.
async fn other_dashboard(
    hang: bool,
) -> (
    String,
    Arc<Mutex<Vec<(String, String)>>>,
    tokio::task::JoinHandle<()>,
) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    let app = axum::Router::new().fallback(move |request: axum::extract::Request| {
        let log = log.clone();
        async move {
            let authorization = request
                .headers()
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_owned();
            let target = request
                .uri()
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_default();
            log.lock().unwrap().push((target, authorization));
            if hang {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            axum::Json(serde_json::json!({ "chats": [], "local_node": "xenia" }))
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, seen, handle)
}

fn peer(url: &str) -> Peer {
    Peer {
        id: "xenia".into(),
        url: url.into(),
        token: Some(PEER_TOKEN.into()),
    }
}

#[tokio::test]
async fn a_configured_machine_is_read_live_with_its_token_kept_here() {
    let root = tempfile::tempdir().unwrap();
    let (url, seen, other) = other_dashboard(false).await;
    let (base, local) = app_signed(root.path(), vec![peer(&url)], "laptop", None).await;
    let client = reqwest::Client::new();

    let viewing: serde_json::Value = client
        .get(format!("{base}/api/viewing"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(viewing, serde_json::json!({ "machines": ["xenia"] }));

    let response = client
        .get(format!("{base}/api/peer/xenia/api/chats?scope=all&q=a%20b"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let headers = format!("{:?}", response.headers());
    let body = response.text().await.unwrap();
    assert!(body.contains("\"local_node\":\"xenia\""), "{body}");
    // The other machine's token is presented there and never comes back here.
    assert!(!body.contains(PEER_TOKEN) && !headers.contains(PEER_TOKEN));
    let requests = seen.lock().unwrap().clone();
    assert_eq!(
        requests,
        vec![(
            "/api/chats?scope=all&q=a%20b".to_owned(),
            format!("Bearer {PEER_TOKEN}")
        )]
    );
    local.abort();
    other.abort();
}

#[tokio::test]
async fn only_history_reads_on_configured_machines_are_forwarded() {
    let root = tempfile::tempdir().unwrap();
    let (url, seen, other) = other_dashboard(false).await;
    let (base, local) = app_signed(root.path(), vec![peer(&url)], "laptop", None).await;
    let client = reqwest::Client::new();
    for path in [
        "/api/peer/xenia/api/services",
        "/api/peer/xenia/api/services/action",
        "/api/peer/xenia/api/console/call",
        "/api/peer/xenia/api/session",
        "/api/peer/xenia/api/chats/../services",
        "/api/peer/xenia/api/openapi.json",
        "/api/peer/elsewhere/api/chats",
    ] {
        let response = client.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(response.status(), 404, "{path}");
    }
    // Only reads: anything else is refused before it is forwarded.
    let post = client
        .post(format!("{base}/api/peer/xenia/api/chats"))
        .send()
        .await
        .unwrap();
    assert_eq!(post.status(), 405);
    assert!(seen.lock().unwrap().is_empty());
    local.abort();
    other.abort();
}

#[tokio::test]
async fn a_machine_that_does_not_answer_is_reported_not_waited_for() {
    let root = tempfile::tempdir().unwrap();
    let (url, _, other) = other_dashboard(true).await;
    let (base, local) = app_signed(root.path(), vec![peer(&url)], "laptop", None).await;
    let started = std::time::Instant::now();
    let response = reqwest::get(format!("{base}/api/peer/xenia/api/chats"))
        .await
        .unwrap();
    assert_eq!(response.status(), 504);
    assert!(started.elapsed() < std::time::Duration::from_secs(4));
    let body = response.text().await.unwrap();
    assert!(body.contains("xenia's dashboard didn't answer"), "{body}");

    // One that is not there at all says so too.
    other.abort();
    let gone = peer("http://127.0.0.1:1");
    let root = tempfile::tempdir().unwrap();
    let (base, unreachable) = app_signed(root.path(), vec![gone], "laptop", None).await;
    let response = reqwest::get(format!("{base}/api/peer/xenia/api/chats"))
        .await
        .unwrap();
    assert!(matches!(response.status().as_u16(), 502 | 504));
    local.abort();
    unreachable.abort();
}

#[tokio::test]
async fn a_signed_in_dashboard_keeps_its_peer_reads_behind_sign_in() {
    let root = tempfile::tempdir().unwrap();
    let (url, seen, other) = other_dashboard(false).await;
    let access = "dashboard-access-token-0123456789abcdefghijkl";
    let (base, local) = app_signed(root.path(), vec![peer(&url)], "laptop", Some(access)).await;
    let client = reqwest::Client::new();
    // A browser that has not signed in cannot use this dashboard to read another.
    let stranger = client
        .get(format!("{base}/api/peer/xenia/api/chats"))
        .send()
        .await
        .unwrap();
    assert_eq!(stranger.status(), 401);
    assert!(seen.lock().unwrap().is_empty());
    local.abort();
    other.abort();
}

#[tokio::test]
async fn the_services_token_reads_history_but_nothing_else() {
    let root = tempfile::tempdir().unwrap();
    let access = "dashboard-access-token-0123456789abcdefghijkl";
    let (base, local) = app_signed(root.path(), Vec::new(), "xenia", Some(access)).await;
    let client = reqwest::Client::new();
    let get = |path: &'static str, token: &'static str| {
        client
            .get(format!("{base}{path}"))
            .bearer_auth(token)
            .send()
    };
    // Another dashboard presenting this machine's services token may read
    // history; the answer may be an error about the test database, but it is
    // not a refusal.
    for path in ["/api/chats", "/api/records", "/api/search?q=x"] {
        let status = get(path, TOKEN).await.unwrap().status();
        assert_ne!(status, 401, "{path}");
    }
    // It may not use the tools, write anything, or sign in as a person.
    for path in ["/api/console/tools", "/api/status", "/api/openapi.json"] {
        assert_eq!(get(path, TOKEN).await.unwrap().status(), 401, "{path}");
    }
    let write = client
        .post(format!("{base}/api/assistant"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .unwrap();
    assert_ne!(write.status(), 200);
    // A wrong token reads nothing.
    let wrong = get("/api/chats", "not-the-services-token-0123456789abcdef")
        .await
        .unwrap();
    assert_eq!(wrong.status(), 401);
    local.abort();
}
