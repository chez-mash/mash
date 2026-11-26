//! One Mash on the machine: the app's bundled `mash` as the command in a
//! terminal, in the background services and in the agents' connections, so
//! an update of the app updates all of them together.
//!
//! The app does not work out what else is installed. The bundled `mash
//! doctor` does (its `copies`, `processes` and `watcher` checks), and the app
//! shows what it found. "Use the App’s Mash Everywhere" then does three
//! things: installs the command that runs the bundled `mash` (cli.rs), runs
//! the repair the doctor names for agent connections (`mash init --yes
//! --no-index`, which registers through each agent's own CLI), and restarts
//! the services that run the app's `mash`. Anything a package manager
//! installed is left where it is, and the doctor's own line says so.

use crate::update::after;
use crate::{cli, engine};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogResult};

/// The doctor check that compares every `mash` on the machine with this one.
const COPIES: &str = "copies";
/// The repair the doctor names when an agent's connection runs another mash.
const REGISTER: &str = "init --yes --no-index";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Finding {
    check: String,
    state: String,
    summary: String,
    #[serde(default)]
    repair: Option<String>,
}

#[derive(Deserialize)]
struct Report {
    findings: Vec<Finding>,
}

/// What the bundled `mash doctor` finds. It exits 1 when it finds a problem,
/// which is still an answer.
fn examine(program: &Path) -> Vec<Finding> {
    after::quiet(program)
        .arg("doctor")
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|output| findings(&output.stdout))
        .unwrap_or_default()
}

fn findings(report: &[u8]) -> Option<Vec<Finding>> {
    serde_json::from_slice::<Report>(report)
        .ok()
        .map(|report| report.findings)
}

/// The findings of `checks` that are not in order.
fn differing(findings: &[Finding], checks: &[&str]) -> Vec<Finding> {
    findings
        .iter()
        .filter(|finding| finding.state != "ok" && checks.contains(&finding.check.as_str()))
        .cloned()
        .collect()
}

/// Findings as lines of a notice: what is so and, `with_repairs`, what to
/// do about it.
fn lines(findings: &[Finding], with_repairs: bool) -> String {
    findings
        .iter()
        .map(|finding| match &finding.repair {
            Some(repair) if with_repairs => format!("• {} ({repair})", finding.summary),
            _ => format!("• {}", finding.summary),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The app's `mash` and the command that runs it: what a service or a
/// dashboard that follows the app's version is started from.
pub fn programs() -> Vec<PathBuf> {
    let mut programs: Vec<PathBuf> = engine::locate()
        .map(|program| std::fs::canonicalize(&program).unwrap_or(program))
        .into_iter()
        .collect();
    programs.extend(cli::command_file());
    programs
}

/// At launch: when another `mash` is the command, or runs a service or an
/// agent's connection, say so once and offer to use the app's own
/// everywhere. Nothing is shown to someone who chose to keep them separate.
pub fn at_launch(app: &AppHandle) {
    if crate::update::keeps_separate(app) {
        return;
    }
    let Ok(program) = engine::locate() else {
        return;
    };
    let separate = differing(&examine(&program), &[COPIES]);
    if separate.is_empty() {
        return;
    }
    let found = lines(&separate, false);
    let handle = app.clone();
    if !cfg!(target_os = "macos") {
        // Elsewhere the command is a separate install with its own updates,
        // so this only says that the versions differ.
        app.dialog()
            .message(format!(
                "Another copy of Mash is in use on this computer:\n\n{found}\n\n\
                 It updates separately from this app."
            ))
            .title("Mash")
            .buttons(MessageDialogButtons::OkCancelCustom(
                "OK".into(),
                "Don’t Show Again".into(),
            ))
            .show(move |ok| {
                if !ok {
                    crate::update::keep_separate(&handle);
                }
            });
        return;
    }
    const EVERYWHERE: &str = "Use the App’s Mash Everywhere";
    const SEPARATE: &str = "Keep Separate";
    app.dialog()
        .message(format!(
            "Another copy of Mash is in use on this Mac:\n\n{found}\n\n\
             Mash can use this app’s own mash for the command, the background services \
             and your agents, so they all update together with the app."
        ))
        .title("Mash")
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            EVERYWHERE.into(),
            SEPARATE.into(),
            "Not Now".into(),
        ))
        .show_with_result(move |answer| match answer {
            MessageDialogResult::Yes => everywhere(&handle),
            MessageDialogResult::Custom(label) if label == EVERYWHERE => everywhere(&handle),
            MessageDialogResult::No => crate::update::keep_separate(&handle),
            MessageDialogResult::Custom(label) if label == SEPARATE => {
                crate::update::keep_separate(&handle);
            }
            _ => {}
        });
}

/// Use the app's `mash` for the command, the agents and the services, then
/// say what was done and what is still separate.
fn everywhere(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Ok(program) = engine::locate() else {
            return;
        };
        let separate = differing(&examine(&program), &[COPIES]);
        let mut done = vec![cli::install_everywhere()];
        if separate
            .iter()
            .any(|finding| finding.repair.as_deref().is_some_and(names_register))
        {
            let registered = after::quiet(&program)
                .args(["init", "--yes", "--no-index"])
                .output()
                .is_ok_and(|output| output.status.success());
            done.push(if registered {
                "Connected your agents to this app’s mash. Restart open agent sessions to use it."
                    .to_owned()
            } else {
                "Couldn’t connect your agents to this app’s mash; open Settings in Mash to try again."
                    .to_owned()
            });
        }
        let restarted = after::restart_services(&programs());
        if !restarted.is_empty() {
            done.push(format!("Restarted {}.", restarted.join(" and ")));
        }
        let left = differing(&examine(&program), &[COPIES]);
        let mut message = done.join("\n");
        if !left.is_empty() {
            message.push_str("\n\nLeft as it is:\n");
            message.push_str(&lines(&left, true));
        }
        app.dialog().message(message).title("Mash").show(|_| {});
    });
}

fn names_register(repair: &str) -> bool {
    repair.contains(REGISTER)
}

/// After an update: what does not yet run the new version, as the text of a
/// notice, or nothing when every part of Mash is on it.
pub fn after_update(version: &str, separate_is_chosen: bool) -> Option<String> {
    let program = engine::locate().ok()?;
    let checks: &[&str] = if separate_is_chosen {
        &["processes", "watcher"]
    } else {
        &[COPIES, "processes", "watcher"]
    };
    let behind = differing(&examine(&program), checks);
    let by_hand = after::hand_started_dashboards(&programs());
    notice(version, &behind, &by_hand)
}

fn notice(version: &str, behind: &[Finding], by_hand: &[String]) -> Option<String> {
    if behind.is_empty() && by_hand.is_empty() {
        return None;
    }
    let mut text = format!("Mash was updated to {version}. Not everything runs it yet:");
    if !behind.is_empty() {
        text.push_str("\n\n");
        text.push_str(&lines(behind, true));
    }
    if !by_hand.is_empty() {
        text.push_str(
            "\n\nThese dashboards were started by hand and run the earlier version until \
             you restart them:\n",
        );
        for dashboard in by_hand {
            text.push_str("• ");
            text.push_str(dashboard);
            text.push('\n');
        }
    }
    Some(text.trim_end().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{differing, findings, lines, names_register, notice};

    const REPORT: &str = r#"{"healthy":true,"problems":0,"warnings":2,"findings":[
        {"check":"configuration","state":"ok","summary":"Configuration loaded."},
        {"check":"copies","state":"warning","summary":"`mash` in a terminal runs /opt/homebrew/bin/mash (0.2.3), not this copy (0.2.4).","cause":"Homebrew installed it.","repair":"brew upgrade mash"},
        {"check":"copies","state":"warning","summary":"Codex runs another mash.","repair":"/Applications/Mash.app/Contents/MacOS/mash init --yes --no-index"},
        {"check":"copies","state":"ok","summary":"The dashboard service runs this copy."},
        {"check":"processes","state":"problem","summary":"An index watcher (process 41) is still running a build that has been replaced.","repair":"launchctl kickstart -k gui/$(id -u)/dev.mash.index-watch"},
        {"check":"search","state":"warning","summary":"Search is behind."}]}"#;

    #[test]
    fn shows_only_what_the_doctor_found_out_of_step() {
        let all = findings(REPORT.as_bytes()).unwrap();
        let copies = differing(&all, &["copies"]);
        assert_eq!(copies.len(), 2);
        assert_eq!(lines(&copies, false).lines().count(), 2);
        assert!(!lines(&copies, false).contains("brew upgrade"));
        assert_eq!(
            lines(&copies, true),
            "• `mash` in a terminal runs /opt/homebrew/bin/mash (0.2.3), not this copy (0.2.4). (brew upgrade mash)\n\
             • Codex runs another mash. (/Applications/Mash.app/Contents/MacOS/mash init --yes --no-index)"
        );
        assert!(names_register(copies[1].repair.as_deref().unwrap()));
        assert!(!names_register(copies[0].repair.as_deref().unwrap()));
        assert_eq!(
            differing(&all, &["copies", "processes", "watcher"]).len(),
            3
        );
        assert!(findings(b"mash: not found").is_none());
    }

    #[test]
    fn an_update_is_quiet_when_everything_runs_it() {
        assert_eq!(notice("0.2.5", &[], &[]), None);
        let all = findings(REPORT.as_bytes()).unwrap();
        let text = notice(
            "0.2.5",
            &differing(&all, &["processes"]),
            &["process 612: /Applications/Mash.app/Contents/MacOS/mash dash".to_owned()],
        )
        .unwrap();
        assert!(text.starts_with("Mash was updated to 0.2.5."), "{text}");
        assert!(text.contains("process 41"), "{text}");
        assert!(text.contains("started by hand"), "{text}");
        assert!(text.ends_with("mash dash"), "{text}");
    }
}
