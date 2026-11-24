//! How long Claude Code keeps its transcripts, which are Mash's source for
//! that agent. Claude Code deletes old ones itself, and Mash then drops the
//! chats they backed, so `doctor` warns about a short setting and `init`
//! says how to change it.
use super::{Check, Finding, State};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Below this, Claude Code's own cleanup removes transcripts Mash would
/// otherwise still hold a year on.
const RETENTION_DAYS: u64 = 365;
/// Claude Code's retention when its settings name none.
const DEFAULT_RETENTION_DAYS: u64 = 30;

const CAUSE: &str = "Sessions not started from the desktop app are removed by Claude Code's own cleanup, and Mash then removes those chats from the index and the shared copy.";

/// The part of Claude Code's settings that decides how long its transcripts,
/// which are Mash's source for that agent, are kept.
#[derive(Deserialize)]
struct ClaudeSettings {
    #[serde(rename = "cleanupPeriodDays")]
    cleanup_period_days: Option<u64>,
}

pub(super) fn claude_settings() -> Option<PathBuf> {
    let directory = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| directories::BaseDirs::new().map(|base| base.home_dir().join(".claude")))?;
    directory.is_dir().then(|| directory.join("settings.json"))
}

/// Claude Code deletes old transcripts itself; Mash then drops those chats,
/// because a source that is gone is treated as deleted history.
pub(super) fn retention(settings: Option<&Path>) -> Option<Finding> {
    let path = settings?;
    let days = kept_days(path);
    Some(if days >= RETENTION_DAYS {
        Finding::ok(
            Check::Retention,
            format!("Claude Code keeps its transcripts for {days} days."),
        )
    } else {
        Finding::found(
            Check::Retention,
            State::Warning,
            format!("Claude Code deletes transcripts {days} days after their last write."),
        )
        .cause(CAUSE)
        .repair(repair(path))
    })
}

/// The days Claude Code keeps a transcript after its last write, as its
/// settings at `path` say, or its default.
fn kept_days(path: &Path) -> u64 {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ClaudeSettings>(&bytes).ok())
        .and_then(|settings| settings.cleanup_period_days)
        .unwrap_or(DEFAULT_RETENTION_DAYS)
}

fn repair(path: &Path) -> String {
    format!("Set \"cleanupPeriodDays\": 3650 in {}.", path.display())
}

/// What `init` says when Claude Code is here and keeps its transcripts for
/// less than [`RETENTION_DAYS`]; None otherwise.
pub(in crate::core) fn claude_cleanup_advice() -> Option<String> {
    cleanup_advice(claude_settings().as_deref())
}

fn cleanup_advice(settings: Option<&Path>) -> Option<String> {
    let path = settings?;
    let days = kept_days(path);
    (days < RETENTION_DAYS).then(|| {
        format!(
            "Claude Code deletes its chats {days} days after their last write, and Mash then removes them too. To keep them, set \"cleanupPeriodDays\": 3650 in {}.",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::cleanup_advice;

    #[test]
    fn init_advises_a_longer_claude_retention_only_when_it_is_short() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join("settings.json");
        assert!(cleanup_advice(None).is_none());
        let advice = cleanup_advice(Some(&settings)).unwrap();
        assert!(
            advice.starts_with("Claude Code deletes its chats 30 days"),
            "{advice}"
        );
        assert!(
            advice.contains("set \"cleanupPeriodDays\": 3650 in "),
            "{advice}"
        );
        std::fs::write(&settings, r#"{"cleanupPeriodDays":3650}"#).unwrap();
        assert!(cleanup_advice(Some(&settings)).is_none());
    }
}
