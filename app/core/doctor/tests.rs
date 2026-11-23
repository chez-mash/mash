use super::*;

fn count(provider: &str, sources: u64) -> SourceCount {
    SourceCount {
        provider: provider.into(),
        sources,
    }
}

/// Found on a machine after an upgrade: Codex sources older than the 90-day
/// range waited to be read. An ordinary pass now reads them, so the repair
/// is the ordinary one, whatever their age.
#[test]
fn sources_awaiting_reading_need_only_an_ordinary_pass() {
    let sources = SourceStates {
        awaiting: vec![count("claude-code", 80), count("copilot", 3)],
        rejected: Vec::new(),
    };
    let finding = unread(&sources);
    assert_eq!(finding.state, State::Problem);
    assert!(finding.summary.contains("83 sources"));
    assert!(finding.summary.contains("claude-code 80, copilot 3"));
    assert_eq!(finding.repair.as_deref(), Some("mash index"));
    let nothing_waiting = unread(&SourceStates::default());
    assert_eq!(nothing_waiting.state, State::Problem);
    assert_eq!(nothing_waiting.repair.as_deref(), Some("mash index"));
}

/// Found on a machine whose 105 transcripts vanished: Claude Code's default
/// keeps a transcript 30 days, and Mash then dropped the chats they backed.
#[test]
fn short_claude_retention_is_a_warning_with_the_setting_that_changes_it() {
    let directory = tempfile::tempdir().unwrap();
    let settings = directory.path().join("settings.json");
    assert!(retention(None).is_none());
    // No file, or a file that does not name the setting, means the default.
    let unset = retention(Some(&settings)).unwrap();
    assert_eq!(unset.state, State::Warning);
    assert!(unset.summary.contains("30 days"));
    assert!(
        unset
            .repair
            .unwrap()
            .contains("\"cleanupPeriodDays\": 3650")
    );
    std::fs::write(&settings, r#"{"model":"x","cleanupPeriodDays":14}"#).unwrap();
    assert!(
        retention(Some(&settings))
            .unwrap()
            .summary
            .contains("14 days")
    );
    std::fs::write(&settings, r#"{"cleanupPeriodDays":3650}"#).unwrap();
    assert_eq!(retention(Some(&settings)).unwrap().state, State::Ok);
}

#[test]
fn a_report_is_unhealthy_only_when_it_holds_a_problem() {
    let warned = Response::new(vec![
        Finding::ok(Check::Index, "fine"),
        Finding::found(Check::Retention, State::Warning, "short"),
    ]);
    assert!(warned.healthy());
    assert_eq!((warned.problems, warned.warnings), (0, 1));
    let broken = Response::new(vec![Finding::found(
        Check::SearchService,
        State::Problem,
        "down",
    )]);
    assert!(!broken.healthy());
    let text = broken.text().unwrap();
    assert!(text.starts_with("Mash doctor: 1 problems, 0 warnings.\n"));
    assert!(text.contains("[problem] search-service: down"));
}
