use crate::chat_catalog::Request;
use crate::silver::subagent::Subagent;
use crate::storage::Storage;
use crate::storage::test_support::chat;
use rusqlite::Connection;

fn codex_chat(id: &str, text: &str, subagent: Option<Subagent>) -> crate::silver::Chat {
    let mut value = chat(text);
    value.id = id.to_owned();
    value.title = Some(format!("{id} title"));
    value.source = crate::silver::SourceIdentity::file(format!("/tmp/{id}.jsonl"));
    value.messages = vec![crate::silver::NormalizedMessage {
        id: format!("{id}-message"),
        ..value.messages.inline()[0].clone()
    }]
    .into();
    value.origin.subagent = subagent;
    value
}

fn spawned(parent: &str) -> Subagent {
    Subagent {
        parent_chat_id: Some(parent.to_owned()),
        label: Some("Locke".into()),
        role: Some("worker".into()),
        depth: Some(1),
        finished: Some(true),
        spawn_call_id: None,
    }
}

fn catalogue(storage: &Storage) -> Vec<crate::chat_catalog::Chat> {
    storage
        .chat_candidates(&Request::default(), "local")
        .unwrap()
}

#[test]
fn subagents_are_listed_with_their_parent_and_parents_count_them() {
    let directory = tempfile::tempdir().unwrap();
    let mut storage = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    storage
        .replace_chat(&codex_chat("parent", "plan", None))
        .unwrap();
    storage
        .replace_chat(&codex_chat("child", "work", Some(spawned("parent"))))
        .unwrap();
    storage
        .replace_chat(&codex_chat("orphan", "review", Some(spawned("elsewhere"))))
        .unwrap();
    let chats = catalogue(&storage);
    let find = |id: &str| chats.iter().find(|chat| chat.chat_id == id).unwrap();
    assert_eq!(find("parent").subagents, 1);
    assert!(find("parent").metadata.subagent.is_none());
    let child = find("child").metadata.subagent.clone().unwrap();
    assert_eq!(child.parent_chat_id.as_deref(), Some("parent"));
    assert_eq!(child.parent_title.as_deref(), Some("parent title"));
    assert_eq!(
        (child.label.as_deref(), child.finished),
        (Some("Locke"), Some(true))
    );
    // A parent this index does not hold keeps its link, without a title.
    let orphan = find("orphan").metadata.subagent.clone().unwrap();
    assert_eq!(orphan.parent_chat_id.as_deref(), Some("elsewhere"));
    assert!(orphan.parent_title.is_none());
    assert_eq!(find("orphan").subagents, 0);

    // Search results take the same facts from silver.
    let facts = super::read(&storage.connection, "child").unwrap().unwrap();
    assert_eq!(facts, child);
    assert!(
        super::read(&storage.connection, "parent")
            .unwrap()
            .is_none()
    );

    // A reread without the marker clears it; removing the chat removes its row.
    storage
        .replace_chat(&codex_chat("child", "work", None))
        .unwrap();
    assert!(super::read(&storage.connection, "child").unwrap().is_none());
    storage
        .replace_chat(&codex_chat("child", "work", Some(spawned("parent"))))
        .unwrap();
    storage
        .connection
        .execute("DELETE FROM chats WHERE id='child'", [])
        .unwrap();
    let rows: i64 = storage
        .connection
        .query_row("SELECT count(*) FROM chat_subagents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 1, "only the orphan's row remains");
}

#[test]
fn a_list_can_leave_subagents_out_in_storage_and_when_pages_merge() {
    let directory = tempfile::tempdir().unwrap();
    let mut storage = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    storage
        .replace_chat(&codex_chat("parent", "plan", None))
        .unwrap();
    storage
        .replace_chat(&codex_chat("child", "work", Some(spawned("parent"))))
        .unwrap();
    let request = Request {
        exclude_subagents: true,
        ..Request::default()
    };
    let own = storage.chat_candidates(&request, "local").unwrap();
    assert_eq!(
        own.iter()
            .map(|chat| chat.chat_id.as_str())
            .collect::<Vec<_>>(),
        ["parent"]
    );
    // Rows merged from other copies, such as a shared server that does not
    // filter, pass the same rule.
    let mut collector = crate::chat_catalog::Collector::new(request).unwrap();
    for chat in catalogue(&storage) {
        collector.push(chat);
    }
    let merged = collector.finish().chats;
    assert_eq!(
        merged
            .iter()
            .map(|chat| chat.chat_id.as_str())
            .collect::<Vec<_>>(),
        ["parent"]
    );
    // A request that keeps subagents says nothing about them on the wire, so
    // older servers, which reject unknown fields, still accept it.
    let ordinary = serde_json::to_value(Request::default()).unwrap();
    assert!(ordinary.get("exclude_subagents").is_none());
}

#[test]
fn a_claude_contribution_without_the_marker_keeps_the_subagent_row() {
    let directory = tempfile::tempdir().unwrap();
    let mut storage = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    let mut child = codex_chat("agent", "work", Some(spawned("session")));
    child.provider = crate::provider::Provider::ClaudeCode;
    storage.replace_chat(&child).unwrap();
    child.origin.subagent = None;
    child.messages = vec![crate::silver::NormalizedMessage {
        content: "more".into(),
        ..child.messages.inline()[0].clone()
    }]
    .into();
    storage.replace_chat(&child).unwrap();
    assert!(super::read(&storage.connection, "agent").unwrap().is_some());
}

#[test]
fn a_chat_never_names_itself_as_its_parent() {
    let directory = tempfile::tempdir().unwrap();
    let mut storage = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    storage
        .replace_chat(&codex_chat("loop", "x", Some(spawned("loop"))))
        .unwrap();
    let row = super::read(&storage.connection, "loop").unwrap().unwrap();
    assert!(row.parent_chat_id.is_none());
}

#[test]
fn upgrading_creates_the_table_and_rereads_only_codex_sources() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("index.sqlite3");
    {
        let mut storage = Storage::open_writer(&path).unwrap();
        storage
            .replace_chat(&codex_chat("kept", "history", None))
            .unwrap();
        storage.commit_writer().unwrap();
    }
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "DROP TABLE chat_subagents; DROP TABLE chat_ssh_copies; PRAGMA user_version=67;
             INSERT INTO source_checkpoints VALUES
               ('codex','file:/a','/a',1,1,'f',3),('claude-code','file:/b','/b',1,1,'f',3),
               ('claude-code','file:/p/ssh-s/s.jsonl','/p/ssh-s/s.jsonl',1,1,'f',3);",
        )
        .unwrap();
    drop(connection);
    let storage = Storage::open(&path).unwrap();
    let versions: Vec<(String, i64)> = storage
        .connection
        .prepare(
            "SELECT provider, interpretation_version FROM source_checkpoints ORDER BY provider",
        )
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    // Sessions copied over SSH are read again; other Claude Code sources are not.
    assert_eq!(
        versions,
        [
            ("claude-code".into(), 3),
            ("claude-code".into(), 0),
            ("codex".into(), 0)
        ]
    );
    assert_eq!(catalogue(&storage).len(), 1);
    let user_version: i64 = storage
        .connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(user_version, crate::storage::schema::SCHEMA_VERSION);
}

#[test]
fn ssh_copies_are_marked_in_lists_and_search_until_read_from_elsewhere() {
    let directory = tempfile::tempdir().unwrap();
    let mut storage = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    let mut copy = codex_chat("session", "deploy", None);
    copy.provider = crate::provider::Provider::ClaudeCode;
    copy.origin.ssh_copy = true;
    storage.replace_chat(&copy).unwrap();
    assert!(catalogue(&storage)[0].metadata.ssh_copy);
    assert!(super::ssh_copy(&storage.connection, "session").unwrap());
    copy.origin.ssh_copy = false;
    storage.replace_chat(&copy).unwrap();
    assert!(!catalogue(&storage)[0].metadata.ssh_copy);
    assert!(!super::ssh_copy(&storage.connection, "session").unwrap());
}

/// The chats a delta after `after` journals, with whether each is resent whole.
fn journaled(storage: &Storage, after: i64) -> Vec<(String, bool)> {
    storage
        .connection
        .prepare("SELECT chat_id, whole FROM shared_changes WHERE revision > ?1 ORDER BY chat_id")
        .unwrap()
        .query_map([after], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn published_revision(storage: &Storage) -> i64 {
    storage
        .connection
        .query_row("SELECT revision FROM shared_publication", [], |row| {
            row.get(0)
        })
        .unwrap()
}

/// Mark everything so far as published to `destination`.
fn publish(storage: &Storage) -> i64 {
    use crate::storage::publication::JournalCoverage;
    storage.prepare_publication_target("destination").unwrap();
    let revision = published_revision(storage);
    storage
        .acknowledge_publication(
            "destination",
            revision,
            JournalCoverage::Incremental { through: revision },
        )
        .unwrap();
    revision
}

/// What a delta after `after` that may amend uploads: the chats resent whole,
/// and each amended chat with its marker, replaced messages and message count.
type Amended = (String, Option<Subagent>, Vec<String>, u64);

fn delta(storage: &Storage, after: i64) -> (Vec<String>, Vec<Amended>, usize) {
    use crate::shared::batches::{Batch, BatchKind};
    use crate::storage::publication::Delta;
    let (mut whole, mut amended, mut messages) = (Vec::new(), Vec::new(), 0);
    storage
        .with_publication_snapshot(|snapshot| {
            let delta = Some(Delta { after, amend: true });
            snapshot.visit_selected(BatchKind::Chats, false, delta, |batch| {
                if let Batch::Chats { records } = batch {
                    whole.extend(records.into_iter().map(|record| record.id));
                }
                Ok(())
            })?;
            snapshot.visit_amendments(after, |amendment| {
                amended.push((
                    amendment.chat.id,
                    amendment.chat.subagent,
                    amendment.replaced,
                    amendment.messages,
                ));
                Ok(())
            })?;
            snapshot.visit_selected(BatchKind::Messages, true, delta, |batch| {
                messages += batch.len();
                Ok(())
            })?;
            snapshot.visit_amended_messages(after, |batch| {
                messages += batch.len();
                Ok(())
            })
        })
        .unwrap();
    (whole, amended, messages)
}

#[test]
fn upgrading_journals_marked_chats_so_a_delta_amends_them_without_messages() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("index.sqlite3");
    {
        let mut storage = Storage::open_writer(&path).unwrap();
        storage
            .replace_chat(&codex_chat("parent", "plan", None))
            .unwrap();
        storage
            .replace_chat(&codex_chat("child", "work", Some(spawned("parent"))))
            .unwrap();
        storage.commit_writer().unwrap();
    }
    // Published by a release that did not publish markers, then upgraded.
    let published = publish(&Storage::open_existing(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("PRAGMA user_version=68;").unwrap();
    drop(connection);
    let storage = Storage::open_existing(&path).unwrap();
    assert!(published_revision(&storage) > published);
    assert!(Storage::publication_is_pending(&path, "destination").unwrap());
    // Only the marked chat, and as an amendment rather than a whole resend.
    assert_eq!(journaled(&storage, published), [("child".into(), false)]);
    let (whole, amended, messages) = delta(&storage, published);
    assert_eq!(whole, [] as [String; 0]);
    assert_eq!(
        amended,
        [("child".into(), Some(spawned("parent")), Vec::new(), 1)]
    );
    assert_eq!(messages, 0, "no message is resent");
}

#[test]
fn a_changed_marker_is_journaled_as_an_amendment_and_an_unchanged_one_is_not() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("index.sqlite3");
    let mut storage = Storage::open_writer(&path).unwrap();
    storage
        .replace_chat(&codex_chat("parent", "plan", None))
        .unwrap();
    let child = codex_chat("child", "work", Some(spawned("parent")));
    storage.replace_chat(&child).unwrap();
    storage.commit_writer().unwrap();
    drop(storage);
    let storage = Storage::open_existing(&path).unwrap();
    let published = publish(&storage);
    // Every record carries its chat's marker, or none.
    let mut markers = Vec::new();
    storage
        .with_publication_snapshot(|snapshot| {
            use crate::shared::batches::{Batch, BatchKind};
            snapshot.visit_selected(BatchKind::Chats, false, None, |batch| {
                if let Batch::Chats { records } = batch {
                    markers.extend(records.into_iter().map(|chat| (chat.id, chat.subagent)));
                }
                Ok(())
            })
        })
        .unwrap();
    assert_eq!(
        markers,
        [
            ("child".into(), Some(spawned("parent"))),
            ("parent".into(), None)
        ]
    );
    drop(storage);

    // Reading the same transcript again changes nothing to publish.
    let mut storage = Storage::open_writer(&path).unwrap();
    storage.replace_chat(&child).unwrap();
    storage.commit_writer().unwrap();
    assert_eq!(journaled(&storage, published), []);

    let mut finished = spawned("parent");
    finished.label = Some("Locke, done".into());
    let mut storage = Storage::open_writer(&path).unwrap();
    storage
        .replace_chat(&codex_chat("child", "work", Some(finished.clone())))
        .unwrap();
    storage.commit_writer().unwrap();
    let (whole, amended, messages) = delta(&storage, published);
    assert_eq!(whole, [] as [String; 0]);
    assert_eq!(amended, [("child".into(), Some(finished), Vec::new(), 1)]);
    assert_eq!(messages, 0);
}
