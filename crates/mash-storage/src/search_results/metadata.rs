//! The facts about a chat that every list and search result carries.
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ChatMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<crate::silver::enrichment::Evidence>,
    pub title: Option<String>,
    pub project: Option<String>,
    pub repository: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    /// Set only when the agent marked this chat as a subagent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagent: Option<crate::subagents::Subagent>,
    /// Read from the Claude desktop app's copy of a session it ran over SSH on
    /// another machine; that machine may share the original under the same ID.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ssh_copy: bool,
}

impl ChatMetadata {
    /// Masks credentials in the descriptive fields; enrichment derived from a
    /// masked field is dropped with it.
    pub fn redact(&mut self, policy: crate::redaction::Policy) {
        if !policy.enabled() {
            return;
        }
        let mut changed = false;
        let subagent = self
            .subagent
            .iter_mut()
            .flat_map(crate::subagents::Subagent::texts);
        for value in [&mut self.title, &mut self.project, &mut self.repository]
            .into_iter()
            .chain(subagent)
            .flatten()
        {
            let safe = crate::redaction::text(value);
            changed |= safe != *value;
            *value = safe;
        }
        if changed {
            self.enrichment = None;
        }
    }
}
