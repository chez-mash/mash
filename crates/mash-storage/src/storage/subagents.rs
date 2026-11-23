//! Writing and reading what agents recorded about the subagents they started,
//! and which chats were read from a copy of a session that ran over SSH.
//!
//! A row is kept beside its chat in silver and goes with it; it is never
//! published. Claude Code chats are assembled from several session files, and
//! only a subagent transcript knows it is one, so a contribution without the
//! marker leaves an existing row alone.
use rusqlite::{Connection, OptionalExtension, params};

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
        let Some(subagent) = &chat.origin.subagent else {
            if chat.provider != Provider::ClaudeCode {
                self.connection
                    .execute("DELETE FROM chat_subagents WHERE chat_id = ?1", [&chat.id])
                    .map_err(AppError::database)?;
            }
            return Ok(());
        };
        let depth = subagent.depth.map(i64::from);
        let finished = subagent.finished.map(i64::from);
        // The chat row may be absent, for a source with no retained messages.
        self.connection
            .execute(
                "INSERT INTO chat_subagents(chat_id,parent_chat_id,label,role,depth,finished,spawn_call_id)
                 SELECT ?1,?2,?3,?4,?5,?6,?7 WHERE EXISTS(SELECT 1 FROM chats WHERE id = ?1)
                 ON CONFLICT(chat_id) DO UPDATE SET parent_chat_id=excluded.parent_chat_id,
                    label=excluded.label, role=excluded.role, depth=excluded.depth,
                    finished=excluded.finished, spawn_call_id=excluded.spawn_call_id",
                params![
                    chat.id,
                    subagent.parent_chat_id.as_deref().filter(|parent| *parent != chat.id),
                    subagent.label,
                    subagent.role,
                    depth,
                    finished,
                    subagent.spawn_call_id
                ],
            )
            .map_err(AppError::database)?;
        Ok(())
    }
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

#[cfg(test)]
mod tests;
