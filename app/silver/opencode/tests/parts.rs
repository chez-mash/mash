//! How synthetic messages, reasoning and tool parts become stored messages.
use super::*;
use crate::{role::Role, tools::Activity};
use serde_json::{Value, json};

type Shape = (Role, String, Option<String>, Activity);

/// Role, content, searchable text and tool activity of each stored message.
fn shape(chat: &mash_silver::Chat) -> Vec<Shape> {
    chat.messages
        .inline()
        .iter()
        .map(|m| {
            (
                m.role.clone(),
                m.content.clone(),
                m.dialogue_text.clone(),
                m.tools.clone(),
            )
        })
        .collect()
}

fn aside(role: Role, text: &str) -> Shape {
    (
        role,
        text.to_owned(),
        Some(String::new()),
        Activity::default(),
    )
}

fn reply(text: &str) -> Shape {
    (
        Role::Assistant,
        text.to_owned(),
        Some(text.to_owned()),
        Activity::default(),
    )
}

fn call(name: &str, id: &str, input: &Value) -> Shape {
    (
        Role::Assistant,
        format!("Tool: {name}\n{input}"),
        Some(String::new()),
        Activity::call(Some(name), Some(id), Some(input)),
    )
}

fn result(id: &str, text: &str, failed: bool) -> Shape {
    (
        Role::Tool,
        text.to_owned(),
        Some(String::new()),
        Activity::result(Some(id), Some(failed)),
    )
}

fn message_id(source: &mash_silver::SourceIdentity, parts: &[&str]) -> String {
    let key = source.key();
    let mut identity = vec![Provider::Opencode.as_str(), key.as_str()];
    identity.extend(parts);
    crate::silver::parsers::stable_id("message", &identity)
}

fn assert_ordered_and_distinct(chat: &mash_silver::Chat) {
    let rows = chat.messages.inline();
    assert!(
        rows.iter()
            .enumerate()
            .all(|(index, row)| row.ordinal == i64::try_from(index).unwrap())
    );
    let ids: std::collections::BTreeSet<_> = rows.iter().map(|row| &row.id).collect();
    assert_eq!(ids.len(), rows.len());
}

#[test]
fn opencode_synthetic_messages_are_context_and_never_prompts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("opencode.db");
    let upstream = fixture(&path);
    upstream.execute_batch("INSERT INTO session_message VALUES('synthetic','alpha','synthetic',3,1790726402000,'{\"text\":\"injectedcontinuation\",\"description\":\"Continue\"}');
        INSERT INTO part VALUES('part2','legacy','beta','{\"type\":\"text\",\"text\":\"injectedwrapper\",\"synthetic\":true}');
        INSERT INTO part VALUES('part3','legacy','beta','{\"type\":\"text\",\"text\":\"ignoredwrapper\",\"ignored\":true}');
        INSERT INTO message VALUES('wrapper','beta',1790726400001,'{\"role\":\"user\"}');
        INSERT INTO part VALUES('part4','wrapper','beta','{\"type\":\"text\",\"text\":\"onlyinjected\",\"synthetic\":true}');").unwrap();
    let sources = discover(&path).unwrap();

    let current = parse(&sources[0].identity).unwrap().chat.unwrap();
    let rows = current.messages.inline();
    let injected = rows.last().unwrap();
    assert_eq!(injected.role, Role::System);
    assert_eq!(injected.content, "injectedcontinuation");
    assert_eq!(injected.dialogue_text.as_deref(), Some(""));
    assert_eq!(
        injected.id,
        message_id(&sources[0].identity, &["synthetic"])
    );
    assert!(
        rows.iter()
            .filter(|m| m.role == Role::User)
            .all(|m| m.content == "Fix the cBioPortal importer")
    );

    // A legacy prompt keeps what was typed; what OpenCode added sits beside it.
    let legacy = parse(&sources[1].identity).unwrap().chat.unwrap();
    assert_eq!(
        shape(&legacy),
        [
            (
                Role::User,
                "independent native history".to_owned(),
                Some("independent native history".to_owned()),
                Activity::default()
            ),
            aside(Role::System, "injectedwrapper\nignoredwrapper"),
            aside(Role::System, "onlyinjected"),
        ]
    );
    assert_eq!(
        legacy.messages.inline()[0].id,
        message_id(&sources[1].identity, &["legacy"])
    );
    assert_ordered_and_distinct(&legacy);

    let mut writer = Storage::open_writer(&directory.path().join("index.sqlite3")).unwrap();
    index(&mut writer, &options(&path)).unwrap();
    writer.commit_writer().unwrap();
    for text in ["injectedcontinuation", "injectedwrapper", "onlyinjected"] {
        assert!(
            crate::storage::test_support::projections_with_text(&writer, text, None).is_empty()
        );
    }
}

/// An assistant message in the current layout, with every tool state.
fn current_step() -> Value {
    json!({"content":[
        {"type":"reasoning","text":"weighing the parser","time":{"created":1}},
        {"type":"text","text":"I will read the importer."},
        {"type":"tool","id":"call_read","name":"read","time":{"created":2},
            "state":{"status":"completed","input":{"filePath":"/work/importer.py"},"metadata":{"truncated":false},
                "content":[{"type":"text","text":"def parse(): ..."},
                    {"type":"file","uri":"file:///work/plot.png","mime":"image/png","name":"plot.png"}]}},
        {"type":"tool","id":"call_bash","name":"bash","time":{"created":3},
            "state":{"status":"error","input":{"command":"false"},
                "error":{"type":"tool","message":"exit status 1"}}},
        {"type":"tool","id":"call_slow","name":"grep","time":{"created":4},
            "state":{"status":"running","input":{"pattern":"mutation"},"metadata":{}}},
        {"type":"tool","id":"call_partial","name":"write","time":{"created":5},
            "state":{"status":"streaming","input":"{\"filePath\":"}},
        {"type":"reasoning","text":"  "},
        {"type":"futurePart","text":"futurepartmarker"},
        {"type":"text","text":"The parser now validates input."}
    ]})
}

#[test]
fn opencode_reasoning_and_tools_are_their_own_messages_beside_the_reply() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("opencode.db");
    let upstream = fixture(&path);
    upstream
        .execute(
            "UPDATE session_message SET data=?1 WHERE id='assistant'",
            [current_step().to_string()],
        )
        .unwrap();
    let source = discover(&path).unwrap().remove(0).identity;
    let chat = parse(&source).unwrap().chat.unwrap();
    assert_eq!(
        shape(&chat)[1..],
        [
            aside(Role::Assistant, "[Thinking] weighing the parser"),
            reply("I will read the importer."),
            call(
                "read",
                "call_read",
                &json!({"filePath":"/work/importer.py"})
            ),
            result(
                "call_read",
                "def parse(): ...\nFile plot.png: file:///work/plot.png",
                false
            ),
            call("bash", "call_bash", &json!({"command":"false"})),
            result("call_bash", "exit status 1", true),
            // A call still running, and one whose arguments are still arriving.
            call("grep", "call_slow", &json!({"pattern":"mutation"})),
            call("write", "call_partial", &json!("{\"filePath\":")),
            reply("The parser now validates input."),
        ]
    );
    let rows = chat.messages.inline();
    // The first passage of the reply is the message stored before the split.
    for (row, suffix) in [
        (1, Some("reasoning:0")),
        (2, None),
        (3, Some("tool:2")),
        (4, Some("result:2")),
        (9, Some("text:8")),
    ] {
        let mut parts = vec!["assistant"];
        parts.extend(suffix);
        assert_eq!(rows[row].id, message_id(&source, &parts), "{suffix:?}");
    }
    assert_ordered_and_distinct(&chat);
    assert!(rows.iter().all(|m| !m.content.contains("futurepartmarker")));

    // Only the replies are searchable, and reading again changes nothing.
    let local = directory.path().join("index.sqlite3");
    let mut writer = Storage::open_writer(&local).unwrap();
    index(&mut writer, &options(&path)).unwrap();
    writer.commit_writer().unwrap();
    for text in ["weighing", "def parse", "exit status"] {
        assert!(
            crate::storage::test_support::projections_with_text(&writer, text, None).is_empty()
        );
    }
    assert!(
        !crate::storage::test_support::projections_with_text(&writer, "validates input", None)
            .is_empty()
    );
    drop(writer);
    let mut writer = Storage::open_writer(&local).unwrap();
    let again = index(&mut writer, &options(&path)).unwrap();
    assert_eq!(again.checkpoint_skipped_sources, 2);
}

#[test]
fn opencode_legacy_parts_split_the_same_way_keyed_by_their_rows() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("opencode.db");
    let upstream = fixture(&path);
    upstream.execute_batch("INSERT INTO message VALUES('step','beta',1790726400001,'{\"role\":\"assistant\"}');
        INSERT INTO part VALUES('prt_1','step','beta','{\"type\":\"step-start\"}');
        INSERT INTO part VALUES('prt_2','step','beta','{\"type\":\"reasoning\",\"text\":\"legacy thought\",\"time\":{\"start\":1}}');
        INSERT INTO part VALUES('prt_3','step','beta','{\"type\":\"text\",\"text\":\"Legacy answer\"}');
        INSERT INTO part VALUES('prt_4','step','beta','{\"type\":\"tool\",\"callID\":\"toolu_1\",\"tool\":\"read\",\"state\":{\"status\":\"completed\",\"input\":{\"filePath\":\"/work/importer.py\"},\"output\":\"legacy output\",\"title\":\"importer.py\",\"metadata\":{},\"time\":{\"start\":1,\"end\":2}}}');
        INSERT INTO part VALUES('prt_5','step','beta','{\"type\":\"tool\",\"callID\":\"toolu_2\",\"tool\":\"bash\",\"state\":{\"status\":\"error\",\"input\":{\"command\":\"false\"},\"error\":\"legacy failure\",\"time\":{\"start\":1,\"end\":2}}}');
        INSERT INTO part VALUES('prt_6','step','beta','{\"type\":\"tool\",\"callID\":\"toolu_3\",\"tool\":\"grep\",\"state\":{\"status\":\"pending\",\"input\":{},\"raw\":\"\"}}');
        INSERT INTO part VALUES('prt_7','step','beta','{\"type\":\"step-finish\",\"reason\":\"tool-calls\"}');").unwrap();
    let source = discover(&path).unwrap().remove(1).identity;
    let chat = parse(&source).unwrap().chat.unwrap();
    assert_eq!(
        shape(&chat)[1..],
        [
            aside(Role::Assistant, "[Thinking] legacy thought"),
            reply("Legacy answer"),
            call("read", "toolu_1", &json!({"filePath":"/work/importer.py"})),
            result("toolu_1", "legacy output", false),
            call("bash", "toolu_2", &json!({"command":"false"})),
            result("toolu_2", "legacy failure", true),
            call("grep", "toolu_3", &json!({})),
        ]
    );
    let rows = chat.messages.inline();
    for (row, suffix) in [
        (1, Some("reasoning:prt_2")),
        (2, None),
        (3, Some("tool:prt_4")),
        (4, Some("result:prt_4")),
    ] {
        let mut parts = vec!["step"];
        parts.extend(suffix);
        assert_eq!(rows[row].id, message_id(&source, &parts), "{suffix:?}");
    }
    assert_ordered_and_distinct(&chat);
}
