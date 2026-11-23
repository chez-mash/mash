//! What a node keeps to itself when some agents are unshared: their chats, and
//! every memory record that cites one of them. Publication drops both sets, so
//! nothing quoted from an unshared chat reaches the server through a memory.
use super::Snapshot;
use crate::AppError;
use crate::provider::Provider;
use std::collections::HashSet;

/// Chats and records a publication must leave out.
#[derive(Debug, Default)]
pub struct Unshared {
    pub providers: HashSet<Provider>,
    pub chats: HashSet<String>,
    pub records: HashSet<String>,
}

impl Unshared {
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Snapshot<'_> {
    /// The chats of `providers`, and the records whose evidence cites any of
    /// them at any revision. A record citing both a shared and an unshared
    /// chat is left out whole: its statement may repeat the unshared chat.
    pub fn unshared(&self, providers: &[Provider]) -> Result<Unshared, AppError> {
        let mut unshared = Unshared {
            providers: providers.iter().copied().collect(),
            ..Unshared::default()
        };
        if unshared.providers.is_empty() {
            return Ok(unshared);
        }
        let mut chats = self
            .connection
            .prepare("SELECT id FROM chats WHERE provider=?1")
            .map_err(AppError::database)?;
        for provider in &unshared.providers {
            let ids = chats
                .query_map([provider], |row| row.get::<_, String>(0))
                .map_err(AppError::database)?;
            for id in ids {
                unshared.chats.insert(id.map_err(AppError::database)?);
            }
        }
        let mut citing = self
            .connection
            .prepare("SELECT DISTINCT record_id FROM record_evidence WHERE chat_id=?1")
            .map_err(AppError::database)?;
        for chat in &unshared.chats {
            let ids = citing
                .query_map([chat], |row| row.get::<_, String>(0))
                .map_err(AppError::database)?;
            for id in ids {
                unshared.records.insert(id.map_err(AppError::database)?);
            }
        }
        Ok(unshared)
    }
}

impl super::Storage {
    /// Messages in the chats of `providers`, so their attachments are never
    /// offered for upload even if the server asked for one.
    pub fn unshared_messages(&self, providers: &[Provider]) -> Result<HashSet<String>, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT m.id FROM messages m JOIN chats c ON c.id=m.chat_id WHERE c.provider=?1",
            )
            .map_err(AppError::database)?;
        let mut ids = HashSet::new();
        for provider in providers {
            let rows = statement
                .query_map([provider], |row| row.get::<_, String>(0))
                .map_err(AppError::database)?;
            for id in rows {
                ids.insert(id.map_err(AppError::database)?);
            }
        }
        Ok(ids)
    }
}
