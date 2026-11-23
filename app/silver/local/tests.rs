use super::*;
use crate::provider::Provider;
use crate::silver::certificate::Completeness;
use std::fs::{self, FileTimes};
use std::path::Path;
use std::time::{Duration, SystemTime};

fn options(provider: Provider, root: &Path) -> silver::IndexOptions {
    silver::IndexOptions {
        providers: [(provider, vec![root.to_path_buf()])].into_iter().collect(),
        roots_are_authoritative: true,
        since_days: None,
    }
}

fn write_codex(path: &Path) {
    fs::write(path, concat!(
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"certified-thread\"}}\n",
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"id\":\"request\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"qdrant filtering\"}]}}\n"
    )).unwrap();
}

#[test]
fn native_rejections_remain_incomplete_across_provider_and_time_scopes_until_repaired() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("history.sqlite3");
    let codex = directory.path().join("codex");
    let claude = directory.path().join("claude");
    fs::create_dir(&codex).unwrap();
    fs::create_dir(&claude).unwrap();
    fs::write(claude.join("session.jsonl"), "{\"type\":\"user\",\"sessionId\":\"claude-thread\",\"uuid\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"postgres pooling\"}}\n").unwrap();
    let bad = codex.join("session.jsonl");
    fs::write(&bad, "{\n").unwrap();
    fs::File::options()
        .write(true)
        .open(&bad)
        .unwrap()
        .set_times(FileTimes::new().set_modified(SystemTime::now() - Duration::from_hours(48)))
        .unwrap();
    let mut all = options(Provider::Codex, &codex);
    all.providers
        .insert(Provider::ClaudeCode, vec![claude.clone()]);
    let mut storage = Storage::open_writer(&database).unwrap();
    let first = ingest(&mut storage, &all).unwrap();
    assert_eq!(first.summary.malformed_records, 1);
    assert_eq!(first.certificate.completeness, Completeness::Incomplete);
    storage.commit_writer().unwrap();
    drop(storage);
    let mut storage = Storage::open_writer(&database).unwrap();
    let unrelated = ingest(&mut storage, &options(Provider::ClaudeCode, &claude)).unwrap();
    assert_eq!(unrelated.summary.malformed_records, 0);
    assert!(!unrelated.is_complete());
    assert_eq!(unrelated.certificate.revision, first.certificate.revision);
    let mut recent = options(Provider::Codex, &codex);
    recent.since_days = Some(1);
    let skipped = ingest(&mut storage, &recent).unwrap();
    assert_eq!(skipped.summary.malformed_records, 0);
    assert!(!skipped.is_complete());
    write_codex(&bad);
    let repaired = ingest(&mut storage, &recent).unwrap();
    assert!(repaired.is_complete());
    assert_eq!(repaired.certificate.rejected_sources, 0);
    assert_eq!(storage.counts().unwrap().messages, 2);
    storage.commit_writer().unwrap();
    drop(storage);
    assert_eq!(
        Storage::open_silver(&database)
            .unwrap()
            .silver_certificate()
            .unwrap(),
        repaired.certificate
    );
}

#[test]
fn deleted_rejection_and_old_interpretation_checkpoint_require_explicit_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("history.sqlite3");
    let root = directory.path().join("codex");
    fs::create_dir(&root).unwrap();
    let source = root.join("session.jsonl");
    write_codex(&source);
    let bad = root.join("bad.jsonl");
    fs::write(&bad, "{\n").unwrap();
    let options = options(Provider::Codex, &root);
    let mut storage = Storage::open_writer(&database).unwrap();
    assert!(!ingest(&mut storage, &options).unwrap().is_complete());
    fs::remove_file(&bad).unwrap();
    let clean = ingest(&mut storage, &options).unwrap();
    assert!(clean.is_complete());
    assert_eq!(clean.summary.checkpoint_skipped_sources, 1);
    storage.commit_writer().unwrap();
    drop(storage);
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute("UPDATE source_checkpoints SET interpretation_version=0", [])
        .unwrap();
    drop(connection);
    let mut storage = Storage::open_writer(&database).unwrap();
    assert_eq!(
        storage.silver_certificate().unwrap().completeness,
        Completeness::Unverified
    );
    let replayed = ingest(&mut storage, &options).unwrap();
    assert!(replayed.is_complete());
    assert_eq!(replayed.summary.checkpoint_skipped_sources, 0);
    assert_eq!(replayed.summary.changed_messages, 0);
    assert_eq!(replayed.certificate.revision, clean.certificate.revision);
}

#[test]
fn a_reread_leaves_sources_outside_the_history_range_certified() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("history.sqlite3");
    let root = directory.path().join("codex");
    fs::create_dir(&root).unwrap();
    let old = root.join("old.jsonl");
    write_codex(&old);
    let all = options(Provider::Codex, &root);
    let mut storage = Storage::open_writer(&database).unwrap();
    assert!(ingest(&mut storage, &all).unwrap().is_complete());
    storage.commit_writer().unwrap();
    drop(storage);
    // The file ages out of a one-day range after it was indexed.
    let aged = SystemTime::now() - Duration::from_hours(72);
    fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_times(FileTimes::new().set_modified(aged))
        .unwrap();
    let mut storage = Storage::open_writer(&database).unwrap();
    assert!(ingest(&mut storage, &all).unwrap().is_complete());
    let mut recent = all.clone();
    recent.since_days = Some(1);
    let cutoff = mash_silver::discovery::cutoff_modified_ns(SystemTime::now(), Some(1)).unwrap();
    storage.reread_sources([Provider::Codex], cutoff).unwrap();
    let reread = ingest(&mut storage, &recent).unwrap();
    assert_eq!(reread.summary.scanned_files, 0);
    assert!(reread.is_complete());
    // Without a range the same request reads the file again.
    storage.reread_sources([Provider::Codex], None).unwrap();
    assert_eq!(
        storage.silver_certificate().unwrap().completeness,
        Completeness::Unverified
    );
    let everything = ingest(&mut storage, &all).unwrap();
    assert_eq!(everything.summary.checkpoint_skipped_sources, 0);
    assert!(everything.is_complete());
}

#[test]
fn a_migration_mark_outside_the_history_range_is_read_again_and_certified() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("history.sqlite3");
    let root = directory.path().join("codex");
    fs::create_dir(&root).unwrap();
    let old = root.join("old.jsonl");
    write_codex(&old);
    let mut recent = options(Provider::Codex, &root);
    let mut storage = Storage::open_writer(&database).unwrap();
    assert!(ingest(&mut storage, &recent).unwrap().is_complete());
    storage.commit_writer().unwrap();
    drop(storage);
    fs::File::options()
        .write(true)
        .open(&old)
        .unwrap()
        .set_times(FileTimes::new().set_modified(SystemTime::now() - Duration::from_hours(72)))
        .unwrap();
    // An upgrade's migration marks every checkpoint, the aged one included,
    // as read by an older release.
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute("UPDATE source_checkpoints SET interpretation_version=0", [])
        .unwrap();
    drop(connection);
    let mut storage = Storage::open_writer(&database).unwrap();
    recent.since_days = Some(1);
    let repaired = ingest(&mut storage, &recent).unwrap();
    assert_eq!(repaired.summary.scanned_files, 1);
    assert_eq!(repaired.summary.checkpoint_skipped_sources, 0);
    assert_eq!(repaired.certificate.completeness, Completeness::Complete);
    // Once current, the aged source is out of range again.
    let next = ingest(&mut storage, &recent).unwrap();
    assert_eq!(next.summary.scanned_files, 0);
    assert_eq!(next.certificate.completeness, Completeness::Complete);
}
