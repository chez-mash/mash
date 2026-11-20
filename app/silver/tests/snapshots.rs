//! Each provider-format fixture read into chats and messages, recorded as
//! snapshots so a change in what a reader produces shows up as a diff.
use super::*;

fn read(parser: fn(&Path) -> Result<ParsedFile, AppError>, fixture: &str) -> serde_json::Value {
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), fixture).unwrap();
    let chat = parser(file.path()).unwrap().chat.unwrap();
    mash_silver::snapshot::snapshot(&chat)
}

#[test]
fn claude_tools_fixture_reads_as_its_snapshot() {
    insta::assert_json_snapshot!(read(
        parse_claude,
        include_str!("../../../fixtures/claude-native-tools-2.1.296.jsonl")
    ));
}

#[test]
fn codex_fixtures_read_as_their_snapshots() {
    for (name, fixture) in [
        (
            "codex_tools",
            include_str!("../../../fixtures/codex-native-tools-0.160.1.jsonl"),
        ),
        (
            "codex_migration",
            include_str!("../../../fixtures/codex-native-migration-0.159.2.jsonl"),
        ),
        (
            "codex_lifecycle_only",
            include_str!("../../../fixtures/codex-native-lifecycle-only-0.159.2.jsonl"),
        ),
    ] {
        insta::assert_json_snapshot!(name, read(parse_codex, fixture));
    }
}
