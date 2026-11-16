//! Canonical SQLite silver storage. Qdrant projection lives in views/local.
use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::AppError;

// View views and materialized retrieval data.
pub mod admission;
mod catalog;
pub mod chat_facts;
pub mod coverage;

// Silver mutation/provenance plus atomic invalidation and publication ownership.
pub mod attachments;
mod claude_sources;
mod enrichment;
pub mod export;
pub mod gold_input;
mod indexing;
pub mod packed;
pub mod publication;
pub mod records;
pub mod search_input;
mod silver_state;
pub mod turns;

// Connection/schema and generated query infrastructure for both layers.
mod queries;
pub use queries::UnmatchedId;
pub use silver_state::SourceCount;
pub mod schema;

#[cfg(test)]
use schema::SCHEMA_VERSION;
// sqlc-gen-rust emits an unused row parameter for :exec result models.
#[allow(dead_code, unused_variables, clippy::all, clippy::pedantic)]
#[path = "storage/generated/queries.rs"]
pub mod generated;

pub struct Storage {
    attachments_config: crate::attachments::Configuration,
    /// This writer's attachment-cache verification, recorded when it commits.
    pub attachment_verification: Option<attachments::PendingVerification>,
    /// Match evidence is computed on text masked under the output policy.
    output_redaction: crate::redaction::Policy,
    pub connection: Connection,
    writer_active: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Counts {
    pub chats: u64,
    pub messages: u64,
    pub searchable_messages: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub searchable_user_messages: Option<u64>,
    pub embeddings: u64,
}

pub struct StatusSnapshot {
    pub silver: Option<crate::silver::certificate::Certificate>,
    pub counts: Counts,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchHit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lookup: Option<crate::search_results::lookup::Lookup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_match: Option<crate::views::matching::Evidence>,
    pub id: String,
    pub chat_id: String,
    pub provider: crate::provider::Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
    pub role: crate::role::Role,
    /// Message position in its chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i64>,
    pub content: String,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub metadata: crate::search_results::ChatMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copies: Vec<crate::search_results::MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<f32>,
    pub fusion_score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Message {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<crate::attachments::Metadata>,
    pub id: String,
    pub ordinal: i64,
    pub role: crate::role::Role,
    pub content: String,
    /// The dialogue in `content`, when the entry holds more than that. Absent:
    /// all of `content` is dialogue. Empty: none of it is; the entry is context
    /// such as injected instructions, thinking or tool traffic. Otherwise what
    /// the person typed, with the context an agent wrapped around it removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialogue: Option<String>,
    pub created_at: Option<i64>,
    /// A native entry's structured tool calls and results.
    #[serde(default, skip_serializing_if = "crate::tools::Activity::is_empty")]
    pub tools: crate::tools::Activity,
}

impl Message {
    /// `dialogue` for an entry read in full. An assistant entry that mixes a
    /// reply with thinking or tool blocks states nothing: its parts are told
    /// apart by `tools` and would only repeat the reply here.
    pub fn dialogue_of(
        role: &crate::role::Role,
        content: &str,
        projection: &str,
    ) -> Option<String> {
        (projection != content && (projection.is_empty() || *role == crate::role::Role::User))
            .then(|| projection.to_owned())
    }

    /// Apply output masking to every text an entry shows.
    pub fn mask_text(&mut self, mask: impl Fn(&str) -> String) {
        self.content = mask(&self.content);
        if let Some(dialogue) = &mut self.dialogue {
            *dialogue = mask(dialogue);
        }
    }
}

/// Where a view's messages sit in their chat, so a short window is not
/// mistaken for the whole conversation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ChatWindow {
    /// The chat the viewed message belongs to.
    pub chat_id: String,
    /// Every message the chat retains, including other branches.
    pub message_count: u64,
    /// Messages earlier than the first returned message.
    pub omitted_before: u64,
    /// Messages later than the last returned message.
    pub omitted_after: u64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MessageContent {
    #[default]
    Canonical,
    SearchProjection,
}

impl MessageContent {
    // serde's skip_serializing_if callback takes a reference.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn is_canonical(&self) -> bool {
        matches!(self, Self::Canonical)
    }
}

#[derive(Default)]
pub struct ChatChange {
    pub unchanged: bool,
    pub tombstoned: bool,
    pub changed_message_ids: Vec<String>,
    pub removed_messages: u64,
}

impl Storage {
    /// Search results leave through CLI or MCP output under this policy.
    pub fn with_output_redaction(mut self, policy: crate::redaction::Policy) -> Self {
        self.output_redaction = policy;
        self
    }

    pub fn open(path: &Path) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AppError::io)?;
        }
        let mut connection = Connection::open(path).map_err(AppError::database)?;
        schema::initialize_silver(&mut connection)?;
        schema::install_publication_triggers(&connection)?;
        Ok(Self {
            attachments_config: crate::attachments::Configuration::default(),
            attachment_verification: None,
            output_redaction: crate::redaction::Policy::ON,
            connection,
            writer_active: false,
        })
    }

    pub fn open_writer(path: &Path) -> Result<Self, AppError> {
        let mut storage = Self::open(path)?;
        storage
            .connection
            .busy_timeout(Duration::ZERO)
            .map_err(AppError::database)?;
        storage
            .connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(AppError::database_at(
                crate::database_error::Operation::BeginWriter,
            ))?;
        storage.writer_active = true;
        Ok(storage)
    }

    #[tracing::instrument(name = "commit_writer", skip_all)]
    pub fn commit_writer(&mut self) -> Result<(), AppError> {
        if self.writer_active {
            let verification = self.verification_to_record();
            self.connection
                .execute_batch("COMMIT")
                .map_err(AppError::database)?;
            self.writer_active = false;
            if let Some(verification) = verification {
                verification.record();
            }
        }
        Ok(())
    }

    pub fn truncate_wal(&self) -> Result<(), AppError> {
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .map_err(AppError::database)
    }

    #[tracing::instrument(name = "checkpoint_writer", skip_all)]
    pub fn checkpoint_writer(&mut self) -> Result<(), AppError> {
        self.commit_and_restart_writer("PRAGMA wal_checkpoint(PASSIVE); BEGIN IMMEDIATE")
    }

    pub fn commit_and_continue_writer(&mut self) -> Result<(), AppError> {
        self.commit_and_restart_writer("BEGIN IMMEDIATE")
    }

    fn commit_and_restart_writer(&mut self, continuation: &str) -> Result<(), AppError> {
        self.require_writer()?;
        self.connection
            .execute_batch("COMMIT")
            .map_err(AppError::database)?;
        self.writer_active = false;
        self.connection
            .execute_batch(continuation)
            .map_err(AppError::database)?;
        self.writer_active = true;
        Ok(())
    }

    fn require_writer(&self) -> Result<(), AppError> {
        if self.writer_active {
            Ok(())
        } else {
            Err(AppError::internal(
                "index mutation requires an active writer",
            ))
        }
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        if self.writer_active {
            let _ = self.connection.execute_batch("ROLLBACK");
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod message_tests {
    use super::Message;
    use crate::role::Role;

    #[test]
    fn dialogue_is_stated_only_where_content_holds_more() {
        // All of it is dialogue: nothing to state.
        assert_eq!(Message::dialogue_of(&Role::User, "hello", "hello"), None);
        // Context only, for any role.
        for role in [Role::User, Role::Assistant, Role::System] {
            assert_eq!(
                Message::dialogue_of(&role, "<system-reminder>x</system-reminder>", ""),
                Some(String::new())
            );
        }
        // What the person typed inside what the agent wrapped around it.
        assert_eq!(
            Message::dialogue_of(
                &Role::User,
                "<system-reminder>x</system-reminder>fix it",
                "fix it"
            ),
            Some("fix it".into())
        );
        // A reply stored with its tool blocks: `tools` tells them apart.
        assert_eq!(
            Message::dialogue_of(&Role::Assistant, "Done.\nTool shell: ls", "Done."),
            None
        );
    }

    #[test]
    fn masking_covers_the_dialogue_as_well_as_the_content() {
        let mut message = Message {
            attachments: Vec::new(),
            id: "m".into(),
            ordinal: 0,
            role: Role::User,
            content: "<system-reminder>x</system-reminder>token abc".into(),
            dialogue: Some("token abc".into()),
            created_at: None,
            tools: crate::tools::Activity::default(),
        };
        message.mask_text(|text| text.replace("abc", "[masked]"));
        assert!(message.content.ends_with("token [masked]"));
        assert_eq!(message.dialogue.as_deref(), Some("token [masked]"));
    }
}
