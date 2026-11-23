use std::time::{Duration, Instant};

use super::writer::Queue;
use super::*;

fn call(tool: &str, arguments: serde_json::Value) -> Call {
    Started::now().call(Surface::Cli, tool, arguments)
}

fn rows(path: &Path) -> Vec<(String, String, String, Option<String>)> {
    let connection = rusqlite::Connection::open(path).unwrap();
    let mut statement = connection
        .prepare("SELECT tool, arguments, outcome, error_kind FROM calls ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn the_log_is_created_at_version_one_and_reopens() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    drop(store::open(&path).unwrap());
    let connection = store::open(&path).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
    let journal: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    assert_eq!(journal, "wal");
}

#[test]
fn a_newer_log_is_refused_rather_than_rewritten() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    rusqlite::Connection::open(&path)
        .unwrap()
        .pragma_update(None, "user_version", 7)
        .unwrap();
    let error = store::open(&path).unwrap_err();
    assert!(
        error.error.message.contains("newer"),
        "{}",
        error.error.message
    );
}

#[cfg(unix)]
#[test]
fn the_log_is_private_to_its_owner() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    drop(store::open(&path).unwrap());
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn calls_are_stored_with_their_arguments_and_errors() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut connection = store::open(&path).unwrap();
    let ok = call(
        "search",
        serde_json::json!({"query": "invoice exports", "limit": "5"}),
    );
    let mut failed = call("view", serde_json::json!({"chat_id": "abc"}));
    failed.fail(&AppError::usage("no such chat"));
    store::insert(&mut connection, &[ok, failed], 3).unwrap();
    let stored = rows(&path);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].0, "search");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored[0].1).unwrap(),
        serde_json::json!({"query": "invoice exports", "limit": "5"})
    );
    assert_eq!(stored[0].2, "ok");
    assert_eq!(stored[1].2, "error");
    assert_eq!(stored[1].3.as_deref(), Some("usage"));
    let dropped: i64 = connection
        .query_row(
            "SELECT value FROM counters WHERE key = 'dropped'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(dropped, 3);
}

#[test]
fn a_log_turned_off_stores_nothing_and_clearing_keeps_the_setting() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("index.sqlite3");
    let path = path_for(&database);
    let mut connection = store::open(&path).unwrap();
    store::insert(&mut connection, &[call("search", serde_json::json!({}))], 0).unwrap();
    assert!(enabled(&database).unwrap());
    set_enabled(&database, false).unwrap();
    assert!(!enabled(&database).unwrap());
    store::insert(&mut connection, &[call("view", serde_json::json!({}))], 0).unwrap();
    assert_eq!(rows(&path).len(), 1, "nothing is stored while off");
    assert_eq!(clear(&database).unwrap(), 1);
    assert_eq!(rows(&path).len(), 0, "clearing deletes every row");
    assert!(!enabled(&database).unwrap(), "clearing keeps the setting");
    // Nothing recorded yet means recording is on and there is nothing to clear.
    let fresh = directory.path().join("other").join("index.sqlite3");
    assert!(enabled(&fresh).unwrap());
    assert_eq!(clear(&fresh).unwrap(), 0);
    assert!(!path_for(&fresh).exists(), "reading does not create a log");
}

#[test]
fn a_stalled_writer_drops_and_counts_instead_of_blocking() {
    let (queue, _receiver) = Queue::stalled(2);
    let started = Instant::now();
    let mut queued = 0;
    for index in 0..100 {
        let row = call(&format!("tool-{index}"), serde_json::json!({}));
        if queue.offer(PathBuf::from(FILE_NAME), row) {
            queued += 1;
        }
    }
    assert_eq!(queued, 2, "the channel holds only its capacity");
    assert_eq!(queue.dropped(), 98);
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "offering never waits"
    );
    assert!(
        !queue.flush(Duration::from_millis(50)),
        "a stalled writer cannot flush"
    );
}

#[test]
fn recorded_calls_reach_the_log_once_flushed() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("index.sqlite3");
    record(
        &database,
        call("list-chats", serde_json::json!({"limit": "3"})),
    );
    record(
        &database,
        call("search", serde_json::json!({"query": "qdrant lock"})),
    );
    assert!(flush(Duration::from_secs(5)));
    let stored = rows(&path_for(&database));
    let tools: Vec<&str> = stored.iter().map(|row| row.0.as_str()).collect();
    assert_eq!(tools, ["list-chats", "search"]);
}

#[test]
fn credentials_in_arguments_are_masked() {
    let mut row = call(
        "init",
        serde_json::json!({
            "join": "mash-join.eyJzZWNyZXQiOiJ4In0",
            "note": "key mash-admin.abc123 here",
            "query": "invoice exports",
        }),
    );
    scrub(&mut row);
    let text = row.arguments.to_string();
    assert!(!text.contains("eyJzZWNyZXQ"), "{text}");
    assert!(!text.contains("abc123"), "{text}");
    assert!(text.contains("invoice exports"), "{text}");
    assert_eq!(row.arguments["note"], "key [REDACTED] here");
}

#[test]
fn a_log_that_cannot_be_created_never_fails_the_caller() {
    // A log under a regular file cannot be created; recording still returns.
    let directory = tempfile::tempdir().unwrap();
    let blocker = directory.path().join("file");
    std::fs::write(&blocker, "not a directory").unwrap();
    record(
        &blocker.join("index.sqlite3"),
        call("search", serde_json::json!({})),
    );
    assert!(flush(Duration::from_secs(5)));
}

#[test]
fn recording_never_creates_the_data_directory() {
    // A call made before Mash has a data directory, or after uninstalling
    // removed it, leaves nothing behind.
    let directory = tempfile::tempdir().unwrap();
    let data = directory.path().join(".mash");
    record(
        &data.join("index.sqlite3"),
        call("status", serde_json::json!({})),
    );
    assert!(flush(Duration::from_secs(5)));
    assert!(!data.exists());
}

/// The example scripts in `tools/usage-log/`, which `docs/cli.md` shows, run
/// against a log Mash wrote and answer their questions; the page's copies are
/// the scripts themselves, so neither can drift.
#[test]
fn the_documented_example_scripts_answer_their_questions() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let examples = repository.join("tools/usage-log");
    let page = std::fs::read_to_string(repository.join("docs/cli.md")).unwrap();
    for script in ["memory_kinds.py", "search_values.py"] {
        let source = std::fs::read_to_string(examples.join(script)).unwrap();
        assert!(
            page.contains(&format!("```python title=\"{script}\"\n{source}```")),
            "docs/cli.md must show tools/usage-log/{script} exactly"
        );
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut connection = store::open(&path).unwrap();
    let record = |id: &str, kind: &str| {
        serde_json::json!({"id": format!("record/o/{id}"), "current_revision": 1,
            "revisions": [{"revision": 1, "body": {"details": {"kind": kind}}}]})
    };
    let mut recall = Started::now().call(Surface::Mcp, "recall_records", serde_json::json!({}));
    recall.at_ms = 1_795_000_000_000; // 2026-11-18
    recall.response = Some(Returned::Json(serde_json::json!({"records": [
        record("1", "decision"), record("2", "decision"), record("3", "preference")
    ]})));
    let mut search = Started::now().call(
        Surface::Mcp,
        "search",
        serde_json::json!({"query": "qdrant lock", "provider": ["codex"]}),
    );
    search.at_ms = 1_795_000_000_000;
    search.response = Some(Returned::Text(
        r#"{"results":[{"chat_id":"a"},{"chat_id":"b"}]}"#.to_owned(),
    ));
    let mut typed = Started::now().call(
        Surface::Dashboard,
        "/api/search",
        serde_json::json!({"q": "qdrant lock"}),
    );
    typed.at_ms = 1_795_000_000_000;
    store::insert(&mut connection, &[recall, search, typed], 0).unwrap();
    drop(connection);

    let run = |script: &str| -> Option<String> {
        let output = std::process::Command::new("python3")
            .arg("-I")
            .arg(examples.join(script))
            .arg(&path)
            .output()
            .ok()?;
        assert!(
            output.status.success(),
            "{script}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Some(String::from_utf8(output.stdout).unwrap())
    };
    let Some(kinds) = run("memory_kinds.py") else {
        eprintln!("python3 is not installed; skipping the example scripts");
        return;
    };
    assert!(
        kinds.contains("2026-11  recall_records       decision    2"),
        "{kinds}"
    );
    assert!(
        kinds.contains("2026-11  recall_records       preference  1"),
        "{kinds}"
    );
    let searches = run("search_values.py").unwrap();
    assert!(searches.contains("2026-11-18  dashboard  1"), "{searches}");
    assert!(searches.contains("2026-11-18  mcp        1"), "{searches}");
    assert!(searches.contains("   2  qdrant lock"), "{searches}");
    assert!(searches.contains("   1  provider"), "{searches}");
}

#[test]
fn readers_query_the_views_while_the_writer_holds_the_log_open() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut writer = store::open(&path).unwrap();
    let reader = rusqlite::Connection::open(&path).unwrap();
    let mut reading = reader.prepare("SELECT COUNT(*) FROM calls").unwrap();
    let before: i64 = reading.query_row([], |row| row.get(0)).unwrap();
    // A reader with an open statement does not stop the writer, in WAL mode.
    store::insert(&mut writer, &[call("search", serde_json::json!({}))], 0).unwrap();
    let after: i64 = reading.query_row([], |row| row.get(0)).unwrap();
    assert_eq!((before, after), (0, 1));
    let columns: Vec<String> = reader
        .prepare("SELECT * FROM calls")
        .unwrap()
        .column_names()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        columns,
        [
            "id",
            "at",
            "at_ms",
            "surface",
            "client",
            "client_version",
            "tool",
            "arguments",
            "outcome",
            "error_kind",
            "error_message",
            "duration_ms",
            "result_count",
            "response_bytes",
            "results_total",
            "results_truncated",
            "session",
            "pid",
            "build",
            "project",
        ],
        "the documented columns of `calls`"
    );
    let results: Vec<String> = reader
        .prepare("SELECT * FROM call_results")
        .unwrap()
        .column_names()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        results,
        ["call_id", "position", "kind", "record_kind", "id", "node"]
    );
}
