//! What an update needs besides new files, done by the new version the first
//! time it starts: bring the index up to the new schema before the engine
//! opens it, and afterwards restart Mash's background services that run the
//! app's own `mash`, so nothing keeps running the version that was replaced.
//!
//! The app decides nothing about schemas or services itself. The index is
//! upgraded by the bundled `mash index`, when the bundled `mash status` says
//! the schema needs it, exactly as a deployment does. A service is restarted
//! only when its definition names the app's `mash` or the command that runs
//! it; a service that runs a separate copy of Mash is left alone. Dashboards
//! someone started by hand are never stopped, only listed.

use crate::engine::{self, Log};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The watcher and dashboard services Mash installs: (launchd label, systemd
/// user unit). See docs/services.md.
const SERVICES: [(&str, &str); 2] = [
    ("dev.mash.index-watch", "mash-index-watch.service"),
    ("dev.mash.dash", "mash-dash.service"),
];

/// Upgrade the index in place when this version's schema is newer than the
/// one on disk. `mash status` refuses such an index and says whether indexing
/// with this version upgrades it; any other answer is left to the engine,
/// which reports it on its own page.
pub fn upgrade_index(program: &Path, log: &Log) {
    let Ok(status) = quiet(program).arg("status").output() else {
        return;
    };
    let said = String::from_utf8_lossy(&status.stderr);
    if status.status.success() || !needs_upgrade_in_place(&said) {
        return;
    }
    engine::remember(log, "Upgrading the index for this version of Mash…");
    match quiet(program).arg("index").output() {
        Ok(output) if output.status.success() => engine::remember(log, "The index is upgraded."),
        Ok(output) => {
            engine::remember(log, "Upgrading the index did not finish:");
            for line in String::from_utf8_lossy(&output.stderr)
                .lines()
                .rev()
                .take(5)
            {
                engine::remember(log, line);
            }
        }
        Err(error) => engine::remember(log, &format!("Couldn’t run mash index: {error}")),
    }
}

/// Whether `mash status` refused the index for a schema that `mash index`
/// upgrades in place (the wording of mash-storage's schema check, which
/// tools/upgrade-local-index.sh reads the same way).
fn needs_upgrade_in_place(status_error: &str) -> bool {
    status_error.contains(r#""kind":"schema-incompatible""#)
        && status_error.contains("requires an in-place upgrade")
}

/// A `mash` command that opens no window and reads nothing from the app.
pub fn quiet(program: &Path) -> Command {
    let mut command = Command::new(program);
    command.stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

/// Restart each installed service whose definition runs one of `programs`
/// (the app's `mash` and the command that leads to it), and name those
/// restarted.
pub fn restart_services(programs: &[PathBuf]) -> Vec<&'static str> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    SERVICES
        .into_iter()
        .filter_map(|(label, unit)| {
            let (name, definition) = if cfg!(target_os = "macos") {
                (
                    label,
                    home.join("Library/LaunchAgents")
                        .join(format!("{label}.plist")),
                )
            } else if cfg!(target_os = "linux") {
                (unit, home.join(".config/systemd/user").join(unit))
            } else {
                return None;
            };
            let text = std::fs::read_to_string(definition).ok()?;
            (runs_one_of(&text, programs) && restart(name)).then_some(name)
        })
        .collect()
}

/// Whether a service definition names one of `programs` as a whole path.
fn runs_one_of(definition: &str, programs: &[PathBuf]) -> bool {
    programs.iter().any(|program| {
        let program = program.to_string_lossy();
        definition.match_indices(program.as_ref()).any(|(at, _)| {
            let after = definition[at + program.len()..].chars().next();
            !after.is_some_and(|next| next.is_alphanumeric() || "-_./".contains(next))
        })
    })
}

/// Restart a loaded service; one that is not loaded stays as it is.
#[cfg(target_os = "macos")]
fn restart(label: &str) -> bool {
    // SAFETY: getuid only reads this process's user number.
    let target = format!("gui/{}/{label}", unsafe { libc::getuid() });
    let run = |arguments: &[&str]| {
        Command::new("/bin/launchctl")
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    };
    run(&["print", &target]) && run(&["kickstart", "-k", &target])
}

#[cfg(target_os = "linux")]
fn restart(unit: &str) -> bool {
    let run = |arguments: &[&str]| {
        Command::new("systemctl")
            .arg("--user")
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    };
    run(&["is-active", "--quiet", unit]) && run(&["restart", unit])
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn restart(_name: &str) -> bool {
    false
}

/// The process a service manager is running for `name`, if any.
#[cfg(target_os = "macos")]
fn service_pid(label: &str) -> Option<u32> {
    // SAFETY: getuid only reads this process's user number.
    let target = format!("gui/{}/{label}", unsafe { libc::getuid() });
    let output = Command::new("/bin/launchctl")
        .args(["print", &target])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.trim().strip_prefix("pid = ")?.parse().ok())
}

#[cfg(target_os = "linux")]
fn service_pid(unit: &str) -> Option<u32> {
    let output = Command::new("systemctl")
        .args(["--user", "show", "--property", "MainPID", "--value", unit])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()
        .filter(|pid| *pid != 0)
}

/// Dashboards started by hand from one of `programs`, as `ps` lists them:
/// they still run the version that was replaced until someone restarts them.
/// The app's own engine (its parent is the app) and the dashboard a service
/// manager runs are not among them. A dashboard left running by a closed
/// terminal has process 1 as its parent, like a service, so services are
/// told apart by asking their manager which process is theirs.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn hand_started_dashboards(programs: &[PathBuf]) -> Vec<String> {
    let supervised: Vec<u32> = SERVICES
        .into_iter()
        .filter_map(|(label, unit)| {
            service_pid(if cfg!(target_os = "macos") {
                label
            } else {
                unit
            })
        })
        .collect();
    Command::new("ps")
        .args(["-axo", "pid=,ppid=,command="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| {
            dashboards_in(
                &String::from_utf8_lossy(&output.stdout),
                programs,
                &supervised,
                std::process::id(),
            )
        })
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn hand_started_dashboards(_programs: &[PathBuf]) -> Vec<String> {
    Vec::new()
}

/// The `mash dash` lines of a `ps -axo pid=,ppid=,command=` listing that run
/// one of `programs` and are neither a service's process (`supervised`) nor
/// a child of this app.
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), allow(dead_code))]
fn dashboards_in(listing: &str, programs: &[PathBuf], supervised: &[u32], app: u32) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| {
            let (pid, rest) = line.trim_start().split_once(char::is_whitespace)?;
            let (parent, command) = rest.trim_start().split_once(char::is_whitespace)?;
            let (pid, parent): (u32, u32) = (pid.parse().ok()?, parent.parse().ok()?);
            let command = command.trim_start();
            let program = programs.iter().find_map(|program| {
                command
                    .strip_prefix(program.to_string_lossy().as_ref())
                    .filter(|rest| rest.starts_with(' '))
            })?;
            let dashboard = program.split_whitespace().any(|word| word == "dash");
            let ours = parent == app || supervised.contains(&pid);
            (dashboard && !ours).then(|| format!("process {pid}: {command}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{dashboards_in, needs_upgrade_in_place, runs_one_of};
    use std::path::PathBuf;

    #[test]
    fn only_an_in_place_schema_upgrade_runs_the_index() {
        let refused = r#"{"error":{"kind":"schema-incompatible","message":"local index schema 70 requires an in-place upgrade to 71; run `mash index` with this release, then reconnect Mash MCP"}}"#;
        assert!(needs_upgrade_in_place(refused));
        for other in [
            "",
            r#"{"error":{"kind":"schema-incompatible","message":"local index schema 99 is newer than this release"}}"#,
            r#"{"error":{"kind":"configuration","message":"requires an in-place upgrade"}}"#,
        ] {
            assert!(!needs_upgrade_in_place(other), "{other}");
        }
    }

    #[test]
    fn a_service_is_ours_only_when_it_names_the_apps_mash() {
        let programs = [
            PathBuf::from("/Applications/Mash.app/Contents/MacOS/mash"),
            PathBuf::from("/Users/sam/.local/bin/mash"),
        ];
        let ours = "<array><string>/Applications/Mash.app/Contents/MacOS/mash</string><string>dash</string>";
        assert!(runs_one_of(ours, &programs));
        assert!(runs_one_of(
            "ExecStart=/Users/sam/.local/bin/mash dash --listen 127.0.0.1:8790",
            &programs
        ));
        for separate in [
            "<string>/opt/homebrew/bin/mash</string>",
            "<string>/Users/sam/.local/bin/mash-index-watch</string>",
            "ExecStart=/Users/sam/.local/bin/mash.old dash",
            "",
        ] {
            assert!(!runs_one_of(separate, &programs), "{separate}");
        }
    }

    #[test]
    fn lists_only_dashboards_someone_started_by_hand() {
        let programs = [PathBuf::from("/Applications/Mash.app/Contents/MacOS/mash")];
        let listing = "\
  501     1 /Applications/Mash.app/Contents/MacOS/mash dash --listen 127.0.0.1:8790
  612     1 /Applications/Mash.app/Contents/MacOS/mash dash --listen 127.0.0.1:9000
  700   900 /Applications/Mash.app/Contents/MacOS/mash dash --listen 127.0.0.1:47290
  702   600 /Applications/Mash.app/Contents/MacOS/mash index --watch
  703   600 /opt/homebrew/bin/mash dash
  704   600 /Applications/Mash.app/Contents/MacOS/mash-desktop
  garbage";
        assert_eq!(
            dashboards_in(listing, &programs, &[501], 900),
            [
                "process 612: /Applications/Mash.app/Contents/MacOS/mash dash --listen 127.0.0.1:9000"
            ]
        );
    }
}
