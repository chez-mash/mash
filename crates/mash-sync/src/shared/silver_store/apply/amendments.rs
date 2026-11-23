//! Amending a chat the server already holds: its new record, its replaced
//! messages removed, and the check that it ends as long as its publisher's.
use super::{AMENDMENT_DIVERGED, Amended, Upload, database, encode, sql};
use crate::AppError;
use crate::shared::protocol::{self, Amendment};
use crate::shared::validation::{conflict, valid_id};
use std::collections::BTreeSet;

impl Upload<'_> {
    /// Apply one chat's amendment: update the chat record and remove the
    /// replaced messages; the upload's message batches supply their new rows.
    pub(super) fn amend(&mut self, amendment: Amendment) -> Result<(), AppError> {
        let Amendment {
            chat: record,
            replaced,
            messages,
        } = amendment;
        record.validate_enrichment()?;
        record.validate_subagent()?;
        valid_id(&record.id)?;
        if replaced.len() > protocol::MAX_AMENDED_MESSAGES
            || self.applied.supplied.contains(&record.id)
        {
            return Err(AppError::usage("invalid chat amendment"));
        }
        let mut ids = BTreeSet::new();
        for id in replaced {
            valid_id(&id)?;
            if !ids.insert(id) {
                return Err(AppError::usage("invalid chat amendment"));
            }
        }
        if self.tombstoned(record.provider, &record.id)? {
            self.remove(&record.id)?;
            return Ok(());
        }
        let prior = match crate::shared::silver_store::chat(self.tx, &record.id)? {
            None => None,
            // Documents carry these chat fields; an amendment keeps documents.
            Some(existing)
                if existing.provider == record.provider
                    && existing.project == record.project
                    && existing.repository == record.repository =>
            {
                Some(
                    sql::ChatFingerprint::builder()
                        .id(&record.id)
                        .build()
                        .query_one(self.tx)
                        .map_err(database)?
                        .chat_fingerprint
                        .unwrap_or_default(),
                )
            }
            Some(_) => return Err(conflict(AMENDMENT_DIVERGED)),
        };
        let removed_embeddings = if prior.is_some() {
            let removed = self.remove_replaced(&record.id, &ids)?;
            sql::UpdateChat::builder()
                .title(record.title.as_deref())
                .created_at(record.created_at)
                .updated_at(record.updated_at)
                .project(record.project.as_deref())
                .repository(record.repository.as_deref())
                .id(&record.id)
                .provider(record.provider.as_str())
                .build()
                .execute(self.tx)
                .map_err(database)?;
            sql::DeleteEnrichment::builder()
                .chat_id(&record.id)
                .build()
                .execute(self.tx)
                .map_err(database)?;
            removed
        } else {
            sql::InsertChat::builder()
                .provider(record.provider.as_str())
                .id(&record.id)
                .title(record.title.as_deref())
                .created_at(record.created_at)
                .updated_at(record.updated_at)
                .project(record.project.as_deref())
                .repository(record.repository.as_deref())
                .build()
                .execute(self.tx)
                .map_err(database)?;
            0
        };
        if let Some(evidence) = &record.enrichment {
            sql::InsertEnrichment::builder()
                .chat_id(&record.id)
                .evidence(&encode(evidence)?)
                .build()
                .execute(self.tx)
                .map_err(database)?;
        }
        self.subagent(&record)?;
        self.applied.deleted.remove(&record.id);
        self.applied.supplied.insert(record.id.clone());
        self.applied.amended.insert(
            record.id,
            Amended {
                prior,
                replaced: ids,
                removed_embeddings,
                messages,
            },
        );
        self.applied.changed = true;
        Ok(())
    }

    /// Delete an amended chat's replaced messages; returns how many of them
    /// were dense-eligible.
    fn remove_replaced(&self, chat: &str, ids: &BTreeSet<String>) -> Result<u64, AppError> {
        let mut embeddings = 0;
        for id in ids {
            let Some(row) = sql::ChatMessageRole::builder()
                .chat_id(chat)
                .id(id)
                .build()
                .query_opt(self.tx)
                .map_err(database)?
            else {
                continue;
            };
            embeddings += u64::from(self.build.includes_dense(
                &crate::role::Role::from(row.role),
                row.search_projection.as_deref().unwrap_or_default(),
            ));
            sql::DeleteChatMessage::builder()
                .chat_id(chat)
                .id(id)
                .build()
                .execute(self.tx)
                .map_err(database)?;
        }
        Ok(embeddings)
    }

    /// Every amended chat now holds exactly as many messages as its publisher's.
    pub(super) fn check_amended(&self) -> Result<(), AppError> {
        for (id, amended) in &self.applied.amended {
            let count = sql::ChatMessageCount::builder()
                .chat_id(id)
                .build()
                .query_one(self.tx)
                .map_err(database)?
                .messages;
            if u64::try_from(count).ok() != Some(amended.messages) {
                return Err(conflict(AMENDMENT_DIVERGED));
            }
        }
        Ok(())
    }
}
