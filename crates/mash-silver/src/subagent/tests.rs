use super::*;

fn codex(source: &str) -> Option<Subagent> {
    let source: CodexSource = serde_json::from_str(source).unwrap();
    source.subagent("child", &CodexNames::default())
}

#[test]
fn codex_thread_spawn_names_its_parent_label_role_and_depth() {
    let found = codex(
        r#"{"subagent":{"thread_spawn":{"parent_thread_id":"parent","depth":1,
            "agent_path":"/root/readme","agent_nickname":"Locke","agent_role":"worker"}}}"#,
    )
    .unwrap();
    assert_eq!(
        found,
        Subagent {
            parent_chat_id: Some("parent".into()),
            label: Some("Locke".into()),
            role: Some("worker".into()),
            depth: Some(1),
            finished: None,
            spawn_call_id: None,
        }
    );
}

#[test]
fn codex_subagents_without_a_parent_keep_their_kind() {
    let review = codex(r#"{"subagent":"review"}"#).unwrap();
    assert_eq!(
        (review.parent_chat_id, review.role.as_deref()),
        (None, Some("review"))
    );
    let guardian = codex(r#"{"subagent":{"other":"guardian"}}"#).unwrap();
    assert_eq!(guardian.role.as_deref(), Some("guardian"));
    assert!(guardian.parent_chat_id.is_none());
    // A future shape is still a subagent, with nothing more known.
    let future = codex(r#"{"subagent":{"swarm":{"size":3}}}"#).unwrap();
    assert_eq!(future, Subagent::default());
}

#[test]
fn codex_sessions_people_started_are_not_subagents() {
    for source in [
        r#""cli""#,
        r#""vscode""#,
        r#""exec""#,
        r#"{"mcp":{}}"#,
        "7",
        "null",
    ] {
        let parsed: Option<CodexSource> = serde_json::from_str(source).unwrap();
        assert!(
            parsed
                .and_then(|s| s.subagent("child", &CodexNames::default()))
                .is_none(),
            "{source}"
        );
    }
    assert!(
        CodexSource::from_column("cli")
            .unwrap()
            .subagent("child", &CodexNames::default())
            .is_none()
    );
}

#[test]
fn codex_spawn_fields_of_the_wrong_type_or_self_parent_read_as_absent() {
    let found = codex(
        r#"{"subagent":{"thread_spawn":{"parent_thread_id":"child","depth":"deep",
            "agent_nickname":{"x":1},"agent_role":null}}}"#,
    )
    .unwrap();
    assert_eq!(found, Subagent::default());
    let named =
        CodexSource::from_column(r#"{"subagent":{"thread_spawn":{"parent_thread_id":"p"}}}"#)
            .unwrap()
            .subagent(
                "child",
                &CodexNames {
                    nickname: Some("  Popper\nsecond line"),
                    role: Some("explorer"),
                },
            )
            .unwrap();
    assert_eq!(named.label.as_deref(), Some("Popper"));
    assert_eq!(named.role.as_deref(), Some("explorer"));
    assert_eq!(named.parent_chat_id.as_deref(), Some("p"));
}

#[test]
fn claude_meta_gives_the_task_type_state_and_spawning_call() {
    let directory = tempfile::tempdir().unwrap();
    let transcript = directory.path().join("agent-a1.jsonl");
    std::fs::write(
        directory.path().join("agent-a1.meta.json"),
        r#"{"agentType":"general-purpose","description":"Fix quick panel dead ends",
            "toolUseId":"toolu_1","spawnDepth":1,"requestShape":"background","finished":true}"#,
    )
    .unwrap();
    assert_eq!(
        claude(&transcript, "session", "a1"),
        Subagent {
            parent_chat_id: Some("session".into()),
            label: Some("Fix quick panel dead ends".into()),
            role: Some("general-purpose".into()),
            depth: Some(1),
            finished: Some(true),
            spawn_call_id: Some("toolu_1".into()),
        }
    );
}

#[test]
fn claude_nested_subagents_point_at_the_subagent_that_started_them() {
    let directory = tempfile::tempdir().unwrap();
    let transcript = directory.path().join("agent-a2.jsonl");
    std::fs::write(
        directory.path().join("agent-a2.meta.json"),
        r#"{"agentType":"Explore","description":"Look","spawnDepth":2,"parentAgentId":"a1"}"#,
    )
    .unwrap();
    let found = claude(&transcript, "session", "a2");
    assert_eq!(
        found.parent_chat_id,
        Some(crate::parsers::stable_id(
            "claude-agent",
            &["session", "a1"]
        ))
    );
    assert_eq!((found.depth, found.finished), (Some(2), None));
}

#[test]
fn claude_subagents_without_readable_meta_still_name_their_session() {
    let directory = tempfile::tempdir().unwrap();
    // Missing, malformed, and a meta file for another agent's name.
    let missing = claude(&directory.path().join("agent-a3.jsonl"), "session", "a3");
    assert_eq!(missing.parent_chat_id.as_deref(), Some("session"));
    assert!(missing.label.is_none());
    std::fs::write(directory.path().join("agent-a4.meta.json"), "{not json").unwrap();
    assert!(
        claude(&directory.path().join("agent-a4.jsonl"), "session", "a4")
            .label
            .is_none()
    );
    std::fs::write(
        directory.path().join("agent-a5.meta.json"),
        r#"{"description":"Wrong file"}"#,
    )
    .unwrap();
    let renamed = claude(&directory.path().join("copy.jsonl"), "session", "a5");
    assert!(renamed.label.is_none());
    assert_eq!(
        claude_meta_candidate(&directory.path().join("session.jsonl")),
        None
    );
}
