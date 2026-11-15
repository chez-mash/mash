//! Reasoning, answers and tool activity from synthetic native steps.
use super::{fixture, history, outcome, parse, proto, store, wire};
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
    history::normalize(&root(), &proto::Metadata::default(), index, step, None).unwrap()
}
/// A stored step read as the store reads it: decoded, then its outcome.
fn read(index: i64, payload: &[u8], metadata: &[u8]) -> Vec<NormalizedMessage> {
    let mut fields = 4096;
    let step = wire::step(payload, Some(metadata), &mut fields).unwrap();
    let outcome = outcome::read(step.kind, payload, &mut fields).unwrap();
    history::normalize(
        &root(),
        &proto::Metadata::default(),
        index,
        &step,
        outcome.as_ref(),
    )
    .unwrap()
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
/// Model-authored metadata (source 3) answering one call (`tool_call` 4).
fn answering(call: &str) -> Vec<u8> {
    [
        number(3, 2),
        nested(4, &tool_call(call, "synthetic_tool", "")),
    ]
    .concat()
}
/// The one result of a tool step of this type and status, whose body is
/// stored in step field `field`.
fn tool_result(kind: u64, status: u64, field: u32, body: &[u8]) -> NormalizedMessage {
    let metadata = answering("call-9");
    // Unknown fields beside and inside the body are skipped.
    let body = [nested(4000, b"unknown"), body.to_vec(), number(4001, 7)].concat();
    let payload = [
        stored(kind, status, &metadata, &nested(field, &body)),
        nested(4321, b"unknown body"),
    ]
    .concat();
    let mut messages = read(3, &payload, &metadata);
    assert_eq!(messages.len(), 1, "step type {kind}");
    let message = messages.remove(0);
    assert_eq!(message.role, Role::Tool);
    assert_eq!(message.dialogue_text.as_deref(), Some(""));
    assert_eq!(message.tools.tool_calls, []);
    assert_eq!(message.tools.tool_results.len(), 1);
    assert_eq!(
        message.tools.tool_results[0].call_id.as_deref(),
        Some("call-9")
    );
    message
}
fn failed(message: &NormalizedMessage) -> Option<bool> {
    message.tools.tool_results[0].is_error
}
/// One named regression per tool step type: step type, body field and the
/// body's fields by number, giving this text from a finished step.
macro_rules! reads_tool_results {
    ($($name:ident: $kind:literal, $field:literal, $body:expr => $text:literal;)*) => {$(
        #[test]
        fn $name() {
            let message = tool_result($kind, 3, $field, &$body);
            assert_eq!(message.content, $text);
            assert_eq!(failed(&message), Some(false));
        }
    )*};
}
reads_tool_results! {
    antigravity_view_file_result_is_read: 8, 14,
        [nested(1, b"file:///synthetic"), number(2, 1), nested(4, b"synthetic file")].concat()
        => "synthetic file";
    antigravity_view_file_raw_content_is_the_fallback: 8, 14, nested(9, b"raw file")
        => "raw file";
    antigravity_list_directory_children_are_read: 9, 15,
        [nested(1, b"file:///synthetic"), nested(2, b"a.txt"), nested(2, b"b")].concat()
        => "a.txt\nb";
    antigravity_list_directory_result_names_are_the_fallback: 9, 15,
        [
            nested(3, &[nested(1, b"a.txt"), number(4, 12)].concat()),
            nested(3, &[nested(1, b"b"), number(2, 1)].concat()),
        ]
        .concat()
        => "a.txt\nb";
    antigravity_grep_search_result_is_read: 7, 13,
        [nested(1, b"needle"), nested(3, b"a.txt:1:needle")].concat() => "a.txt:1:needle";
    antigravity_find_result_is_read: 25, 34,
        [nested(1, b"*.txt"), nested(11, b"a.txt"), nested(14, b"a")].concat() => "a.txt";
    antigravity_find_truncated_output_is_the_fallback: 25, 34, nested(14, b"a.txt")
        => "a.txt";
    antigravity_view_file_outline_result_is_read: 47, 58, nested(7, b"fn main") => "fn main";
    antigravity_view_file_outline_items_are_the_fallback: 47, 58,
        [nested(9, b"fn a"), nested(9, b"fn b")].concat() => "fn a\nfn b";
    antigravity_run_command_result_is_read: 21, 28,
        [
            nested(23, b"synthetic --flag"),
            number(6, 0),
            nested(21, &[nested(1, b"full output"), nested(2, b"output")].concat()),
            nested(4, b"stdout"),
        ]
        .concat()
        => "full output";
    antigravity_run_command_truncated_output_is_the_fallback: 21, 28,
        nested(21, &nested(2, b"output")) => "output";
    antigravity_run_command_stdout_is_the_fallback: 21, 28,
        [nested(4, b"stdout"), nested(5, b"stderr")].concat() => "stdout";
    antigravity_command_status_result_is_read: 28, 37,
        [nested(1, b"command-1"), nested(9, b"combined"), nested(12, b"delta")].concat()
        => "combined";
    antigravity_send_command_input_result_is_read: 100, 113,
        [nested(2, b"y"), nested(8, &nested(1, b"continued"))].concat() => "continued";
    antigravity_read_terminal_result_is_read: 65, 77,
        [nested(2, b"zsh"), nested(3, b"terminal text")].concat() => "terminal text";
    antigravity_shell_exec_result_is_read: 112, 127,
        [nested(1, b"synthetic"), number(2, 0), nested(3, b"shell output")].concat()
        => "shell output";
    antigravity_run_extension_code_result_is_read: 57, 68,
        [nested(1, b"code"), nested(3, b"extension output")].concat() => "extension output";
    antigravity_mcp_tool_result_is_read: 38, 47,
        [nested(1, b"server"), nested(3, b"mcp result")].concat() => "mcp result";
    antigravity_list_resources_result_is_read: 51, 62,
        [
            nested(3, &[nested(1, b"res://a"), nested(2, b"A")].concat()),
            nested(3, &nested(1, b"res://b")),
        ]
        .concat()
        => "res://a\nres://b";
    antigravity_read_resource_result_is_read: 52, 63,
        nested(3, &[nested(1, b"res://a"), nested(2, &nested(1, b"resource text"))].concat())
        => "resource text";
    antigravity_read_url_content_result_is_read: 31, 40,
        [nested(1, b"https://synthetic.invalid"), nested(2, &nested(2, b"page text"))].concat()
        => "page text";
    antigravity_view_content_chunk_result_is_read: 32, 41,
        nested(4, &[nested(1, b"document"), nested(2, b"chunk text")].concat())
        => "chunk text";
    antigravity_search_web_result_is_read: 33, 42,
        [nested(1, b"query"), nested(5, b"search summary")].concat() => "search summary";
    antigravity_workspace_api_result_is_read: 122, 136,
        [nested(1, b"https://synthetic.invalid"), number(5, 200), nested(6, b"response")]
            .concat()
        => "response";
    antigravity_browser_subagent_result_is_read: 85, 97,
        [nested(1, b"task"), nested(2, b"subagent result")].concat() => "subagent result";
    antigravity_execute_browser_javascript_result_is_read: 61, 73,
        [nested(2, b"1 + 1"), nested(8, b"2"), nested(11, b"diff")].concat() => "2";
    antigravity_read_browser_page_result_is_read: 67, 79,
        nested(2, &nested(2, b"page text")) => "page text";
    antigravity_browser_get_dom_result_is_read: 68, 80, nested(3, b"<p>dom</p>")
        => "<p>dom</p>";
    antigravity_capture_browser_console_logs_result_is_read: 66, 78,
        nested(
            3,
            &[
                nested(1, &[nested(2, b"log"), nested(3, b"first")].concat()),
                nested(1, &nested(3, b"second")),
            ]
            .concat(),
        )
        => "first\nsecond";
    antigravity_browser_list_network_requests_result_is_read: 123, 137,
        nested(5, b"GET /synthetic") => "GET /synthetic";
    antigravity_browser_get_network_request_result_is_read: 124, 138,
        nested(4, b"200 OK") => "200 OK";
    antigravity_open_browser_url_result_is_read: 56, 67,
        [nested(1, b"https://synthetic.invalid"), nested(9, b"page opened")].concat()
        => "page opened";
    antigravity_click_browser_pixel_result_is_read: 64, 76, nested(8, b"clicked")
        => "clicked";
    antigravity_browser_click_element_result_is_read: 79, 91, nested(7, b"clicked")
        => "clicked";
    antigravity_browser_resize_window_result_is_read: 96, 109, nested(7, b"resized")
        => "resized";
    antigravity_browser_input_result_is_read: 74, 86,
        [nested(3, b"typed"), nested(7, b"field changed")].concat() => "field changed";
    antigravity_browser_move_mouse_result_is_read: 75, 87, nested(7, b"moved") => "moved";
    antigravity_browser_select_option_result_is_read: 76, 88,
        [nested(3, b"option"), nested(7, b"selected")].concat() => "selected";
    antigravity_browser_scroll_up_result_is_read: 77, 89, nested(5, b"scrolled up")
        => "scrolled up";
    antigravity_browser_scroll_down_result_is_read: 78, 90, nested(5, b"scrolled down")
        => "scrolled down";
    antigravity_browser_press_key_result_is_read: 80, 92,
        [nested(2, b"Enter"), nested(4, b"submitted")].concat() => "submitted";
    antigravity_browser_scroll_result_is_read: 88, 101, nested(8, b"scrolled") => "scrolled";
    antigravity_browser_mouse_wheel_result_is_read: 113, 125, nested(7, b"wheeled")
        => "wheeled";
    antigravity_browser_mouse_up_result_is_read: 120, 134, nested(4, b"released")
        => "released";
    antigravity_browser_mouse_down_result_is_read: 121, 135, nested(4, b"pressed")
        => "pressed";
    antigravity_browser_refresh_page_result_is_read: 125, 139, nested(3, b"refreshed")
        => "refreshed";
    antigravity_read_notebook_result_is_read: 129, 144,
        [nested(1, b"read_notebook"), nested(3, &nested(1, b"cell text"))].concat()
        => "cell text";
    antigravity_execute_notebook_result_is_read: 134, 150,
        nested(
            3,
            &[
                number(1, 1),
                nested(3, &[nested(1, b"cell-1"), nested(4, b"one")].concat()),
                nested(3, &nested(4, b"two")),
            ]
            .concat(),
        )
        => "one\ntwo";
    antigravity_compile_applet_result_is_read: 93, 106, nested(3, b"compiled")
        => "compiled";
    antigravity_install_applet_dependencies_result_is_read: 94, 107, nested(2, b"installed")
        => "installed";
    antigravity_install_applet_package_result_is_read: 95, 108,
        [nested(1, b"package"), nested(3, b"installed")].concat() => "installed";
    antigravity_lint_applet_result_is_read: 114, 126,
        [number(1, 0), nested(2, b"no problems")].concat() => "no problems";
    antigravity_cloud_sql_update_schema_result_is_read: 135, 151, nested(4, b"updated")
        => "updated";
    antigravity_cloud_sql_execute_sql_result_is_read: 137, 153,
        [nested(3, b"select 1"), nested(5, b"1 row")].concat() => "1 row";
    antigravity_generic_result_is_read: 132, 140,
        [nested(1, &nested(1, b"path")), nested(2, &nested(1, b"generic result"))].concat()
        => "generic result";
}

#[test]
fn antigravity_tool_result_failures_are_reported() {
    // A non-zero exit code (6), also when negative, and a rejected call (14).
    let exited = tool_result(
        21,
        3,
        28,
        &[number(6, 2), nested(21, &nested(1, b"failed output"))].concat(),
    );
    assert_eq!(exited.content, "failed output");
    assert_eq!(failed(&exited), Some(true));
    let negative = [number(6, u64::MAX), nested(4, b"killed")].concat();
    assert_eq!(failed(&tool_result(21, 3, 28, &negative)), Some(true));
    let rejected = [number(14, 1), nested(4, b"not run")].concat();
    assert_eq!(failed(&tool_result(21, 3, 28, &rejected)), Some(true));
    for (kind, field, body) in [
        (28, 37, [number(5, 1), nested(9, b"x")].concat()),
        (
            100,
            113,
            [number(10, 1), nested(8, &nested(1, b"x"))].concat(),
        ),
        (
            100,
            113,
            [number(4, 1), nested(8, &nested(1, b"x"))].concat(),
        ),
        (112, 127, [number(2, 1), nested(3, b"x")].concat()),
        (114, 126, [number(1, 1), nested(2, b"x")].concat()),
        (9, 15, [number(4, 1), nested(2, b"x")].concat()),
        (38, 47, [number(7, 1), nested(3, b"x")].concat()),
        (57, 68, [number(4, 1), nested(3, b"x")].concat()),
        (31, 40, [number(5, 1), nested(2, &nested(2, b"x"))].concat()),
        (61, 73, [number(4, 1), nested(8, b"x")].concat()),
        (56, 67, [number(3, 1), nested(9, b"x")].concat()),
        (64, 76, [number(4, 1), nested(8, b"x")].concat()),
        (79, 91, [number(4, 1), nested(7, b"x")].concat()),
        (96, 109, [number(4, 1), nested(7, b"x")].concat()),
    ] {
        let message = tool_result(kind, 3, field, &body);
        assert_eq!(message.content, "x");
        assert_eq!(failed(&message), Some(true), "step type {kind}");
    }
    // Error text marks a failure and is the result when nothing else is.
    for (kind, field, error) in [
        (7, 13, 5),
        (25, 34, 8),
        (112, 127, 4),
        (93, 106, 1),
        (94, 107, 1),
        (95, 108, 2),
        (114, 126, 3),
        (135, 151, 1),
        (137, 153, 4),
    ] {
        let message = tool_result(kind, 3, field, &nested(error, b"synthetic error"));
        assert_eq!(message.content, "synthetic error", "step type {kind}");
        assert_eq!(failed(&message), Some(true), "step type {kind}");
    }
    let output = [nested(3, b"partial"), nested(5, b"synthetic error")].concat();
    let message = tool_result(7, 3, 13, &output);
    assert_eq!(message.content, "partial");
    assert_eq!(failed(&message), Some(true));
    // A notebook's own error trace (reply 3, trace 4), then a cell's (7).
    for reply in [
        nested(4, b"synthetic error"),
        nested(3, &nested(7, b"synthetic error")),
    ] {
        let message = tool_result(134, 3, 150, &nested(3, &reply));
        assert_eq!(message.content, "synthetic error");
        assert_eq!(failed(&message), Some(true));
    }
    // The step's status reports a failure the body does not, and an
    // unfinished step reports neither outcome.
    let body = nested(4, b"synthetic file");
    assert_eq!(failed(&tool_result(8, 7, 14, &body)), Some(true));
    assert_eq!(failed(&tool_result(8, 2, 14, &body)), None);
    assert_eq!(
        failed(&tool_result(132, 7, 140, &nested(2, &nested(1, b"x")))),
        Some(true)
    );
}

#[test]
fn antigravity_failed_tool_step_without_output_shows_the_step_error() {
    let metadata = answering("call-9");
    // Step error 31: the model's message 9 wins over the user's 1 and the
    // short form 2.
    for (details, text) in [
        (
            [nested(1, b"for the user"), nested(9, b"for the model")].concat(),
            "for the model",
        ),
        (
            [nested(2, b"short"), nested(1, b"for the user")].concat(),
            "for the user",
        ),
        (nested(2, b"short"), "short"),
    ] {
        let payload = [
            stored(
                8,
                7,
                &metadata,
                &nested(14, &nested(1, b"file:///synthetic")),
            ),
            nested(31, &details),
        ]
        .concat();
        let messages = read(3, &payload, &metadata);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, text);
        assert_eq!(failed(&messages[0]), Some(true));
        // A step with no body at all still reports its error.
        let bare = [stored(8, 7, &metadata, &[]), nested(31, &details)].concat();
        assert_eq!(read(3, &bare, &metadata)[0].content, text);
    }
    // Output the step did record is kept.
    let payload = [
        stored(8, 7, &metadata, &nested(14, &nested(4, b"synthetic file"))),
        nested(31, &nested(9, b"for the model")),
    ]
    .concat();
    assert_eq!(read(3, &payload, &metadata)[0].content, "synthetic file");
}

#[test]
fn antigravity_tool_step_without_a_call_or_output_is_skipped() {
    // A file view nobody called for: no tool call in its metadata.
    let model = number(3, 2);
    let viewed = nested(14, &nested(4, b"synthetic file"));
    assert_eq!(read(3, &stored(8, 3, &model, &viewed), &model), []);
    // A generic step is a tool's result even when its call is unnamed.
    let generic = nested(140, &nested(2, &nested(1, b"generic result")));
    let unnamed = read(3, &stored(132, 3, &model, &generic), &model);
    assert_eq!(unnamed.len(), 1);
    assert_eq!(unnamed[0].tools.tool_results[0].call_id, None);
    // A called step with no output, or only blank output, is dropped.
    let metadata = answering("call-9");
    for body in [Vec::new(), nested(4, b" \n")] {
        let payload = stored(8, 3, &metadata, &nested(14, &body));
        assert_eq!(read(3, &payload, &metadata), []);
    }
    // Another step type's body field is not this type's result.
    let misplaced = nested(140, &nested(2, &nested(1, b"generic result")));
    assert_eq!(read(3, &stored(8, 3, &metadata, &misplaced), &metadata), []);
    // Tool steps this reader has no result text for stay skipped: a file
    // edit (5, body 10) and an unknown type.
    for (kind, field) in [(5, 10), (9999, 4321)] {
        let payload = stored(kind, 3, &metadata, &nested(field, &nested(4, b"unread")));
        assert_eq!(read(3, &payload, &metadata), []);
    }
}

#[test]
fn antigravity_malformed_tool_result_is_rejected() {
    let metadata = answering("call-9");
    let mut fields = 64;
    let read = |payload: &[u8], fields: &mut usize| outcome::read(8, payload, fields);
    // Two bodies for one step.
    let body = nested(14, &nested(4, b"synthetic file"));
    let twice = [stored(8, 3, &metadata, &body), body.clone()].concat();
    assert!(read(&twice, &mut fields).is_err());
    // Text that is not UTF-8, and a body cut short.
    let binary = stored(8, 3, &metadata, &nested(14, &nested(4, &[0xff, 0xfe])));
    assert!(read(&binary, &mut fields).is_err());
    let cut = stored(8, 3, &metadata, &nested(14, &[0x22, 0x7f, b'a']));
    assert!(read(&cut, &mut fields).is_err());
    // Repeated result values count against the structure limit.
    let children = nested(2, b"a").repeat(5);
    let listing = stored(9, 3, &metadata, &nested(15, &children));
    let mut few = 4;
    assert!(outcome::read(9, &listing, &mut few).is_err());
    let mut enough = 5;
    assert_eq!(
        outcome::read(9, &listing, &mut enough)
            .unwrap()
            .unwrap()
            .text,
        "a\na\na\na\na"
    );
}

#[test]
fn antigravity_tool_result_ids_are_unchanged_and_shared_by_step_position() {
    // The identity a generic result had before other step types were read:
    // the step's unsuffixed one, as `antigravity_reply_id_is_unchanged` pins
    // for the same position.
    let metadata = answering("call-9");
    let generic = nested(140, &nested(2, &nested(1, b"generic result")));
    let messages = read(7, &stored(132, 3, &metadata, &generic), &metadata);
    assert_eq!(messages[0].id, "27653bedd44c20aa805e4de5");
    let viewed = nested(14, &nested(4, b"synthetic file"));
    let other = read(7, &stored(8, 3, &metadata, &viewed), &metadata);
    assert_eq!(other[0].id, messages[0].id);
    assert_ne!(
        read(8, &stored(8, 3, &metadata, &viewed), &metadata)[0].id,
        messages[0].id
    );
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
fn antigravity_stored_view_file_step_answers_its_call() {
    let directory = tempfile::tempdir().unwrap();
    let path = fixture::write(directory.path());
    let connection = Connection::open(&path).unwrap();
    let model = number(3, 2);
    let calling = nested(7, &tool_call("call-2", "view_file", "{}"));
    replace(
        &connection,
        8,
        15,
        &model,
        &stored(15, 3, &model, &nested(20, &calling)),
    );
    // View-file step 9 (type 8): body 14, content 4.
    let viewing = answering("call-2");
    replace(
        &connection,
        9,
        8,
        &viewing,
        &stored(8, 3, &viewing, &nested(14, &nested(4, b"synthetic file"))),
    );
    drop(connection);

    let source = super::discover(&path).unwrap().remove(0).identity;
    let chat = parse(&source).unwrap().chat.unwrap();
    let messages = chat.messages.inline();
    let call = messages
        .iter()
        .position(|m| m.content == "Tool: view_file\n{}")
        .unwrap();
    let viewed = &messages[call + 1];
    assert_eq!(viewed.role, Role::Tool);
    assert_eq!(viewed.content, "synthetic file");
    assert_eq!(viewed.dialogue_text.as_deref(), Some(""));
    assert_eq!(viewed.ordinal, messages[call].ordinal + 1);
    assert_eq!(
        viewed.tools.tool_results[0].call_id,
        messages[call].tools.tool_calls[0].call_id
    );
    assert_eq!(viewed.tools.tool_results[0].is_error, Some(false));
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
    let message = tool_result(132, 7, 140, &nested(2, &nested(1, b"synthetic failure")));
    assert_eq!(message.content, "synthetic failure");
    assert_eq!(failed(&message), Some(true));
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
    assert_eq!(read(1, &stored(132, 3, &model, &[]), &model), []);
    let generic = nested(140, &nested(2, &nested(1, b"not a tool result")));
    assert_eq!(read(2, &stored(17, 3, &model, &generic), &model), []);

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
