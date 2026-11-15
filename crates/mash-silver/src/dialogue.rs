//! Silver interpretation of visible dialogue; canonical records retain all context.
use crate::role::Role;
use std::borrow::Cow;

const CONTEXT_TAGS: &[&str] = &[
    "system-reminder",
    "environment_context",
    "permissions",
    "collaboration_mode",
    "skills_instructions",
    "recommended_plugins",
    "multi_agent_role",
    "multi_agent_mode",
    "in-app-browser-context",
    "codex_internal_context",
    "realtime_delegation",
    "task-notification",
    "local-command-caveat",
    "local-command-stdout",
    "local-command-stderr",
];

/// Remove known agent-injected context without matching ordinary mentions of it.
pub fn visible<'a>(role: &Role, text: &'a str) -> Cow<'a, str> {
    if !matches!(role, Role::User | Role::Assistant) {
        return Cow::Borrowed("");
    }
    // Skill bodies have no reliable closing delimiter. These are dedicated
    // injected messages, not arbitrary references to skills in ordinary prose.
    let trimmed = text.trim_start();
    if trimmed.starts_with("Base directory for this skill:")
        || trimmed.starts_with(
            "The following is the Codex agent history added since your last approval assessment.",
        )
        || trimmed.starts_with(
            "This session is being continued from a previous conversation that ran out of context.",
        )
        || trimmed.starts_with("This conversation is a continuation of an earlier chat.")
    {
        return Cow::Borrowed("");
    }
    if *role == Role::User
        && let Some(typed) = command(trimmed)
            .or_else(|| answers(trimmed))
            .or_else(|| request_after_mentions(trimmed))
    {
        return Cow::Owned(without_context_blocks(&typed, CONTEXT_TAGS).into_owned());
    }
    let mut result = without_context_blocks(text, CONTEXT_TAGS);
    if trimmed.starts_with("# AGENTS.md instructions") && text.contains("<INSTRUCTIONS>") {
        result = remove_blocks(result, "INSTRUCTIONS");
        let content = result.as_ref();
        let end = content.find('\n').unwrap_or(content.len());
        result = Cow::Owned(content[end..].trim_start().to_owned());
    }
    result
}

/// The text between `<name>` and `</name>`, when the block is complete.
fn block<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let start = text.find(&format!("<{name}>"))? + name.len() + 2;
    let end = text[start..].find(&format!("</{name}>"))? + start;
    Some(&text[start..end])
}

/// A slash command as the person typed it. Claude Code records `/goal fix it`
/// as `<command-name>`, `<command-message>` and `<command-args>` blocks.
fn command(text: &str) -> Option<String> {
    if !(text.starts_with("<command-name>") || text.starts_with("<command-message>")) {
        return None;
    }
    let name = block(text, "command-name")?.trim();
    let arguments = block(text, "command-args").unwrap_or("").trim();
    Some(format!("{name} {arguments}").trim().to_owned())
}

#[derive(serde::Deserialize)]
struct QuestionReply {
    answer: Option<String>,
}

/// The answers a person gave to questions Codex asked. The record repeats each
/// question and its internal identity; only the answers are theirs.
fn answers(text: &str) -> Option<String> {
    if !text.starts_with("<send_user_message_question_reply>") {
        return None;
    }
    let replies = block(text, "send_user_message_question_reply")
        .and_then(|body| serde_json::from_str::<Vec<QuestionReply>>(body.trim()).ok())
        .unwrap_or_default();
    Some(
        replies
            .into_iter()
            .filter_map(|reply| reply.answer)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// The request under Codex's list of mentioned files. The list is paths Codex
/// wrote; an unreadable preamble leaves nothing the person typed.
fn request_after_mentions(text: &str) -> Option<String> {
    if !text.starts_with("# Files mentioned by the user:") {
        return None;
    }
    let request = text.find("\n## My request").map_or("", |heading| {
        let after = &text[heading + 1..];
        after.find('\n').map_or("", |end| &after[end + 1..])
    });
    Some(request.trim().to_owned())
}

pub fn without_context_blocks<'a>(text: &'a str, tags: &[&str]) -> Cow<'a, str> {
    tags.iter()
        .fold(Cow::Borrowed(text), |text, tag| remove_blocks(text, tag))
}

fn remove_blocks<'a>(text: Cow<'a, str>, tag: &str) -> Cow<'a, str> {
    let opening = format!("<{tag}");
    let closing = format!("</{tag}>");
    let mut cursor = 0;
    let mut copied = 0;
    let mut output = None::<String>;
    while let Some(relative) = text[cursor..].find(&opening) {
        let start = cursor + relative;
        let boundary = start + opening.len();
        // Markdown examples are user-authored evidence, not injected context.
        if text[..start].bytes().filter(|b| *b == b'`').count() % 2 != 0 {
            cursor = boundary;
            continue;
        }
        // Attribute-bearing context blocks are allowed; longer tag names aren't.
        if !text[boundary..].starts_with(['>', ' ', '\n', '\t']) {
            cursor = boundary;
            continue;
        }
        let ending = text[boundary..]
            .find(&closing)
            .map(|end| boundary + end + closing.len());
        let ending = ending.or_else(|| {
            let opening_end = text[boundary..].find('>')? + boundary;
            let labeled_close = format!("</{}>", &text[start + 1..opening_end]);
            text[opening_end..]
                .find(&labeled_close)
                .map(|end| opening_end + end + labeled_close.len())
        });
        let Some(end) = ending else {
            let line = text[..start].rsplit('\n').next().unwrap_or("");
            if !line.trim().is_empty() {
                cursor = boundary;
                continue;
            }
            // Incomplete serialized context is still context, rather than query evidence.
            output
                .get_or_insert_with(String::new)
                .push_str(&text[copied..start]);
            cursor = text.len();
            copied = cursor;
            break;
        };
        output
            .get_or_insert_with(String::new)
            .push_str(&text[copied..start]);
        cursor = end;
        copied = end;
    }
    if let Some(mut output) = output {
        output.push_str(&text[copied..]);
        Cow::Owned(output.trim().to_owned())
    } else {
        text
    }
}

pub fn title(text: &str) -> Option<String> {
    // Older indexes may have truncated an injected header before its delimiter.
    if text.trim_start().starts_with("# AGENTS.md instructions") {
        return None;
    }
    let clean = visible(&Role::User, text);
    let clean = clean.trim();
    (!clean.is_empty()).then(|| clean.chars().take(120).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slash_command_reads_as_it_was_typed() {
        let recorded = "<command-name>/goal</command-name>\n            <command-message>goal</command-message>\n            <command-args>fix the watcher</command-args>";
        assert_eq!(visible(&Role::User, recorded), "/goal fix the watcher");
        let bare = "<command-message>clear</command-message>\n<command-name>/clear</command-name>";
        assert_eq!(visible(&Role::User, bare), "/clear");
        // Only a record that is a command; a mention of the tag stays prose.
        let prose = "what does <command-name> mean in the log?";
        assert_eq!(visible(&Role::User, prose), prose);
    }

    #[test]
    fn command_output_and_notifications_are_context() {
        for text in [
            "<local-command-stdout>Goal set: fix it</local-command-stdout>",
            "<local-command-caveat>Caveat: the messages below were generated by the user while running local commands.</local-command-caveat>",
            "<task-notification>\n<task-id>a1</task-id>\n<status>completed</status>\n</task-notification>",
            "<realtime_delegation>\n  <source>transcript_tail_flush</source>\n  <input>handoff</input>\n</realtime_delegation>",
        ] {
            assert_eq!(visible(&Role::User, text).trim(), "", "{text}");
        }
    }

    #[test]
    fn only_the_answers_of_a_question_reply_are_the_persons() {
        let reply = "<send_user_message_question_reply>\n[{\"question\":\"Switch to Bun?\",\"questionItemId\":\"[\\\"request_user_input_async\\\",\\\"call_1\\\",0]\",\"answer\":\"Yes, and keep pnpm for now\"},{\"question\":\"Which port?\",\"answer\":\"8790\"}]\n</send_user_message_question_reply>";
        assert_eq!(
            visible(&Role::User, reply),
            "Yes, and keep pnpm for now\n8790"
        );
        let damaged = "<send_user_message_question_reply>\n[{\"question\":";
        assert_eq!(visible(&Role::User, damaged), "");
    }

    #[test]
    fn a_file_mention_preamble_leaves_the_request() {
        let mentioned = "# Files mentioned by the user:\n\n## Photo 1.jpg: /tmp/attachments/1-Photo-1.jpg\n\n## My request:\nwhy is one dot elevated?\nsecond line";
        assert_eq!(
            visible(&Role::User, mentioned),
            "why is one dot elevated?\nsecond line"
        );
        let older = "# Files mentioned by the user:\n\n## a.rs: /src/a.rs\n\n## My request for Codex:\nrename it";
        assert_eq!(visible(&Role::User, older), "rename it");
        let alone = "# Files mentioned by the user:\n\n## a.rs: /src/a.rs";
        assert_eq!(visible(&Role::User, alone), "");
        // The heading as ordinary prose is not a preamble.
        let prose = "Add a section called # Files mentioned by the user: to the doc";
        assert_eq!(visible(&Role::User, prose), prose);
    }

    #[test]
    fn dedicated_review_transcripts_and_compaction_summaries_are_not_user_evidence() {
        for text in [
            "The following is the Codex agent history added since your last approval assessment.\nsearch authentication",
            "This session is being continued from a previous conversation that ran out of context.\nredis",
            "This conversation is a continuation of an earlier chat.\ncbioportal",
            "<in-app-browser-context>automatic state</in-app-browser-context>",
            "<codex_internal_context source=\"goal\">automatic continuation</codex_internal_context>",
        ] {
            assert_eq!(visible(&Role::User, text).trim(), "");
        }
        let ordinary = "Can we remove automatic approval-review transcripts from search?";
        assert_eq!(visible(&Role::User, ordinary), ordinary);
        let mixed = "<codex_internal_context source=\"goal\">automatic continuation</codex_internal_context>fix the importer";
        assert_eq!(visible(&Role::User, mixed), "fix the importer");
        let example =
            "Explain `<codex_internal_context source=\"goal\">example</codex_internal_context>`";
        assert_eq!(visible(&Role::User, example), example);
    }

    #[test]
    fn removes_context_preserves_requests_and_ordinary_mentions() {
        let input = "# AGENTS.md instructions for /work\n<INSTRUCTIONS>cbioportal rules</INSTRUCTIONS><environment_context>cwd</environment_context>\nfix the importer";
        assert_eq!(visible(&Role::User, input), "fix the importer");
        assert_eq!(
            visible(
                &Role::User,
                "<system-reminder>cbioportal skill</system-reminder>review PR 180"
            ),
            "review PR 180"
        );
        assert_eq!(
            visible(
                &Role::User,
                "Base directory for this skill: /skills/cbioportal\nsecret instructions"
            ),
            ""
        );
        assert_eq!(
            visible(
                &Role::User,
                "How do I edit AGENTS.md or write a system-reminder?"
            ),
            "How do I edit AGENTS.md or write a system-reminder?"
        );
    }

    #[test]
    fn handles_repeated_and_incomplete_blocks_without_overmatching_tags() {
        assert_eq!(
            visible(
                &Role::User,
                "A<system-reminder>x</system-reminder>B<system-reminder>y</system-reminder>C"
            ),
            "ABC"
        );
        assert_eq!(
            visible(&Role::User, "ask\n<system-reminder>unfinished"),
            "ask"
        );
        assert_eq!(
            visible(
                &Role::User,
                "<permissions-extra>example</permissions-extra>"
            ),
            "<permissions-extra>example</permissions-extra>"
        );
        assert_eq!(
            visible(
                &Role::User,
                "<permissions-extra>x</permissions-extra><permissions instructions>noise</permissions instructions>"
            ),
            "<permissions-extra>x</permissions-extra>"
        );
    }

    #[test]
    fn context_roles_are_unsearchable_and_titles_use_real_text() {
        assert_eq!(title("# AGENTS.md instructions for /work"), None);
        for role in [
            Role::System,
            Role::Developer,
            Role::Tool,
            Role::Other("toolResult".into()),
        ] {
            assert_eq!(visible(&role, "cbioportal"), "");
        }
        assert_eq!(
            title("<recommended_plugins>list</recommended_plugins>"),
            None
        );
        assert_eq!(
            title("<system-reminder>noise</system-reminder>cBioPortal importer"),
            Some("cBioPortal importer".into())
        );
    }

    #[test]
    fn preserves_literal_examples_and_inline_unclosed_tags() {
        for input in [
            "what does `<system-reminder>example</system-reminder>` mean?",
            "```xml\n<system-reminder>example</system-reminder>\n```",
            "what does <system-reminder> mean?",
            "<INSTRUCTIONS>user XML</INSTRUCTIONS>",
        ] {
            assert_eq!(visible(&Role::User, input), input);
        }
    }
}
