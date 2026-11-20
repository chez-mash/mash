//! Progressive backfill: a baseline's manifest, which chats it covers, and
//! applying it without replacing what the node published since.
use super::{
    Accepted, App, AppError, BackfillComplete, CatalogHandle, CommitState, Event, Failed,
    FinishBackfill, Identity, ManifestStart, OwnedBegin, Progress, Sink, begin_operation, catalog,
    protocol, reject, send, server_frame, wire,
};

#[expect(
    clippy::too_many_lines,
    reason = "baseline identity, staleness and manifest checks before one durable insert"
)]
pub(super) async fn handle_begin_backfill(
    sink: &mut Sink,
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation_id: &str,
    begin: wire::BeginBackfill,
) -> Result<(), AppError> {
    let valid = !begin.backfill_id.is_empty()
        && begin.backfill_id.len() <= 4096
        && !begin.database_instance.is_empty()
        && begin.baseline_revision >= 0
        && !begin.views_fingerprint.is_empty()
        && begin.manifest_digest.len() == 64
        && begin.manifest_digest.bytes().all(|b| b.is_ascii_hexdigit())
        && begin.total_chats > 0
        && begin.decoded_bytes <= app.store.max_node_bytes();
    if !valid {
        return reject(
            sink,
            operation_id,
            "begin",
            "invalid_backfill",
            "backfill metadata or manifest is invalid",
        )
        .await;
    }
    let Ok(records) = u32::try_from(begin.total_chats) else {
        return reject(
            sink,
            operation_id,
            "begin",
            "invalid_backfill",
            "backfill has too many baseline members",
        )
        .await;
    };
    // A baseline frozen at a revision the node has already moved past cannot be
    // merged safely: silver no longer has per-chat ordering for it.
    let head = match app.store.head_revision(identity).await {
        Ok(head) => head,
        Err(error) => {
            return reject(
                sink,
                operation_id,
                "begin",
                "invalid_backfill",
                &error.error.message,
            )
            .await;
        }
    };
    if head > begin.baseline_revision {
        return reject(
            sink,
            operation_id,
            "begin",
            "backfill_baseline_stale",
            "baseline revision is older than the node's current revision",
        )
        .await;
    }
    let backfill_id = begin.backfill_id.clone();
    let database_instance = begin.database_instance.clone();
    let views_fingerprint = begin.views_fingerprint.clone();
    let manifest_digest = begin.manifest_digest.clone();
    let baseline_revision = begin.baseline_revision;
    let total_chats = begin.total_chats;
    let start_json = serde_json::json!({
        "backfill_id": backfill_id,
        "database_instance": database_instance,
        "baseline_revision": baseline_revision,
        "views_fingerprint": views_fingerprint,
    })
    .to_string();
    let group = identity.group_id.clone();
    let node = identity.node_id.clone();
    let (seed_id, seed_group, seed_node) = (backfill_id.clone(), group.clone(), node.clone());
    let (seed_instance, seed_fingerprint, seed_manifest) = (
        database_instance.clone(),
        views_fingerprint.clone(),
        manifest_digest.clone(),
    );
    let opened = catalog(handle, move |catalog| {
        catalog.begin_backfill(&catalog::BackfillBegin {
            backfill_id: &seed_id,
            group_id: &seed_group,
            node_id: &seed_node,
            database_instance: &seed_instance,
            baseline_revision,
            views_fingerprint: &seed_fingerprint,
            manifest_digest: &seed_manifest,
            total_chats,
        })
    })
    .await;
    if let Err(error) = opened {
        return send(
            sink,
            operation_id,
            server_frame::Body::Rejected(Failed {
                phase: "begin".into(),
                code: "begin_rejected".into(),
                commit_state: CommitState::NotCommitted as i32,
                retryable: error.error.retryable,
                recovery_action: "start a new backfill identity".into(),
                kind: error.error.kind.name(),
                message: error.error.message,
            }),
        )
        .await;
    }
    let owned = OwnedBegin {
        operation_id: operation_id.to_owned(),
        group_id: group,
        node_id: node,
        kind: "backfill_manifest".into(),
        manifest_digest,
        decoded_bytes: begin.decoded_bytes,
        record_count: records,
        start_json,
    };
    begin_operation(sink, handle, operation_id, owned).await
}

/// Baseline evidence carried by one committed backfill batch. Member digests
/// are transfer-level content identities, not canonical silver fingerprints.
pub(super) fn covered_members(events: &[Event]) -> Result<Vec<(String, String)>, AppError> {
    let mut members = Vec::new();
    for event in events {
        match event {
            Event::Batch(protocol::Batch::Chats { records }) => {
                for record in records {
                    members.push((record.id.clone(), wire::member_digest(record)?));
                }
            }
            Event::Batch(protocol::Batch::Tombstones { records }) => {
                for record in records {
                    members.push((
                        record.chat_id.clone(),
                        wire::tombstone_member_digest(&record.chat_id),
                    ));
                }
            }
            Event::Batch(protocol::Batch::Records { .. } | protocol::Batch::Messages { .. })
            | Event::Remove { .. }
            | Event::Amend(_)
            | Event::Start(_)
            | Event::Finish => {}
        }
    }
    Ok(members)
}

/// Outcome of applying per-chat ordering to a baseline batch.
pub(super) enum BackfillPreparation {
    /// A newer live publication covers every record in this batch.
    Superseded {
        members: Vec<(String, String)>,
        revision: i64,
    },
    /// Some chats are still baseline-only; apply them at the current
    /// node revision so silver's node-level ordering stays monotonic.
    Apply {
        start: protocol::Start,
        events: Vec<Event>,
        members: Vec<(String, String)>,
    },
}

/// Drop records superseded by a newer live publication and rebase the rest.
pub(super) async fn prepare_backfill(
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    backfill_id: &str,
    start: protocol::Start,
    events: Vec<Event>,
) -> Result<BackfillPreparation, AppError> {
    let members = covered_members(&events)?;
    let revision = start.revision.unwrap_or_default();
    let group = identity.group_id.clone();
    let node = identity.node_id.clone();
    let backfill = backfill_id.to_owned();
    let stale = catalog(handle, move |catalog| {
        catalog.superseded(&backfill, &group, &node, revision)
    })
    .await?;
    let events = filter_superseded(events, &stale);
    let head = app.store.head_revision(identity).await?;
    if events.is_empty() {
        return Ok(BackfillPreparation::Superseded {
            members,
            revision: head,
        });
    }
    let mut start = start;
    // Baseline state merges into current history; it is not a node revision
    // rollback. Per-chat ordering already removed stale records.
    start.revision = Some(head.saturating_add(1));
    start.base_revision = Some(head);
    Ok(BackfillPreparation::Apply {
        start,
        events,
        members,
    })
}

/// Remove chat records, their messages and their tombstones for
/// chats a newer live publication already owns.
pub(super) fn filter_superseded(events: Vec<Event>, stale: &[String]) -> Vec<Event> {
    let stale = stale
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    if stale.is_empty() {
        return events;
    }
    events
        .into_iter()
        .filter_map(|event| match event {
            Event::Batch(protocol::Batch::Chats { mut records }) => {
                records.retain(|record| !stale.contains(record.id.as_str()));
                (!records.is_empty()).then_some(Event::Batch(protocol::Batch::Chats { records }))
            }
            Event::Batch(protocol::Batch::Messages { mut records }) => {
                records.retain(|record| !stale.contains(record.chat_id.as_str()));
                (!records.is_empty()).then_some(Event::Batch(protocol::Batch::Messages { records }))
            }
            Event::Batch(protocol::Batch::Tombstones { mut records }) => {
                records.retain(|record| !stale.contains(record.chat_id.as_str()));
                (!records.is_empty())
                    .then_some(Event::Batch(protocol::Batch::Tombstones { records }))
            }
            other => Some(other),
        })
        .collect()
}

/// Decode, install and durably record a baseline manifest. Shared by the
/// socket handler and startup recovery.
pub(super) async fn apply_manifest(
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation: &catalog::Operation,
    pieces: &[wire::Piece],
) -> Result<catalog::Backfill, AppError> {
    let max_decoded = usize::try_from(app.store.max_node_bytes()).unwrap_or(usize::MAX);
    let bytes = wire::decode_bytes(pieces, max_decoded)
        .map_err(|_| AppError::usage("backfill manifest bytes were not valid"))?;
    let limit = usize::try_from(operation.record_count).unwrap_or(usize::MAX);
    let members = wire::decode_backfill_manifest(&bytes, limit)
        .map_err(|_| AppError::usage("backfill manifest did not match its member count"))?;
    let start: ManifestStart = serde_json::from_str(&operation.start_json)
        .map_err(|_| AppError::internal("backfill manifest metadata is not valid"))?;
    let group = identity.group_id.clone();
    let node = identity.node_id.clone();
    let backfill_id = start.backfill_id;
    let stored = catalog(handle, move |catalog| {
        catalog.store_manifest(&backfill_id, &group, &node, &members)
    })
    .await?;
    let operation_id = operation.operation_id.clone();
    let digest = operation.manifest_digest.clone();
    let _ = catalog(handle, move |catalog| {
        catalog.completed(&operation_id, &digest)
    })
    .await;
    Ok(stored)
}

pub(super) async fn finish_manifest(
    sink: &mut Sink,
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation_id: &str,
    operation: &catalog::Operation,
    pieces: &[wire::Piece],
) -> Result<(), AppError> {
    match apply_manifest(app, identity, handle, operation, pieces).await {
        Ok(backfill) => {
            send(
                sink,
                operation_id,
                server_frame::Body::Progress(Progress {
                    phase: "baseline_manifest".into(),
                    completed: backfill.total_chats,
                    total: backfill.total_chats,
                }),
            )
            .await?;
            send(
                sink,
                operation_id,
                server_frame::Body::Accepted(Accepted { credit_bytes: 0 }),
            )
            .await
        }
        Err(error) => {
            reject(
                sink,
                operation_id,
                "validating",
                "invalid_manifest",
                &error.error.message,
            )
            .await
        }
    }
}

pub(super) async fn handle_finish_backfill(
    sink: &mut Sink,
    handle: &CatalogHandle,
    identity: &Identity,
    operation_id: &str,
    finish: FinishBackfill,
) -> Result<(), AppError> {
    let group = identity.group_id.clone();
    let node = identity.node_id.clone();
    let result = catalog(handle, move |catalog| {
        catalog.finish_backfill(
            &finish.backfill_id,
            &group,
            &node,
            finish.baseline_revision,
            &finish.manifest_digest,
        )
    })
    .await;
    match result {
        Ok(backfill) => {
            send(
                sink,
                operation_id,
                server_frame::Body::BackfillComplete(BackfillComplete {
                    backfill_id: backfill.backfill_id,
                    covered_chats: backfill.covered,
                }),
            )
            .await
        }
        Err(error) => {
            send(
                sink,
                operation_id,
                server_frame::Body::Rejected(Failed {
                    phase: "backfill".into(),
                    code: "backfill_incomplete".into(),
                    commit_state: CommitState::NotCommitted as i32,
                    retryable: true,
                    recovery_action: "transfer remaining baseline members before finishing".into(),
                    kind: error.error.kind.name(),
                    message: error.error.message,
                }),
            )
            .await
        }
    }
}
