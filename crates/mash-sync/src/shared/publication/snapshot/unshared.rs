//! The one place a publication drops what the node keeps to itself: the chats
//! of unshared agents, their messages, amendments and tombstones, and every
//! memory record that cites one of those chats. Every batch a snapshot,
//! delta or backfill writes passes through `keep` first.
use crate::records::transfer::Row;
use crate::shared::protocol::Batch;
use crate::storage::publication::Unshared;

/// What this node chose to publish, as the end of its publication target. A
/// new target makes the next publication a complete snapshot, and a snapshot
/// replaces everything this node published. So changing the masking replaces
/// what was sent; turning an agent off removes its chats from the server; and
/// turning it back on sends its older chats, which no delta would carry.
pub(in crate::shared::publication) fn target_choices(
    config: &crate::config::SharedConfig,
) -> String {
    let masking = if config.publication_redaction.enabled() {
        ""
    } else {
        "\0unmasked"
    };
    let mut names: Vec<&str> = config
        .unshared
        .iter()
        .map(|provider| provider.as_str())
        .collect();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        masking.to_owned()
    } else {
        format!("{masking}\0unshared:{}", names.join(","))
    }
}

/// Write `batch` without anything unshared, or nothing when all of it was.
pub(super) fn write_kept(
    writer: &mut impl std::io::Write,
    unshared: &Unshared,
    batch: Batch,
    redaction: crate::redaction::Policy,
) -> Result<(), crate::AppError> {
    match keep(unshared, batch) {
        Some(batch) => super::write_event(
            writer,
            &crate::shared::protocol::Event::Batch(batch),
            redaction,
        ),
        None => Ok(()),
    }
}

/// `batch` without anything unshared, or nothing when every row was unshared.
pub(super) fn keep(unshared: &Unshared, batch: Batch) -> Option<Batch> {
    if unshared.is_empty() {
        return Some(batch);
    }
    let batch = match batch {
        Batch::Tombstones { mut records } => {
            records.retain(|tombstone| !unshared.providers.contains(&tombstone.provider));
            Batch::Tombstones { records }
        }
        Batch::Chats { mut records } => {
            records.retain(|chat| {
                !unshared.providers.contains(&chat.provider) && !unshared.chats.contains(&chat.id)
            });
            Batch::Chats { records }
        }
        Batch::Messages { mut records } => {
            records.retain(|message| !unshared.chats.contains(&message.chat_id));
            Batch::Messages { records }
        }
        Batch::Records { mut records } => {
            records.retain(|row| shared_row(unshared, row));
            Batch::Records { records }
        }
    };
    (!is_empty(&batch)).then_some(batch)
}

/// Whether a chat's amendment may be published.
pub(super) fn keeps_chat(unshared: &Unshared, chat: &str) -> bool {
    !unshared.chats.contains(chat)
}

/// A record row is published only when neither it nor anything it points at
/// belongs to an unshared chat or a record citing one.
fn shared_row(unshared: &Unshared, row: &Row) -> bool {
    let record = |id: &str| !unshared.records.contains(id);
    match row {
        Row::Owner { .. } => true,
        Row::Header { id, .. } | Row::Revision { id, .. } | Row::Confirmation { id, .. } => {
            record(id)
        }
        Row::Evidence { id, source, .. } => record(id) && !unshared.chats.contains(&source.chat_id),
        Row::Link { link } => record(&link.source_id) && record(&link.target_id),
        Row::Operation { result, .. } => {
            record(&result.record_id)
                && result
                    .related
                    .as_ref()
                    .is_none_or(|related| record(&related.id))
        }
        Row::ExtractionOperation { result, .. } => result.records.iter().all(|receipt| {
            record(&receipt.record_id)
                && receipt
                    .related
                    .as_ref()
                    .is_none_or(|related| record(&related.id))
        }),
        Row::Checkpoint { chat_id, .. } => !unshared.chats.contains(chat_id),
    }
}

fn is_empty(batch: &Batch) -> bool {
    match batch {
        Batch::Tombstones { records } => records.is_empty(),
        Batch::Chats { records } => records.is_empty(),
        Batch::Messages { records } => records.is_empty(),
        Batch::Records { records } => records.is_empty(),
    }
}
