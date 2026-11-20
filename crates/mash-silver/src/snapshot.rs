//! A reader's output as reviewable JSON, for snapshot tests of the provider
//! fixtures in `fixtures/`: the chat's metadata and every message it yields.
//! Review a changed snapshot with `cargo insta review`, or rerun the test with
//! `INSTA_UPDATE=always` and read the diff.
use crate::{Chat, NormalizedMessage};

#[derive(serde::Serialize)]
struct ChatSnapshot<'a> {
    id: &'a str,
    provider: mash_model::provider::Provider,
    project: Option<&'a str>,
    repository: Option<&'a str>,
    title: Option<&'a str>,
    created_at: Option<i64>,
    updated_at: Option<i64>,
    messages: &'a [NormalizedMessage],
}

/// The chat as a snapshot records it. Its source location is left out: it
/// names the test's temporary directory. Message IDs become their position
/// (`#0`, `#1`, ...), here and wherever a message's context names one,
/// because some providers' IDs are derived from that location.
pub fn snapshot(chat: &Chat) -> serde_json::Value {
    let positions: std::collections::HashMap<&str, String> = chat
        .messages
        .inline()
        .iter()
        .enumerate()
        .map(|(position, message)| (message.id.as_str(), format!("#{position}")))
        .collect();
    let mut value = serde_json::to_value(ChatSnapshot {
        id: &chat.id,
        provider: chat.provider,
        project: chat.project.as_deref(),
        repository: chat.repository.as_deref(),
        title: chat.title.as_deref(),
        created_at: chat.created_at,
        updated_at: chat.updated_at,
        messages: chat.messages.inline(),
    })
    .expect("a chat serializes");
    let messages = value["messages"].as_array_mut().into_iter().flatten();
    for message in messages {
        for pointer in ["/id", "/context/id"] {
            let Some(id) = message.pointer_mut(pointer) else {
                continue;
            };
            if let Some(position) = id.as_str().and_then(|text| positions.get(text)) {
                *id = serde_json::Value::String(position.clone());
            }
        }
    }
    value
}
