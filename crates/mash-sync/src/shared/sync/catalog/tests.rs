use super::*;

#[test]
fn sync_catalog_rejects_other_schemas_without_modification() {
    for version in [0, SCHEMA_VERSION + 1] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("catalog.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE sentinel(value TEXT); INSERT INTO sentinel VALUES('retained');",
            )
            .unwrap();
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        drop(connection);
        let before = std::fs::read(&path).unwrap();
        let error = Catalog::open(root.path())
            .err()
            .expect("foreign schema rejected");
        assert!(
            error
                .error
                .message
                .contains("unsupported sync catalog schema")
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}

fn manifest(bytes: u64, pieces: &[(u64, u64, &str)]) -> String {
    let entries = pieces
        .iter()
        .map(|(sequence, offset, digest)| wire::ManifestPiece {
            sequence: *sequence,
            decoded_offset: *offset,
            decoded_length: 2,
            digest,
        })
        .collect::<Vec<_>>();
    wire::manifest_digest(bytes, 2, &entries)
}

fn begin(catalog: &Catalog, id: &str, manifest: &str, bytes: u64) -> Operation {
    match catalog
        .begin(&Begin {
            operation_id: id,
            group_id: "group",
            node_id: "node",
            kind: &crate::shared::sync::catalog::Kind::Publication,
            manifest_digest: manifest,
            decoded_bytes: bytes,
            record_count: 2,
            start_json: "{\"type\":\"start\"}",
        })
        .unwrap()
    {
        BeginOutcome::New(operation) | BeginOutcome::Existing(operation) => operation,
    }
}

fn chunk(sequence: u64, offset: u64, digest: &str, encoded: &[u8]) -> StoredChunk {
    StoredChunk {
        sequence,
        decoded_offset: offset,
        decoded_length: 2,
        digest: digest.into(),
        encoded: encoded.to_vec(),
        compression: 1,
    }
}

fn backfill(catalog: &Catalog, id: &str, manifest: &str, total: u64) -> Backfill {
    catalog
        .begin_backfill(&BackfillBegin {
            backfill_id: id,
            group_id: "group",
            node_id: "node",
            database_instance: "instance",
            baseline_revision: 7,
            views_fingerprint: "generation",
            manifest_digest: manifest,
            total_chats: total,
        })
        .unwrap();
    catalog.backfill(id, "group", "node").unwrap()
}

/// Freshness requests used to open the catalog again and restamp its
/// version: a commit that made SQLite refuse a transfer's next piece.
#[test]
fn opening_an_existing_catalog_commits_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let version = |catalog: &Catalog| -> i64 {
        catalog
            .connection
            .pragma_query_value(None, "data_version", |row| row.get(0))
            .unwrap()
    };
    let before = version(&catalog);
    drop(Catalog::open(directory.path()).unwrap());
    assert_eq!(version(&catalog), before);
}

#[test]
fn retiring_a_node_forgets_its_transfers_and_backfills_but_no_other_nodes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = "a".repeat(64);
    begin(&catalog, "mine", &manifest(2, &[(0, 0, &digest)]), 2);
    backfill(&catalog, "backfill", &digest, 2);
    let other = Begin {
        operation_id: "theirs",
        group_id: "group",
        node_id: "other",
        kind: &crate::shared::sync::catalog::Kind::Publication,
        manifest_digest: &manifest(2, &[(0, 0, &digest)]),
        decoded_bytes: 2,
        record_count: 2,
        start_json: "{\"type\":\"start\"}",
    };
    catalog.begin(&other).unwrap();
    // A spooled piece directory, as an interrupted upload leaves behind.
    let spool = catalog.piece_directory("mine");
    std::fs::create_dir_all(&spool).unwrap();
    std::fs::write(spool.join("0.zst"), b"piece").unwrap();

    assert_eq!(catalog.retire("group", "node").unwrap(), 1);
    let count = |table: &str, node: &str| -> i64 {
        catalog
            .connection
            .query_row(
                &format!("SELECT count(*) FROM {table} WHERE node_id = ?1"),
                params![node],
                |row| row.get(0),
            )
            .unwrap()
    };
    assert_eq!(count("operations", "node"), 0);
    assert_eq!(count("backfills", "node"), 0);
    assert_eq!(count("operations", "other"), 1);
    assert!(!spool.exists());
    // Nothing left to retire is not an error.
    assert_eq!(catalog.retire("group", "node").unwrap(), 0);
}

#[test]
fn backfill_requires_the_declared_members_and_complete_coverage() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let manifest = "a".repeat(64);
    let opened = backfill(&catalog, "backfill", &manifest, 2);
    assert_eq!(opened.state, "receiving_manifest");
    assert_eq!(opened.covered, 0);

    // A manifest with the wrong member count cannot be installed.
    assert!(
        catalog
            .store_manifest(
                "backfill",
                "group",
                "node",
                &[wire::BackfillMember {
                    chat_id: "a".into(),
                    digest: "b".repeat(64),
                }],
            )
            .is_err()
    );

    let members = [
        wire::BackfillMember {
            chat_id: "a".into(),
            digest: "b".repeat(64),
        },
        wire::BackfillMember {
            chat_id: "b".into(),
            digest: "c".repeat(64),
        },
    ];
    let ready = catalog
        .store_manifest("backfill", "group", "node", &members)
        .unwrap();
    assert_eq!(ready.state, "ready");
    // Repeating the same manifest is idempotent.
    assert_eq!(
        catalog
            .store_manifest("backfill", "group", "node", &members)
            .unwrap(),
        ready
    );
    // Finish before coverage fails closed.
    assert!(
        catalog
            .finish_backfill("backfill", "group", "node", 7, &manifest)
            .is_err()
    );

    // A digest that does not match the frozen member is not evidence.
    let stale = catalog
        .cover("backfill", "group", "node", &[("a".into(), "d".repeat(64))])
        .unwrap();
    assert_eq!(stale, 0);

    let covered = catalog
        .cover(
            "backfill",
            "group",
            "node",
            &[
                ("a".into(), "b".repeat(64)),
                ("extra".into(), "e".repeat(64)),
            ],
        )
        .unwrap();
    assert_eq!(covered, 1, "unknown live members are not baseline evidence");
    assert!(
        catalog
            .finish_backfill("backfill", "group", "node", 7, &manifest)
            .is_err(),
        "one uncovered member still blocks completion"
    );

    catalog
        .cover("backfill", "group", "node", &[("b".into(), "c".repeat(64))])
        .unwrap();
    let complete = catalog
        .finish_backfill("backfill", "group", "node", 7, &manifest)
        .unwrap();
    assert_eq!(complete.state, "complete");
    assert_eq!(complete.covered, 2);
}

#[test]
fn live_publications_supersede_baseline_members() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let manifest = "a".repeat(64);
    catalog
        .begin_backfill(&BackfillBegin {
            backfill_id: "backfill",
            group_id: "group",
            node_id: "node",
            database_instance: "instance",
            baseline_revision: 5,
            views_fingerprint: "generation",
            manifest_digest: &manifest,
            total_chats: 2,
        })
        .unwrap();
    let members = [
        wire::BackfillMember {
            chat_id: "a".into(),
            digest: "b".repeat(64),
        },
        wire::BackfillMember {
            chat_id: "b".into(),
            digest: "c".repeat(64),
        },
    ];
    catalog
        .store_manifest("backfill", "group", "node", &members)
        .unwrap();

    // A live record above the frozen baseline covers its baseline member.
    catalog
        .record_live("backfill", "group", "node", &["a".into()], 6)
        .unwrap();
    let backfill = catalog.backfill("backfill", "group", "node").unwrap();
    assert_eq!(backfill.covered, 1);
    assert_eq!(
        catalog.superseded("backfill", "group", "node", 5).unwrap(),
        vec!["a".to_owned()]
    );
    // A batch at the live revision already contains the newer state.
    assert_eq!(
        catalog.superseded("backfill", "group", "node", 6).unwrap(),
        Vec::<String>::new()
    );
    // A live record at or below the baseline proves nothing.
    catalog
        .record_live("backfill", "group", "node", &["b".into()], 5)
        .unwrap();
    assert_eq!(
        catalog.superseded("backfill", "group", "node", 5).unwrap(),
        vec!["a".to_owned()]
    );
    assert_eq!(
        catalog
            .backfill("backfill", "group", "node")
            .unwrap()
            .covered,
        1
    );
}

#[test]
fn backfill_identity_is_immutable_across_sessions() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let manifest = "a".repeat(64);
    backfill(&catalog, "backfill", &manifest, 1);
    // Same identity with a different baseline revision conflicts.
    assert!(
        catalog
            .begin_backfill(&BackfillBegin {
                backfill_id: "backfill",
                group_id: "group",
                node_id: "node",
                database_instance: "instance",
                baseline_revision: 8,
                views_fingerprint: "generation",
                manifest_digest: &manifest,
                total_chats: 1,
            })
            .is_err()
    );
    // Another node cannot read this session.
    assert!(catalog.backfill("backfill", "group", "other").is_err());
}

/// Model-based property test: random operation sequences must preserve the
/// catalog invariants the TLA+ models state. Independent expected state.
#[expect(
    clippy::too_many_lines,
    reason = "one model-based sequence applies and checks every catalog transition"
)]
#[test]
fn random_operation_sequences_preserve_catalog_invariants() {
    use proptest::prelude::*;

    #[derive(Clone, Copy, Debug)]
    enum Op {
        Begin,
        AppendNext,
        DuplicateLast,
        WrongDigest,
        Gap,
        Finish,
        Expire,
        Reopen,
        Cancel,
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop::sample::select(vec![
            Op::Begin,
            Op::AppendNext,
            Op::AppendNext,
            Op::DuplicateLast,
            Op::WrongDigest,
            Op::Gap,
            Op::Finish,
            Op::Expire,
            Op::Reopen,
            Op::Cancel,
        ])
    }

    proptest!(|(ops in prop::collection::vec(op_strategy(), 1..40))| {
        let directory = tempfile::tempdir().unwrap();
        let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
        let mut catalog = Catalog::open(directory.path()).unwrap();
        let mut begun = false;
        let mut next = 0u64;
        for op in ops {
            match op {
                Op::Begin if !begun => {
                    begin(&catalog, "op", &digest, 4);
                    begun = true;
                }
                Op::Begin => {
                    // Re-begin of the same bytes is idempotent.
                    begin(&catalog, "op", &digest, 4);
                }
                Op::AppendNext if begun && next < 2 && receiving(&catalog) => {
                    let seq = next;
                    let offset = seq * 2;
                    let digest = if seq == 0 { "a" } else { "b" };
                    let bytes: &[u8] = if seq == 0 { b"first" } else { b"second" };
                    catalog
                        .append("op", "group", "node", &chunk(seq, offset, digest, bytes))
                        .unwrap();
                    next += 1;
                }
                Op::DuplicateLast if begun && next > 0 && receiving(&catalog) => {
                    let seq = next - 1;
                    let digest = if seq == 0 { "a" } else { "b" };
                    let bytes: &[u8] = if seq == 0 { b"first" } else { b"second" };
                    catalog
                        .append("op", "group", "node", &chunk(seq, seq * 2, digest, bytes))
                        .unwrap();
                }
                // A retry of a retained sequence with different bytes.
                Op::WrongDigest if begun && next > 0 && receiving(&catalog) => {
                    let seq = next - 1;
                    let rejected = catalog.append(
                        "op",
                        "group",
                        "node",
                        &chunk(seq, seq * 2, "wrong", b"xx"),
                    );
                    assert!(rejected.is_err(), "changed retry bytes must fail closed");
                }
                // Skipping the next expected sequence.
                Op::Gap if begun && next == 0 && receiving(&catalog) => {
                    let rejected =
                        catalog.append("op", "group", "node", &chunk(1, 2, "b", b"second"));
                    assert!(rejected.is_err(), "gaps must fail closed");
                }
                Op::Finish if begun && next == 2 && receiving(&catalog) => {
                    catalog.finish("op", "group", "node", &digest).unwrap();
                }
                Op::Expire => {
                    catalog.expire_incomplete(now() + 1).unwrap();
                }
                Op::Reopen => {
                    catalog = Catalog::open(directory.path()).unwrap();
                }
                Op::Cancel if begun => {
                    catalog.cancel("op").unwrap();
                }
                _ => {}
            }

            if !begun {
                continue;
            }
            // Invariants: checkpoint equals retained pieces, and terminal
            // states never keep a spool.
            let operation = catalog.select("op", "group", "node").unwrap();
            assert_eq!(
                operation.next_sequence, next,
                "checkpoint must equal the retained pieces"
            );
            assert_eq!(operation.decoded_offset, next * 2);
            let spool = directory
                .path()
                .join("pieces")
                .join(blake3::hash(b"op").to_hex().as_str());
            if matches!(operation.state, State::Expired | State::Cancelled) {
                assert!(!spool.exists(), "terminal states must release their spool");
                assert!(
                    !catalog
                        .recoverable()
                        .unwrap()
                        .iter()
                        .any(|row| row.operation_id == "op")
                );
            }
            if operation.state == State::Published {
                assert!(operation.revision.is_some());
            }
        }
    });
}

fn receiving(catalog: &Catalog) -> bool {
    catalog
        .select("op", "group", "node")
        .is_ok_and(|operation| operation.state == State::Receiving)
}

#[test]
fn spool_quota_rejects_a_piece_and_keeps_the_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    // Two pieces of two encoded bytes fit in three bytes only once.
    let catalog = Catalog::open_with_limit(directory.path(), 3).unwrap();
    begin(&catalog, "op", &digest, 4);
    catalog
        .append("op", "group", "node", &chunk(0, 0, "a", b"xx"))
        .unwrap();
    let rejected = catalog.append("op", "group", "node", &chunk(1, 2, "b", b"yy"));
    assert!(
        rejected.is_err(),
        "the spool ceiling rejects the second piece"
    );
    assert_eq!(
        catalog.select("op", "group", "node").unwrap().next_sequence,
        1
    );
    let count: i64 = catalog
        .connection
        .query_row(
            "SELECT COUNT(*) FROM pieces WHERE operation_id = 'op' AND sequence = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    assert!(
        !directory
            .path()
            .join("pieces")
            .join(blake3::hash(b"op").to_hex().as_str())
            .join("1.zst")
            .exists(),
        "a rejected piece must not leave a file behind"
    );
}

#[cfg(unix)]
#[test]
fn an_unwritable_spool_reports_an_io_failure_without_advancing() {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = tempfile::tempdir().unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    let catalog = Catalog::open(directory.path()).unwrap();
    begin(&catalog, "op", &digest, 4);
    let spool = directory
        .path()
        .join("pieces")
        .join(blake3::hash(b"op").to_hex().as_str());
    std::fs::create_dir_all(&spool).unwrap();
    std::fs::set_permissions(&spool, std::fs::Permissions::from_mode(0o500)).unwrap();
    let rejected = catalog.append("op", "group", "node", &chunk(0, 0, "a", b"xx"));
    std::fs::set_permissions(&spool, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(rejected.is_err(), "an unwritable spool fails closed");
    assert_eq!(
        catalog.select("op", "group", "node").unwrap().next_sequence,
        0
    );
}
#[test]
fn an_expired_transfer_rejects_further_pieces_from_another_connection() {
    let directory = tempfile::tempdir().unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    let writer = Catalog::open(directory.path()).unwrap();
    begin(&writer, "op", &digest, 4);
    writer
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();

    // A second connection, like another /v1/sync session, expires the row.
    let sweeper = Catalog::open(directory.path()).unwrap();
    assert_eq!(sweeper.expire_incomplete(now() + 1).unwrap(), 1);
    assert_eq!(
        writer.select("op", "group", "node").unwrap().state,
        State::Expired
    );

    // The writer's next piece must fail closed instead of acknowledging
    // bytes that the sweep already deleted.
    let rejected = writer.append("op", "group", "node", &chunk(1, 2, "b", b"second"));
    assert!(rejected.is_err(), "expired transfers cannot accept pieces");
    assert_eq!(
        writer.select("op", "group", "node").unwrap().decoded_offset,
        2
    );

    // No piece row or file survives for the rejected sequence.
    let count: i64 = writer
        .connection
        .query_row(
            "SELECT COUNT(*) FROM pieces WHERE operation_id = 'op' AND sequence = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    assert!(
        !directory
            .path()
            .join("pieces")
            .join(blake3::hash(b"op").to_hex().as_str())
            .join("1.zst")
            .exists()
    );
}

#[test]
fn abandoned_receiving_transfers_expire_and_release_their_spool() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    begin(&catalog, "abandoned", &digest, 4);
    catalog
        .append("abandoned", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    // A completed receipt is not an abandoned receiving transfer.
    begin(&catalog, "queued", &digest, 4);
    for (sequence, offset, digest) in [(0u64, 0u64, "a"), (1, 2, "b")] {
        catalog
            .append(
                "queued",
                "group",
                "node",
                &chunk(sequence, offset, digest, b"xx"),
            )
            .unwrap();
    }
    catalog.finish("queued", "group", "node", &digest).unwrap();

    // A cutoff before any progress expires nothing.
    assert_eq!(catalog.expire_incomplete(0).unwrap(), 0);
    assert_eq!(
        catalog.select("abandoned", "group", "node").unwrap().state,
        State::Receiving
    );

    // A cutoff after the last update expires only the receiving row.
    assert_eq!(catalog.expire_incomplete(now() + 1).unwrap(), 1);
    assert_eq!(
        catalog.select("abandoned", "group", "node").unwrap().state,
        State::Expired
    );
    assert_eq!(
        catalog.select("queued", "group", "node").unwrap().state,
        State::QueuedForApply
    );
    // The spool directory is gone, so a reconnect cannot resume it.
    assert!(
        !directory
            .path()
            .join("pieces")
            .join(blake3::hash(b"abandoned").to_hex().as_str())
            .exists()
    );
    assert!(
        !catalog
            .recoverable()
            .unwrap()
            .iter()
            .any(|row| row.operation_id == "abandoned")
    );
}

#[test]
fn restart_preserves_checkpoint_and_exact_retained_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    begin(&catalog, "op", &digest, 4);
    let operation = catalog
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    assert_eq!(operation.next_sequence, 1);
    assert_eq!(operation.decoded_offset, 2);
    drop(catalog);

    let reopened = Catalog::open(directory.path()).unwrap();
    let operation = reopened.select("op", "group", "node").unwrap();
    assert_eq!(operation.next_sequence, 1);
    assert_eq!(operation.decoded_offset, 2);
    // Replaying the retained piece with the same digest is idempotent.
    let operation = reopened
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    assert_eq!(operation.next_sequence, 1);
    // The next piece continues from the persisted offset.
    let operation = reopened
        .append("op", "group", "node", &chunk(1, 2, "b", b"second"))
        .unwrap();
    assert_eq!(operation.next_sequence, 2);
    assert_eq!(operation.decoded_offset, 4);
    assert!(reopened.finish("op", "group", "node", &digest).is_ok());
}

#[test]
fn conflicting_retries_gaps_and_foreign_owners_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = manifest(6, &[(0, 0, "a"), (1, 2, "b"), (2, 4, "c")]);
    begin(&catalog, "op", &digest, 6);
    catalog
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    assert!(
        catalog
            .append("op", "group", "node", &chunk(0, 0, "b", b"other"))
            .is_err()
    );
    assert!(
        catalog
            .append("op", "group", "node", &chunk(2, 4, "c", b"third"))
            .is_err()
    );
    assert!(
        catalog
            .append("op", "group", "other", &chunk(1, 2, "c", b"third"))
            .is_err()
    );
    assert!(
        catalog
            .begin(&Begin {
                operation_id: "op",
                group_id: "group",
                node_id: "node",
                kind: &crate::shared::sync::catalog::Kind::Publication,
                manifest_digest: "changed",
                decoded_bytes: 6,
                record_count: 2,
                start_json: "{}",
            })
            .is_err()
    );
}

#[test]
fn incomplete_or_tampered_receipts_never_queue_for_apply() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    begin(&catalog, "op", &digest, 4);
    catalog
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    assert!(catalog.finish("op", "group", "node", &digest).is_err());
    catalog
        .append("op", "group", "node", &chunk(1, 2, "b", b"second"))
        .unwrap();
    assert!(catalog.finish("op", "group", "node", "other").is_err());
    let (operation, pieces) = catalog.finish("op", "group", "node", &digest).unwrap();
    assert_eq!(operation.state, State::QueuedForApply);
    assert_eq!(pieces.len(), 2);
}

#[test]
fn published_receipts_survive_restart_without_retaining_piece_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = Catalog::open(directory.path()).unwrap();
    let digest = manifest(4, &[(0, 0, "a"), (1, 2, "b")]);
    begin(&catalog, "op", &digest, 4);
    catalog
        .append("op", "group", "node", &chunk(0, 0, "a", b"first"))
        .unwrap();
    catalog
        .append("op", "group", "node", &chunk(1, 2, "b", b"second"))
        .unwrap();
    catalog.finish("op", "group", "node", &digest).unwrap();
    catalog.published("op", 9, "receipt").unwrap();
    drop(catalog);
    let reopened = Catalog::open(directory.path()).unwrap();
    let operation = reopened.select("op", "group", "node").unwrap();
    assert_eq!(operation.state, State::Published);
    assert_eq!(operation.revision, Some(9));
    assert_eq!(operation.receipt_digest.as_deref(), Some("receipt"));
    assert!(
        reopened
            .append("op", "group", "node", &chunk(2, 4, "c", b"third"))
            .is_err()
    );
    assert!(reopened.finish("op", "group", "node", &digest).is_err());
}
