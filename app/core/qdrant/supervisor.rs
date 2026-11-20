//! Qdrant's service under the user's own supervisor: the definitions Mash
//! writes for it and how supervisor failures are worded. Starting, stopping
//! and removing it go through [`crate::service`], which every user service
//! Mash installs shares. Definitions and commands are plain data, so tests
//! check them without running a supervisor.
use super::{Layout, Ports, install_error};
use crate::AppError;
use crate::service::{Failure, UserService};
use std::path::Path;

pub(crate) use crate::service::Step;
#[cfg(test)]
pub(crate) use crate::service::{Invocation, parse_state};
pub(crate) use crate::views::qdrant::service::{ServiceState, Supervisor};

/// Qdrant's own configuration file (YAML, Qdrant's format). It binds only
/// to loopback, keeps telemetry off and places data under `layout.data`.
pub(crate) fn qdrant_config(layout: &Layout, ports: Ports) -> String {
    let quoted = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "''"));
    let mut text = format!(
        "# Written by `mash qdrant install`, which replaces it on every run.\n\
         service:\n  host: 127.0.0.1\n  http_port: {}\n  grpc_port: {}\n\
         storage:\n  storage_path: {}\n  snapshots_path: {}\n\
         telemetry_disabled: true\n\
         logger:\n  log_level: INFO\n",
        ports.http,
        ports.grpc,
        quoted(&layout.storage()),
        quoted(&layout.snapshots()),
    );
    // A scheduled task has no output stream to capture, so Qdrant writes
    // its own log; an unbuffered one survives a forced stop.
    if layout.platform.supervisor() == Supervisor::ScheduledTask {
        text.push_str("  on_disk:\n    enabled: true\n    log_file: ");
        text.push_str(&quoted(&layout.log()));
        text.push_str("\n    log_level: INFO\n    buffer_size_bytes: 1\n");
    }
    text
}

/// The supervisor's definition of the service running `binary`.
pub(crate) fn definition(layout: &Layout, binary: &Path) -> Result<String, AppError> {
    for path in [binary, &layout.root, &layout.qdrant_config(), &layout.log()] {
        let text = path.to_str().ok_or_else(|| unexpressible(path))?;
        if text.chars().any(|c| c.is_control() || c == '"') {
            return Err(unexpressible(path));
        }
    }
    let text = |path: &Path| path.display().to_string();
    Ok(match layout.platform.supervisor() {
        Supervisor::SystemdUser => {
            // `%` starts a specifier and `\` an escape in systemd values.
            let escape = |path: &Path| text(path).replace('\\', "\\\\").replace('%', "%%");
            format!(
                "# Written by `mash qdrant install`; rerun it to repair, or run `mash qdrant uninstall`.\n\
                 [Unit]\nDescription=Qdrant search service for Mash\n\n\
                 [Service]\nType=simple\nWorkingDirectory={}\n\
                 ExecStart=\"{}\" --config-path \"{}\"\n\
                 Restart=on-failure\nRestartSec=5\nTimeoutStopSec=60\n\n\
                 [Install]\nWantedBy=default.target\n",
                text(&layout.root).replace('%', "%%"),
                escape(binary),
                escape(&layout.qdrant_config()),
            )
        }
        Supervisor::LaunchAgent => {
            let string = |value: &str| format!("<string>{}</string>", xml(value));
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                 <!-- Written by `mash qdrant install`; rerun it to repair, or run `mash qdrant uninstall`. -->\n\
                 <plist version=\"1.0\"><dict>\n\
                 <key>Label</key>{}\n\
                 <key>ProgramArguments</key><array>{}<string>--config-path</string>{}</array>\n\
                 <key>WorkingDirectory</key>{}\n\
                 <key>RunAtLoad</key><true/>\n\
                 <key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>\n\
                 <key>ThrottleInterval</key><integer>10</integer>\n\
                 <key>SoftResourceLimits</key><dict><key>NumberOfFiles</key><integer>10240</integer></dict>\n\
                 <key>StandardOutPath</key>{}\n\
                 <key>StandardErrorPath</key>{}\n\
                 </dict></plist>\n",
                string(&layout.service),
                string(&text(binary)),
                string(&text(&layout.qdrant_config())),
                string(&text(&layout.root)),
                string(&text(&layout.log())),
                string(&text(&layout.log())),
            )
        }
        Supervisor::ScheduledTask => {
            let user = &layout.account.user;
            if user.is_empty() {
                return Err(install_error(
                    "USERNAME is not set, so the logon task has no user",
                ));
            }
            // A console program started at logon opens a window; the headless
            // console host runs it without one.
            let conhost = format!("{}\\System32\\conhost.exe", layout.account.system_root);
            format!(
                "<Task version=\"1.2\" xmlns=\"http://schemas.microsoft.com/windows/2004/02/mit/task\">\n\
                 <RegistrationInfo><Description>Qdrant search service for Mash. Written by mash qdrant install; rerun it to repair, or run mash qdrant uninstall.</Description></RegistrationInfo>\n\
                 <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{user}</UserId></LogonTrigger></Triggers>\n\
                 <Principals><Principal id=\"Author\"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>\n\
                 <Settings>\
                 <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>\
                 <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>\
                 <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>\
                 <AllowHardTerminate>true</AllowHardTerminate>\
                 <StartWhenAvailable>true</StartWhenAvailable>\
                 <IdleSettings><StopOnIdleEnd>false</StopOnIdleEnd><RestartOnIdle>false</RestartOnIdle></IdleSettings>\
                 <AllowStartOnDemand>true</AllowStartOnDemand>\
                 <Enabled>true</Enabled>\
                 <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>\
                 <Priority>5</Priority>\
                 <RestartOnFailure><Interval>PT1M</Interval><Count>3</Count></RestartOnFailure>\
                 </Settings>\n\
                 <Actions Context=\"Author\"><Exec><Command>{}</Command><Arguments>{}</Arguments><WorkingDirectory>{}</WorkingDirectory></Exec></Actions>\n\
                 </Task>\n",
                xml(&conhost),
                xml(&format!(
                    "--headless \"{}\" --config-path \"{}\"",
                    text(binary),
                    text(&layout.qdrant_config())
                )),
                xml(&text(&layout.root)),
                user = xml(user),
            )
        }
    })
}

fn unexpressible(path: &Path) -> AppError {
    install_error(format!(
        "{} cannot be written into a service definition (it is not UTF-8, or has quotes or control characters); set a plain data directory and rerun",
        path.display()
    ))
}

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl Layout {
    /// Qdrant's service, as the shared supervisor code drives it. On Windows,
    /// ending the task ends the console host; wait for Qdrant beneath it to
    /// exit, then end any copy of Mash's Qdrant still running.
    pub(crate) fn user_service(&self) -> UserService {
        let bin = crate::service::literal(&format!("{}\\", self.root.join("bin").display()));
        UserService {
            supervisor: self.platform.supervisor(),
            name: self.service.clone(),
            definition: self.definition(),
            windows_cleanup: Some(format!(
                "$owned = @(Get-Process -Name qdrant -ErrorAction SilentlyContinue | Where-Object {{ $_.Path -and $_.Path.StartsWith({bin}, [StringComparison]::OrdinalIgnoreCase) }}); \
                 if ($owned) {{ $owned | Wait-Process -Timeout 15 -ErrorAction SilentlyContinue; $owned | Where-Object {{ -not $_.HasExited }} | Stop-Process -Force }}"
            )),
        }
    }
}

/// The command lines for `step`, which the tests check. `domain` is
/// launchd's `gui/UID`.
#[cfg(test)]
pub(crate) fn commands(layout: &Layout, step: Step, domain: &str) -> Vec<Invocation> {
    crate::service::commands(&layout.user_service(), step, domain)
}

/// How supervisor steps are carried out.
pub(crate) enum Control {
    /// Run the real supervisor's commands.
    System,
    /// Simulate a supervisor in tests.
    #[cfg(test)]
    Fake(std::sync::Arc<super::tests::FakeSupervisor>),
}

impl Control {
    pub(crate) async fn state(&self, layout: &Layout) -> Result<ServiceState, AppError> {
        match self {
            Self::System => crate::service::state(&layout.user_service())
                .await
                .map_err(|failure| supervisor_failure(layout, failure)),
            #[cfg(test)]
            Self::Fake(fake) => Ok(fake.state()),
        }
    }

    pub(crate) async fn stop(&self, layout: &Layout) -> Result<(), AppError> {
        self.perform(layout, Step::Stop).await
    }

    pub(crate) async fn start(&self, layout: &Layout) -> Result<(), AppError> {
        self.perform(layout, Step::Start).await
    }

    /// Stop and unregister the service; the caller removes its files.
    pub(crate) async fn remove(&self, layout: &Layout) -> Result<(), AppError> {
        self.perform(layout, Step::Remove).await
    }

    async fn perform(&self, layout: &Layout, step: Step) -> Result<(), AppError> {
        match self {
            Self::System => crate::service::perform(&layout.user_service(), step)
                .await
                .map_err(|failure| supervisor_failure(layout, failure)),
            #[cfg(test)]
            Self::Fake(fake) => fake.perform(layout, step).await,
        }
    }
}

/// A supervisor failure, worded for `mash qdrant install` and its uninstall.
fn supervisor_failure(layout: &Layout, failure: Failure) -> AppError {
    match failure {
        Failure::TimedOut { program } => install_error(format!(
            "`{program}` did not finish within {} s",
            crate::service::COMMAND_TIMEOUT.as_secs()
        )),
        Failure::Unavailable { program, reason } => install_error(format!(
            "could not run `{program}` ({reason}); `mash qdrant install` needs the user's service manager on this machine. Run Qdrant yourself, for example with Docker, as the installation guide describes"
        )),
        Failure::NoUser => install_error("could not determine this user's ID for launchd"),
        Failure::Failed { command, output } => {
            let detail: String = output.trim().chars().take(600).collect();
            let hint = match layout.platform.supervisor() {
                Supervisor::SystemdUser => {
                    " If there is no systemd user session here (some containers and WSL setups), run Qdrant yourself, for example with Docker."
                }
                Supervisor::LaunchAgent | Supervisor::ScheduledTask => "",
            };
            install_error(format!("`{}` failed: {detail}.{hint}", command.display()))
        }
    }
}
