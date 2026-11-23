use super::search::{LiveParams, SearchParams, live_operation, search_operation};
use super::{Operation, allowed_hosts, content_type};

fn record_fixture() -> (tempfile::TempDir, crate::core::Paths, String) {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    std::fs::write(&config, "version=1\nproviders={}\n").unwrap();
    let database = directory.path().join("index.sqlite3");
    drop(crate::storage::Storage::open(&database).unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    crate::storage::packed::register(&connection).unwrap();
    connection.execute_batch("INSERT INTO chats(id,provider) VALUES('fixture','codex'); INSERT INTO messages(id,chat_id,ordinal,role,content_packed) VALUES('source','fixture',0,'user','The fixture uses port 8080.');").unwrap();
    drop(connection);
    let body = crate::records::Body {
        statement: "Fixture uses port 8080".into(),
        rationale: "Reported in chat".into(),
        review: crate::records::Review::Established,
        details: crate::records::Details::Fact {
            state: crate::records::FactState::Current,
            reported_by: "fixture user".into(),
            valid_until: None,
        },
        effective_at: None,
        recorded_by: "HTTP test".into(),
    };
    let mut storage = crate::storage::Storage::open_existing(&database).unwrap();
    let receipt = storage
        .mutate_record(&crate::records::Mutation {
            operation_id: "fixture-create".into(),
            change: crate::records::Change::Create {
                scope: crate::records::Scope::Personal,
                body,
                evidence: vec![crate::records::Citation {
                    message_id: "source".into(),
                    session_id: None,
                    quote: "port 8080".into(),
                    purpose: crate::records::EvidenceUse::Report,
                    expected_fingerprint: None,
                }],
                on_duplicate: crate::records::DuplicatePolicy::Report,
            },
        })
        .unwrap();
    (
        directory,
        crate::core::Paths {
            config: Some(config),
            db: Some(database),
            models_dir: None,
        },
        receipt.record_id,
    )
}

#[tokio::test]
async fn only_a_signed_in_caller_mints_a_code_that_signs_another_browser_in() {
    const TOKEN: &str = "0123456789abcdef0123456789abcdef";
    let (_directory, paths, _) = record_fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts: allowed_hosts(address),
        loopback: false,
        access: super::access::Access::new(Some(TOKEN)),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let url = |path: &str| format!("http://{address}{path}");
    let stranger = client.post(url("/api/session/code")).send().await.unwrap();
    assert_eq!(stranger.status(), reqwest::StatusCode::UNAUTHORIZED);
    let minted: serde_json::Value = client
        .post(url("/api/session/code"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let code = minted["code"].as_str().unwrap().to_lowercase();
    let with_code = client
        .post(url("/api/session"))
        .json(&serde_json::json!({ "token": code }))
        .send()
        .await
        .unwrap();
    assert_eq!(with_code.status(), reqwest::StatusCode::OK);
    assert!(with_code.headers().contains_key("set-cookie"));
    // The code is spent.
    let again = client
        .post(url("/api/session"))
        .json(&serde_json::json!({ "token": code }))
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), reqwest::StatusCode::UNAUTHORIZED);
    server.abort();
}

#[tokio::test]
async fn a_reachable_dashboard_serves_history_only_after_sign_in() {
    const TOKEN: &str = "0123456789abcdef0123456789abcdef";
    let (_directory, paths, _) = record_fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut hosts = allowed_hosts(address);
    hosts.push("mash.example.ts.net".into());
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts,
        loopback: false,
        access: super::access::Access::new(Some(TOKEN)),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let url = |path: &str| format!("http://{address}{path}");
    // The page and build stamp are public; nothing that reads history is.
    for path in ["/", "/api/health", "/api/session"] {
        let status = client.get(url(path)).send().await.unwrap().status();
        assert_ne!(status, reqwest::StatusCode::UNAUTHORIZED, "{path}");
    }
    for path in [
        "/api/status",
        "/api/chats",
        "/api/search?q=port",
        "/api/view?id=source",
        "/api/records",
        "/api/console/tools",
        "/api/services",
        "/api/services/access",
        "/api/openapi.json",
    ] {
        let status = client.get(url(path)).send().await.unwrap().status();
        assert_eq!(status, reqwest::StatusCode::UNAUTHORIZED, "{path}");
    }
    let sign_in = |token: &'static str, origin: Option<&'static str>| {
        let mut request = client
            .post(url("/api/session"))
            .json(&serde_json::json!({ "token": token }));
        if let Some(origin) = origin {
            request = request.header("Origin", origin);
        }
        request.send()
    };
    assert_eq!(
        sign_in("wrong", None).await.unwrap().status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    // Another website cannot sign a visitor's browser in.
    assert_eq!(
        sign_in(TOKEN, Some("https://evil.invalid"))
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::FORBIDDEN
    );
    let accepted = sign_in(TOKEN, None).await.unwrap();
    assert_eq!(accepted.status(), reqwest::StatusCode::OK);
    let set = accepted.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(set.contains("HttpOnly") && !set.contains(TOKEN));
    let cookie = set.split(';').next().unwrap().to_owned();
    for path in ["/api/status", "/api/view?id=source&node=local"] {
        let response = client
            .get(url(path))
            .header("Cookie", &cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK, "{path}");
    }
    // The proxy's name is accepted; the session works under it too.
    let named = client
        .get(url("/api/session"))
        .header("Host", "mash.example.ts.net")
        .header("Cookie", &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(named.status(), reqwest::StatusCode::OK);
    let named: serde_json::Value = named.json().await.unwrap();
    assert_eq!(named["signed_in"], true);
    let ended = client.post(url("/api/session/end")).send().await.unwrap();
    assert!(
        ended.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    server.abort();
}

#[tokio::test]
async fn dashboard_shared_chat_source_never_falls_back_to_local_node() {
    let (_directory, paths, _) = record_fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts: allowed_hosts(address),
        loopback: address.ip().is_loopback(),
        access: super::access::Access::new(None),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    for path in ["/api/view?id=source", "/api/chat?id=fixture"] {
        let url = format!("http://{address}{path}&node=local");
        let local = client.get(&url).send().await.unwrap();
        assert_eq!(local.status(), reqwest::StatusCode::OK);
        if path.starts_with("/api/chat?") {
            // A page states the chat's length so a reader can jump to its end.
            let page: serde_json::Value = local.json().await.unwrap();
            let returned = page["messages"].as_array().unwrap().len() as u64;
            assert!(returned > 0);
            assert!(page["message_count"].as_u64().unwrap() >= returned);
        }
        let shared = client
            .get(format!("{url}&shared=true"))
            .send()
            .await
            .unwrap();
        assert_eq!(shared.status(), reqwest::StatusCode::BAD_REQUEST);
        assert!(
            shared
                .text()
                .await
                .unwrap()
                .contains("shared index is not configured")
        );
        let no_node = client
            .get(format!("http://{address}{path}&shared=true"))
            .send()
            .await
            .unwrap();
        assert_eq!(no_node.status(), reqwest::StatusCode::BAD_REQUEST);
        assert!(
            no_node
                .text()
                .await
                .unwrap()
                .contains("require a source node")
        );
    }
    server.abort();
}

#[tokio::test]
async fn dashboard_records_expose_real_filtered_history_without_mutation() {
    let (_directory, paths, id) = record_fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts: allowed_hosts(address),
        loopback: address.ip().is_loopback(),
        access: super::access::Access::new(None),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let list = format!(
        "http://{address}/api/records?kind=fact&state=current&review=established&chat=fixture&q=8080"
    );
    let response = client.get(&list).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let page: serde_json::Value = response.json().await.unwrap();
    assert_eq!(page["records"].as_array().unwrap().len(), 1);
    assert_eq!(page["records"][0]["id"], id);
    let detail = format!("http://{address}/api/record?id={id}");
    let record: serde_json::Value = client
        .get(&detail)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(record["current_revision"], 1);
    assert_eq!(
        record["revisions"][0]["evidence"][0]["message_id"],
        "source"
    );
    assert_eq!(record["revisions"][0]["evidence_current"], true);
    assert_eq!(
        client.post(&detail).send().await.unwrap().status(),
        reqwest::StatusCode::METHOD_NOT_ALLOWED
    );
    assert_eq!(
        client
            .get(&detail)
            .header("Host", "evil.invalid")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::MISDIRECTED_REQUEST
    );
    for path in [
        "/api/records?kind=unknown",
        "/api/records?state=unknown",
        "/api/records?node=remote&shared=true",
        "/api/record?id=missing&node=remote",
        "/api/record?id=missing&unknown=true",
    ] {
        assert_eq!(
            client
                .get(format!("http://{address}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
    let after: serde_json::Value = client
        .get(&detail)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after, record);
    server.abort();
}

#[tokio::test]
async fn dashboard_reports_status_read_only_and_compresses_responses() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    std::fs::write(&config, "version = 1\nproviders = {}\n").unwrap();
    let database = directory.path().join("index.sqlite3");
    let mut storage = crate::storage::Storage::open_writer(&database).unwrap();
    storage.commit_writer().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths: crate::core::Paths {
            config: Some(config),
            db: Some(database),
            models_dir: None,
        },
        hosts: allowed_hosts(address),
        loopback: address.ip().is_loopback(),
        access: super::access::Access::new(None),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    // A client that leaves the body as sent shows the encoding on the wire.
    let client = reqwest::Client::builder().no_gzip().build().unwrap();
    let url = format!("http://{address}/api/status");
    // Settings and readiness are a read of what `mash status` reports.
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let status: serde_json::Value = response.json().await.unwrap();
    assert_eq!(status["configuration"]["loaded"], true);
    assert_eq!(status["configuration"]["providers"], serde_json::json!({}));
    assert_eq!(status["chats"], 0);
    assert_eq!(
        client.post(&url).send().await.unwrap().status(),
        reqwest::StatusCode::METHOD_NOT_ALLOWED
    );
    // The console lists every tool an agent sees and runs only readers.
    let tools: serde_json::Value = client
        .get(format!("http://{address}/api/console/tools"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "search" && tool["inputSchema"].is_object())
    );
    let call = |query: &'static str| {
        let request = client.get(format!("http://{address}/api/console/call?{query}"));
        async move { request.send().await.unwrap() }
    };
    let refused: serde_json::Value = call("name=forget&arguments=%7B%22chat_id%22%3A%22x%22%7D")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(refused["reply"]["error"]["kind"], "not-read-only");
    assert_eq!(
        call("name=help&arguments=%5B%5D").await.status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    let help: serde_json::Value = call("name=help").await.json().await.unwrap();
    assert_eq!(help["failed"], false);
    let compressed = client
        .get(&url)
        .header("accept-encoding", "gzip")
        .send()
        .await
        .unwrap();
    assert_eq!(compressed.headers()["content-encoding"], "gzip");
    assert_eq!(compressed.headers()["cache-control"], "no-store");
    // The page and the logo, whose name never changes, are rechecked on every visit.
    for (path, policy) in [("/", "no-cache"), ("/mash-mark.svg", "no-cache")] {
        let response = client
            .get(format!("http://{address}{path}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.headers()["cache-control"], policy, "{path}");
    }
    server.abort();
}

#[tokio::test]
async fn dashboard_http_guards_host_methods_and_serves_real_catalogue() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    std::fs::write(&config, "version = 1\nproviders = {}\n").unwrap();
    let database = directory.path().join("index.sqlite3");
    let mut storage = crate::storage::Storage::open_writer(&database).unwrap();
    storage.commit_writer().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths: crate::core::Paths {
            config: Some(config),
            db: Some(database),
            models_dir: None,
        },
        hosts: allowed_hosts(address),
        loopback: address.ip().is_loopback(),
        access: super::access::Access::new(None),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/api/chats");
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["local_node"], "local");
    assert_eq!(body["chats"], serde_json::json!([]));
    // Resolving the local node must not require shared configuration or a
    // catalogue request. Remote nodes must still fail rather than fall back.
    for node in ["", "local"] {
        let response = client
            .get(format!("http://{address}/api/view?id=missing&node={node}"))
            .send()
            .await
            .unwrap();
        if node.is_empty() {
            // An explicitly empty name is not the local identity.
            assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        } else {
            // The local node resolves, and a missing message is not found.
            assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
            let body: serde_json::Value = response.json().await.unwrap();
            assert_eq!(body["error"]["kind"], "not-found");
        }
    }
    assert_eq!(
        client
            .get(format!("http://{address}/api/view?id=missing&node=remote"))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    assert_eq!(
        client
            .get(&url)
            .header("Host", "attacker.invalid")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::MISDIRECTED_REQUEST
    );
    assert_eq!(
        client.post(&url).send().await.unwrap().status(),
        reqwest::StatusCode::METHOD_NOT_ALLOWED
    );
    server.abort();
}

#[test]
fn live_search_reads_this_machine_with_the_box_filters() {
    let params = || LiveParams {
        q: "chec".into(),
        provider: "codex".into(),
        project: "/p".into(),
        days: Some(7),
        limit: Some(5),
    };
    let Operation::LiveSearch {
        query,
        limit,
        filters,
    } = live_operation(params(), false).unwrap()
    else {
        panic!("expected a live search")
    };
    assert_eq!((query.as_str(), limit), ("chec", Some(5)));
    assert_eq!(filters.project, ["/p"]);
    assert_eq!(filters.days, Some(7));
    // A dashboard that serves only the shared index has no index of its own.
    assert!(live_operation(params(), true).is_err());
    assert!(
        live_operation(
            LiveParams {
                provider: "nope".into(),
                ..params()
            },
            false
        )
        .is_err()
    );
}

#[test]
fn empty_queries_list_chats_and_queries_search_with_filters() {
    let params = || SearchParams {
        provider: "claude-code, codex".into(),
        machine: "xenia".into(),
        ..Default::default()
    };
    let listed = search_operation(params(), false).unwrap();
    let Operation::ListChats { input, shared, .. } = listed else {
        panic!("expected a chat listing")
    };
    assert!(!shared);
    assert_eq!(input.provider.len(), 2);
    assert_eq!(input.node, ["xenia"]);
    let query = || SearchParams {
        q: "backfill".into(),
        project: "/p".into(),
        days: Some(7),
        sort: crate::search_filter::Sort::Recent,
        ..Default::default()
    };
    let searched = search_operation(query(), false).unwrap();
    let Operation::Search {
        query: text,
        intent,
        filters,
        shared,
        ..
    } = searched
    else {
        panic!("expected a search")
    };
    assert!(!shared);
    assert_eq!(intent, None);
    let explicit = crate::views::exact::Intent::ExactMention;
    assert!(matches!(
        search_operation(SearchParams { intent: Some(explicit), ..query() }, true).unwrap(),
        Operation::Search { intent: Some(value), shared: true, .. } if value == explicit
    ));
    let query_text = text;
    // A machine with no history of its own reads only the shared index.
    assert!(matches!(
        search_operation(params(), true).unwrap(),
        Operation::ListChats { shared: true, .. }
    ));
    assert!(matches!(
        search_operation(query(), true).unwrap(),
        Operation::Search { shared: true, .. }
    ));
    assert_eq!(query_text, "backfill");
    assert_eq!(filters.project, ["/p"]);
    assert_eq!(filters.days, Some(7));
    assert_eq!(filters.sort, crate::search_filter::Sort::Recent);
    assert!(
        search_operation(
            SearchParams {
                provider: "nope".into(),
                ..Default::default()
            },
            false
        )
        .is_err()
    );
}

#[test]
fn loopback_accepts_its_own_names_and_nothing_else() {
    let hosts = allowed_hosts("127.0.0.1:8790".parse().unwrap());
    for host in ["127.0.0.1:8790", "localhost:8790", "[::1]:8790"] {
        assert!(hosts.iter().any(|allowed| allowed == host), "{host}");
    }
    for host in ["evil.example:8790", "localhost:9999", "127.0.0.1"] {
        assert!(!hosts.iter().any(|allowed| allowed == host), "{host}");
    }
    let tailnet = allowed_hosts("100.64.0.20:8790".parse().unwrap());
    assert_eq!(tailnet, vec!["100.64.0.20:8790".to_owned()]);
}

#[test]
fn known_assets_get_their_media_types() {
    assert_eq!(content_type("/index.html"), "text/html; charset=utf-8");
    assert_eq!(
        content_type("/assets/index.js"),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(content_type("/agents/claude.svg"), "image/svg+xml");
    assert_eq!(content_type("/favicon.ico"), "image/x-icon");
    assert_eq!(content_type("/unknown"), "application/octet-stream");
}
#[tokio::test]
async fn setup_runs_only_where_the_dashboard_is_private_and_against_the_file_it_showed() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    let paths = crate::core::Paths {
        config: Some(config.clone()),
        ..crate::core::Paths::default()
    };
    let mut addresses = Vec::new();
    for loopback in [true, false] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = super::router(std::sync::Arc::new(super::Dash {
            console: crate::mcp::console::Console::new(crate::core::Paths::default()),
            paths: paths.clone(),
            hosts: allowed_hosts(address),
            loopback,
            access: super::access::Access::new(None),
            shared_only: false,
            model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
            services: super::services::Services::load(None).unwrap(),
            live: super::live::Live::new(None, None),
            questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
        }));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addresses.push(address);
    }
    let client = reqwest::Client::new();
    let urls = addresses
        .iter()
        .map(|address| format!("http://{address}/api/services/setup"))
        .collect::<Vec<_>>();
    for (url, editable) in [(&urls[0], true), (&urls[1], false)] {
        let view: serde_json::Value = client.get(url).send().await.unwrap().json().await.unwrap();
        // No file yet: nothing names its contents, and no index was started.
        assert_eq!(view["configured"], false);
        assert_eq!(view["revision"], "");
        assert_eq!(view["editable"], editable);
        assert_eq!(view["index"]["state"], "idle");
        assert_eq!(view["path"], config.display().to_string());
    }
    let choice = |revision: &str| {
        serde_json::json!({
            "revision": revision, "agents": {}, "since_days": 90, "connect": [], "index": false
        })
    };
    // A dashboard other machines can reach does not run setup.
    let refused = client
        .post(&urls[1])
        .json(&choice(""))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), reqwest::StatusCode::FORBIDDEN);
    // Nor does a page from another origin, even on this machine.
    let foreign = client
        .post(&urls[0])
        .header("origin", "http://evil.example")
        .json(&choice(""))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), reqwest::StatusCode::FORBIDDEN);
    // Choices made against other contents, or naming unknown fields, change nothing.
    for body in [
        choice("not what the page showed"),
        serde_json::json!({"revision": "", "agents": {}, "connect": [], "index": false, "roots": ["/"]}),
    ] {
        let stale = client.post(&urls[0]).json(&body).send().await.unwrap();
        assert_eq!(stale.status(), reqwest::StatusCode::BAD_REQUEST);
    }
    assert!(!config.exists());

    std::fs::write(
        &config,
        format!(
            "version = 1\n\n[providers.codex]\nroots = ['{}']\n",
            directory.path().join("codex").display()
        ),
    )
    .unwrap();
    let view: serde_json::Value = client
        .get(&urls[0])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let saved: serde_json::Value = client
        .post(&urls[0])
        .json(&choice(view["revision"].as_str().unwrap()))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(saved["setup"]["index"]["state"], "idle");
    for line in saved["report"]["next"].as_array().unwrap() {
        assert!(!line.as_str().unwrap().contains("mash index"), "{saved}");
    }
}

#[tokio::test]
#[expect(clippy::too_many_lines)]
async fn settings_are_changed_only_where_the_dashboard_is_private() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    // Configured roots must be absolute on this host.
    std::fs::write(
        &config,
        format!(
            "version = 1\n\n[providers.codex]\nroots = ['{}']\n\n[index]\nall_history = true\n",
            directory.path().join("codex").display()
        ),
    )
    .unwrap();
    let paths = crate::core::Paths {
        config: Some(config.clone()),
        ..crate::core::Paths::default()
    };
    let mut addresses = Vec::new();
    for loopback in [true, false] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = super::router(std::sync::Arc::new(super::Dash {
            console: crate::mcp::console::Console::new(crate::core::Paths::default()),
            paths: paths.clone(),
            hosts: allowed_hosts(address),
            loopback,
            access: super::access::Access::new(None),
            shared_only: false,
            model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
            services: super::services::Services::load(None).unwrap(),
            live: super::live::Live::new(None, None),
            questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
        }));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addresses.push(address);
    }
    let client = reqwest::Client::new();
    let private = format!("http://{}/api/services/settings", addresses[0]);
    let view: serde_json::Value = client
        .get(&private)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["agents"]["codex"]["indexed"], true);
    assert_eq!(view["editable"], true);
    let shown: serde_json::Value = client
        .get(format!("http://{}/api/services/settings", addresses[1]))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(shown["editable"], false);
    assert_eq!(view["since_days"], serde_json::Value::Null);
    let change = |revision: &serde_json::Value| {
        serde_json::json!({
            "revision": revision,
            "agents": {},
            "since_days": 30,
            "destination": "local",
            "memory_capture": false,
            "hide_credentials": { "output": true, "publication": true },
        })
    };
    // A dashboard other machines can reach, without the services opt-in, refuses.
    let open = format!("http://{}/api/services/settings", addresses[1]);
    let refused = client
        .post(&open)
        .json(&change(&view["revision"]))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), reqwest::StatusCode::FORBIDDEN);
    // Another site cannot submit the change through the reader's browser.
    let foreign = client
        .post(&private)
        .header("Origin", "http://evil.invalid")
        .json(&change(&view["revision"]))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(
        std::fs::read_to_string(&config)
            .unwrap()
            .contains("all_history = true")
    );
    let saved: serde_json::Value = client
        .post(&private)
        .json(&change(&view["revision"]))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(saved["since_days"], 30);
    assert_eq!(saved["memory_capture"], false);
    // Unknown fields, such as a path from the browser, are not accepted.
    let mut extra = change(&saved["revision"]);
    extra["roots"] = serde_json::json!(["/etc"]);
    let rejected = client.post(&private).json(&extra).send().await.unwrap();
    assert_eq!(rejected.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_open_network_dashboard_never_joins_or_manages_machines() {
    // Reachable by its network and signed in to by nobody: it may read the
    // shared server's state, but joining, saving an admin key and changing
    // machines are refused before any request leaves this machine.
    let (_directory, paths, _) = record_fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = super::router(std::sync::Arc::new(super::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts: allowed_hosts(address),
        loopback: false,
        access: super::access::Access::new(None),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(0)),
        services: super::services::Services::load(None).unwrap(),
        live: super::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::new();
    let url = |path: &str| format!("http://{address}{path}");
    let key = format!("mash-admin.{}", "a".repeat(64));
    for (path, body) in [
        (
            "/api/shared/join",
            serde_json::json!({"join": "mash-join.x", "apply": true}),
        ),
        ("/api/shared/admin/key", serde_json::json!({"key": key})),
        (
            "/api/shared/admin/change",
            serde_json::json!({"change": "add", "node_id": "desk"}),
        ),
    ] {
        let response = client.post(url(path)).json(&body).send().await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN, "{path}");
        let text = response.text().await.unwrap();
        assert!(!text.contains(&key), "{path} repeats no key");
    }
    let response = client.get(url("/api/shared/admin")).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    let view: serde_json::Value = client
        .get(url("/api/shared/server"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["may_change"], false);
    server.abort();
}
