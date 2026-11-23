//! Chat catalogues across a group's node silvers: every published chat with
//! the subagent marker its node published, and each parent's subagent count.
use super::{Identity, Store, silver_store};
use crate::AppError;
use crate::chat_catalog::{Chat, Collector, Page, Request};

impl Store {
    pub async fn list_chats(&self, identity: &Identity, input: Request) -> Result<Page, AppError> {
        let mut collector = Collector::new(input.clone())?;
        for chat in self.catalog(identity, &input).await? {
            collector.push(chat);
        }
        Ok(collector.finish())
    }

    /// Rows carry markers, so `exclude_subagents` and folding work for
    /// other machines' chats as they do for this machine's.
    pub async fn catalog(
        &self,
        identity: &Identity,
        input: &Request,
    ) -> Result<Vec<Chat>, AppError> {
        let mut chats = Vec::new();
        for node in self.heads(&identity.group_id).await?.into_keys() {
            if !input.admits_node(&node, &identity.node_id) {
                continue;
            }
            let scope = silver_store::node_identity(&identity.group_id, &node);
            chats.extend(
                self.read(&scope, move |connection| node_catalog(connection, &node))
                    .await?,
            );
        }
        Ok(chats)
    }
}

fn node_catalog(connection: &rusqlite::Connection, node: &str) -> Result<Vec<Chat>, AppError> {
    let threads = silver_store::read::list_chats(connection, node)?;
    let mut subagents = crate::storage::subagents::catalog(connection)?;
    let mut chats = Vec::with_capacity(threads.len());
    for thread in threads {
        let row = thread.chat;
        chats.push(Chat {
            subagents: subagents.started.get(&row.id).copied().unwrap_or_default(),
            metadata: crate::search_results::ChatMetadata {
                subagent: subagents.markers.remove(&row.id),
                enrichment: row.enrichment,
                title: row.title,
                project: row.project,
                repository: row.repository,
                created_at: row.created_at,
                updated_at: row.updated_at,
                ..Default::default()
            },
            chat_id: row.id,
            provider: row.provider,
            node_id: thread.node_id,
            first_message_id: thread.first_message_id,
        });
    }
    Ok(chats)
}

/// Rows as clients older than published markers read them: they reject the
/// marker and count fields. Applied after any exclusion, which still holds.
pub fn without_subagents(chats: &mut [Chat]) {
    for chat in chats {
        chat.metadata.subagent = None;
        chat.subagents = 0;
    }
}
