//! How a response's parts and a request's attachments become stored messages.
use super::*;
use crate::tools::Activity;

fn uri(path: &str) -> Value {
    json!({"$mid":1,"fsPath":path,"external":format!("file://{path}"),"path":path,"scheme":"file"})
}

fn message_id(parts: &[&str]) -> String {
    let mut identity = vec!["copilot-chat", "chat-session"];
    identity.extend(parts);
    crate::silver::parsers::stable_id("message", &identity)
}

fn agentic() -> Value {
    let mut request = request("request-1", "Validate the importer");
    request["variableData"] = json!({"variables":[
        {"kind":"file","id":"file-1","name":"importer.py","modelDescription":"attached",
            "value":{"uri":uri("/work/src/importer.py"),"range":{"startLineNumber":1,"startColumn":1,"endLineNumber":9,"endColumn":1}}},
        {"kind":"promptFile","id":"prompt-1","name":"prompt:AGENTS.md","value":uri("/work/AGENTS.md"),
            "isRoot":true,"automaticallyAdded":true},
        {"kind":"file","id":"file-1-again","name":"importer.py","value":uri("/work/src/importer.py")},
        {"kind":"paste","id":"paste-1","name":"Pasted text","value":"privatepastedtext"}
    ]});
    request["response"] = json!([
        {"kind":"mcpServersStarting","didStartServerIds":[]},
        {"kind":"thinking","value":"plan the validation","id":"thought-1"},
        {"kind":"thinking","value":"","id":"thought-1","metadata":{"vscodeReasoningDone":true}},
        {"value":"I will check ","supportThemeIcons":false},
        {"kind":"inlineReference","inlineReference":uri("/work/src/importer.py")},
        {"value":" and the ","supportThemeIcons":false},
        {"kind":"inlineReference","inlineReference":{"name":"parse_row","kind":12,
            "location":{"uri":uri("/work/src/rows.py"),"range":{"startLineNumber":3,"startColumn":1,"endLineNumber":3,"endColumn":9}}}},
        {"value":" helper first.","supportThemeIcons":false},
        {"kind":"progressTaskSerialized","content":{"value":"Searching the workspace"},"progress":[]},
        {"kind":"toolInvocationSerialized","invocationMessage":{"value":"Reading importer.py"},
            "pastTenseMessage":{"value":"Read importer.py"},"isConfirmed":{"type":1},"isComplete":true,
            "source":{"type":"internal","label":"Built-In"},"toolCallId":"call-read","toolId":"copilot_readFile"},
        {"kind":"toolInvocationSerialized","invocationMessage":"Running the linter","isComplete":true,
            "toolCallId":"call-lint","toolId":"mcp_lint","toolSpecificData":{"kind":"input","rawInput":{"path":"src"}},
            "resultDetails":{"input":"{}","output":[{"type":"embed","value":"lint failed"}],"isError":true}},
        {"value":"\n```python\n"},
        {"kind":"undoStop","id":"stop-1"},
        {"kind":"codeblockUri","uri":uri("/work/src/importer.py"),"isEdit":true},
        {"kind":"textEditGroup","uri":uri("/work/src/importer.py"),"edits":[[{"text":"privateedittext","range":{}}]],"done":true},
        {"value":"\n```\n"},
        {"value":"The importer now validates rows; see "},
        {"kind":"inlineReference","inlineReference":{"$mid":1,"path":"/docs/import","scheme":"https","authority":"example.org","external":"https://example.org/docs/import"}},
        {"value":"."}
    ]);
    json!({"version":3,"sessionId":"chat-session","creationDate":50,"requests":[request]})
}

#[test]
fn copilot_chat_thinking_tools_and_references_stay_out_of_the_reply() {
    let f = Fixture::new(false);
    f.write("json", &agentic().to_string());
    let chat = f.parse().unwrap();
    let rows = chat.messages.inline();
    let aside = |role: Role, text: &str, tools: Activity| {
        (role, text.to_owned(), Some(String::new()), tools)
    };
    assert_eq!(
        rows.iter()
            .map(|m| (
                m.role.clone(),
                m.content.clone(),
                m.dialogue_text.clone(),
                m.tools.clone()
            ))
            .collect::<Vec<_>>(),
        [
            // Attached files stay with the request; the search text is what was typed.
            (
                Role::User,
                "Validate the importer\nFile importer.py: file:///work/src/importer.py\nFile prompt:AGENTS.md: file:///work/AGENTS.md".to_owned(),
                Some("Validate the importer".to_owned()),
                Activity::default()
            ),
            aside(
                Role::Assistant,
                "[Thinking] plan the validation",
                Activity::default()
            ),
            // A reference continues its sentence, as it does on screen.
            (
                Role::Assistant,
                "I will check importer.py and the `parse_row` helper first.\nFile importer.py: file:///work/src/importer.py\nFile rows.py: file:///work/src/rows.py".to_owned(),
                Some("I will check importer.py and the `parse_row` helper first.".to_owned()),
                Activity::default()
            ),
            aside(
                Role::Assistant,
                "Searching the workspace",
                Activity::default()
            ),
            aside(
                Role::Assistant,
                "Tool: copilot_readFile\nReading importer.py\nRead importer.py",
                Activity::call(Some("copilot_readFile"), Some("call-read"), None)
            ),
            aside(
                Role::Assistant,
                "Tool: mcp_lint\nRunning the linter\n{\"kind\":\"input\",\"rawInput\":{\"path\":\"src\"}}",
                Activity::call(Some("mcp_lint"), Some("call-lint"), None)
            ),
            aside(
                Role::Tool,
                "{\"input\":\"{}\",\"isError\":true,\"output\":[{\"type\":\"embed\",\"value\":\"lint failed\"}]}",
                Activity::result(Some("call-lint"), Some(true))
            ),
            (
                Role::Assistant,
                "\n```python\n\n```\nThe importer now validates rows; see https://example.org/docs/import.".to_owned(),
                Some("\n```python\n\n```\nThe importer now validates rows; see https://example.org/docs/import.".to_owned()),
                Activity::default()
            ),
        ]
    );
    // The prompt and the first passage of the reply are the messages stored
    // before thinking and tools were split out.
    assert_eq!(rows[0].id, message_id(&["request-1", "user"]));
    assert_eq!(rows[2].id, message_id(&["request-1", "assistant"]));
    assert_eq!(
        rows[1].id,
        message_id(&["request-1", "assistant", "thinking:1"])
    );
    assert_eq!(
        rows[4].id,
        message_id(&["request-1", "assistant", "tool:call-read"])
    );
    assert_eq!(
        rows[6].id,
        message_id(&["request-1", "assistant", "result:call-lint"])
    );
    assert_eq!(
        rows[7].id,
        message_id(&["request-1", "assistant", "text:11"])
    );
    assert!(
        rows.iter()
            .enumerate()
            .all(|(index, row)| row.ordinal == i64::try_from(index).unwrap())
    );
    assert!(rows.iter().all(|row| {
        !row.content.contains("privatepastedtext") && !row.content.contains("privateedittext")
    }));
    assert_eq!(
        rows.iter()
            .filter(|row| row.created_at == Some(200))
            .count(),
        7
    );
}

#[test]
fn copilot_chat_only_what_was_typed_and_replied_is_searchable() {
    let f = Fixture::new(false);
    f.write("json", &agentic().to_string());
    let mut storage = Storage::open_writer(&f.dir.path().join("index.sqlite3")).unwrap();
    index(&mut storage, &f.options()).unwrap();
    storage.commit_writer().unwrap();
    assert_eq!(storage.counts().unwrap().searchable_messages, 3);
    for text in ["plan the validation", "Searching", "linter", "AGENTS"] {
        assert!(
            crate::storage::test_support::projections_with_text(&storage, text, None).is_empty(),
            "{text}"
        );
    }
}

#[test]
fn copilot_chat_logged_attachments_and_streamed_parts_keep_identities() {
    let f = Fixture::new(false);
    let mut snapshot = agentic();
    let request = snapshot["requests"][0].take();
    snapshot["requests"] = json!([]);
    let mut bare = request.clone();
    bare.as_object_mut().unwrap().remove("variableData");
    bare["response"] = json!([]);
    let parts = request["response"].as_array().unwrap();
    f.write(
        "jsonl",
        &log(&[
            json!({"kind":0,"v":snapshot}),
            json!({"kind":2,"k":["requests"],"v":[bare]}),
            json!({"kind":1,"k":["requests",0,"variableData"],"v":{"variables":[request["variableData"]["variables"][0]]}}),
            json!({"kind":2,"k":["requests",0,"variableData","variables"],"v":[request["variableData"]["variables"][1]]}),
            json!({"kind":2,"k":["requests",0,"response"],"v":parts[..8]}),
        ]),
    );
    let streaming = f.parse().unwrap();
    let before = streaming.messages.inline();
    assert_eq!(before.len(), 3);
    assert!(before[0].content.ends_with(
        "\nFile importer.py: file:///work/src/importer.py\nFile prompt:AGENTS.md: file:///work/AGENTS.md"
    ));
    std::fs::OpenOptions::new()
        .append(true)
        .open(f.chats.join("chat-session.jsonl"))
        .unwrap()
        .write_all(
            log(&[json!({"kind":2,"k":["requests",0,"response"],"i":8,"v":parts[8..]})]).as_bytes(),
        )
        .unwrap();
    let finished = f.parse().unwrap();
    let after = finished.messages.inline();
    assert_eq!(after.len(), 8);
    assert_eq!(
        after[..3]
            .iter()
            .map(|m| (&m.id, &m.content))
            .collect::<Vec<_>>(),
        before
            .iter()
            .map(|m| (&m.id, &m.content))
            .collect::<Vec<_>>(),
        "messages already read keep their identity while the response grows"
    );
}

#[test]
fn copilot_chat_prompts_written_by_the_editor_are_not_user_messages() {
    let f = Fixture::new(false);
    let mut snapshot = session();
    let mut automatic = request("automatic", "privateautomaticrequest");
    automatic["isSystemInitiated"] = json!(true);
    let mut hidden = request("hidden-prompt", "privatehiddenprompt");
    hidden["requestHiddenFromTranscript"] = json!(true);
    snapshot["requests"]
        .as_array_mut()
        .unwrap()
        .extend([automatic, hidden]);
    f.write("json", &snapshot.to_string());
    let chat = f.parse().unwrap();
    let rows = chat.messages.inline();
    for (request, text) in [
        ("automatic", "privateautomaticrequest"),
        ("hidden-prompt", "privatehiddenprompt"),
    ] {
        let row = rows.iter().find(|row| row.content == text).unwrap();
        assert_eq!(row.role, Role::System);
        assert_eq!(row.dialogue_text.as_deref(), Some(""));
        assert_eq!(row.id, message_id(&[request, "user"]));
    }
    assert_eq!(rows.iter().filter(|row| row.role == Role::User).count(), 1);
}
