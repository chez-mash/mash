//! Reasoning, answers and tool activity from synthetic native steps.
use super::{fixture, history, parse, proto, store, wire};
use crate::{NormalizedMessage, role::Role};
use prost::{
    Message,
    encoding::{WireType, encode_key, encode_varint},
};
use rusqlite::{Connection, params};

fn root() -> store::Root {
    store::Root {
        trajectory: "trajectory".into(),
        cascade: "cascade".into(),
        kind: proto::TrajectoryKind::Cascade as i32,
        source: proto::TrajectorySource::CascadeClient as i32,
    }
}
fn answered(assistant: proto::Assistant) -> proto::Step {
    proto::Step {
        kind: proto::Kind::Assistant as i32,
        status: proto::Status::Done as i32,
        metadata: Some(proto::StepMetadata {
            source: proto::Source::Model as i32,
            ..Default::default()
        }),
        assistant: Some(assistant),
        ..Default::default()
    }
}
fn normalize(index: i64, step: &proto::Step) -> Vec<NormalizedMessage> {
    history::normalize(&root(), &proto::Metadata::default(), index, step).unwrap()
}

/// Protobuf written by field number, independent of the reader's own models.
fn number(tag: u32, value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_key(tag, WireType::Varint, &mut bytes);
    encode_varint(value, &mut bytes);
    bytes
}
fn nested(tag: u32, value: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_key(tag, WireType::LengthDelimited, &mut bytes);
    encode_varint(value.len() as u64, &mut bytes);
    bytes.extend_from_slice(value);
    bytes
}
fn tool_call(id: &str, name: &str, arguments: &str) -> Vec<u8> {
    [
        nested(1, id.as_bytes()),
        nested(2, name.as_bytes()),
        nested(3, arguments.as_bytes()),
        // A field this reader does not know.
        nested(9, b"original_name"),
    ]
    .concat()
}
/// A step as stored: type 1, status 4, metadata 5 and one body field.
fn stored(kind: u64, status: u64, metadata: &[u8], body: &[u8]) -> Vec<u8> {
    [
        number(1, kind),
        number(4, status),
        nested(5, metadata),
        body.to_vec(),
    ]
    .concat()
}
fn replace(connection: &Connection, index: i64, kind: i64, metadata: &[u8], payload: &[u8]) {
    let changed = connection
        .execute(
            "UPDATE steps SET step_type=?1,status=3,metadata=?2,step_payload=?3 WHERE idx=?4",
            params![kind, metadata, payload, index],
        )
        .unwrap();
    assert_eq!(changed, 1);
}

#[test]
fn antigravity_tool_call_and_result_are_decoded_and_paired_by_call_id() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture::write(directory.path());
    let connection = Connection::open(&path).unwrap();
    let model = number(3, 2);
    // Assistant step 8: response 1, thinking 3, tool calls 7.
    let assistant = [
        nested(1, b"I will look."),
        nested(3, b"synthetic plan"),
        nested(
            7,
            &tool_call("call-1", "list_dir", r#"{"path":"/synthetic"}"#),
        ),
        nested(7, &tool_call("call-2", "view_file", "not json")),
    ]
    .concat();
    replace(
        &connection,
        8,
        15,
        &model,
        &stored(15, 3, &model, &nested(20, &assistant)),
    );
    // Generic step 9 (type 132): body 140, result 2, text 1; the answered
    // call is metadata field 4.
    let answering = [
        model.clone(),
        nested(4, &tool_call("call-1", "list_dir", "")),
    ]
    .concat();
    let generic = [
        nested(1, &nested(1, b"path")),
        nested(2, &nested(1, b"synthetic listing")),
    ]
    .concat();
    replace(
        &connection,
        9,
        132,
        &answering,
        &stored(132, 3, &answering, &nested(140, &generic)),
    );
    // An unknown step type holding unknown fields stays skipped.
    replace(
        &connection,
        11,
        9999,
        &model,
        &stored(9999, 3, &model, &nested(4321, b"unknown body")),
    );
    drop(connection);

    let source = super::discover(&path).unwrap().remove(0).identity;
    let chat = parse(&source).unwrap().chat.unwrap();
    let messages = chat.messages.inline();
    for (position, message) in messages.iter().enumerate() {
        assert_eq!(message.ordinal, i64::try_from(position).unwrap());
    }
    let start = messages
        .iter()
        .position(|m| m.content == "[Thinking] synthetic plan")
        .unwrap();
    let [thinking, reply, first, second, result] = &messages[start..start + 5] else {
        panic!("the step's messages are missing");
    };
    assert_eq!(thinking.dialogue_text.as_deref(), Some(""));
    assert!(thinking.tools.is_empty());
    assert_eq!(reply.content, "I will look.");
    assert_eq!(reply.dialogue_text.as_deref(), Some("I will look."));
    assert!(reply.tools.is_empty());

    assert_eq!(first.role, Role::Assistant);
    assert_eq!(first.dialogue_text.as_deref(), Some(""));
    assert_eq!(first.content, "Tool: list_dir\n{\"path\":\"/synthetic\"}");
    let call = &first.tools.tool_calls[0];
    assert_eq!(call.name.as_deref(), Some("list_dir"));
    assert_eq!(call.call_id.as_deref(), Some("call-1"));
    assert_eq!(
        call.input,
        Some(serde_json::json!({ "path": "/synthetic" }))
    );
    // Arguments that are not JSON are kept verbatim.
    assert_eq!(
        second.tools.tool_calls[0].input,
        Some(serde_json::Value::String("not json".into()))
    );
    assert_eq!(
        second.tools.tool_calls[0].call_id.as_deref(),
        Some("call-2")
    );

    assert_eq!(result.role, Role::Tool);
    assert_eq!(result.content, "synthetic listing");
    assert_eq!(result.dialogue_text.as_deref(), Some(""));
    assert_eq!(result.tools.tool_calls, []);
    assert_eq!(result.tools.tool_results.len(), 1);
    assert_eq!(result.tools.tool_results[0].call_id, call.call_id);
    assert_eq!(result.tools.tool_results[0].is_error, Some(false));

    let mut ids: Vec<&str> = messages.iter().map(|m| m.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), messages.len());
    assert!(messages.iter().all(|m| !m.content.contains("unknown body")));
}

#[test]
fn antigravity_tool_only_step_is_kept() {
    let step = answered(proto::Assistant {
        tool_calls: vec![proto::ToolCall {
            id: "call-1".into(),
            name: "run_command".into(),
            arguments_json: String::new(),
        }],
        ..Default::default()
    });
    let messages = normalize(4, &step);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Tool: run_command");
    assert_eq!(messages[0].dialogue_text.as_deref(), Some(""));
    let call = &messages[0].tools.tool_calls[0];
    assert_eq!(call.name.as_deref(), Some("run_command"));
    assert_eq!(call.call_id.as_deref(), Some("call-1"));
    assert_eq!(call.input, None);
}

#[test]
fn antigravity_failed_tool_result_reports_its_error() {
    let step = proto::Step {
        kind: proto::Kind::Generic as i32,
        status: proto::Status::Error as i32,
        metadata: Some(proto::StepMetadata {
            source: proto::Source::Model as i32,
            tool_call: Some(proto::ToolCall {
                id: "call-1".into(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        generic: Some(proto::Generic {
            result: Some(proto::GenericResult {
                text: "synthetic failure".into(),
            }),
        }),
        ..Default::default()
    };
    let messages = normalize(5, &step);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, Role::Tool);
    assert_eq!(messages[0].content, "synthetic failure");
    assert_eq!(messages[0].dialogue_text.as_deref(), Some(""));
    let result = &messages[0].tools.tool_results[0];
    assert_eq!(result.call_id.as_deref(), Some("call-1"));
    assert_eq!(result.is_error, Some(true));
}

#[test]
fn antigravity_modified_answer_wins_over_the_original() {
    let messages = normalize(
        1,
        &answered(proto::Assistant {
            response: "original answer".into(),
            modified_response: "edited answer".into(),
            ..Default::default()
        }),
    );
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "edited answer");
    assert_eq!(messages[0].dialogue_text.as_deref(), Some("edited answer"));

    let unedited = normalize(
        1,
        &answered(proto::Assistant {
            response: "original answer".into(),
            ..Default::default()
        }),
    );
    assert_eq!(unedited[0].content, "original answer");
    assert_eq!(unedited[0].id, messages[0].id);
}

#[test]
fn antigravity_thinking_is_split_from_the_reply() {
    let messages = normalize(
        1,
        &answered(proto::Assistant {
            response: "the answer".into(),
            thinking: "internal analysis".into(),
            raw_thinking: "unsummarized analysis".into(),
            ..Default::default()
        }),
    );
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, Role::Assistant);
    assert_eq!(messages[0].content, "[Thinking] internal analysis");
    assert_eq!(messages[0].dialogue_text.as_deref(), Some(""));
    assert_eq!(messages[1].content, "the answer");
    assert_eq!(messages[1].dialogue_text.as_deref(), Some("the answer"));
    assert_ne!(messages[0].id, messages[1].id);

    // Reasoning with no answer is still only reasoning.
    let alone = normalize(
        1,
        &answered(proto::Assistant {
            raw_thinking: "unsummarized analysis".into(),
            ..Default::default()
        }),
    );
    assert_eq!(alone.len(), 1);
    assert_eq!(alone[0].content, "[Thinking] unsummarized analysis");
    assert_eq!(alone[0].dialogue_text.as_deref(), Some(""));
    assert_eq!(alone[0].id, messages[0].id);
}

#[test]
fn antigravity_empty_thinking_is_dropped() {
    let messages = normalize(
        1,
        &answered(proto::Assistant {
            response: "the answer".into(),
            thinking: " \n".into(),
            raw_thinking: String::new(),
            ..Default::default()
        }),
    );
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "the answer");
}

#[test]
fn antigravity_reply_id_is_unchanged() {
    // The identity this reader gave the step's one message before reasoning
    // and tool calls became messages of their own.
    let messages = normalize(
        7,
        &answered(proto::Assistant {
            response: "the answer".into(),
            thinking: "internal analysis".into(),
            tool_calls: vec![proto::ToolCall::default()],
            ..Default::default()
        }),
    );
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1].content, "the answer");
    assert_eq!(messages[1].id, "27653bedd44c20aa805e4de5");
    assert_eq!(messages[2].content, "Tool: unknown");
}

#[test]
fn antigravity_unknown_fields_and_step_types_are_skipped() {
    let model = number(3, 2);
    let mut fields = 64;
    let step = wire::step(
        &stored(9999, 3, &model, &nested(4321, b"unknown body")),
        Some(&model),
        &mut fields,
    )
    .unwrap();
    assert_eq!(normalize(0, &step), []);

    // A generic step with no body, and a body on another step type.
    let empty = wire::step(&stored(132, 3, &model, &[]), Some(&model), &mut fields).unwrap();
    assert_eq!(normalize(1, &empty), []);
    let other = proto::Step {
        kind: 17,
        generic: Some(proto::Generic {
            result: Some(proto::GenericResult {
                text: "not a tool result".into(),
            }),
        }),
        ..Default::default()
    };
    assert_eq!(normalize(2, &other), []);

    // Repeated tool calls count against the structure limit.
    let calls = nested(7, &tool_call("a", "b", "")).repeat(3);
    let mut few = 6;
    assert!(
        wire::step(
            &stored(15, 3, &model, &nested(20, &calls)),
            Some(&model),
            &mut few
        )
        .is_err()
    );
    assert_eq!(
        proto::Step::decode(stored(15, 3, &model, &nested(20, &calls)).as_slice())
            .unwrap()
            .assistant
            .unwrap()
            .tool_calls
            .len(),
        3
    );
}
