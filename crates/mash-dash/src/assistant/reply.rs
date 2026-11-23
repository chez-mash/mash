//! The model service's streamed reply: one `data:` line of JSON per chunk,
//! carrying text, fragments of requested lookups, and what the reply cost.
//! Lines arrive through the shared line decoder; this file only reads them.
use super::{Call, MAX_LOOKUPS, MAX_TURN};

#[derive(serde::Deserialize)]
pub(super) struct Chunk {
    #[serde(default)]
    pub(super) choices: Vec<Choice>,
    /// Some services report a failure inside the stream.
    #[serde(default)]
    pub(super) error: Option<serde::de::IgnoredAny>,
    #[serde(default)]
    pub(super) usage: Option<Usage>,
}

/// What the service says a reply cost, when it says.
#[derive(Clone, Copy, Default, serde::Deserialize)]
pub(super) struct Usage {
    #[serde(default)]
    pub(super) prompt_tokens: u64,
    #[serde(default)]
    pub(super) completion_tokens: u64,
}

#[derive(serde::Deserialize)]
pub(super) struct Choice {
    #[serde(default)]
    pub(super) delta: Delta,
    /// Why the reply ended: `stop`, `tool_calls`, `length` and others.
    #[serde(default)]
    pub(super) finish_reason: Option<String>,
}

#[derive(Default, serde::Deserialize)]
pub(super) struct Delta {
    #[serde(default)]
    pub(super) content: Option<String>,
    #[serde(default)]
    pub(super) tool_calls: Vec<CallDelta>,
}

#[derive(serde::Deserialize)]
pub(super) struct CallDelta {
    #[serde(default)]
    pub(super) index: usize,
    #[serde(default)]
    pub(super) id: Option<String>,
    #[serde(default)]
    pub(super) function: Option<FunctionDelta>,
}

#[derive(serde::Deserialize)]
pub(super) struct FunctionDelta {
    #[serde(default)]
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) arguments: Option<String>,
}

/// One line of the model's stream, when it carries data.
pub(super) fn chunk(line: &[u8]) -> Option<Chunk> {
    let data = std::str::from_utf8(line)
        .ok()?
        .strip_prefix("data:")?
        .trim();
    serde_json::from_str(data).ok()
}

/// Adds one fragment of a requested lookup to the ones gathered so far.
pub(super) fn gather(calls: &mut Vec<Call>, delta: CallDelta) {
    // More calls than lookups allowed are never run, so they are not kept.
    if delta.index > MAX_LOOKUPS {
        return;
    }
    while calls.len() <= delta.index {
        calls.push(Call {
            r#type: "function",
            ..Call::default()
        });
    }
    let call = &mut calls[delta.index];
    if let Some(id) = delta.id {
        call.id = id;
    }
    if let Some(function) = delta.function {
        if let Some(name) = function.name {
            call.function.name.push_str(&name);
        }
        if let Some(arguments) = function.arguments
            && call.function.arguments.len() < MAX_TURN
        {
            call.function.arguments.push_str(&arguments);
        }
    }
}
