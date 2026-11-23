//! Subagent markers as the JSONL readers see them, accepted and rejected.
use super::{parse_claude, parse_codex, stable_id};
use std::fs;

fn claude_line(agent: Option<&str>, sidechain: Option<bool>, uuid: &str, text: &str) -> String {
    let mut record = serde_json::json!({
        "type": "user", "sessionId": "session", "uuid": uuid,
        "message": {"role": "user", "content": text},
    });
    if let Some(agent) = agent {
        record["agentId"] = agent.into();
    }
    if let Some(sidechain) = sidechain {
        record["isSidechain"] = sidechain.into();
    }
    format!("{record}\n")
}

#[test]
fn claude_sidechain_transcripts_are_subagents_of_their_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("agent-a1.jsonl");
    // Claude Code's request bookkeeping names the agent without the mark.
    let bookkeeping = r#"{"type":"api-request","agentId":"a1","sessionId":"session"}"#;
    let text = claude_line(Some("a1"), Some(true), "m1", "Work ONLY in /tmp") + bookkeeping;
    fs::write(&path, text + "\n").unwrap();
    fs::write(
        directory.path().join("agent-a1.meta.json"),
        r#"{"agentType":"general-purpose","description":"Fix the tests","toolUseId":"toolu_1","spawnDepth":1}"#,
    )
    .unwrap();
    let chat = parse_claude(&path).unwrap().chat.unwrap();
    assert_eq!(chat.id, stable_id("claude-agent", &["session", "a1"]));
    let subagent = chat.origin.subagent.unwrap();
    assert_eq!(subagent.parent_chat_id.as_deref(), Some("session"));
    assert_eq!(subagent.label.as_deref(), Some("Fix the tests"));
    assert_eq!(subagent.spawn_call_id.as_deref(), Some("toolu_1"));
}

#[test]
fn claude_sessions_and_unmarked_agent_records_are_not_subagents() {
    let directory = tempfile::tempdir().unwrap();
    // An ordinary session whose first prompt reads like a subagent brief.
    let session = directory.path().join("session.jsonl");
    fs::write(
        &session,
        claude_line(None, None, "m1", "Work ONLY in /home/me"),
    )
    .unwrap();
    assert!(
        parse_claude(&session)
            .unwrap()
            .chat
            .unwrap()
            .origin
            .subagent
            .is_none()
    );
    // An agent ID without the sidechain mark, even beside a meta file.
    let unmarked = directory.path().join("agent-a2.jsonl");
    fs::write(
        &unmarked,
        claude_line(Some("a2"), Some(false), "m2", "hello"),
    )
    .unwrap();
    fs::write(
        directory.path().join("agent-a2.meta.json"),
        r#"{"description":"x"}"#,
    )
    .unwrap();
    assert!(
        parse_claude(&unmarked)
            .unwrap()
            .chat
            .unwrap()
            .origin
            .subagent
            .is_none()
    );
    // A sidechain mark on some records only.
    let mixed = directory.path().join("agent-a3.jsonl");
    fs::write(
        &mixed,
        claude_line(Some("a3"), Some(true), "m3", "one")
            + &claude_line(Some("a3"), None, "m4", "two"),
    )
    .unwrap();
    assert!(
        parse_claude(&mixed)
            .unwrap()
            .chat
            .unwrap()
            .origin
            .subagent
            .is_none()
    );
}

fn codex_meta(id: &str, session: &str, extra: &serde_json::Value) -> String {
    let mut payload = serde_json::json!({"id": id, "session_id": session, "cwd": "/repo"});
    for (key, value) in extra.as_object().unwrap() {
        payload[key] = value.clone();
    }
    format!(
        "{}\n",
        serde_json::json!({"type": "session_meta", "payload": payload})
    )
}

fn codex_message(text: &str) -> String {
    format!(
        "{}\n",
        serde_json::json!({"type": "response_item", "payload": {"type": "message", "role": "user",
            "content": [{"type": "input_text", "text": text}]}})
    )
}

#[test]
fn codex_spawned_rollouts_are_subagents_and_ignore_the_copied_parent_meta() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rollout-child.jsonl");
    let spawn = serde_json::json!({
        "forked_from_id": "parent", "parent_thread_id": "parent",
        "source": {"subagent": {"thread_spawn": {"parent_thread_id": "parent", "depth": 1,
            "agent_nickname": "Locke", "agent_role": "worker"}}},
        "agent_nickname": "Locke", "agent_role": "worker",
    });
    fs::write(
        &path,
        codex_meta("child", "parent", &spawn)
            + &codex_meta("parent", "parent", &serde_json::json!({"source": "vscode"}))
            + &codex_message("Update the README"),
    )
    .unwrap();
    let chat = parse_codex(&path).unwrap().chat.unwrap();
    assert_eq!(chat.id, "child");
    let subagent = chat.origin.subagent.unwrap();
    assert_eq!(subagent.parent_chat_id.as_deref(), Some("parent"));
    assert_eq!(subagent.label.as_deref(), Some("Locke"));
    assert_eq!(subagent.role.as_deref(), Some("worker"));
}

#[test]
fn codex_forks_and_parent_links_without_a_subagent_source_are_not_subagents() {
    let directory = tempfile::tempdir().unwrap();
    for (name, extra) in [
        (
            "fork",
            serde_json::json!({"forked_from_id": "original", "source": "vscode", "thread_source": "user"}),
        ),
        (
            "linked",
            serde_json::json!({"parent_thread_id": "other", "source": "cli"}),
        ),
        (
            "odd",
            serde_json::json!({"source": {"subagents": "review"}, "agent_nickname": 5}),
        ),
    ] {
        let path = directory.path().join(format!("rollout-{name}.jsonl"));
        fs::write(
            &path,
            codex_meta(name, name, &extra) + &codex_message("hello"),
        )
        .unwrap();
        let chat = parse_codex(&path).unwrap().chat.unwrap();
        assert!(chat.origin.subagent.is_none(), "{name}");
    }
}

#[test]
fn claude_copies_of_ssh_sessions_are_marked_with_their_subagents_beside_them() {
    let directory = tempfile::tempdir().unwrap();
    // The desktop app's flat layout: the session and its subagents side by side.
    let folder = directory.path().join("ssh-session");
    fs::create_dir(&folder).unwrap();
    let session = folder.join("session.jsonl");
    fs::write(&session, claude_line(None, None, "m1", "deploy it")).unwrap();
    let agent = folder.join("agent-a1.jsonl");
    fs::write(
        &agent,
        claude_line(Some("a1"), Some(true), "m2", "check the logs"),
    )
    .unwrap();
    fs::write(
        folder.join("agent-a1.meta.json"),
        r#"{"description":"Check the logs"}"#,
    )
    .unwrap();
    let parent = parse_claude(&session).unwrap().chat.unwrap();
    assert!(parent.origin.ssh_copy);
    assert!(parent.origin.subagent.is_none());
    let child = parse_claude(&agent).unwrap().chat.unwrap();
    assert!(child.origin.ssh_copy);
    let subagent = child.origin.subagent.unwrap();
    assert_eq!(subagent.parent_chat_id.as_deref(), Some(parent.id.as_str()));
    assert_eq!(subagent.label.as_deref(), Some("Check the logs"));

    // A folder named for another session, or an ordinary project, is not a copy.
    let other = directory.path().join("ssh-other");
    fs::create_dir(&other).unwrap();
    let elsewhere = other.join("session.jsonl");
    fs::write(&elsewhere, claude_line(None, None, "m1", "deploy it")).unwrap();
    assert!(
        !parse_claude(&elsewhere)
            .unwrap()
            .chat
            .unwrap()
            .origin
            .ssh_copy
    );
    let project = directory.path().join("-home-me-ssh-session");
    fs::create_dir(&project).unwrap();
    let local = project.join("session.jsonl");
    fs::write(&local, claude_line(None, None, "m1", "deploy it")).unwrap();
    assert!(!parse_claude(&local).unwrap().chat.unwrap().origin.ssh_copy);
}

#[test]
fn discovered_subagent_transcripts_are_captured_whole_beside_their_metadata() {
    use crate::provider::Provider;
    let directory = tempfile::tempdir().unwrap();
    let folder = directory.path().join("project/session/subagents");
    fs::create_dir_all(&folder).unwrap();
    let transcript = folder.join("agent-a1.jsonl");
    fs::write(
        &transcript,
        claude_line(Some("a1"), Some(true), "m1", "one")
            + &claude_line(Some("a1"), Some(true), "m2", "two"),
    )
    .unwrap();
    // Metadata larger than the transcript must not stretch the capture.
    let meta = format!(
        r#"{{"description":"Check","padding":"{}"}}"#,
        "x".repeat(4096)
    );
    fs::write(folder.join("agent-a1.meta.json"), meta).unwrap();
    let sources =
        crate::discovered_sources(Provider::ClaudeCode, &[directory.path().to_path_buf()]);
    let source = sources
        .iter()
        .find(|s| s.identity.path == transcript)
        .unwrap();
    assert_eq!(source.dependencies.len(), 1);
    let chat = crate::parse_test_source(Provider::ClaudeCode, source)
        .chat
        .unwrap();
    assert_eq!(chat.messages.len(), 2);
    assert_eq!(
        chat.origin.subagent.unwrap().label.as_deref(),
        Some("Check")
    );
}
