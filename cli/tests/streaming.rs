use super::*;

/// `mash search --stream` prints one line of JSON per update: the opening
/// update with every source's state, then one after each source answers.
/// Sources that are out of scope or not set up say so instead of failing.
/// A word-search index of three chats, two of which mention "invoice exports".
fn indexed() -> (tempfile::TempDir, Vec<String>) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("claude");
    std::fs::create_dir(&root).unwrap();
    for (session, text) in [
        ("first", "invoice exports time out for large accounts"),
        ("second", "a queue for invoice exports in postgres"),
        ("third", "nothing to do with the question"),
    ] {
        std::fs::write(
            root.join(format!("{session}.jsonl")),
            serde_json::to_vec(&serde_json::json!({
                "type": "user", "sessionId": session, "uuid": format!("{session}-message"),
                "message": {"role": "user", "content": text}
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let config = directory.path().join("config.toml");
    std::fs::write(
        &config,
        format!(
            "version=1\n[providers.claude-code]\nroots=[{}]\n[views.build]\nrepresentations='lexical-only'\n[views.query]\nintent='exact-mention'\n",
            toml::Value::String(root.to_str().unwrap().into())
        ),
    )
    .unwrap();
    configure_qdrant(&config);
    let path = |name: &str| directory.path().join(name).to_str().unwrap().to_owned();
    let base = vec![
        "--config".to_owned(),
        path("config.toml"),
        "--db".to_owned(),
        path("index.sqlite3"),
        "--models-dir".to_owned(),
        path("missing-models"),
    ];
    let mut index: Vec<&str> = base.iter().map(String::as_str).collect();
    index.push("index");
    run_json(&index);
    (directory, base)
}

#[test]
fn stream_prints_one_update_per_source_and_merges_into_one_ranking() {
    let (directory, base) = indexed();
    let base: Vec<&str> = base.iter().map(String::as_str).collect();
    let database = directory.path().join("index.sqlite3");
    let lines = |extra: &[&str]| -> Vec<Value> {
        let output = Command::cargo_bin("mash")
            .unwrap()
            .args(
                [
                    base.as_slice(),
                    &["search", "invoice exports", "--stream"],
                    extra,
                ]
                .concat(),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).expect("each line is one JSON update"))
            .collect()
    };
    for extra in [&["--live"][..], &[][..]] {
        let updates = lines(extra);
        // Opening update, then words: meaning and other machines are skipped here.
        assert_eq!(updates.len(), 2, "{updates:?}");
        let opening = &updates[0];
        assert_eq!(opening["done"], false);
        assert_eq!(opening["results"], serde_json::json!([]));
        let sources = &opening["sources"];
        assert_eq!(sources[0]["source"], "words");
        assert_eq!(sources[0]["state"], "searching");
        assert_eq!(sources[1]["state"], "skipped");
        assert!(
            sources[1]["reason"]
                .as_str()
                .unwrap()
                .contains("isn't set up")
        );
        assert!(
            sources[2]["reason"]
                .as_str()
                .unwrap()
                .contains("hasn't joined")
        );
        let last = updates.last().unwrap();
        assert_eq!(last["done"], true);
        assert_eq!(last["sources"][0]["state"], "done");
        let chats: Vec<&str> = last["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["chat_id"].as_str().unwrap())
            .collect();
        assert_eq!(chats.len(), 2, "{chats:?}");
        assert!(!chats.contains(&"third"));
        for row in last["results"].as_array().unwrap() {
            assert_eq!(row["matched_by"], serde_json::json!(["words"]));
        }
    }
    // This machine only never reaches for other machines.
    let local = lines(&["--scope", "local"]);
    assert!(
        local[0]["sources"][2]["reason"]
            .as_str()
            .unwrap()
            .contains("only this machine")
    );
    // Each streamed search is in the usage log with how many results its
    // last update listed, as Raycast's searches, which all stream, need.
    let usage = rusqlite::Connection::open(directory.path().join("usage.sqlite3")).unwrap();
    let counts: Vec<Option<i64>> = usage
        .prepare("SELECT result_count FROM calls WHERE tool = 'search' ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get::<_, Option<i64>>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(counts.len(), 3, "{counts:?}");
    assert!(counts.iter().all(Option::is_some), "{counts:?}");
    assert_eq!(counts[0], Some(2));
    cleanup_local_gold(&database, "local");
}

#[test]
fn stream_rejects_an_unknown_scope_and_needs_stream_for_scope() {
    Command::cargo_bin("mash")
        .unwrap()
        .args(["search", "x", "--stream", "--scope", "everywhere"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("local, all or node:"));
    Command::cargo_bin("mash")
        .unwrap()
        .args(["search", "x", "--scope", "local"])
        .assert()
        .failure();
}
