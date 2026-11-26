//! Whether everything on this machine that runs Mash runs this copy of it:
//! the `mash` a terminal finds, the installed services, the agents'
//! connections, and the long-lived processes already running.
//!
//! Two copies at different versions is how search stops after an upgrade: a
//! watcher or an agent's connection keeps the earlier build, which then
//! refuses the newer schema. Nothing here changes any of it. Each finding
//! names the other copy, who installed it when that can be told, and the
//! command that brings it into step; the schema and contract checks in the
//! commands themselves remain what actually refuses a mismatch.
use super::{Check, Finding, State, restart_watcher};
use crate::core::installation::{Agent, Registration, registration};
use crate::runtime;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long another copy may take to say what it is.
const ASK: Duration = Duration::from_secs(5);

/// What another `mash` says of itself with `--version-info`.
#[derive(serde::Deserialize)]
struct Told {
    version: String,
    /// Absent from releases before 0.2.5.
    build: Option<String>,
}

/// How a copy at some path relates to the running one.
enum Relation {
    /// The same file, or another file of the same build (such as a command
    /// that runs this copy).
    Same,
    /// Another build, with its version when it gave one.
    Other(Option<String>),
}

async fn relation(path: &Path, this: &Path) -> Relation {
    if std::fs::canonicalize(path).is_ok_and(|path| path == this) {
        return Relation::Same;
    }
    let told = crate::service::output(path, &["--version-info"], ASK)
        .await
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| serde_json::from_slice::<Told>(&output.stdout).ok());
    match told {
        Some(told) if told.build.as_deref() == Some(crate::BUILD_REVISION) => Relation::Same,
        Some(told) => Relation::Other(Some(told.version)),
        None => Relation::Other(None),
    }
}

/// Who installed the copy at `path`, and how that copy is updated, when the
/// path says so.
fn installer(path: &Path) -> Option<(&'static str, &'static str)> {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = resolved.to_string_lossy().replace('\\', "/");
    if text.contains("/Cellar/") || text.contains("/homebrew/") || text.contains("/linuxbrew/") {
        Some(("Homebrew", "brew upgrade mash"))
    } else if text.contains("/scoop/") {
        Some(("Scoop", "scoop update mash"))
    } else if text == "/usr/bin/mash" && Path::new("/var/lib/dpkg/info/mash-cli.list").exists() {
        Some(("apt", "sudo apt install --only-upgrade mash-cli"))
    } else {
        None
    }
}

fn described(version: Option<&str>) -> String {
    version.map_or_else(String::new, |version| format!(" (version {version})"))
}

/// The findings: one line when everything is this copy, otherwise a warning
/// for each thing that is not.
pub(super) async fn copies() -> Vec<Finding> {
    let Some(this) = std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::canonicalize(path).ok())
    else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    findings.extend(command(&this).await);
    findings.extend(services(&this).await);
    findings.extend(connections(&this).await);
    if findings.is_empty() {
        findings.push(Finding::ok(
            Check::Copies,
            "The mash command, the installed services and the agents' connections all run this copy of Mash.",
        ));
    }
    findings
}

/// The `mash` a terminal would run, found the way a login shell finds it.
async fn command(this: &Path) -> Option<Finding> {
    let found = crate::blocking(|| Ok(crate::core::setup::which("mash")))
        .await
        .ok()
        .flatten()?;
    let Relation::Other(version) = relation(&found, this).await else {
        return None;
    };
    let finding = Finding::found(
        Check::Copies,
        State::Warning,
        format!(
            "`mash` in a terminal runs {}{}, not this copy ({}, version {}).",
            found.display(),
            described(version.as_deref()),
            this.display(),
            env!("CARGO_PKG_VERSION"),
        ),
    );
    Some(match installer(&found) {
        Some((manager, update)) => finding
            .cause(format!(
                "{manager} installed that copy and updates it separately; it is left in place."
            ))
            .repair(update),
        None => finding.cause("That copy was installed separately and updates separately."),
    })
}

/// The program a service definition starts: the first of a launch agent's
/// `ProgramArguments`, or a systemd unit's `ExecStart` command.
fn service_program(definition: &str) -> Option<PathBuf> {
    let program = if let Some((_, arguments)) = definition.split_once("<key>ProgramArguments</key>")
    {
        let (_, rest) = arguments.split_once("<string>")?;
        let (program, _) = rest.split_once("</string>")?;
        program
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&")
    } else {
        let line = definition
            .lines()
            .find_map(|line| line.trim().strip_prefix("ExecStart="))?;
        let line = line.trim_start_matches(['-', '@', '+', '!', ':']);
        match line.strip_prefix('"') {
            Some(quoted) => quoted.split_once('"')?.0.to_owned(),
            None => line.split_whitespace().next()?.to_owned(),
        }
    };
    (!program.is_empty()).then(|| PathBuf::from(program))
}

fn is_mash(program: &Path) -> bool {
    program
        .file_stem()
        .is_some_and(|name| name.eq_ignore_ascii_case("mash"))
}

/// The dashboard and watcher services, by what their definitions start. A
/// watcher started through a launcher script is not judged here; the
/// processes check sees the build it actually runs.
async fn services(this: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let dashboard = crate::remote::installed();
    let definitions = [
        (
            "dashboard",
            dashboard
                .as_ref()
                .map(|(layout, _)| layout.unit_file.clone()),
            format!("{} dash remote", this.display()),
        ),
        (
            "watcher",
            crate::watch::installed_definition(),
            restart_watcher().to_owned(),
        ),
    ];
    let mut dashboard_differs = false;
    for (name, definition, repair) in definitions {
        let Some(program) = definition
            .and_then(|file| std::fs::read_to_string(file).ok())
            .and_then(|text| service_program(&text))
            .filter(|program| is_mash(program))
        else {
            continue;
        };
        if let Relation::Other(version) = relation(&program, this).await {
            dashboard_differs |= name == "dashboard";
            findings.push(
                Finding::found(
                    Check::Copies,
                    State::Warning,
                    format!(
                        "The {name} service runs {}{}, not this copy.",
                        program.display(),
                        described(version.as_deref()),
                    ),
                )
                .repair(repair),
            );
        }
    }
    // A service that starts this copy may still be running the one before.
    if !dashboard_differs
        && let Some((layout, Some(port))) = dashboard
        && let Some(build) = served_build(port).await
        && build != crate::BUILD_REVISION
    {
        findings.push(
            Finding::found(
                Check::Copies,
                State::Warning,
                format!(
                    "The dashboard service on port {port} is still running build {}, not this one ({}).",
                    short(&build),
                    short(crate::BUILD_REVISION),
                ),
            )
            .cause("It started before this copy was installed and has not restarted since.")
            .repair(restart_dashboard(layout.service())),
        );
    }
    findings
}

fn short(build: &str) -> &str {
    build.get(..12).unwrap_or(build)
}

fn restart_dashboard(service: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("launchctl kickstart -k gui/$(id -u)/{service}")
    } else {
        format!("systemctl --user restart {service}")
    }
}

/// The build a dashboard on this machine's `port` says it is.
async fn served_build(port: u16) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Health {
        build: String,
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    let health = client
        .get(format!("http://127.0.0.1:{port}/api/health"))
        .send()
        .await
        .ok()?;
    Some(health.json::<Health>().await.ok()?.build)
}

/// Agents whose Mash connection starts some other program.
async fn connections(this: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for agent in [Agent::Codex, Agent::ClaudeCode, Agent::Copilot] {
        let found = crate::blocking(move || Ok(registration(agent))).await;
        if let Ok(Registration::Stale { command }) = found {
            findings.push(
                Finding::found(
                    Check::Copies,
                    State::Warning,
                    format!(
                        "{}'s Mash connection runs `{command}`, not this copy.",
                        agent.display_name()
                    ),
                )
                .repair(format!("{} init --yes --no-index", this.display())),
            );
        }
    }
    findings
}

/// Long-lived Mash processes left running on a build that has been replaced.
pub(super) fn processes(inventory: &runtime::Inventory) -> Finding {
    let stale = |kind: fn(&runtime::Kind) -> bool| {
        inventory
            .processes
            .iter()
            .filter(|process| process.state != runtime::State::Current && kind(&process.kind))
            .map(|process| process.pid.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let watchers = stale(|kind| matches!(kind, runtime::Kind::IndexWatch));
    let servers = stale(|kind| matches!(kind, runtime::Kind::Mcp));
    if !watchers.is_empty() {
        return Finding::found(
            Check::Processes,
            State::Problem,
            format!("An index watcher (process {watchers}) is still running a build that has been replaced."),
        )
        .cause("A replaced watcher refuses new writes, so indexing has stopped until it restarts.")
        .repair(restart_watcher());
    }
    if !servers.is_empty() {
        return Finding::found(
            Check::Processes,
            State::Warning,
            format!("An agent's Mash connection (process {servers}) is still running a build that has been replaced."),
        )
        .repair("Reconnect Mash in that agent, or restart the agent session.");
    }
    Finding::ok(
        Check::Processes,
        "Every running Mash process is this build.",
    )
}

#[cfg(test)]
mod tests {
    use super::{installer, is_mash, service_program};
    use std::path::{Path, PathBuf};

    #[test]
    fn reads_the_program_a_service_starts() {
        let agent = "<plist version=\"1.0\"><dict>\n<key>Label</key><string>dev.mash.dash</string>\n\
            <key>ProgramArguments</key><array><string>/Applications/My &amp; Apps/Mash.app/Contents/MacOS/mash</string><string>dash</string></array>\n</dict></plist>";
        assert_eq!(
            service_program(agent),
            Some(PathBuf::from(
                "/Applications/My & Apps/Mash.app/Contents/MacOS/mash"
            ))
        );
        let unit = "[Service]\nEnvironmentFile=/home/sam/.mash/dashboard.env\nExecStart=/home/sam/.local/bin/mash dash --listen 127.0.0.1:8790\n";
        assert_eq!(
            service_program(unit),
            Some(PathBuf::from("/home/sam/.local/bin/mash"))
        );
        let quoted = "ExecStart=\"/opt/my mash/mash\" dash\n";
        assert_eq!(
            service_program(quoted),
            Some(PathBuf::from("/opt/my mash/mash"))
        );
        assert_eq!(
            service_program("[Unit]\nDescription=nothing to start\n"),
            None
        );
    }

    #[test]
    fn a_launcher_script_is_not_taken_for_mash() {
        assert!(is_mash(Path::new("/home/sam/.local/bin/mash")));
        assert!(is_mash(Path::new(r"C:\Users\sam\scoop\shims\mash.exe")) || cfg!(unix));
        assert!(!is_mash(Path::new("/home/sam/.local/bin/mash-index-watch")));
        assert!(!is_mash(Path::new("/bin/sh")));
    }

    #[test]
    fn names_the_package_manager_only_when_the_path_shows_it() {
        assert_eq!(
            installer(Path::new("/opt/homebrew/Cellar/mash/0.2.4/bin/mash")).map(|found| found.0),
            Some("Homebrew")
        );
        assert_eq!(
            installer(Path::new("/home/linuxbrew/.linuxbrew/bin/mash")).map(|found| found.0),
            Some("Homebrew")
        );
        assert_eq!(installer(Path::new("/home/sam/.local/bin/mash")), None);
        assert_eq!(
            installer(Path::new("/Applications/Mash.app/Contents/MacOS/mash")),
            None
        );
    }
}
