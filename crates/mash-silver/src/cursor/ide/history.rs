//! Ordered native bubbles, visibility normalization and stable message identities.
use super::super::text;
use super::{
    AppError, invalid,
    models::{self, Bubble, Kind, Root},
    records,
    store::Store,
    timestamp,
};
use crate::tools::Activity;
use crate::{NormalizedMessage, message_context::MessageContext, provider::Provider, role::Role};
use std::{collections::BTreeSet, fmt::Write};

fn recorded_time(bubble: &Bubble, header: &models::Header) -> Result<Option<i64>, AppError> {
    let time = bubble
        .created_at
        .as_deref()
        .or(header.created_at.as_deref());
    if let Some(time) = time.filter(|time| !time.is_empty()) {
        return timestamp(Some(
            chrono::DateTime::parse_from_rfc3339(time)
                .map_err(|_| invalid("invalid native ISO timestamp"))?
                .timestamp_millis(),
        ));
    }
    timestamp(bubble.started_at_ms.or(bubble.completed_at_ms))
}

/// A tool payload as recorded: encoded JSON is decoded, anything else kept.
fn payload(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => {
            serde_json::from_str(text).unwrap_or_else(|_| value.clone())
        }
        other => other.clone(),
    }
}

fn shown(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// One displayed step of a bubble. `suffix` keeps the bubble's own identity
/// for its text and gives its thinking and tool steps their own.
struct Part {
    suffix: Option<&'static str>,
    role: Role,
    text: text::Text,
    tools: Activity,
}

impl Part {
    fn step(suffix: &'static str, role: Role, canonical: String, tools: Activity) -> Self {
        Self {
            suffix: Some(suffix),
            role,
            text: text::Text {
                canonical,
                projection: String::new(),
            },
            tools,
        }
    }
}

fn normalized(bubble: &Bubble, child: bool) -> Result<Vec<Part>, AppError> {
    let role = match bubble.kind {
        Kind::Human => Role::User,
        Kind::Assistant => Role::Assistant,
        kind => Role::Other(format!("cursor-message-type-{}", i32::from(kind))),
    };
    // Older releases attach a tool record holding only `additionalData` to
    // ordinary prompts and replies. Only one that names a tool or carries a
    // payload is a tool step.
    let tool = bubble.tool_former_data.as_ref().filter(|tool| {
        tool.id.is_some() || tool.name.is_some() || tool.params.is_some() || tool.result.is_some()
    });
    let mut parts = Vec::new();
    if let Some(thinking) = bubble
        .thinking
        .as_ref()
        .filter(|thinking| !thinking.text.trim().is_empty())
    {
        parts.push(Part::step(
            "thinking",
            Role::Assistant,
            format!("[Thinking] {}", thinking.text),
            Activity::default(),
        ));
    }
    let mut text = text::Text {
        canonical: bubble.text.clone(),
        projection: text::projection(&role, &bubble.text),
    };
    if child
        || bubble.is_display_only
        || bubble.is_simulated_msg
        || bubble.simulated_msg_reason.is_some_and(|r| r != 0)
        || bubble
            .simulated_message_metadata
            .as_ref()
            .is_some_and(|m| m.suppress_ui_display)
        || bubble.is_placeholder
        || bubble.service_status_update.is_some()
        || bubble.event_notification.is_some()
        || bubble.capability_type.is_some()
        || tool.is_some()
        || bubble.sent_by_agent_id.is_some()
    {
        text.projection.clear();
    }
    if let Some(status) = &bubble.service_status_update {
        let message = match status {
            models::ProtoRecord::Object(status) => status.message.clone(),
            models::ProtoRecord::Json(json) => {
                records::decode::<models::Status>(json.as_bytes())?.message
            }
        };
        let _ = write!(text.canonical, "\nStatus: {message}");
    }
    if let Some(notification) = &bubble.event_notification {
        let _ = write!(text.canonical, "\nNotification: {}", notification.headline);
        if let Some(detail) = &notification.detail {
            let _ = write!(text.canonical, "\n{detail}");
        }
    }
    parts.push(Part {
        suffix: None,
        role,
        text,
        tools: Activity::default(),
    });
    if let Some(tool) = tool {
        let name = tool
            .name
            .clone()
            .or_else(|| tool.id.map(|id| format!("tool {id}")));
        let input = tool.params.as_ref().map(payload);
        let mut call = format!("Tool: {}", name.as_deref().unwrap_or("unknown"));
        if let Some(params) = &tool.params {
            call.push('\n');
            call.push_str(&shown(params));
        }
        let name = super::super::label(name.as_deref());
        let call_id = super::super::label(tool.tool_call_id.as_deref());
        let call_id = call_id.as_deref();
        parts.push(Part::step(
            "tool",
            Role::Assistant,
            call,
            Activity::call(name.as_deref(), call_id, input.as_ref()),
        ));
        let failed = match tool.status.as_deref() {
            Some("completed") => Some(false),
            Some("error") => Some(true),
            _ => None,
        };
        if let Some(output) = tool.result.as_ref().or(tool.error.as_ref()) {
            parts.push(Part::step(
                "result",
                Role::Tool,
                shown(output),
                Activity::result(call_id, failed),
            ));
        }
    }
    Ok(parts)
}

/// The bubble order: inline on older roots, in numbered chunk records on
/// roots that keep a header log.
fn headers(store: &mut Store, root: &Root, id: &str) -> Result<Vec<models::Header>, AppError> {
    let Some(log) = root
        .header_log
        .as_ref()
        .filter(|log| root.full_conversation_headers_only.is_empty() && log.chunk_count > 0)
    else {
        return Ok(Vec::new());
    };
    if log.chunk_count > 4096 {
        return Err(invalid("too many header chunks"));
    }
    let mut headers = Vec::new();
    for chunk in 0..log.chunk_count {
        let part: Vec<models::Header> =
            store.json(&format!("composerConvHeaders:{id}:{chunk:06}"))?;
        headers.extend(part);
    }
    if u64::try_from(headers.len()).ok() != Some(log.count) {
        return Err(invalid("header chunks disagree with the header log"));
    }
    Ok(headers)
}

pub fn read(store: &mut Store, root: &Root, id: &str) -> Result<Vec<NormalizedMessage>, AppError> {
    let mut ids = BTreeSet::new();
    let mut messages = Vec::new();
    let mut bytes = 0usize;
    let chunked = headers(store, root, id)?;
    let listed = if chunked.is_empty() {
        &root.full_conversation_headers_only
    } else {
        &chunked
    };
    for header in listed {
        if header.bubble_id.trim().is_empty() || !ids.insert(&header.bubble_id) {
            return Err(invalid("invalid or duplicate reachable bubble identity"));
        }
        if header.is_display_only {
            continue;
        }
        let bubble: Bubble = store.json(&format!("bubbleId:{id}:{}", header.bubble_id))?;
        if bubble.id != header.bubble_id || bubble.kind != header.kind {
            return Err(invalid("bubble and header identities or types disagree"));
        }
        if bubble.is_display_only {
            continue;
        }
        let created_at = recorded_time(&bubble, header)?;
        for part in normalized(&bubble, root.subagent_info.is_some())? {
            if part.text.canonical.trim().is_empty() {
                continue;
            }
            bytes = bytes
                .saturating_add(part.text.canonical.capacity())
                .saturating_add(part.text.projection.capacity())
                .saturating_add(part.tools.retained_size())
                .saturating_add(512);
            if bytes > 32 * 1024 * 1024 || messages.len() == 65_536 {
                return Err(invalid(
                    "normalized history exceeds the retained replay limit",
                ));
            }
            let mut identity = vec![Provider::Cursor.as_str(), "ide", id, &bubble.id];
            identity.extend(part.suffix);
            messages.push(NormalizedMessage {
                attachments: Vec::new(),
                id: crate::parsers::stable_id("message", &identity),
                ordinal: i64::try_from(messages.len()).map_err(|_| invalid("too many bubbles"))?,
                context: MessageContext::Linear,
                role: part.role,
                content: part.text.canonical,
                dialogue_text: Some(part.text.projection),
                created_at,
                tools: part.tools,
            });
        }
    }
    Ok(messages)
}
