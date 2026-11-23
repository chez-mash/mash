//! Chats an agent started for part of a task, as the agent itself marks them.
//!
//! Only the agent's own marker makes a chat a subagent: Claude Code's
//! sidechain transcripts (every record `isSidechain` with one `agentId`) and a
//! Codex rollout whose `session_meta.source` is `{"subagent": ...}`. Titles,
//! prompts and file names never do. A parent is named only when the marker
//! names it exactly; a fork (`forked_from_id` without a subagent source) is a
//! chat the person started, not a subagent.
use serde::Deserialize;
use serde::de::IgnoredAny;
use std::path::Path;

/// The longest label kept; labels are one-line descriptions or nicknames.
const MAX_LABEL_CHARS: usize = 200;
/// Claude Code's `.meta.json` files are a few hundred bytes.
const MAX_META_BYTES: u64 = 64 * 1024;

/// Where a chat came from, as its history records it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Origin {
    /// Set only when the agent marked this chat as one it started.
    pub subagent: Option<Subagent>,
    /// Read from the Claude desktop app's copy of a session it ran over SSH on
    /// another machine, which may index the original under the same chat ID.
    pub ssh_copy: bool,
}

impl Origin {
    pub fn of(subagent: Option<Subagent>) -> Self {
        Self {
            subagent,
            ssh_copy: false,
        }
    }
}

/// What the agent recorded about a chat it started.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Subagent {
    /// The chat that started it, from the same agent on the same machine.
    /// `None` when the marker names no parent (a Codex review, for example).
    pub parent_chat_id: Option<String>,
    /// Claude Code's task description or Codex's nickname.
    pub label: Option<String>,
    /// Claude Code's agent type or Codex's agent role.
    pub role: Option<String>,
    pub depth: Option<u32>,
    /// Only what the agent wrote; absence means unknown, not running.
    pub finished: Option<bool>,
    /// The parent's tool call that started it (Claude Code's `toolUseId`).
    pub spawn_call_id: Option<String>,
}

pub(crate) fn label(value: Option<&str>) -> Option<String> {
    let line = value?
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let text: String = line
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_LABEL_CHARS)
        .collect();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn identity(value: Option<&str>, own: &str) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != own && value.len() <= 256)
        .map(str::to_owned)
}

/// Codex's `session_meta.source`: a plain name such as `"cli"`, or an object.
/// Any other shape is accepted and means "not a subagent", so a new Codex
/// vocabulary never rejects a rollout.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum CodexSource {
    Subagent { subagent: CodexSubagent },
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CodexSubagent {
    Spawn {
        thread_spawn: ThreadSpawn,
    },
    /// `{"other": "guardian"}`.
    Other {
        other: String,
    },
    /// A plain kind such as `"review"`.
    Named(String),
    Unknown(IgnoredAny),
}

#[derive(Deserialize)]
pub struct ThreadSpawn {
    #[serde(default, deserialize_with = "lenient")]
    parent_thread_id: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    depth: Option<u32>,
    #[serde(default, deserialize_with = "lenient")]
    agent_nickname: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    agent_role: Option<String>,
}

/// A field of an unexpected type reads as absent rather than rejecting.
pub(crate) fn lenient<'de, D, T>(input: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = serde_json::Value::deserialize(input)?;
    Ok(serde_json::from_value(value).ok())
}

/// The fields beside `source` that also describe a Codex subagent.
#[derive(Default)]
pub struct CodexNames<'a> {
    pub nickname: Option<&'a str>,
    pub role: Option<&'a str>,
}

impl CodexSource {
    /// Decode the `source` column of Codex's thread store, which holds the
    /// same JSON or a bare name such as `cli`.
    pub fn from_column(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.starts_with('{') {
            serde_json::from_str(text).ok()
        } else {
            Some(Self::Other(IgnoredAny))
        }
    }

    /// The subagent this source describes, for the thread `own_id`.
    pub fn subagent(&self, own_id: &str, names: &CodexNames<'_>) -> Option<Subagent> {
        let Self::Subagent { subagent } = self else {
            return None;
        };
        let mut found = Subagent {
            label: label(names.nickname),
            role: label(names.role),
            ..Subagent::default()
        };
        match subagent {
            CodexSubagent::Spawn { thread_spawn } => {
                found.parent_chat_id = identity(thread_spawn.parent_thread_id.as_deref(), own_id);
                found.depth = thread_spawn.depth;
                if let Some(nickname) = label(thread_spawn.agent_nickname.as_deref()) {
                    found.label = Some(nickname);
                }
                if let Some(role) = label(thread_spawn.agent_role.as_deref()) {
                    found.role = Some(role);
                }
            }
            CodexSubagent::Other { other: kind } | CodexSubagent::Named(kind) => {
                found.role = found.role.or_else(|| label(Some(kind)));
            }
            CodexSubagent::Unknown(_) => {}
        }
        Some(found)
    }
}

/// Claude Code's `agent-<id>.meta.json` beside a subagent transcript.
#[derive(Deserialize)]
struct ClaudeMeta {
    #[serde(rename = "agentType", default, deserialize_with = "lenient")]
    agent_type: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    description: Option<String>,
    #[serde(rename = "toolUseId", default, deserialize_with = "lenient")]
    tool_use_id: Option<String>,
    #[serde(rename = "spawnDepth", default, deserialize_with = "lenient")]
    spawn_depth: Option<u32>,
    #[serde(default, deserialize_with = "lenient")]
    finished: Option<bool>,
    /// Set when another subagent, not the session, started this one.
    #[serde(rename = "parentAgentId", default, deserialize_with = "lenient")]
    parent_agent_id: Option<String>,
}

/// The metadata file Claude Code writes beside `agent-<agent>.jsonl`.
pub fn claude_meta_path(transcript: &Path, agent: &str) -> Option<std::path::PathBuf> {
    let name = transcript.file_name()?.to_str()?;
    (name == format!("agent-{agent}.jsonl"))
        .then(|| transcript.with_file_name(format!("agent-{agent}.meta.json")))
}

/// Whether a Claude Code history file may be a subagent transcript with a
/// metadata file beside it; discovery watches that file.
pub fn claude_meta_candidate(transcript: &Path) -> Option<std::path::PathBuf> {
    let name = transcript.file_name()?.to_str()?;
    let agent = name.strip_prefix("agent-")?.strip_suffix(".jsonl")?;
    (!agent.is_empty()).then(|| transcript.with_file_name(format!("agent-{agent}.meta.json")))
}

fn read_claude_meta(path: &Path) -> Option<ClaudeMeta> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    let mut text = String::new();
    file.take(MAX_META_BYTES).read_to_string(&mut text).ok()?;
    serde_json::from_str(&text).ok()
}

/// A Claude Code sidechain transcript of `agent` in `session`. The parent is
/// the session's own chat, or the subagent that started this one; both
/// identities are the ones the parser gives those chats.
pub fn claude(transcript: &Path, session: &str, agent: &str) -> Subagent {
    let meta = claude_meta_path(transcript, agent).and_then(|path| read_claude_meta(&path));
    let Some(meta) = meta else {
        return Subagent {
            parent_chat_id: Some(session.to_owned()),
            ..Subagent::default()
        };
    };
    let parent = match meta.parent_agent_id.as_deref().map(str::trim) {
        Some(parent) if !parent.is_empty() && parent != agent => {
            crate::parsers::stable_id("claude-agent", &[session, parent])
        }
        _ => session.to_owned(),
    };
    Subagent {
        parent_chat_id: Some(parent),
        label: label(meta.description.as_deref()),
        role: label(meta.agent_type.as_deref()),
        depth: meta.spawn_depth,
        finished: meta.finished,
        spawn_call_id: identity(meta.tool_use_id.as_deref(), ""),
    }
}

#[cfg(test)]
mod tests;
