//! The status route a machine reads with its own token, and the admin routes
//! that need an admin key. The failure modes each test pins down:
//!
//! - a machine's token used for management (refused, and changes nothing);
//! - a wrong, malformed or revoked admin key (refused; a rotated key stops
//!   working at once);
//! - guessing admin keys (the server refuses every admin request for a
//!   minute after too many wrong ones, even a right key: it fails closed);
//! - a machine reading another group's machines, or any secret, in status;
//! - deleting history without the name typed again, or before removal;
//! - a removed machine still publishing or searching with its old token;
//! - a join string replayed after it was redeemed.
use std::collections::HashMap;
use std::sync::Arc;

use mash_app::shared::server::{admin, enrollment};
use reqwest::StatusCode;
use serde_json::{Value, json};

use super::super::{App, Store, roster, router};
use crate::shared::server::Identity;

const FIRST: &str = "first-token-0123456789abcdef0123456789";
const OTHER: &str = "other-token-0123456789abcdef0123456789";

struct Service {
    url: String,
    admin_key: String,
    configuration: std::path::PathBuf,
    _directory: tempfile::TempDir,
    task: tokio::task::JoinHandle<()>,
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn identity(group: &str, node: &str) -> Identity {
    Identity {
        group_id: group.into(),
        node_id: node.into(),
        generation: "test".into(),
        dimensions: 2,
    }
}

/// A server with `team/first` and `other/first` in its own configuration,
/// an admin key named `ops`, and `team/laptop` waiting to join.
async fn service() -> Service {
    super::super::admin::reset_failures();
    let directory = tempfile::tempdir().unwrap();
    let configuration = directory.path().join("server.toml");
    let silver = directory.path().join("silver");
    std::fs::write(
        &configuration,
        format!(
            "version = 1\nsilver_dir = {silver:?}\n[[nodes]]\ngroup_id = 'team'\nnode_id = 'first'\ntoken_hash = '{}'\n[[nodes]]\ngroup_id = 'other'\nnode_id = 'first'\ntoken_hash = '{}'\n",
            blake3::hash(FIRST.as_bytes()).to_hex(),
            blake3::hash(OTHER.as_bytes()).to_hex(),
        ),
    )
    .unwrap();
    let admin_key = enrollment::update(&configuration, |added| {
        added.api_url = Some("http://127.0.0.1:9".into());
        enrollment::add(added, &[], "team", "laptop", enrollment::now())?;
        admin::create(added, "ops", enrollment::now())
    })
    .unwrap();
    let members = [
        identity("team", "first"),
        identity("other", "first"),
        identity("team", "laptop"),
    ];
    let store = Store::new(
        crate::shared::silver_store::SilverStore::open(&silver, members.iter()).unwrap(),
        crate::views::qdrant::Store::unavailable(),
        64 * 1024 * 1024,
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let live = roster::Live::new(configuration.clone(), address.to_string());
    let app = App {
        activity: crate::shared::activity::Registry::default(),
        mutation_capacity: 1,
        attachments_dir: None,
        catalog: Arc::new(std::sync::Mutex::new(
            super::super::sync::open_catalog(&directory.path().join("sync")).unwrap(),
        )),
        store,
        nodes: roster::Nodes::fixed(HashMap::new()),
        live: Some(live.clone()),
        joins: Arc::new(tokio::sync::Semaphore::new(super::super::join::CONCURRENT)),
        uploads: Arc::new(tokio::sync::Semaphore::new(1)),
        presence: super::super::presence::Presence::default(),
        waiters: Arc::new(tokio::sync::Semaphore::new(
            super::super::admission::MAX_WAITERS,
        )),
    };
    roster::refresh(&app, &live, true).await.unwrap();
    // The file is followed as a running server follows it.
    let watch = roster::spawn_watch(app.clone(), live);
    let task = tokio::spawn(async move {
        let _watch = AbortOnDrop(watch);
        axum::serve(listener, router(app)).await.unwrap();
    });
    Service {
        url: format!("http://{address}"),
        admin_key,
        configuration,
        _directory: directory,
        task,
    }
}

impl Service {
    async fn call(
        &self,
        method: reqwest::Method,
        route: &str,
        key: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = reqwest::Client::new()
            .request(method, format!("{}/v1/{route}", self.url))
            .bearer_auth(key)
            .header("x-mash-from", "laptop");
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.unwrap();
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }
    async fn admin(&self, route: &str, key: &str, body: Value) -> (StatusCode, Value) {
        self.call(
            reqwest::Method::POST,
            &format!("admin/{route}"),
            key,
            Some(body),
        )
        .await
    }
}

#[tokio::test]
async fn a_machine_reads_its_own_group_and_no_secret() {
    let service = service().await;
    let (status, body) = service
        .call(reqwest::Method::GET, "status", FIRST, None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["group_id"], "team");
    let names: Vec<_> = body["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|machine| {
            (
                machine["node_id"].as_str().unwrap(),
                machine["state"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(names, [("first", "configured"), ("laptop", "waiting")]);
    // Neither published, so neither has a search state to report.
    assert!(
        body["machines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|machine| machine.get("search").is_none())
    );
    let text = body.to_string();
    assert!(
        !text.contains("other"),
        "another group is never named: {text}"
    );
    let file = std::fs::read_to_string(enrollment::nodes_path(&service.configuration)).unwrap();
    for line in file.lines().filter(|line| line.contains("hash")) {
        let hash = line.split('"').nth(1).unwrap();
        assert!(!text.contains(hash), "status holds no stored hash");
    }
    assert!(!text.contains(&service.admin_key));
}

#[tokio::test]
async fn a_machine_checking_in_is_seen_by_its_group_only() {
    let service = service().await;
    let seen = |body: &Value, node: &str| {
        body["machines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|machine| machine["node_id"] == node)
            .and_then(|machine| machine.get("last_seen_at"))
            .and_then(Value::as_u64)
    };
    let (_, before) = service
        .call(reqwest::Method::GET, "status", FIRST, None)
        .await;
    assert_eq!(
        seen(&before, "first"),
        None,
        "nothing is seen before a check-in"
    );
    let (status, beat) = service
        .call(
            reqwest::Method::POST,
            "heartbeat",
            FIRST,
            Some(serde_json::json!({ "os": "linux" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{beat}");
    assert_eq!(beat["every_seconds"], 60);
    let (_, after) = service
        .call(reqwest::Method::GET, "status", FIRST, None)
        .await;
    assert_eq!(seen(&after, "first"), beat["now"].as_u64());
    let first = after["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|machine| machine["node_id"] == "first")
        .unwrap();
    assert_eq!(first["os"], "linux", "the system it said it runs");
    // A machine of the same name in another group is a different machine.
    let (_, other) = service
        .call(reqwest::Method::GET, "status", OTHER, None)
        .await;
    assert_eq!(seen(&other, "first"), None);
    assert_eq!(
        seen(&after, "laptop"),
        None,
        "a machine that never checked in"
    );
    // Checking in takes the machine's own token.
    let (refused, _) = service
        .call(
            reqwest::Method::POST,
            "heartbeat",
            "not-a-token",
            Some(serde_json::json!({})),
        )
        .await;
    assert_eq!(refused, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_machine_token_never_manages_machines() {
    let service = service().await;
    for (route, body) in [
        ("add", json!({"group_id": "team", "node_id": "desk"})),
        ("remove", json!({"group_id": "team", "node_id": "laptop"})),
        ("replace", json!({"group_id": "team", "node_id": "laptop"})),
        (
            "delete",
            json!({"group_id": "team", "node_id": "laptop", "confirm": "laptop"}),
        ),
    ] {
        let (status, answer) = service.admin(route, FIRST, body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{route}: {answer}");
        assert_eq!(answer["error"]["kind"], "not-admin");
    }
    let (status, _) = service
        .call(reqwest::Method::GET, "admin/machines", FIRST, None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let added = enrollment::read(&service.configuration).unwrap();
    assert_eq!(added.nodes.len(), 1, "nothing changed");
    assert_eq!(added.audit.len(), 0);
    // And an admin key is not a machine's token.
    let (status, _) = service
        .call(reqwest::Method::GET, "status", &service.admin_key, None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_admin_adds_replaces_removes_and_deletes_with_a_record() {
    let service = service().await;
    let key = service.admin_key.clone();
    let (status, added) = service
        .admin("add", &key, json!({"group_id": "team", "node_id": "desk"}))
        .await;
    assert_eq!(status, StatusCode::OK, "{added}");
    let join = added["join"].as_str().unwrap();
    assert!(join.starts_with("mash-join."));
    assert_eq!(added["expires_in_seconds"], enrollment::JOIN_LIFETIME);
    // The name is taken now.
    let (status, _) = service
        .admin("add", &key, json!({"group_id": "team", "node_id": "desk"}))
        .await;
    assert!(status.is_client_error());
    let (status, replaced) = service
        .admin(
            "replace",
            &key,
            json!({"group_id": "team", "node_id": "desk"}),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(replaced["join"], added["join"], "a new string");
    // Deleting needs removal first, and the name typed again.
    let (status, _) = service
        .admin(
            "delete",
            &key,
            json!({"group_id": "team", "node_id": "desk", "confirm": "desk"}),
        )
        .await;
    assert!(status.is_client_error());
    let (status, _) = service
        .admin(
            "remove",
            &key,
            json!({"group_id": "team", "node_id": "desk"}),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, answer) = service
        .admin(
            "delete",
            &key,
            json!({"group_id": "team", "node_id": "desk", "confirm": "Desk"}),
        )
        .await;
    assert!(status.is_client_error(), "{answer}");
    let (status, answer) = service
        .admin(
            "delete",
            &key,
            json!({"group_id": "team", "node_id": "desk", "confirm": "desk"}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let (status, overview) = service
        .call(reqwest::Method::GET, "admin/machines", &key, None)
        .await;
    assert_eq!(status, StatusCode::OK, "{overview}");
    let actions: Vec<_> = overview["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|action| {
            (
                action["action"].as_str().unwrap(),
                action["key"].as_str().unwrap(),
                action["from"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        actions,
        [
            ("delete", "ops", "laptop"),
            ("remove", "ops", "laptop"),
            ("replace", "ops", "laptop"),
            ("add", "ops", "laptop")
        ]
    );
    let text = overview.to_string();
    assert!(
        !text.contains(&key) && !text.contains(join),
        "no secret comes back"
    );
}

#[tokio::test]
async fn wrong_keys_are_refused_and_too_many_shut_the_gate() {
    let service = service().await;
    let rotated = enrollment::update(&service.configuration, |added| {
        admin::create(added, "ops", enrollment::now())
    })
    .unwrap();
    // The running server takes the rotation in within its next refresh.
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    let wrong = format!("mash-admin.{}", "0".repeat(64));
    for key in [
        service.admin_key.as_str(),
        wrong.as_str(),
        "mash-admin.short",
        "",
    ] {
        let (status, _) = service
            .call(reqwest::Method::GET, "admin/machines", key, None)
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{key}");
    }
    let (status, _) = service
        .call(reqwest::Method::GET, "admin/machines", &rotated, None)
        .await;
    assert_eq!(status, StatusCode::OK, "the new key works");
    for _ in 0..super::super::admin::MAX_FAILURES {
        service
            .call(reqwest::Method::GET, "admin/machines", &wrong, None)
            .await;
    }
    let (status, _) = service
        .call(reqwest::Method::GET, "admin/machines", &rotated, None)
        .await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "fails closed, even for a right key"
    );
    // Machines keep publishing and searching meanwhile.
    let (status, _) = service
        .call(reqwest::Method::GET, "status", FIRST, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    super::super::admin::reset_failures();
}

#[tokio::test]
async fn a_removed_machine_is_told_and_a_redeemed_string_cannot_be_replayed() {
    let service = service().await;
    let key = service.admin_key.clone();
    let (_, added) = service
        .admin("add", &key, json!({"group_id": "team", "node_id": "desk"}))
        .await;
    let join = mash_app::shared::join::Join::decode(added["join"].as_str().unwrap()).unwrap();
    let mash_app::shared::join::Credential::Secret(secret) = join.credential else {
        panic!("a current string carries a secret");
    };
    let token = enrollment::secret().unwrap();
    let redeem = json!({"secret": secret, "token_hash": blake3::hash(token.as_bytes()).to_hex().to_string()});
    let (status, answer) = service
        .call(reqwest::Method::POST, "join", "", Some(redeem.clone()))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(answer["status"], "ready");
    let (status, _) = service
        .call(reqwest::Method::GET, "status", &token, None)
        .await;
    assert_eq!(status, StatusCode::OK, "the joined machine reads its group");
    // Someone else replaying the string gets nothing.
    let stolen = enrollment::secret().unwrap();
    let replay = json!({"secret": secret, "token_hash": blake3::hash(stolen.as_bytes()).to_hex().to_string()});
    let (_, answer) = service
        .call(reqwest::Method::POST, "join", "", Some(replay))
        .await;
    assert_eq!(answer["status"], "used");
    let (status, _) = service
        .call(reqwest::Method::GET, "status", &stolen, None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // Removed, the machine is told why, and reads nothing more.
    service
        .admin(
            "remove",
            &key,
            json!({"group_id": "team", "node_id": "desk"}),
        )
        .await;
    let (status, answer) = service
        .call(reqwest::Method::GET, "status", &token, None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(answer["error"]["kind"], "credential-revoked");
}
