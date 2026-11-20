//! Planning a progressive backfill: grouping a spool's chats into bounded
//! batches, their manifest, and where a resumed baseline starts again.
use super::{AppError, Event, protocol, read_events, wire};

/// Progressive-backfill manifest membership derived from one frozen batch.
pub(super) fn backfill_members(events: &[Event]) -> Result<Vec<wire::BackfillMember>, AppError> {
    let mut members = std::collections::BTreeMap::new();
    for event in events {
        match event {
            Event::Batch(protocol::Batch::Chats { records }) => {
                for record in records {
                    members.insert(record.id.clone(), wire::member_digest(record)?);
                }
            }
            Event::Batch(protocol::Batch::Tombstones { records }) => {
                for record in records {
                    members.insert(
                        record.chat_id.clone(),
                        wire::tombstone_member_digest(&record.chat_id),
                    );
                }
            }
            Event::Batch(protocol::Batch::Records { .. } | protocol::Batch::Messages { .. })
            | Event::Remove { .. }
            | Event::Amend(_)
            | Event::Start(_)
            | Event::Finish => {}
        }
    }
    Ok(members
        .into_iter()
        .map(|(chat_id, digest)| wire::BackfillMember { chat_id, digest })
        .collect())
}

/// Encode, compress and digest a baseline manifest as one transfer piece.
pub(super) fn encode_backfill_manifest(
    members: &[wire::BackfillMember],
) -> Result<(wire::Piece, String, u64), AppError> {
    use prost::Message as _;
    let mut bytes = Vec::new();
    for member in members {
        member
            .encode_length_delimited(&mut bytes)
            .map_err(|_| AppError::internal("encode backfill manifest"))?;
    }
    if bytes.len() > wire::DEFAULT_MAX_DECODED_BYTES {
        return Err(AppError::usage("backfill manifest exceeds its size limit"));
    }
    let encoded = wire::compress(&bytes)?;
    let piece = wire::Piece {
        sequence: 0,
        decoded_offset: 0,
        decoded_length: bytes.len() as u64,
        digest: blake3::hash(&bytes).to_hex().to_string(),
        encoded,
        compression: wire::Compression::Zstd as i32,
    };
    let records = u32::try_from(members.len())
        .map_err(|_| AppError::usage("backfill manifest has too many members"))?;
    let entry = wire::ManifestPiece {
        sequence: 0,
        decoded_offset: 0,
        decoded_length: piece.decoded_length,
        digest: &piece.digest,
    };
    let digest = wire::manifest_digest(piece.decoded_length, records, &[entry]);
    Ok((piece, digest, bytes.len() as u64))
}

/// How many chats and decoded bytes one baseline batch may carry.
#[derive(Clone, Copy, Debug)]
pub struct BackfillLimits {
    pub chats: usize,
    pub decoded_bytes: u64,
}

impl Default for BackfillLimits {
    fn default() -> Self {
        // Baseline commits are apply-bound: fewer, larger batches amortize the
        // per-operation transaction, fsync, Qdrant flush and certification.
        // The cost is coarser resume granularity.
        Self {
            chats: 512,
            decoded_bytes: 4 * 1024 * 1024,
        }
    }
}

/// One independently committed baseline batch.
#[derive(Clone)]
pub struct PlannedBatch {
    pub encoded: wire::EncodedBatch,
    pub members: Vec<wire::BackfillMember>,
}

impl PlannedBatch {
    pub fn digest(&self) -> &str {
        &self.encoded.manifest_digest
    }
}

/// A frozen baseline split into commits, with the membership manifest that
/// proves completion.
pub struct BackfillPlan {
    pub start: protocol::Start,
    pub baseline_revision: i64,
    pub members: Vec<wire::BackfillMember>,
    pub manifest_piece: wire::Piece,
    pub manifest_digest: String,
    pub manifest_bytes: u64,
    pub batches: Vec<PlannedBatch>,
    pub record_batch: Option<wire::EncodedBatch>,
}

/// What a previous attempt already completed for this baseline.
pub struct BackfillResume<'a> {
    pub completed: &'a std::collections::BTreeSet<String>,
    pub last_revision: i64,
}

/// A batch that just committed, with a gate the sender waits on before
/// continuing. Tests use the gate to observe progressive visibility.
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "progress gate is consumed by tests and future operator surfaces"
    )
)]
pub struct BackfillProgress {
    pub index: usize,
    pub total: usize,
    pub revision: i64,
    pub acknowledge: tokio::sync::oneshot::Sender<()>,
}

pub(super) struct Group {
    id: String,
    updated_at: i64,
    events: Vec<Event>,
}

pub(super) fn entry<'a>(
    groups: &'a mut std::collections::BTreeMap<String, Group>,
    id: &str,
) -> &'a mut Group {
    groups.entry(id.to_owned()).or_insert_with(|| Group {
        id: id.to_owned(),
        updated_at: 0,
        events: Vec::new(),
    })
}

/// Split a frozen batch into independently publishable commits: whole
/// chats, most-recently-updated first, packed to the byte and
/// chat targets. A chat is never split across batches.
pub fn plan_backfill(
    start: protocol::Start,
    events: Vec<Event>,
    limits: BackfillLimits,
) -> Result<BackfillPlan, AppError> {
    plan_backfill_bounded(start, events, limits, wire::DEFAULT_MAX_DECODED_BYTES)
}

#[expect(
    clippy::too_many_lines,
    reason = "grouping, ordering, packing and manifest assembly in one pure planner"
)]
pub(super) fn plan_backfill_bounded(
    start: protocol::Start,
    events: Vec<Event>,
    limits: BackfillLimits,
    record_bytes: usize,
) -> Result<BackfillPlan, AppError> {
    let baseline_revision = start
        .revision
        .ok_or_else(|| AppError::usage("a backfill batch requires a frozen revision"))?;
    if start.snapshot {
        return Err(AppError::usage(
            "a backfill batch is not a snapshot; send deltas from the frozen baseline",
        ));
    }

    let mut groups: std::collections::BTreeMap<String, Group> = std::collections::BTreeMap::new();
    let mut record_events = Vec::new();
    for event in events {
        match event {
            Event::Batch(protocol::Batch::Records { records }) => {
                record_events.push(Event::Batch(protocol::Batch::Records { records }));
            }
            Event::Batch(protocol::Batch::Chats { records }) => {
                for record in records {
                    let updated_at = record.updated_at.unwrap_or(0);
                    let group = entry(&mut groups, &record.id);
                    group.updated_at = group.updated_at.max(updated_at);
                    group.events.push(Event::Batch(protocol::Batch::Chats {
                        records: vec![record],
                    }));
                }
            }
            Event::Batch(protocol::Batch::Messages { records }) => {
                for record in records {
                    entry(&mut groups, &record.chat_id)
                        .events
                        .push(Event::Batch(protocol::Batch::Messages {
                            records: vec![record],
                        }));
                }
            }
            Event::Batch(protocol::Batch::Tombstones { records }) => {
                for record in records {
                    entry(&mut groups, &record.chat_id)
                        .events
                        .push(Event::Batch(protocol::Batch::Tombstones {
                            records: vec![record],
                        }));
                }
            }
            Event::Remove { id } => entry(&mut groups, &id).events.push(Event::Remove { id }),
            // A baseline captures whole chats; amendments belong to deltas.
            Event::Amend(_) => {
                return Err(AppError::usage("a backfill batch cannot amend chats"));
            }
            Event::Start(_) | Event::Finish => {}
        }
    }
    // Preparation has already applied publication redaction to every event.
    // Validate the final record transfer before admitting any chat uploads;
    // retain compressed pieces rather than a second copy of its typed rows.
    let record_batch = (!record_events.is_empty())
        .then(|| wire::encode_publication(&record_events, record_bytes))
        .transpose()?;
    drop(record_events);
    if groups.is_empty() {
        return Err(AppError::usage(
            "a backfill batch must carry at least one chat",
        ));
    }
    let mut ordered = groups.into_values().collect::<Vec<_>>();
    // Newest activity first; the identifier breaks ties deterministically.
    ordered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));

    let mut batches = Vec::new();
    let mut current = Vec::new();
    let mut chats = 0usize;
    let mut decoded = 0u64;
    let flush = |events: Vec<Event>, batches: &mut Vec<PlannedBatch>| -> Result<(), AppError> {
        if events.is_empty() {
            return Ok(());
        }
        let encoded = wire::encode_publication(&events, wire::DEFAULT_MAX_DECODED_BYTES)?;
        let members = backfill_members(&events)?;
        batches.push(PlannedBatch { encoded, members });
        Ok(())
    };
    for group in ordered {
        let group_encoded =
            wire::encode_publication(&group.events, wire::DEFAULT_MAX_DECODED_BYTES)?;
        let group_bytes = group_encoded.decoded_bytes;
        let over_limit =
            chats + 1 > limits.chats || decoded.saturating_add(group_bytes) > limits.decoded_bytes;
        if !current.is_empty() && over_limit {
            flush(std::mem::take(&mut current), &mut batches)?;
            chats = 0;
            decoded = 0;
        }
        current.extend(group.events);
        chats += 1;
        decoded = decoded.saturating_add(group_bytes);
    }
    flush(current, &mut batches)?;
    if batches.is_empty() {
        return Err(AppError::usage(
            "a backfill batch must carry at least one chat",
        ));
    }

    let mut members = batches
        .iter()
        .flat_map(|batch| batch.members.iter().cloned())
        .collect::<Vec<_>>();
    members.sort_by(|a, b| a.chat_id.cmp(&b.chat_id));
    members.dedup_by(|a, b| a.chat_id == b.chat_id);
    let (manifest_piece, manifest_digest, manifest_bytes) = encode_backfill_manifest(&members)?;

    Ok(BackfillPlan {
        start,
        baseline_revision,
        members,
        manifest_piece,
        manifest_digest,
        manifest_bytes,
        batches,
        record_batch,
    })
}

/// Plan a sealed baseline spool with the production batch targets.
pub fn plan_backfill_file(
    file: &tempfile::NamedTempFile,
    limits: BackfillLimits,
) -> Result<BackfillPlan, AppError> {
    let (start, events) = read_events(file)?;
    plan_backfill(start, events, limits)
}
