use super::*;
use crate::silver::NormalizedMessage;
use crate::silver::{IndexOptions, discovered_sources, index};
use crate::storage::Storage;
use models::{Bubble, Kind};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;

mod failures;

#[derive(Deserialize)]
struct Fixture {
    rows: Vec<Row>,
    #[serde(rename = "expectedBubbleIds")]
    expected_bubble_ids: Vec<String>,
}
#[derive(Deserialize)]
struct Row {
    key: String,
    value: String,
}

pub(in crate::silver::cursor) fn fixture(path: &Path) -> Connection {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../fixtures/cursor-ide-native-v18.json"
    ))
    .unwrap();
    assert_eq!(fixture.expected_bubble_ids.len(), 6);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let connection = Connection::open(path).unwrap();
    connection.execute_batch("PRAGMA user_version=1; PRAGMA journal_mode=WAL; CREATE TABLE ItemTable(key TEXT UNIQUE ON CONFLICT REPLACE,value BLOB); CREATE TABLE cursorDiskKV(key TEXT UNIQUE ON CONFLICT REPLACE,value BLOB);").unwrap();
    for row in fixture.rows {
        connection
            .execute(
                "INSERT INTO cursorDiskKV VALUES(?1,?2)",
                params![row.key, row.value],
            )
            .unwrap();
    }
    connection
}
fn path(root: &Path) -> PathBuf {
    root.join("User/globalStorage/state.vscdb")
}
fn root_key() -> &'static str {
    "composerData:fixture-ide-session"
}
fn bubble_key(id: &str) -> String {
    format!("bubbleId:fixture-ide-session:{id}")
}
fn read<T: DeserializeOwned>(connection: &Connection, key: &str) -> T {
    let value: String = connection
        .query_row("SELECT value FROM cursorDiskKV WHERE key=?1", [key], |r| {
            r.get(0)
        })
        .unwrap();
    serde_json::from_str(&value).unwrap()
}
fn write<T: Serialize>(connection: &Connection, key: &str, value: &T) {
    connection
        .execute(
            "INSERT OR REPLACE INTO cursorDiskKV VALUES(?1,?2)",
            params![key, serde_json::to_string(value).unwrap()],
        )
        .unwrap();
}
fn parsed(path: &Path) -> Chat {
    super::super::parse(&super::super::discover(path).unwrap().remove(0).identity)
        .unwrap()
        .chat
        .unwrap()
}
fn options(path: &Path) -> IndexOptions {
    IndexOptions {
        providers: [(Provider::Cursor, vec![path.into()])].into(),
        since_days: None,
        roots_are_authoritative: true,
    }
}
fn searchable(chat: &Chat) -> Vec<&NormalizedMessage> {
    chat.messages
        .inline()
        .iter()
        .filter(|m| !m.dialogue_text.as_deref().unwrap_or_default().is_empty())
        .collect()
}

#[test]
fn native_root_owns_order_metadata_and_distinct_repeated_requests() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let chat = parsed(&path);
    // Six bubbles; the answer's thinking and the tool call are their own entries.
    assert_eq!(chat.messages.len(), 8);
    assert_eq!(searchable(&chat).len(), 4);
    assert_eq!(chat.project.as_deref(), Some("/work/cbioportal"));
    assert_eq!(chat.repository.as_deref(), Some("/work/cbioportal"));
    assert_eq!(chat.title.as_deref(), Some("cBioPortal native IDE fixture"));
    assert_eq!(chat.created_at, Some(1_790_762_400_000));
    assert_eq!(chat.updated_at, Some(1_790_762_400_001));
    assert_eq!(
        chat.messages.inline()[0].content,
        chat.messages.inline()[6].content
    );
    assert_ne!(chat.messages.inline()[0].id, chat.messages.inline()[6].id);
    assert_eq!(
        chat.messages.inline()[0].created_at,
        Some(1_790_762_400_000)
    );
    assert_eq!(chat.messages.inline()[1].created_at, None);
    assert_eq!(
        chat.messages.inline()[1].content,
        "[Thinking] private-ide-reasoning"
    );
    assert_eq!(chat.messages.inline()[1].role, crate::role::Role::Assistant);
    assert_eq!(
        chat.messages.inline()[2].content,
        "Validate the study metadata before import."
    );
    let call = &chat.messages.inline()[5];
    assert!(call.content.starts_with("Tool: tool 51\n"));
    assert_eq!(call.tools.tool_calls.len(), 1);
    assert_eq!(call.tools.tool_calls[0].name.as_deref(), Some("tool 51"));
    assert!(
        chat.messages
            .inline()
            .iter()
            .all(|m| !m.content.contains("unsent-draft")
                && !m.content.contains("ide-summary")
                && !m.content.contains("abandoned-branch")
                && !m.content.contains("AAAAAAAA"))
    );
    assert!(
        searchable(&chat)
            .iter()
            .all(|m| !m.dialogue_text.as_deref().unwrap().contains("private-"))
    );
    let mut bubble: Bubble = read(&connection, &bubble_key("request-1"));
    bubble.text = "Updated cBioPortal importer request".into();
    write(&connection, &bubble_key("request-1"), &bubble);
    let edited = parsed(&path);
    assert_eq!(edited.messages.inline()[0].id, chat.messages.inline()[0].id);
    assert_ne!(
        edited.messages.inline()[0].content,
        chat.messages.inline()[0].content
    );
    let mut root: Root = read(&connection, root_key());
    root.full_conversation_headers_only.truncate(2);
    write(&connection, root_key(), &root);
    let rewind = parsed(&path);
    assert_eq!(rewind.messages.len(), 3);
    assert_eq!(rewind.messages.inline()[1].id, chat.messages.inline()[1].id);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
}

#[test]
fn logical_composers_are_discovered_independently_and_schema_detection_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let mut root: Root = read(&connection, root_key());
    root.composer_id = "second".into();
    root.full_conversation_headers_only.clear();
    write(&connection, "composerData:second", &root);
    let sources = discovered_sources(Provider::Cursor, &[dir.path().into()]);
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].identity.key(), sources[1].identity.key());
    assert!(
        sources
            .iter()
            .all(|s| s.identity.session.as_deref().unwrap().starts_with("ide:"))
    );
    let db = dir.path().join("index.sqlite3");
    let mut storage = Storage::open_writer(&db).unwrap();
    index(&mut storage, &options(dir.path())).unwrap();
    storage.commit_writer().unwrap();
    // The second composer holds no conversation, so it is not a chat.
    assert_eq!(storage.counts().unwrap().chats, 1);
    assert_eq!(storage.counts().unwrap().messages, 8);
    connection
        .execute_batch(
            "CREATE TABLE meta(key TEXT,value TEXT); CREATE TABLE blobs(id TEXT,data BLOB);",
        )
        .unwrap();
    assert!(super::super::discover(&path).is_err());
}

/// Found on macOS with Cursor 3.30.8: 19 of 27 workspace databases had no
/// chat table, and discovery refused the whole provider.
#[test]
fn workspace_state_without_a_chat_table_holds_no_chats_and_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    fixture(&path(dir.path()));
    let settings_only = dir
        .path()
        .join("workspaceStorage/settings-only/state.vscdb");
    std::fs::create_dir_all(settings_only.parent().unwrap()).unwrap();
    Connection::open(&settings_only)
        .unwrap()
        .execute_batch("CREATE TABLE ItemTable(key TEXT UNIQUE ON CONFLICT REPLACE,value BLOB);")
        .unwrap();
    assert!(super::super::discover(&settings_only).unwrap().is_empty());
    let sources = discovered_sources(Provider::Cursor, &[dir.path().into()]);
    assert_eq!(sources.len(), 1);
    let db = dir.path().join("index.sqlite3");
    let mut storage = Storage::open_writer(&db).unwrap();
    index(&mut storage, &options(dir.path())).unwrap();
    storage.commit_writer().unwrap();
    assert_eq!(storage.counts().unwrap().chats, 1);
}

/// A composer layout this reader has not been checked against rejects only
/// itself, and a deleted composer (a row without a value) is not a source.
#[test]
fn unknown_composer_versions_reject_themselves_and_deleted_composers_are_not_sources() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    connection
        .execute(
            "INSERT INTO cursorDiskKV VALUES('composerData:unknown',?1)",
            [r#"{"_v":5,"composerId":"unknown","conversation":[],"createdAt":1}"#],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO cursorDiskKV VALUES('composerData:deleted',NULL)",
            [],
        )
        .unwrap();
    let sources = discovered_sources(Provider::Cursor, &[dir.path().into()]);
    assert_eq!(sources.len(), 2);
    let unknown = sources
        .iter()
        .find(|s| s.identity.session.as_deref() == Some("ide:unknown"))
        .unwrap();
    let rejected = super::super::parse(&unknown.identity).unwrap();
    assert!(rejected.chat.is_none());
    assert!(rejected.first_record_issue.is_some());
    let db = dir.path().join("index.sqlite3");
    let mut storage = Storage::open_writer(&db).unwrap();
    let summary = index(&mut storage, &options(dir.path())).unwrap();
    storage.commit_writer().unwrap();
    assert_eq!(summary.rejected_sources, 1);
    assert_eq!(storage.counts().unwrap().chats, 1);
    assert_eq!(storage.counts().unwrap().messages, 8);
}

/// Found on macOS with Cursor 3.30.8: 80 of 91 composers were saved by older
/// releases (versions 6, 9, 10 and 11). They keep the same headers and bubble
/// rows; prompts and replies carry a tool record with nothing in it, and most
/// composers never held a conversation.
#[test]
fn older_composer_versions_are_read_and_empty_ones_are_not_chats() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let rows = [
        (
            "composerData:older",
            r#"{"_v":9,"composerId":"older","name":"Older chat","createdAt":1700000000000,"lastUpdatedAt":1700000005000,"fullConversationHeadersOnly":[{"bubbleId":"q","type":1},{"bubbleId":"t","type":2},{"bubbleId":"a","type":2}]}"#,
        ),
        (
            "bubbleId:older:q",
            r#"{"bubbleId":"q","type":1,"text":"olderquestion sentinel","toolFormerData":{"additionalData":{}}}"#,
        ),
        (
            "bubbleId:older:t",
            r#"{"bubbleId":"t","type":2,"text":"","capabilityType":15,"toolFormerData":{"tool":5,"name":"read_file","toolCallId":"call-1","status":"completed","params":"{\"path\":\"a.rs\"}","result":"{\"ok\":true}"}}"#,
        ),
        (
            "bubbleId:older:a",
            r#"{"bubbleId":"a","type":2,"text":"olderanswer sentinel","createdAt":"2023-11-14T22:13:25.000Z","thinking":{"text":"olderthought"}}"#,
        ),
        (
            "composerData:draft",
            r#"{"_v":6,"composerId":"draft","createdAt":1,"fullConversationHeadersOnly":[]}"#,
        ),
        (
            "composerData:bare",
            r#"{"_v":11,"composerId":"bare","createdAt":1}"#,
        ),
    ];
    for (key, value) in rows {
        connection
            .execute("INSERT INTO cursorDiskKV VALUES(?1,?2)", [key, value])
            .unwrap();
    }
    let sources = discovered_sources(Provider::Cursor, &[dir.path().into()]);
    let parse = |session: &str| {
        let source = sources
            .iter()
            .find(|s| s.identity.session.as_deref() == Some(session))
            .unwrap();
        super::super::parse(&source.identity).unwrap()
    };
    let older = parse("ide:older").chat.unwrap();
    assert_eq!(older.title.as_deref(), Some("Older chat"));
    let messages = older.messages.inline();
    assert_eq!(messages.len(), 5);
    assert_eq!(messages[0].role, crate::role::Role::User);
    assert_eq!(
        messages[0].dialogue_text.as_deref(),
        Some("olderquestion sentinel")
    );
    // The tool step is kept for viewing, typed, and stays out of search.
    assert!(messages[1].content.contains("a.rs"));
    assert_eq!(messages[1].dialogue_text.as_deref(), Some(""));
    assert_eq!(
        messages[1].tools.tool_calls[0].name.as_deref(),
        Some("read_file")
    );
    assert_eq!(
        messages[1].tools.tool_calls[0].call_id.as_deref(),
        Some("call-1")
    );
    assert_eq!(messages[2].role, crate::role::Role::Tool);
    assert_eq!(
        messages[2].tools.tool_results[0].call_id.as_deref(),
        Some("call-1")
    );
    assert_eq!(messages[2].tools.tool_results[0].is_error, Some(false));
    assert_eq!(messages[3].content, "[Thinking] olderthought");
    assert_eq!(messages[3].dialogue_text.as_deref(), Some(""));
    assert_eq!(
        messages[4].dialogue_text.as_deref(),
        Some("olderanswer sentinel")
    );
    for session in ["ide:draft", "ide:bare"] {
        let empty = parse(session);
        assert!(empty.chat.is_none());
        assert!(empty.first_record_issue.is_none());
    }
    let db = dir.path().join("index.sqlite3");
    let mut storage = Storage::open_writer(&db).unwrap();
    let summary = index(&mut storage, &options(dir.path())).unwrap();
    storage.commit_writer().unwrap();
    assert_eq!(summary.rejected_sources, 0);
    assert_eq!(storage.counts().unwrap().chats, 2);
    assert_eq!(storage.counts().unwrap().messages, 13);
}

#[test]
fn synthetic_operational_and_unknown_bubbles_remain_context_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let key = bubble_key("request-1");
    let original: String = connection
        .query_row("SELECT value FROM cursorDiskKV WHERE key=?1", [&key], |r| {
            r.get(0)
        })
        .unwrap();
    let variants: [fn(&mut Bubble); 7] = [
        |b| b.is_placeholder = true,
        |b| b.is_simulated_msg = true,
        |b| b.simulated_msg_reason = Some(999),
        |b| {
            b.simulated_message_metadata = Some(models::Simulated {
                suppress_ui_display: true,
            });
        },
        |b| b.sent_by_agent_id = Some("sender".into()),
        |b| {
            b.service_status_update = Some(models::ProtoRecord::Json(
                "{\"message\":\"private-status-context\"}".into(),
            ));
        },
        |b| {
            b.event_notification = Some(models::Notification {
                headline: "private-notification-context".into(),
                detail: None,
            });
        },
    ];
    for modify in variants {
        let mut bubble: Bubble = serde_json::from_str(&original).unwrap();
        modify(&mut bubble);
        write(&connection, &key, &bubble);
        let chat = parsed(&path);
        assert_eq!(searchable(&chat).len(), 3);
        assert_eq!(chat.messages.inline()[0].dialogue_text.as_deref(), Some(""));
    }
    let mut bubble: Bubble = serde_json::from_str(&original).unwrap();
    bubble.kind = Kind::Other(999);
    write(&connection, &key, &bubble);
    let mut root: Root = read(&connection, root_key());
    root.full_conversation_headers_only[0].kind = bubble.kind;
    write(&connection, root_key(), &root);
    assert_eq!(searchable(&parsed(&path)).len(), 3);
    root.subagent_info = Some(models::Subagent {
        parent_composer_id: "parent".into(),
    });
    write(&connection, root_key(), &root);
    assert_eq!(searchable(&parsed(&path)), Vec::<&NormalizedMessage>::new());
    assert_eq!(parsed(&path).messages.len(), 8);
}

#[test]
fn wal_changes_refresh_sources_without_mixing_snapshot_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA wal_autocheckpoint=0;")
        .unwrap();
    let source = super::super::discover(&path).unwrap().remove(0);
    let stamp = source.stamp().unwrap();
    let mut snapshot = Store::open(&path).unwrap();
    let _: Root = snapshot.json(root_key()).unwrap();
    let key = bubble_key("request-1");
    let mut bubble: Bubble = read(&connection, &key);
    bubble.text = "WAL cBioPortal changes".into();
    write(&connection, &key, &bubble);
    let old: Bubble = snapshot.json(&key).unwrap();
    assert_ne!(old.text, bubble.text);
    assert_eq!(parsed(&path).messages.inline()[0].content, bubble.text);
    assert_ne!(stamp.fingerprint, source.stamp().unwrap().fingerprint);
    assert!(path.with_file_name("state.vscdb-wal").exists());
}

/// Found on macOS with Cursor 3.30.8: a version 18 root had no inline headers
/// and a header log instead, and the chat came out with a title and nothing in it.
#[test]
fn headers_kept_in_chunk_records_are_read_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let inline = parsed(&path);
    let mut root: Root = read(&connection, root_key());
    let headers = std::mem::take(&mut root.full_conversation_headers_only);
    root.header_log = Some(models::HeaderLog {
        chunk_count: 2,
        count: 6,
    });
    write(&connection, root_key(), &root);
    write(
        &connection,
        "composerConvHeaders:fixture-ide-session:000000",
        &&headers[..4],
    );
    write(
        &connection,
        "composerConvHeaders:fixture-ide-session:000001",
        &&headers[4..],
    );
    let ids = |chat: &Chat| {
        chat.messages
            .inline()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&parsed(&path)), ids(&inline));
    // A log that disagrees with its chunks is not a shorter chat.
    root.header_log = Some(models::HeaderLog {
        chunk_count: 2,
        count: 7,
    });
    write(&connection, root_key(), &root);
    let source = super::super::discover(&path).unwrap().remove(0);
    assert!(super::super::parse(&source.identity).is_err());
}

/// Older roots name no workspace; the workspace's own state lists its chats.
#[test]
fn a_chat_without_a_recorded_workspace_takes_the_folder_that_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = path(dir.path());
    let connection = fixture(&path);
    let mut root: Root = read(&connection, root_key());
    root.workspace_identifier = None;
    root.tracked_git_repos.clear();
    root.conversation_state = None;
    write(&connection, root_key(), &root);
    assert_eq!(parsed(&path).project, None);
    let workspace = dir.path().join("User/workspaceStorage/abc");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(
        workspace.join("workspace.json"),
        r#"{"folder":"file:///work/listed"}"#,
    )
    .unwrap();
    let state = Connection::open(workspace.join("state.vscdb")).unwrap();
    state
        .execute_batch("CREATE TABLE ItemTable(key TEXT UNIQUE ON CONFLICT REPLACE,value BLOB);")
        .unwrap();
    state
        .execute(
            "INSERT INTO ItemTable VALUES('composer.composerData',?1)",
            [r#"{"allComposers":[{"composerId":"other"},{"composerId":"fixture-ide-session"}]}"#],
        )
        .unwrap();
    assert_eq!(parsed(&path).project.as_deref(), Some("/work/listed"));
}
