//! Machines joining and leaving a running store: whether it serves a group,
//! admitting a machine added while it runs, and what one machine published.
use super::Store;
use crate::AppError;
use crate::shared::server::Identity;
use crate::shared::silver_store;

/// What one machine has published to this server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Published {
    pub instance: String,
    pub chats: u64,
    pub messages: u64,
    pub last_synced_at: Option<String>,
}

impl Store {
    /// Configured nodes of one group, in identity order.
    pub fn nodes(&self, group: &str) -> Vec<String> {
        self.silver.nodes(group)
    }

    /// Whether this running store serves a group, so a machine added to it
    /// can be admitted without a restart.
    pub fn serves(&self, group: &str) -> bool {
        self.silver.serves(group)
    }

    /// Admit a machine added to a group this store serves.
    pub async fn admit(&self, identity: &Identity) -> Result<(), AppError> {
        let silver = self.silver.clone();
        let identity = identity.clone();
        crate::blocking(move || silver.admit(&identity)).await
    }

    /// What the server holds under one name: the index that published it
    /// and how many chats and messages, or nothing when it never published.
    pub async fn published(&self, identity: &Identity) -> Result<Option<Published>, AppError> {
        let Some(head) = self.head(identity).await? else {
            return Ok(None);
        };
        let counts = self.read(identity, silver_store::counts).await?;
        Ok(Some(Published {
            instance: head.instance,
            chats: counts.chats,
            messages: counts.messages,
            last_synced_at: head.last_synced_at,
        }))
    }
}
