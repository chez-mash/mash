//! A user service under the user's own service manager: a systemd user unit
//! on Linux, a launchd `LaunchAgent` on macOS and a logon scheduled task on
//! Windows. None needs administrator rights. Callers write the definition
//! file; this module starts, stops, queries and removes the service. The
//! command lines are plain data, so tests check them without a manager.
use std::path::PathBuf;

pub(crate) use crate::views::qdrant::service::{ServiceState, Supervisor};

/// One service, as its manager knows it.
#[derive(Debug, Clone)]
pub(crate) struct UserService {
    pub(crate) supervisor: Supervisor,
    /// The systemd unit (with `.service`), launchd label or task name.
    pub(crate) name: String,
    /// The unit file, property list or task XML that callers write.
    pub(crate) definition: PathBuf,
    /// PowerShell run after a scheduled task is told to stop: ending a task
    /// ends its console host, not always the program beneath it.
    pub(crate) windows_cleanup: Option<String>,
}

/// The steps a manager can be asked to take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Ask for the state; parsed by [`parse_state`].
    Query,
    /// Stop the running service but keep it registered.
    Stop,
    /// Register the current definition, enable it at login and (re)start it.
    Start,
    /// Stop it and remove its registration.
    Remove,
}

/// A manager command line and whether its failure matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Invocation {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    /// Stopping or removing something already absent is not a failure.
    pub(crate) tolerate_failure: bool,
}

impl Invocation {
    pub(crate) fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Why a step did not complete. Callers word it for their own command.
#[derive(Debug)]
pub(crate) enum Failure {
    /// The program could not be started: no such manager here.
    Unavailable { program: String, reason: String },
    TimedOut {
        program: String,
        after: std::time::Duration,
    },
    /// The command ran and failed; `output` is its combined output.
    Failed { command: Invocation, output: String },
    /// launchd's user domain could not be determined.
    NoUser,
}

impl Failure {
    /// A plain description, for callers without wording of their own.
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Unavailable { program, reason } => {
                format!("`{program}` could not be run ({reason})")
            }
            Self::TimedOut { program, after } => {
                format!("`{program}` did not finish within {} s", after.as_secs())
            }
            Self::Failed { command, output } => {
                let detail: String = output.trim().chars().take(600).collect();
                format!("`{}` failed: {detail}", command.display())
            }
            Self::NoUser => "could not determine this user's ID for launchd".to_owned(),
        }
    }
}

/// Bounds each manager command; none should take this long.
pub(crate) const COMMAND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

pub(crate) fn invocation(program: &str, args: &[&str]) -> Invocation {
    Invocation {
        program: program.to_owned(),
        args: args.iter().map(|&arg| arg.to_owned()).collect(),
        tolerate_failure: false,
    }
}

fn tolerant(mut invocation: Invocation) -> Invocation {
    invocation.tolerate_failure = true;
    invocation
}

pub(crate) fn powershell(script: &str) -> Invocation {
    invocation(
        "powershell.exe",
        &[
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ],
    )
}

/// A PowerShell single-quoted literal.
pub(crate) fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// The command lines for `step`. `domain` is launchd's `gui/UID`.
pub(crate) fn commands(service: &UserService, step: Step, domain: &str) -> Vec<Invocation> {
    let name = service.name.as_str();
    let definition = service.definition.display().to_string();
    match service.supervisor {
        Supervisor::SystemdUser => {
            let systemctl = |args: &[&str]| {
                let mut all = vec!["--user"];
                all.extend_from_slice(args);
                invocation("systemctl", &all)
            };
            match step {
                Step::Query => vec![systemctl(&[
                    "show",
                    name,
                    "--property=LoadState,ActiveState",
                ])],
                Step::Stop => vec![systemctl(&["stop", name])],
                Step::Start => vec![
                    systemctl(&["daemon-reload"]),
                    systemctl(&["enable", name]),
                    systemctl(&["restart", name]),
                ],
                Step::Remove => vec![
                    tolerant(systemctl(&["disable", "--now", name])),
                    tolerant(systemctl(&["reset-failed", name])),
                ],
            }
        }
        Supervisor::LaunchAgent => {
            let target = format!("{domain}/{name}");
            match step {
                Step::Query => vec![tolerant(invocation("launchctl", &["print", &target]))],
                Step::Stop | Step::Remove => {
                    vec![tolerant(invocation("launchctl", &["bootout", &target]))]
                }
                // `RunAtLoad` and `KeepAlive` spawns are deferred, possibly
                // indefinitely, while the domain is in on-demand-only mode;
                // `kickstart` is an explicit demand that starts it regardless.
                Step::Start => vec![
                    tolerant(invocation("launchctl", &["bootout", &target])),
                    invocation("launchctl", &["enable", &target]),
                    invocation("launchctl", &["bootstrap", domain, &definition]),
                    invocation("launchctl", &["kickstart", &target]),
                ],
            }
        }
        Supervisor::ScheduledTask => {
            let task = literal(name);
            let mut halt =
                format!("Stop-ScheduledTask -TaskName {task} -ErrorAction SilentlyContinue");
            if let Some(cleanup) = &service.windows_cleanup {
                halt.push_str("; ");
                halt.push_str(cleanup);
            }
            match step {
                Step::Query => vec![powershell(&format!(
                    "$task = Get-ScheduledTask -TaskName {task} -ErrorAction SilentlyContinue; if ($task) {{ [string]$task.State }} else {{ 'Missing' }}"
                ))],
                Step::Stop => vec![powershell(&halt)],
                Step::Start => vec![
                    powershell(&halt),
                    powershell(&format!(
                        "Register-ScheduledTask -TaskName {task} -Xml (Get-Content -LiteralPath {} -Raw) -Force | Out-Null",
                        literal(&definition)
                    )),
                    powershell(&format!("Start-ScheduledTask -TaskName {task}")),
                ],
                Step::Remove => vec![
                    powershell(&halt),
                    tolerant(powershell(&format!(
                        "Unregister-ScheduledTask -TaskName {task} -Confirm:$false -ErrorAction Stop"
                    ))),
                ],
            }
        }
    }
}

/// The service state from the query's exit status and standard output.
pub(crate) fn parse_state(supervisor: Supervisor, succeeded: bool, output: &str) -> ServiceState {
    match supervisor {
        Supervisor::SystemdUser => {
            if !succeeded {
                return ServiceState::Unknown;
            }
            let value = |key: &str| {
                output
                    .lines()
                    .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                    .map(str::trim)
            };
            if value("LoadState") == Some("not-found") {
                return ServiceState::NotInstalled;
            }
            match value("ActiveState") {
                Some("active" | "reloading" | "refreshing") => ServiceState::Running,
                Some("activating") => ServiceState::Starting,
                Some("inactive" | "deactivating") => ServiceState::Stopped,
                Some("failed") => ServiceState::Failed,
                _ => ServiceState::Unknown,
            }
        }
        Supervisor::LaunchAgent => {
            if !succeeded {
                return ServiceState::NotInstalled;
            }
            match output
                .lines()
                .find_map(|line| line.trim().strip_prefix("state = "))
                .map(str::trim)
            {
                Some("running") => ServiceState::Running,
                Some("spawn scheduled" | "xpcproxy") => ServiceState::Starting,
                Some(_) => ServiceState::Stopped,
                None => ServiceState::Unknown,
            }
        }
        Supervisor::ScheduledTask if succeeded => match output.trim() {
            "Missing" => ServiceState::NotInstalled,
            "Running" => ServiceState::Running,
            "Queued" => ServiceState::Starting,
            "Ready" | "Disabled" => ServiceState::Stopped,
            _ => ServiceState::Unknown,
        },
        Supervisor::ScheduledTask => ServiceState::Unknown,
    }
}

/// Ask the manager for the service's state; `Unknown` when it cannot answer.
pub(crate) async fn state(service: &UserService) -> Result<ServiceState, Failure> {
    let domain = launchd_domain(service.supervisor).await?;
    let [query] = <[Invocation; 1]>::try_from(commands(service, Step::Query, &domain))
        .unwrap_or_else(|_| unreachable!("every manager has one query command"));
    Ok(match run(&query).await {
        Ok((succeeded, output)) => parse_state(service.supervisor, succeeded, &output),
        Err(_) => ServiceState::Unknown,
    })
}

/// Take `step`, stopping at the first command whose failure matters.
pub(crate) async fn perform(service: &UserService, step: Step) -> Result<(), Failure> {
    let domain = launchd_domain(service.supervisor).await?;
    for command in commands(service, step, &domain) {
        let (succeeded, output) = run(&command).await?;
        if !succeeded && !command.tolerate_failure {
            return Err(Failure::Failed { command, output });
        }
        if step == Step::Stop || command.args.first().is_some_and(|a| a == "bootout") {
            wait_unloaded(service, &domain).await;
        }
    }
    Ok(())
}

/// Stop the service, unregister it and delete its definition file. A
/// service that is already gone is the wanted outcome.
pub(crate) async fn remove(service: &UserService) -> Result<(), Failure> {
    perform(service, Step::Remove).await?;
    match std::fs::remove_file(&service.definition) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        // A task registered without a definition file has a directory here.
        Err(_) if service.definition.as_os_str().is_empty() => {}
        Err(error) => {
            return Err(Failure::Failed {
                command: invocation("rm", &[&service.definition.display().to_string()]),
                output: error.to_string(),
            });
        }
    }
    if service.supervisor == Supervisor::SystemdUser {
        // systemd forgets a deleted unit only when it reloads.
        let _ = run(&invocation("systemctl", &["--user", "daemon-reload"])).await;
    }
    Ok(())
}

/// launchd finishes a `bootout` after the command returns; bootstrapping
/// before then fails, so wait (bounded) until the label is gone.
async fn wait_unloaded(service: &UserService, domain: &str) {
    if service.supervisor != Supervisor::LaunchAgent {
        return;
    }
    let Ok([query]) = <[Invocation; 1]>::try_from(commands(service, Step::Query, domain)) else {
        return;
    };
    for _ in 0..40 {
        if matches!(run(&query).await, Ok((false, _))) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

/// launchd's `gui/UID` domain for this user; empty for other managers.
async fn launchd_domain(supervisor: Supervisor) -> Result<String, Failure> {
    if supervisor != Supervisor::LaunchAgent {
        return Ok(String::new());
    }
    let (succeeded, output) = run(&invocation("id", &["-u"])).await?;
    let uid = output.trim();
    if !succeeded || uid.is_empty() || !uid.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Failure::NoUser);
    }
    Ok(format!("gui/{uid}"))
}

/// Run a helper to completion with no input, killed if it outlives
/// `limit`: one that stops to ask something would otherwise hang Mash.
/// `Err` only when it could not be started or timed out.
pub(crate) async fn output(
    program: impl AsRef<std::ffi::OsStr>,
    args: &[impl AsRef<std::ffi::OsStr>],
    limit: std::time::Duration,
) -> Result<std::process::Output, Failure> {
    finish(
        tokio::process::Command::new(program.as_ref()),
        program.as_ref(),
        args,
        limit,
    )
    .await
}

/// [`output`], with `path` as the helper's PATH.
pub(crate) async fn output_with_path(
    program: impl AsRef<std::ffi::OsStr>,
    args: &[impl AsRef<std::ffi::OsStr>],
    limit: std::time::Duration,
    path: &std::ffi::OsStr,
) -> Result<std::process::Output, Failure> {
    let mut command = tokio::process::Command::new(program.as_ref());
    command.env("PATH", path);
    finish(command, program.as_ref(), args, limit).await
}

async fn finish(
    mut command: tokio::process::Command,
    program: &std::ffi::OsStr,
    args: &[impl AsRef<std::ffi::OsStr>],
    limit: std::time::Duration,
) -> Result<std::process::Output, Failure> {
    let running = command
        .args(args)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true)
        .output();
    tokio::time::timeout(limit, running)
        .await
        .map_err(|_| Failure::TimedOut {
            program: program.to_string_lossy().into_owned(),
            after: limit,
        })?
        .map_err(|error| Failure::Unavailable {
            program: program.to_string_lossy().into_owned(),
            reason: error.to_string(),
        })
}

/// Run one manager command and return whether it succeeded, with its
/// combined output.
pub(crate) async fn run(command: &Invocation) -> Result<(bool, String), Failure> {
    let output = output(&command.program, &command.args, COMMAND_TIMEOUT).await?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok((output.status.success(), text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(service: &UserService, step: Step, domain: &str) -> Vec<String> {
        commands(service, step, domain)
            .iter()
            .map(|command| {
                let tolerant = if command.tolerate_failure { "? " } else { "" };
                format!("{tolerant}{}", command.display())
            })
            .collect()
    }

    fn service(supervisor: Supervisor, name: &str, definition: &str) -> UserService {
        UserService {
            supervisor,
            name: name.to_owned(),
            definition: definition.into(),
            windows_cleanup: None,
        }
    }

    /// The sequences `mash dash remote` ran before it shared this module.
    #[test]
    fn systemd_and_launchd_steps_are_the_ones_each_manager_needs() {
        let unit = service(
            Supervisor::SystemdUser,
            "mash-dash.service",
            "/home/me/.config/systemd/user/mash-dash.service",
        );
        assert_eq!(
            lines(&unit, Step::Start, ""),
            [
                "systemctl --user daemon-reload",
                "systemctl --user enable mash-dash.service",
                "systemctl --user restart mash-dash.service",
            ]
        );
        assert_eq!(
            lines(&unit, Step::Remove, ""),
            [
                "? systemctl --user disable --now mash-dash.service",
                "? systemctl --user reset-failed mash-dash.service",
            ]
        );
        let agent = service(
            Supervisor::LaunchAgent,
            "dev.mash.dash",
            "/Users/me/Library/LaunchAgents/dev.mash.dash.plist",
        );
        assert_eq!(
            lines(&agent, Step::Start, "gui/501"),
            [
                "? launchctl bootout gui/501/dev.mash.dash",
                "launchctl enable gui/501/dev.mash.dash",
                "launchctl bootstrap gui/501 /Users/me/Library/LaunchAgents/dev.mash.dash.plist",
                "launchctl kickstart gui/501/dev.mash.dash",
            ]
        );
        assert_eq!(
            lines(&agent, Step::Remove, "gui/501"),
            ["? launchctl bootout gui/501/dev.mash.dash"]
        );
    }

    /// The watcher task `mash-fleet` registers has no definition file here.
    #[test]
    fn a_scheduled_task_stops_runs_cleanup_and_unregisters() {
        let mut task = service(Supervisor::ScheduledTask, "mash-index-watch", "");
        assert_eq!(
            lines(&task, Step::Remove, ""),
            [
                "powershell.exe -NoLogo -NoProfile -NonInteractive -Command Stop-ScheduledTask -TaskName 'mash-index-watch' -ErrorAction SilentlyContinue",
                "? powershell.exe -NoLogo -NoProfile -NonInteractive -Command Unregister-ScheduledTask -TaskName 'mash-index-watch' -Confirm:$false -ErrorAction Stop",
            ]
        );
        task.windows_cleanup = Some("Get-Process -Name x | Stop-Process".into());
        assert!(
            lines(&task, Step::Stop, "")[0]
                .ends_with("-ErrorAction SilentlyContinue; Get-Process -Name x | Stop-Process")
        );
        assert_eq!(
            parse_state(Supervisor::ScheduledTask, true, "Missing\r\n"),
            ServiceState::NotInstalled
        );
    }
}
