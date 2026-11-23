//! Published subagent markers in a node's shared silver: kept from the
//! records that carry them, gone with the records that do not, and outside
//! every fingerprint.
use super::*;
use crate::shared::activity::Progress;
use crate::shared::protocol::Amendment;
use crate::shared::silver_store::SilverStore;
use crate::silver::subagent::Subagent;
use std::io::Write;

fn identity() -> Identity {
    Identity {
        group_id: "subagents".into(),
        node_id: "node".into(),
        generation: "fixture:test:4".to_owned(),
        dimensions: 4,
    }
}

fn marker(label: &str) -> Subagent {
    Subagent {
        parent_chat_id: Some("parent".into()),
        label: Some(label.into()),
        role: Some("worker".into()),
        depth: Some(1),
        finished: Some(true),
        spawn_call_id: Some("call-1".into()),
    }
}

fn chat(id: &str, subagent: Option<Subagent>) -> ChatRecord {
    ChatRecord {
        enrichment: None,
        id: id.into(),
        provider: Provider::ClaudeCode,
        title: Some(format!("{id} title")),
        project: None,
        repository: None,
        created_at: Some(1),
        updated_at: Some(2),
        subagent,
    }
}

fn message(chat: &str) -> MessageRecord {
    MessageRecord {
        attachments: vec![],
        id: format!("{chat}-message"),
        chat_id: chat.into(),
        ordinal: 0,
        context: crate::message_context::MessageContext::Linear,
        role: crate::role::Role::User,
        content: "question".into(),
        projection: "question".into(),
        created_at: Some(1),
        generation: None,
        vector: None,
        tools: crate::tools::Activity::default(),
    }
}

fn start(revision: i64, base: Option<i64>) -> Event {
    Event::Start(Start {
        views_build: None,
        source_silver: None,
        instance: "instance".into(),
        revision: Some(revision),
        base_revision: base,
        snapshot: base.is_none(),
        forgets: vec![],
    })
}

/// A snapshot of a parent and its subagent `child`, carrying `subagent`.
fn snapshot(revision: i64, subagent: Option<Subagent>) -> Vec<Event> {
    vec![
        start(revision, None),
        Event::Batch(Batch::Chats {
            records: vec![chat("parent", None), chat("child", subagent)],
        }),
        Event::Batch(Batch::Messages {
            records: vec![message("parent"), message("child")],
        }),
        Event::Finish,
    ]
}

/// A delta that amends `child` with `subagent` and changes none of its messages.
fn amendment(revision: i64, subagent: Option<Subagent>) -> Vec<Event> {
    vec![
        start(revision, Some(revision - 1)),
        Event::Amend(Box::new(Amendment {
            chat: chat("child", subagent),
            replaced: vec![],
            messages: 1,
        })),
        Event::Finish,
    ]
}

fn upload(store: &SilverStore, events: &[Event]) -> Result<Applied, AppError> {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    for event in events {
        serde_json::to_writer(&mut file, event).unwrap();
        writeln!(file).unwrap();
    }
    let build = Build::default();
    write(
        store,
        &identity(),
        &file,
        &build,
        &Progress::default(),
        None,
        false,
    )
}

fn stored(store: &SilverStore) -> Option<mash_storage::subagents::Subagent> {
    let connection = store.connect(&identity()).unwrap();
    crate::storage::subagents::read(&connection, "child").unwrap()
}

/// The silver revision fingerprint and the child's thread fingerprint.
fn fingerprints(store: &SilverStore) -> (String, Option<String>) {
    let connection = store.connect(&identity()).unwrap();
    let head = head(&connection).unwrap().unwrap();
    let thread = sql::ChatFingerprint::builder()
        .id("child")
        .build()
        .query_one(&connection)
        .unwrap()
        .chat_fingerprint;
    (head.silver_revision.unwrap().fingerprint, thread)
}

#[test]
fn a_snapshot_keeps_the_markers_it_carries_outside_every_fingerprint() {
    let marked = tempfile::tempdir().unwrap();
    let marked = SilverStore::open(marked.path(), [&identity()]).unwrap();
    upload(&marked, &snapshot(1, Some(marker("Tidy the docs")))).unwrap();
    let read = stored(&marked).unwrap();
    assert_eq!(read.label.as_deref(), Some("Tidy the docs"));
    assert_eq!(read.parent_chat_id.as_deref(), Some("parent"));
    assert_eq!(read.parent_title.as_deref(), Some("parent title"));
    assert_eq!((read.depth, read.finished), (Some(1), Some(true)));
    assert_eq!(read.spawn_call_id.as_deref(), Some("call-1"));

    let plain = tempfile::tempdir().unwrap();
    let plain = SilverStore::open(plain.path(), [&identity()]).unwrap();
    upload(&plain, &snapshot(1, None)).unwrap();
    assert_eq!(stored(&plain), None);
    // Views and certification never see the marker.
    assert_eq!(fingerprints(&marked), fingerprints(&plain));

    // A later snapshot without it replaces every row.
    upload(&marked, &snapshot(2, None)).unwrap();
    assert_eq!(stored(&marked), None);
}

#[test]
fn an_amendment_sets_changes_or_clears_the_marker_without_touching_messages() {
    let directory = tempfile::tempdir().unwrap();
    let store = SilverStore::open(directory.path(), [&identity()]).unwrap();
    // The chat was published before markers were.
    upload(&store, &snapshot(1, None)).unwrap();
    let before = fingerprints(&store);
    let applied = upload(&store, &amendment(2, Some(marker("Tidy the docs")))).unwrap();
    assert!(applied.amended.contains_key("child"));
    assert_eq!(
        stored(&store).and_then(|read| read.label).as_deref(),
        Some("Tidy the docs")
    );
    assert_eq!(fingerprints(&store), before);
    upload(&store, &amendment(3, Some(marker("Tidy the guide")))).unwrap();
    assert_eq!(
        stored(&store).and_then(|read| read.label).as_deref(),
        Some("Tidy the guide")
    );
    // The record is the node's whole chat: without a marker it is not one.
    upload(&store, &amendment(4, None)).unwrap();
    assert_eq!(stored(&store), None);
}

#[test]
fn a_chat_resent_whole_takes_the_marker_its_record_carries() {
    let directory = tempfile::tempdir().unwrap();
    let store = SilverStore::open(directory.path(), [&identity()]).unwrap();
    upload(&store, &snapshot(1, Some(marker("Tidy the docs")))).unwrap();
    let resend = |revision: i64, subagent: Option<Subagent>| {
        vec![
            start(revision, Some(revision - 1)),
            Event::Batch(Batch::Chats {
                records: vec![chat("child", subagent)],
            }),
            Event::Batch(Batch::Messages {
                records: vec![message("child")],
            }),
            Event::Finish,
        ]
    };
    upload(&store, &resend(2, Some(marker("Tidy the guide")))).unwrap();
    assert_eq!(
        stored(&store).and_then(|read| read.label).as_deref(),
        Some("Tidy the guide")
    );
    upload(&store, &resend(3, None)).unwrap();
    assert_eq!(stored(&store), None);
}

#[test]
fn a_marker_that_names_its_own_chat_is_rejected_whole() {
    let directory = tempfile::tempdir().unwrap();
    let store = SilverStore::open(directory.path(), [&identity()]).unwrap();
    let mut own = marker("Loop");
    own.parent_chat_id = Some("child".into());
    let error = upload(&store, &snapshot(1, Some(own))).unwrap_err();
    assert!(error.error.message.contains("own parent"), "{error:?}");
    let connection = store.connect(&identity()).unwrap();
    assert!(head(&connection).unwrap().is_none());
}
