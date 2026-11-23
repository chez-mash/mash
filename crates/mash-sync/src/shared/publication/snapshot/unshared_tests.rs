//! An unshared agent's chats, and the memories citing them, never reach an
//! upload: not in a snapshot, a delta, a backfill, or the snapshot that a
//! change to the unshared list forces.
use super::*;
use crate::provider::Provider;
use crate::records::{
    Body, Change, Citation, DecidedBy, DecisionState, Details, DuplicatePolicy, EvidenceUse,
    Mutation, Scope,
};
use crate::silver::{Chat, NormalizedMessage, SourceIdentity};

const SECRET: &str = "kept-on-this-machine";

fn chat(id: &str, provider: Provider, text: &str) -> Chat {
    Chat {
        id: id.into(),
        provider,
        title_origin: None,
        timestamp_origins: None,
        claude_lineage: None,
        origin: mash_silver::subagent::Origin::default(),
        source: SourceIdentity::file(format!("/nonexistent/mash-{id}.jsonl")),
        project: None,
        repository: None,
        title: Some(format!("Topic {id}")),
        created_at: Some(1),
        updated_at: Some(2),
        messages: vec![NormalizedMessage {
            attachments: Vec::new(),
            id: format!("{id}-message"),
            ordinal: 0,
            context: crate::message_context::MessageContext::Linear,
            role: crate::role::Role::User,
            content: text.into(),
            dialogue_text: None,
            created_at: Some(1),
            tools: crate::tools::Activity::default(),
        }]
        .into(),
    }
}

fn memory(statement: &str, message: &str, quote: &str) -> Mutation {
    Mutation {
        operation_id: format!("remember-{message}-{statement}"),
        change: Change::Create {
            scope: Scope::Personal,
            body: Body {
                statement: statement.into(),
                rationale: String::new(),
                details: Details::Decision {
                    state: DecisionState::Accepted,
                    decided_by: DecidedBy::User,
                    alternatives: Vec::new(),
                    consequences: Vec::new(),
                },
                review: crate::records::Review::Established,
                effective_at: None,
                recorded_by: "test agent".into(),
            },
            evidence: vec![Citation {
                session_id: None,
                message_id: message.into(),
                quote: quote.into(),
                purpose: EvidenceUse::Endorsement,
                expected_fingerprint: None,
            }],
            on_duplicate: DuplicatePolicy::Report,
        },
    }
}

/// A Codex chat and a Cursor chat holding `SECRET`, each with a memory citing it.
fn seed(directory: &Path) -> std::path::PathBuf {
    let database = directory.join("silver.sqlite3");
    let mut storage = Storage::open_writer(&database).unwrap();
    storage
        .replace_chat(&chat("shared", Provider::Codex, "use qdrant for search"))
        .unwrap();
    storage
        .replace_chat(&chat(
            "private",
            Provider::Cursor,
            &format!("token {SECRET} here"),
        ))
        .unwrap();
    storage.commit_writer().unwrap();
    let mut storage = Storage::open_existing(&database).unwrap();
    storage
        .mutate_record(&memory("Use Qdrant", "shared-message", "use qdrant"))
        .unwrap();
    storage
        .mutate_record(&memory(
            &format!("Keep {SECRET}"),
            "private-message",
            SECRET,
        ))
        .unwrap();
    database
}

fn target(instance: Option<String>, revision: i64, unshared: Vec<Provider>) -> Target {
    Target {
        build: crate::views::config::Build::default(),
        generation: "unused".into(),
        dimensions: 384,
        instance,
        publication_revision: revision,
        redaction: crate::redaction::Policy::ON,
        unshared,
    }
}

fn written(source: &Source) -> String {
    std::fs::read_to_string(source.file.path()).unwrap()
}

fn assert_private_absent(upload: &str) {
    assert!(!upload.contains(SECRET), "unshared text reached the upload");
    assert!(
        !upload.contains("\"private\""),
        "the unshared chat reached the upload"
    );
    assert!(
        !upload.contains("private-message"),
        "an unshared message reached the upload"
    );
    assert!(
        !upload.contains("\"cursor\""),
        "an unshared agent's row reached the upload"
    );
}

#[test]
fn a_snapshot_and_a_backfill_leave_out_unshared_chats_and_the_memories_citing_them() {
    let directory = tempfile::tempdir().unwrap();
    let database = seed(directory.path());
    for selection in [Selection::Replace, Selection::Backfill] {
        let source = capture(
            &database,
            &target(None, 0, vec![Provider::Cursor]),
            selection,
        )
        .unwrap();
        let upload = written(&source);
        assert_private_absent(&upload);
        // What is shared still goes: the Codex chat and the memory citing it.
        assert!(upload.contains("use qdrant for search"));
        assert!(upload.contains("Use Qdrant"));
        assert_eq!(source.chats.keys().collect::<Vec<_>>(), vec!["shared"]);
    }
    // Without the setting, the same database publishes both.
    let everything = capture(&database, &target(None, 0, Vec::new()), Selection::Replace).unwrap();
    assert!(written(&everything).contains(SECRET));
}

#[test]
fn a_delta_leaves_out_unshared_changes_and_memories() {
    let directory = tempfile::tempdir().unwrap();
    let database = seed(directory.path());
    let storage = Storage::open_existing(&database).unwrap();
    storage.prepare_publication_target("journal").unwrap();
    let mut instance = String::new();
    let base = storage
        .with_publication_snapshot(|snapshot| {
            instance.clone_from(&snapshot.instance);
            Ok(())
        })
        .unwrap();
    storage
        .acknowledge_publication(
            "journal",
            base,
            crate::storage::publication::JournalCoverage::Incremental { through: base },
        )
        .unwrap();
    drop(storage);
    // Both chats change after the acknowledged base, and a new memory cites the private one.
    let mut writer = Storage::open_writer(&database).unwrap();
    writer
        .replace_chat(&chat(
            "shared",
            Provider::Codex,
            "use qdrant for search, still",
        ))
        .unwrap();
    writer
        .replace_chat(&chat(
            "private",
            Provider::Cursor,
            &format!("new {SECRET} again"),
        ))
        .unwrap();
    writer.commit_writer().unwrap();
    Storage::open_existing(&database)
        .unwrap()
        .mutate_record(&memory(
            &format!("Again {SECRET}"),
            "private-message",
            SECRET,
        ))
        .unwrap();
    let source = capture(
        &database,
        &target(Some(instance), base, vec![Provider::Cursor]),
        Selection::Delta { journal_base: base },
    )
    .unwrap();
    let upload = written(&source);
    let Some(Ok(Event::Start(start))) = upload.lines().next().map(serde_json::from_str::<Event>)
    else {
        panic!("expected start");
    };
    assert!(!start.snapshot, "this is an incremental upload");
    assert_private_absent(&upload);
    assert!(upload.contains("still"));
}

#[test]
fn changing_the_unshared_list_forces_a_snapshot_that_replaces_everything() {
    let config = |unshared: Vec<Provider>| crate::config::SharedConfig {
        group_id: "group".into(),
        node_id: "node".into(),
        api_url: "https://example.invalid".into(),
        token_env: "MASH_TEST_TOKEN".into(),
        publication_redaction: crate::redaction::Policy::ON,
        unshared,
    };
    let before = super::super::target(&config(Vec::new()));
    let after = super::super::target(&config(vec![Provider::Cursor]));
    assert_ne!(
        before, after,
        "a new list must start a new publication target"
    );
    // Order and repeats don't matter.
    assert_eq!(
        super::super::target(&config(vec![Provider::Cursor, Provider::Copilot])),
        super::super::target(&config(vec![
            Provider::Copilot,
            Provider::Cursor,
            Provider::Copilot
        ]))
    );

    // A delta prepared after the target changes is a complete snapshot, which
    // the server applies by replacing everything this node published before.
    let directory = tempfile::tempdir().unwrap();
    let database = seed(directory.path());
    let storage = Storage::open_existing(&database).unwrap();
    storage.prepare_publication_target(&before).unwrap();
    let mut instance = String::new();
    let base = storage
        .with_publication_snapshot(|snapshot| {
            instance.clone_from(&snapshot.instance);
            Ok(())
        })
        .unwrap();
    storage
        .acknowledge_publication(
            &before,
            base,
            crate::storage::publication::JournalCoverage::Incremental { through: base },
        )
        .unwrap();
    storage.prepare_publication_target(&after).unwrap();
    drop(storage);
    let source = capture(
        &database,
        &target(Some(instance), base, vec![Provider::Cursor]),
        Selection::Delta { journal_base: base },
    )
    .unwrap();
    let upload = written(&source);
    let Some(Ok(Event::Start(start))) = upload.lines().next().map(serde_json::from_str::<Event>)
    else {
        panic!("expected start");
    };
    assert!(start.snapshot, "a changed list republishes everything");
    assert_private_absent(&upload);
}
