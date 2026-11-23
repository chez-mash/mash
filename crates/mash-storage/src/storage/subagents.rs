//! Writing and reading what agents recorded about the subagents they started,
//! and which chats were read from a copy of a session that ran over SSH.
//!
//! A row is kept beside its chat in silver and goes with it. Claude Code chats
//! are assembled from several session files, and only a subagent transcript
//! knows it is one, so a contribution without the marker leaves an existing
//! row alone.
//!
//! A node publishes the row with its chat record, and the shared server keeps
//! it in that node's silver, so other machines can tell its subagents apart.
//! A published record is read from the node's whole silver chat rather than
//! from one session file, so on the server a record without a marker means
//! the chat is not a subagent. Publication journals the row like the chat's
//! other fields.
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashMap;

use super::{ChatChange, Storage};
use crate::AppError;
use crate::provider::Provider;
use crate::silver::Chat;
use crate::subagents::{Row, Subagent};

impl Storage {
    pub fn replace_chat(&mut self, chat: &Chat) -> Result<ChatChange, AppError> {
        let change = if chat.provider == Provider::ClaudeCode && chat.claude_lineage.is_some() {
            self.replace_claude_source(chat)?
        } else {
            self.replace_canonical_chat(chat)?
        };
        if !change.tombstoned {
            self.store_subagent(chat)?;
            self.store_ssh_copy(chat)?;
        }
        Ok(change)
    }

    /// The latest contribution decides; a chat is not read from a copy and
    /// from its own transcript on one machine.
    fn store_ssh_copy(&self, chat: &Chat) -> Result<(), AppError> {
        let sql = if chat.origin.ssh_copy {
            "INSERT OR IGNORE INTO chat_ssh_copies(chat_id)
             SELECT ?1 WHERE EXISTS(SELECT 1 FROM chats WHERE id = ?1)"
        } else {
            "DELETE FROM chat_ssh_copies WHERE chat_id = ?1"
        };
        self.connection
            .execute(sql, [&chat.id])
            .map_err(AppError::database)?;
        Ok(())
    }

    fn store_subagent(&self, chat: &Chat) -> Result<(), AppError> {
        match &chat.origin.subagent {
            Some(subagent) => store(&self.connection, &chat.id, subagent),
            None if chat.provider != Provider::ClaudeCode => remove(&self.connection, &chat.id),
            None => Ok(()),
        }
    }
}

/// Keep `subagent` as the row of `chat_id` when that chat is in silver; the
/// chat may be absent, for a source with no retained messages. An unchanged
/// row is left as it is, so reading a transcript again journals nothing for
/// publication.
pub fn store(
    connection: &Connection,
    chat_id: &str,
    subagent: &crate::silver::subagent::Subagent,
) -> Result<(), AppError> {
    let depth = subagent.depth.map(i64::from);
    let finished = subagent.finished.map(i64::from);
    connection
        .prepare_cached(
            "INSERT INTO chat_subagents(chat_id,parent_chat_id,label,role,depth,finished,spawn_call_id)
             SELECT ?1,?2,?3,?4,?5,?6,?7 WHERE EXISTS(SELECT 1 FROM chats WHERE id = ?1)
             ON CONFLICT(chat_id) DO UPDATE SET parent_chat_id=excluded.parent_chat_id,
                label=excluded.label, role=excluded.role, depth=excluded.depth,
                finished=excluded.finished, spawn_call_id=excluded.spawn_call_id
             WHERE (parent_chat_id,label,role,depth,finished,spawn_call_id) IS NOT
                (excluded.parent_chat_id,excluded.label,excluded.role,excluded.depth,
                 excluded.finished,excluded.spawn_call_id)",
        )
        .map_err(AppError::database)?
        .execute(params![
            chat_id,
            subagent.parent_chat_id.as_deref().filter(|parent| *parent != chat_id),
            subagent.label,
            subagent.role,
            depth,
            finished,
            subagent.spawn_call_id
        ])
        .map_err(AppError::database)?;
    Ok(())
}

/// Forget that `chat_id` is a subagent.
pub fn remove(connection: &Connection, chat_id: &str) -> Result<(), AppError> {
    connection
        .prepare_cached("DELETE FROM chat_subagents WHERE chat_id = ?1")
        .map_err(AppError::database)?
        .execute([chat_id])
        .map_err(AppError::database)?;
    Ok(())
}

/// Whether a chat was read from a copy of a session that ran over SSH.
pub fn ssh_copy(connection: &Connection, chat_id: &str) -> Result<bool, AppError> {
    connection
        .prepare_cached("SELECT EXISTS(SELECT 1 FROM chat_ssh_copies WHERE chat_id = ?1)")
        .map_err(AppError::database)?
        .query_row([chat_id], |row| row.get(0))
        .map_err(AppError::database)
}

/// The subagent row of one chat, with its parent's title when indexed.
pub fn read(connection: &Connection, chat_id: &str) -> Result<Option<Subagent>, AppError> {
    connection
        .prepare_cached(
            "SELECT a.parent_chat_id, p.title, a.label, a.role, a.depth, a.finished, a.spawn_call_id
               FROM chat_subagents a JOIN chats c ON c.id = a.chat_id
               LEFT JOIN chats p ON p.id = a.parent_chat_id AND p.provider = c.provider
              WHERE a.chat_id = ?1",
        )
        .map_err(AppError::database)?
        .query_row([chat_id], |row| {
            Ok(Row {
                parent_chat_id: row.get(0)?,
                parent_title: row.get(1)?,
                label: row.get(2)?,
                role: row.get(3)?,
                depth: row.get(4)?,
                finished: row.get(5)?,
                spawn_call_id: row.get(6)?,
            })
        })
        .optional()
        .map(|row| row.map(Subagent::from))
        .map_err(AppError::database)
}

/// What a catalogue of a whole silver shows about subagents: each marked
/// chat's row, with its parent's title when indexed, and how many marked
/// chats each parent started. Both are keyed by chat ID.
pub struct Catalog {
    pub markers: HashMap<String, Subagent>,
    pub started: HashMap<String, u32>,
}

/// Every subagent row of one silver at once, for catalogues that list it all,
/// as the shared server's lists each node's silver.
pub fn catalog(connection: &Connection) -> Result<Catalog, AppError> {
    let mut statement = connection
        .prepare_cached(
            "SELECT a.chat_id, a.parent_chat_id, p.title, a.label, a.role, a.depth, a.finished,
                    a.spawn_call_id
               FROM chat_subagents a JOIN chats c ON c.id = a.chat_id
               LEFT JOIN chats p ON p.id = a.parent_chat_id AND p.provider = c.provider",
        )
        .map_err(AppError::database)?;
    let rows = statement
        .query_map([], |row| {
            let marker = Row {
                parent_chat_id: row.get(1)?,
                parent_title: row.get(2)?,
                label: row.get(3)?,
                role: row.get(4)?,
                depth: row.get(5)?,
                finished: row.get(6)?,
                spawn_call_id: row.get(7)?,
            };
            Ok((row.get::<_, String>(0)?, marker))
        })
        .map_err(AppError::database)?;
    let mut catalog = Catalog {
        markers: HashMap::new(),
        started: HashMap::new(),
    };
    for row in rows {
        let (chat_id, marker) = row.map_err(AppError::database)?;
        // Counted as the local chat list counts them: by the parent named.
        if let Some(parent) = &marker.parent_chat_id {
            *catalog.started.entry(parent.clone()).or_default() += 1;
        }
        catalog.markers.insert(chat_id, Subagent::from(marker));
    }
    Ok(catalog)
}

#[cfg(test)]
mod tests;
