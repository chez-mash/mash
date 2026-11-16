use super::models::{Content, Message, Part};
use crate::tools::Activity;
use crate::{dialogue, role::Role};

const CONTEXT: &[&str] = &[
    "user_info",
    "project_layout",
    "rules",
    "always_applied_workspace_rules",
    "agent_requestable_workspace_rules",
    "user_rules",
    "agent_skills",
    "available_skills",
    "cloud_instructions",
    "cloud_task_instructions",
    "open_and_recently_viewed_files",
    "system_reminder",
    "system-reminder",
    "instructions_update",
    "mcp_instructions",
    "mcp_file_system",
    "mcp_file_system_servers",
    "git_status",
    "agent_transcripts",
    "cursor_rules_context",
    "attached_files",
    "system_notification",
    "task_notification",
    "agent_notification",
];

#[derive(Default)]
pub struct Text {
    pub canonical: String,
    pub projection: String,
}

/// One displayed step of a native message: prose, a thinking block, a tool
/// call or a tool result.
pub struct Entry {
    pub role: Role,
    pub text: Text,
    pub tools: Activity,
}

impl Entry {
    fn step(role: Role, canonical: String, tools: Activity) -> Self {
        Self {
            role,
            text: Text {
                canonical,
                projection: String::new(),
            },
            tools,
        }
    }
}

pub fn projection(role: &Role, text: &str) -> String {
    let clean = dialogue::without_context_blocks(text, CONTEXT);
    let clean = dialogue::without_context_blocks(&clean, &["think", "thinking"]);
    dialogue::visible(role, &clean).trim().to_owned()
}

/// The folder named by the context Cursor injects into the first prompt.
pub fn working_directory(message: &Message) -> Option<String> {
    let text = match &message.content {
        Content::Text(text) => text.as_str(),
        Content::Parts(parts) => parts.iter().find_map(|part| match part {
            Part::Text { text } => Some(text.as_str()),
            _ => None,
        })?,
    };
    let (_, rest) = text.split_once("<user_info>")?;
    let (block, _) = rest.split_once("</user_info>")?;
    block
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("Current working directory:")
                .or_else(|| line.strip_prefix("Workspace Path:"))
        })
        .map(str::trim)
        .filter(|path| path.starts_with('/') || crate::platform::is_drive_path(path))
        .map(str::to_owned)
}

/// The text the reader typed or was shown, for matching recorded turns.
pub fn spoken(message: &Message) -> String {
    let text = match &message.content {
        Content::Text(text) => text.clone(),
        Content::Parts(parts) => parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect(),
    };
    projection(&message.role, &text)
}

/// A message in display order. Consecutive text is one entry; each thinking
/// block, tool call and tool result is its own, so none reads as the reply.
pub fn entries(message: &Message) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut prose = Text::default();
    let flush = |prose: &mut Text, entries: &mut Vec<Entry>| {
        let mut text = std::mem::take(prose);
        if text.canonical.is_empty() {
            return;
        }
        text.projection = projection(&message.role, &text.projection);
        entries.push(Entry {
            role: message.role.clone(),
            text,
            tools: Activity::default(),
        });
    };
    let context = |prose: &mut Text, text: &str| {
        if !prose.canonical.is_empty() {
            prose.canonical.push('\n');
        }
        prose.canonical.push_str(text);
        prose.canonical.push('\n');
    };
    match &message.content {
        Content::Text(text) => {
            prose.canonical.push_str(text);
            prose.projection.push_str(text);
        }
        Content::Parts(parts) => {
            for part in parts {
                match part {
                    Part::Text { text } => {
                        prose.canonical.push_str(text);
                        prose.projection.push_str(text);
                    }
                    Part::File { filename } => context(
                        &mut prose,
                        &format!("File: {}", filename.as_deref().unwrap_or("attachment")),
                    ),
                    Part::Image => context(&mut prose, "Image attachment"),
                    Part::Unknown => context(&mut prose, "Unknown message part"),
                    // Redacted reasoning has nothing to show.
                    Part::RedactedReasoning => {}
                    Part::Reasoning { text } => {
                        flush(&mut prose, &mut entries);
                        if !text.trim().is_empty() {
                            entries.push(Entry::step(
                                Role::Assistant,
                                format!("[Thinking] {text}"),
                                Activity::default(),
                            ));
                        }
                    }
                    Part::ToolCall {
                        tool_call_id,
                        tool_name,
                        args,
                    } => {
                        flush(&mut prose, &mut entries);
                        entries.push(Entry::step(
                            Role::Assistant,
                            format!("Tool: {tool_name}\n{args}"),
                            Activity::call(
                                super::label(Some(tool_name)).as_deref(),
                                super::label(tool_call_id.as_deref()).as_deref(),
                                Some(args),
                            ),
                        ));
                    }
                    Part::ToolResult {
                        tool_call_id,
                        tool_name,
                        result,
                        is_error,
                    } => {
                        flush(&mut prose, &mut entries);
                        // The output as recorded; a result without one still
                        // marks that the call was answered.
                        let output = match result {
                            serde_json::Value::String(text) => text.clone(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        let output = if output.trim().is_empty() {
                            format!("Tool result: {tool_name}")
                        } else {
                            output
                        };
                        entries.push(Entry::step(
                            Role::Tool,
                            output,
                            Activity::result(
                                super::label(tool_call_id.as_deref()).as_deref(),
                                *is_error,
                            ),
                        ));
                    }
                }
            }
        }
    }
    flush(&mut prose, &mut entries);
    entries
}
