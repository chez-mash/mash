//! A subagent chat as lists and search results describe it.
//!
//! Silver keeps what the agent recorded (`chat_subagents`); readers add the
//! parent's title when the parent is indexed in the same silver. Nodes publish
//! their rows, so the shared index marks other machines' subagents too; a
//! chat published by a release older than that, or to an older server, is
//! unmarked there until it is published again.
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ChatSubagent")]
pub struct Subagent {
    /// The chat that started this one, from the same agent and machine.
    /// Absent when the agent named none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_chat_id: Option<String>,
    /// The parent's title. Absent when the parent is not indexed here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_title: Option<String>,
    /// The agent's name for the task: Claude Code's description, Codex's nickname.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Claude Code's agent type or Codex's agent role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// Whether the agent recorded it as finished. Absent means not recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished: Option<bool>,
    /// The parent's tool call that started it, when recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_call_id: Option<String>,
}

/// The columns `chat_subagents` holds for one chat, joined with its parent.
pub struct Row {
    pub parent_chat_id: Option<String>,
    pub parent_title: Option<String>,
    pub label: Option<String>,
    pub role: Option<String>,
    pub depth: Option<i64>,
    pub finished: Option<i64>,
    pub spawn_call_id: Option<String>,
}

impl From<Row> for Subagent {
    fn from(row: Row) -> Self {
        Self {
            parent_chat_id: row.parent_chat_id,
            parent_title: row.parent_title.map(|mut title| {
                crate::chat_catalog::headline(&mut title);
                title
            }),
            label: row.label,
            role: row.role,
            depth: row.depth.and_then(|depth| u32::try_from(depth).ok()),
            finished: row.finished.map(|finished| finished != 0),
            spawn_call_id: row.spawn_call_id,
        }
    }
}

impl Subagent {
    /// One line for text output: `Locke (worker), started by Tidy the docs`.
    pub fn describe(&self) -> String {
        let mut text = self.label.clone().unwrap_or_else(|| "unnamed".into());
        if let Some(role) = &self.role {
            text = format!("{text} ({role})");
        }
        match (&self.parent_title, &self.parent_chat_id) {
            (Some(title), _) => format!("{text}, started by {title}"),
            (None, Some(id)) => format!("{text}, started by chat {id}, not indexed here"),
            (None, None) => text,
        }
    }

    /// The descriptive text that can hold what a person wrote.
    pub fn texts(&mut self) -> [&mut Option<String>; 2] {
        [&mut self.label, &mut self.parent_title]
    }
}

/// `Subagent::describe` for a chat that may not be a subagent.
pub fn describe(marker: Option<&Subagent>) -> Option<String> {
    marker.map(Subagent::describe)
}
