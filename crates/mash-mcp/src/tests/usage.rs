use super::*;
use rusqlite::Connection;

/// One recorded call: tool, arguments, surface, client, client version, session.
type Row = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// Every tool call is one row in the usage log beside the index: the tool's
/// name, the arguments the client sent, the client's name and version from
/// its handshake, this connection's session and how the call ended.
#[tokio::test]
async fn each_tool_call_is_recorded_once_with_its_client_and_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    std::fs::write(&config, "version = 1\n[providers]\n").unwrap();
    let database = directory.path().join("index.sqlite3");
    let server = Mash {
        runtime: None,
        paths: Arc::new(Paths {
            config: Some(config),
            db: Some(database.clone()),
            models_dir: Some(directory.path().join("models")),
        }),
        gate: Arc::new(tokio::sync::Mutex::new(())),
        model_work: Arc::new(tokio::sync::Semaphore::new(1)),
    };
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let serving = tokio::spawn(async move { server.serve(server_io).await.unwrap() });
    let client = ().serve(client_io).await.unwrap();
    let server = serving.await.unwrap();
    let help = client
        .call_tool(
            CallToolRequestParams::new("help")
                .with_arguments(json!({"topic": "discovery"}).as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    assert_ne!(help.is_error, Some(true), "{help:?}");
    let missing = client
        .call_tool(
            CallToolRequestParams::new("view").with_arguments(
                json!({"chat_id": "no-such-chat"})
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await;
    drop(missing);
    client.cancel().await.unwrap();
    drop(server);
    assert!(mash_app::usage::flush(std::time::Duration::from_secs(5)));

    let log = Connection::open(database.with_file_name("usage.sqlite3")).unwrap();
    let rows: Vec<Row> = log
        .prepare("SELECT tool, arguments, surface, client, client_version, session FROM calls ORDER BY id")
        .unwrap()
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(rows.len(), 2, "{rows:?}");
    let (tool, arguments, surface, client, version, session) = &rows[0];
    assert_eq!(tool, "help");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(arguments).unwrap(),
        json!({"topic": "discovery"})
    );
    assert_eq!(surface, "mcp");
    assert!(
        client.is_some() && version.is_some(),
        "{client:?} {version:?}"
    );
    assert!(
        session
            .as_deref()
            .is_some_and(|session| session.starts_with("mcp-"))
    );
    assert_eq!(rows[1].0, "view");
    assert_eq!(rows[0].5, rows[1].5, "one connection, one session");
}
