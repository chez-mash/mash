//! Native displayed prose is independent of attachments, reasoning, tool
//! activity and copies.
use super::{invalid, outcome::Outcome, proto, store::Root, timestamp};
use crate::{
    AppError, dialogue,
    message_context::MessageContext,
    provider::Provider,
    role::Role,
    tools::Activity,
    {NormalizedMessage, parsers},
};

/// One stored message from a step. An assistant step can hold reasoning, a
/// reply and tool calls; each becomes its own message, and only the reply
/// keeps the step's unsuffixed identity.
struct Part {
    suffix: Option<String>,
    role: Role,
    content: String,
    /// Displayed prose that may be searched; `None` is context only.
    prose: Option<String>,
    tools: Activity,
}

impl Part {
    fn prose(role: Role, prose: String, content: String) -> Self {
        Self {
            suffix: None,
            role,
            content,
            prose: Some(prose),
            tools: Activity::default(),
        }
    }
}

fn present(value: &str) -> Option<&str> {
    Some(value).filter(|value| !value.is_empty())
}

fn user(user: &proto::User) -> Part {
    let prose = if user.response.is_empty() {
        let fallback = user
            .items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<String>();
        if fallback.is_empty() {
            user.legacy_query.clone()
        } else {
            fallback
        }
    } else {
        user.response.clone()
    };
    let mut content = prose.clone();
    for block in user
        .items
        .iter()
        .filter_map(|i| i.scope.as_ref()?.text_block.as_ref())
    {
        if !block.content.is_empty() {
            content.push('\n');
            content.push_str(&block.label);
            content.push('\n');
            content.push_str(&block.content);
        }
    }
    Part::prose(Role::User, prose, content)
}

fn call(position: usize, call: &proto::ToolCall) -> Part {
    let name = present(&call.name);
    // Arguments are recorded as encoded JSON; anything else stays a string.
    let input = present(&call.arguments_json).map(|arguments| {
        serde_json::from_str(arguments)
            .unwrap_or_else(|_| serde_json::Value::String(arguments.to_owned()))
    });
    let mut content = format!("Tool: {}", name.unwrap_or("unknown"));
    if let Some(arguments) = present(&call.arguments_json) {
        content.push('\n');
        content.push_str(arguments);
    }
    Part {
        // A position cannot collide within the step, unlike a repeated ID.
        suffix: Some(format!("tool:{position}")),
        role: Role::Assistant,
        content,
        prose: None,
        tools: Activity::call(name, present(&call.id), input.as_ref()),
    }
}

fn assistant(assistant: &proto::Assistant) -> Vec<Part> {
    let mut parts = Vec::new();
    let thinking = if assistant.thinking.trim().is_empty() {
        &assistant.raw_thinking
    } else {
        &assistant.thinking
    };
    if !thinking.trim().is_empty() {
        parts.push(Part {
            suffix: Some("reasoning".to_owned()),
            role: Role::Assistant,
            content: format!("[Thinking] {thinking}"),
            prose: None,
            tools: Activity::default(),
        });
    }
    // The answer as displayed: an edited answer replaces the original.
    let reply = if assistant.modified_response.is_empty() {
        &assistant.response
    } else {
        &assistant.modified_response
    };
    parts.push(Part::prose(Role::Assistant, reply.clone(), reply.clone()));
    parts.extend(
        assistant
            .tool_calls
            .iter()
            .enumerate()
            .map(|(position, tool)| call(position, tool)),
    );
    parts
}

/// The call a tool step answers, named by its metadata.
fn answered(step: &proto::Step) -> Option<&proto::ToolCall> {
    step.metadata.as_ref()?.tool_call.as_ref()
}

/// A tool step is one tool's result. A generic step always is; any other
/// step type is one only when its metadata names the call it answers.
fn result(step: &proto::Step, outcome: &Outcome) -> Part {
    let is_error = match proto::Status::try_from(step.status) {
        _ if outcome.failed => Some(true),
        Ok(proto::Status::Done) => Some(false),
        Ok(proto::Status::Error) => Some(true),
        _ => None,
    };
    Part {
        suffix: None,
        role: Role::Tool,
        content: outcome.text.clone(),
        prose: None,
        tools: Activity::result(answered(step).and_then(|call| present(&call.id)), is_error),
    }
}

/// IDE cascades and the app's agent manager hold the user's own dialogue.
fn interactive(root: &Root) -> bool {
    matches!(
        proto::TrajectorySource::try_from(root.source),
        Ok(proto::TrajectorySource::CascadeClient | proto::TrajectorySource::GeminiApp)
    )
}

pub fn normalize(
    root: &Root,
    metadata: &proto::Metadata,
    index: i64,
    step: &proto::Step,
    outcome: Option<&Outcome>,
) -> Result<Vec<NormalizedMessage>, AppError> {
    let kind = proto::Kind::try_from(step.kind);
    if kind == Ok(proto::Kind::User) && step.user.is_none()
        || kind == Ok(proto::Kind::Assistant) && step.assistant.is_none()
    {
        return Err(invalid("missing native dialogue payload"));
    }
    if step.user.is_some() && (step.assistant.is_some() || kind != Ok(proto::Kind::User))
        || step.assistant.is_some() && kind != Ok(proto::Kind::Assistant)
    {
        return Err(invalid("native step type and dialogue payload disagree"));
    }
    let parts = if let Some(user_input) = &step.user {
        vec![user(user_input)]
    } else if let Some(response) = &step.assistant {
        assistant(response)
    } else if let Some(outcome) =
        outcome.filter(|_| kind == Ok(proto::Kind::Generic) || answered(step).is_some())
    {
        vec![result(step, outcome)]
    } else {
        return Ok(Vec::new());
    };
    let origin = step.metadata.as_ref();
    let copied = origin.and_then(|m| m.origin.as_ref()).is_some_and(|r| {
        r.trajectory != root.trajectory || r.cascade != root.cascade || i64::from(r.step) != index
    });
    let shown = proto::TrajectoryKind::try_from(root.kind) == Ok(proto::TrajectoryKind::Cascade)
        && interactive(root)
        && metadata.subagent.is_none()
        && !copied
        && proto::Status::try_from(step.status) == Ok(proto::Status::Done);
    let created_at = timestamp(origin.and_then(|m| m.created_at.as_ref()))?;
    let position = index.to_string();
    let mut messages = Vec::new();
    for part in parts {
        if part.content.trim().is_empty() {
            continue;
        }
        let author = match part.role {
            Role::User => proto::Source::UserExplicit,
            _ => proto::Source::Model,
        };
        let visible =
            shown && origin.is_some_and(|m| proto::Source::try_from(m.source) == Ok(author));
        let projection = match &part.prose {
            Some(prose) if visible => dialogue::visible(&part.role, prose).trim().to_owned(),
            _ => String::new(),
        };
        let mut identity = vec![
            Provider::Antigravity.as_str(),
            &root.cascade,
            &root.trajectory,
            &position,
        ];
        if let Some(suffix) = &part.suffix {
            identity.push(suffix);
        }
        messages.push(NormalizedMessage {
            attachments: Vec::new(),
            id: parsers::stable_id("message", &identity),
            ordinal: 0,
            context: MessageContext::Linear,
            role: part.role,
            content: part.content,
            dialogue_text: Some(projection),
            created_at,
            tools: part.tools,
        });
    }
    Ok(messages)
}
