use super::*;

/// Every command a person or agent runs is one row in the usage log beside
/// the index, with the arguments Clap parsed; `mash usage off` stops that and
/// `mash usage clear --yes` empties it.
fn calls(database: &Path) -> Vec<(String, Value, String)> {
    let log = database.with_file_name("usage.sqlite3");
    if !log.exists() {
        return Vec::new();
    }
    let connection = Connection::open(log).unwrap();
    let mut statement = connection
        .prepare("SELECT tool, arguments, surface FROM calls ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| {
            let arguments: String = row.get(1)?;
            Ok((
                row.get(0)?,
                serde_json::from_str(&arguments).unwrap(),
                row.get(2)?,
            ))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn mash(home: &Path, arguments: &[&str]) -> std::process::Output {
    Command::cargo_bin("mash")
        .unwrap()
        .args(arguments)
        .env("HOME", home)
        .env("MASH_CLIENT", "usage-test")
        .env_remove("MASH_USAGE")
        .env_remove("MASH_TRACE")
        .output()
        .unwrap()
}

#[test]
fn each_command_is_recorded_once_and_the_log_can_be_turned_off_and_cleared() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("index.sqlite3");
    let db = database.to_str().unwrap();
    let config = directory.path().join("missing.toml");
    let config = config.to_str().unwrap();

    mash(
        directory.path(),
        &["--db", db, "--config", config, "list-chats", "--limit", "3"],
    );
    let recorded = calls(&database);
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    let (tool, arguments, surface) = &recorded[0];
    assert_eq!(tool, "list-chats");
    assert_eq!(surface, "cli");
    assert_eq!(arguments["limit"], "3");
    assert_eq!(arguments["db"], db);
    // Only flags given, never the group Clap makes for a flattened set of
    // them, which is named for its Rust struct (here `Request`).
    let mut keys: Vec<&str> = arguments
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["config", "db", "limit"], "{arguments}");

    // `mash usage` itself is not a call to record.
    let off = mash(directory.path(), &["--db", db, "usage", "off"]);
    assert!(off.status.success(), "{off:?}");
    let answer: Value = serde_json::from_slice(&off.stdout).unwrap();
    assert_eq!(answer["recording"], false);
    mash(
        directory.path(),
        &["--db", db, "--config", config, "list-chats"],
    );
    assert_eq!(calls(&database).len(), 1, "nothing is recorded while off");

    // Clearing needs a confirmation it cannot ask for without a terminal.
    let refused = mash(directory.path(), &["--db", db, "usage", "clear"]);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--yes"));
    assert_eq!(calls(&database).len(), 1);

    let cleared = mash(directory.path(), &["--db", db, "usage", "clear", "--yes"]);
    assert!(cleared.status.success(), "{cleared:?}");
    let answer: Value = serde_json::from_slice(&cleared.stdout).unwrap();
    assert_eq!(answer["deleted"], 1);
    assert_eq!(calls(&database).len(), 0, "nothing is recorded while off");

    mash(directory.path(), &["--db", db, "usage", "on"]);
    mash(
        directory.path(),
        &["--db", db, "--config", config, "list-chats"],
    );
    assert_eq!(calls(&database).len(), 1, "recording resumes");
}

#[test]
fn mash_usage_0_records_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("index.sqlite3");
    let config = directory.path().join("missing.toml");
    Command::cargo_bin("mash")
        .unwrap()
        .args([
            "--db",
            database.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "list-chats",
        ])
        .env("HOME", directory.path())
        .env("MASH_USAGE", "0")
        .output()
        .unwrap();
    assert!(!database.with_file_name("usage.sqlite3").exists());
}
