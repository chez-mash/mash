//! Silver rows read back into the records a publication uploads.
use crate::AppError;
use crate::provider::Provider;
use crate::shared::batches::{ChatRecord, MessageRecord};

/// One row of the publication's chat query, marker columns included.
pub(super) fn chat_record(row: &rusqlite::Row<'_>) -> Result<ChatRecord, AppError> {
    Ok(ChatRecord {
        enrichment: row
            .get::<_, Option<String>>(7)
            .map_err(AppError::database)?
            .map(|json| super::super::enrichment::decode(&json))
            .transpose()?,
        id: row.get::<_, String>(0).map_err(AppError::database)?,
        provider: row.get::<_, Provider>(1).map_err(AppError::database)?,
        title: row
            .get::<_, Option<String>>(2)
            .map_err(AppError::database)?,
        created_at: row.get::<_, Option<i64>>(3).map_err(AppError::database)?,
        updated_at: row.get::<_, Option<i64>>(4).map_err(AppError::database)?,
        project: row
            .get::<_, Option<String>>(5)
            .map_err(AppError::database)?,
        repository: row
            .get::<_, Option<String>>(6)
            .map_err(AppError::database)?,
        subagent: subagent(row)?,
    })
}

/// The chat's `chat_subagents` row, published exactly as silver keeps it.
/// Silver holds the whole chat, so a record without one means the chat is not
/// a subagent here, whichever session file was read last.
fn subagent(
    row: &rusqlite::Row<'_>,
) -> Result<Option<crate::silver::subagent::Subagent>, AppError> {
    if !row.get::<_, bool>(8).map_err(AppError::database)? {
        return Ok(None);
    }
    let text = |index: usize| {
        row.get::<_, Option<String>>(index)
            .map_err(AppError::database)
    };
    let depth = row
        .get::<_, Option<i64>>(12)
        .map_err(AppError::database)?
        .map(u32::try_from)
        .transpose()
        .map_err(|_| AppError::database_data("invalid subagent depth"))?;
    let finished = row.get::<_, Option<i64>>(13).map_err(AppError::database)?;
    Ok(Some(crate::silver::subagent::Subagent {
        parent_chat_id: text(9)?,
        label: text(10)?,
        role: text(11)?,
        depth,
        finished: finished.map(|finished| finished != 0),
        spawn_call_id: text(14)?,
    }))
}

pub(super) fn message_record(row: &rusqlite::Row<'_>) -> Result<MessageRecord, AppError> {
    Ok(MessageRecord {
        attachments: serde_json::from_str(
            &row.get::<_, String>("attachments")
                .map_err(AppError::database)?,
        )
        .map_err(|_| AppError::database_data("invalid attachment metadata"))?,
        id: row.get::<_, String>(0).map_err(AppError::database)?,
        chat_id: row.get::<_, String>(1).map_err(AppError::database)?,
        ordinal: row.get::<_, i64>(2).map_err(AppError::database)?,
        context: crate::message_context::MessageContext::from_parent(
            row.get("context_parent").map_err(AppError::database)?,
        ),
        role: row
            .get::<_, crate::role::Role>(3)
            .map_err(AppError::database)?,
        content: row.get::<_, String>(4).map_err(AppError::database)?,
        projection: row.get::<_, String>(5).map_err(AppError::database)?,
        created_at: row.get::<_, Option<i64>>(6).map_err(AppError::database)?,
        generation: None,
        vector: None,
        tools: crate::tools::Activity::decode(
            row.get::<_, Option<String>>("metadata")
                .map_err(AppError::database)?
                .as_deref(),
        )?,
    })
}
