//! Reinstalling the agent plugins on this machine, as the setup warning's
//! manual step does: `mash init --yes --no-index`, run by the installed Mash.
//!
//! The request arrives through the service action route, so it carries that
//! route's token, origin and forwarding rules, and a machine only ever runs it
//! for itself. The command runs as a child of the Mash executable on disk,
//! never inside the dashboard, so the plugin and the MCP registration come
//! from the release installed there. Runs are kept in memory only: unlike a
//! supervisor action, repeating one is harmless, because `mash init` checks
//! each agent before it changes anything.
use super::model::{Action, now};
use super::operations::{ActionRequest, Operation, Outcome};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_app::core::installation::Agent;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;
use tokio::io::AsyncReadExt;

/// The one target a reinstall names: every agent's plugin on the machine.
pub(super) const TARGET: &str = "agent-plugins";
/// Each agent CLI call inside `mash init` has its own two-minute limit.
const LIMIT: Duration = Duration::from_mins(10);
const KEPT: usize = 10;
const DETAIL: usize = 300;

/// The reinstalls this dashboard started, newest first.
#[derive(Default)]
pub(super) struct Runs(Mutex<Vec<Operation>>);

impl Runs {
    pub(super) fn history(&self) -> Vec<Operation> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn finish(&self, id: &str, outcome: Outcome, message: String) {
        let mut runs = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(run) = runs.iter_mut().find(|run| run.request.request_id == id) {
            run.outcome = outcome;
            run.message = message;
            run.completed_at = Some(now());
        }
    }
}

/// Answer a reinstall request addressed to this machine.
pub(super) fn reinstall(state: &Arc<super::super::Dash>, request: ActionRequest) -> Response {
    // The same machines that may run first-run setup, which is `mash init`
    // too: the machine itself, or a dashboard started with a services
    // configuration, which is how a controller reaches its peers.
    if !state.loopback && !state.services.managed() {
        return super::error(
            StatusCode::FORBIDDEN,
            "The plugin can be reinstalled from the machine itself, or from a dashboard started with a services configuration",
        );
    }
    match start(state, request) {
        Ok(operation) => axum::Json(operation).into_response(),
        Err(message) => super::error(StatusCode::CONFLICT, &message),
    }
}

fn start(state: &Arc<super::super::Dash>, request: ActionRequest) -> Result<Operation, String> {
    if request.action != Action::ReinstallPlugin || request.service != TARGET {
        return Err("A plugin reinstall names this machine's agent plugins".into());
    }
    // The person confirmed against a setup check made by one build. A
    // dashboard restarted since then may judge the plugin differently.
    if request.control_revision != crate::BUILD_REVISION {
        return Err(
            "The dashboard on this machine runs a different build from the one that checked its setup. Check again, then reinstall."
                .into(),
        );
    }
    let mut runs = state
        .services
        .plugin_runs
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(prior) = runs
        .iter()
        .find(|run| run.request.request_id == request.request_id)
    {
        return if prior.request == request {
            Ok(prior.clone())
        } else {
            Err("Request ID was already used for a different action".into())
        };
    }
    if runs.iter().any(|run| run.outcome == Outcome::Pending) {
        return Err("The plugin is already being reinstalled on this machine.".into());
    }
    let program = match &state.services.init_program {
        Some(program) => program.clone(),
        None => installed(std::env::current_exe())?,
    };
    let mut command = tokio::process::Command::new(program);
    for (flag, value) in [
        ("--config", &state.paths.config),
        ("--db", &state.paths.db),
        ("--models-dir", &state.paths.models_dir),
    ] {
        if let Some(value) = value {
            command.arg(flag).arg(value);
        }
    }
    command
        .args(["init", "--yes", "--no-index"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let child = command
        .spawn()
        .map_err(|_| "Could not start `mash init` on this machine.".to_owned())?;
    let operation = Operation {
        request,
        requested_at: now(),
        completed_at: None,
        outcome: Outcome::Pending,
        message: "Reinstalling the plugin with `mash init --yes --no-index`.".into(),
    };
    runs.insert(0, operation.clone());
    runs.truncate(KEPT);
    drop(runs);
    let state = state.clone();
    let id = operation.request.request_id.clone();
    tokio::spawn(async move {
        let (outcome, message) = finished(child).await;
        state.services.plugin_runs.finish(&id, outcome, message);
    });
    Ok(operation)
}

/// The Mash executable on disk that this dashboard was started from.
///
/// Deploying Mash replaces that file while the dashboard keeps running.
/// Linux then names the running image `<path> (deleted)`, and the file now at
/// `<path>` is the installed release: the one whose plugin the agents need.
fn installed(current: std::io::Result<PathBuf>) -> Result<PathBuf, String> {
    let current = current.map_err(|_| "Could not find the Mash executable on this machine.")?;
    if current.is_file() {
        return Ok(current);
    }
    current
        .to_str()
        .and_then(|path| path.strip_suffix(" (deleted)"))
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .ok_or_else(|| {
            "The Mash executable this dashboard started from is gone. Restart the dashboard on this machine, then try again."
                .into()
        })
}

async fn finished(mut child: tokio::process::Child) -> (Outcome, String) {
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let run = tokio::time::timeout(LIMIT, async {
        let mut output = String::new();
        let mut errors = String::new();
        let read_output = async {
            if let Some(stdout) = stdout.as_mut() {
                let _ = stdout.read_to_string(&mut output).await;
            }
        };
        let read_errors = async {
            if let Some(stderr) = stderr.as_mut() {
                let _ = stderr.read_to_string(&mut errors).await;
            }
        };
        tokio::join!(read_output, read_errors);
        let success = child.wait().await.is_ok_and(|status| status.success());
        (success, output, errors)
    })
    .await;
    match run {
        Ok((success, output, errors)) => judge(success, &output, &errors),
        Err(_) => {
            let _ = child.kill().await;
            (
                Outcome::Unknown,
                "`mash init` did not finish within 10 minutes and was stopped. Check the agent setup again before retrying.".into(),
            )
        }
    }
}

/// What `mash init` reported, judged by its plugin steps. Its JSON report on
/// stdout names each agent it found and what happened to that agent's plugin.
fn judge(success: bool, output: &str, errors: &str) -> (Outcome, String) {
    let error = last_error(errors);
    let Ok(report) = serde_json::from_str::<serde_json::Value>(output) else {
        return (
            Outcome::Failed,
            error.unwrap_or_else(|| "`mash init` failed on this machine.".into()),
        );
    };
    let agents: Vec<(String, &serde_json::Value)> = report["agents"]
        .as_array()
        .map(|agents| {
            agents
                .iter()
                .map(|agent| (name(&agent["agent"]), &agent["plugin"]))
                .collect()
        })
        .unwrap_or_default();
    if agents.is_empty() {
        return (
            Outcome::Failed,
            "`mash init` found no Claude Code, Codex or GitHub Copilot CLI on the PATH this dashboard runs with, so it reinstalled nothing. Run it in a terminal on this machine instead.".into(),
        );
    }
    if let Some((agent, plugin)) = agents
        .iter()
        .find(|(_, plugin)| plugin["state"] == "failed")
    {
        let cause = plugin["message"].as_str().unwrap_or("no reason given");
        return (
            Outcome::Failed,
            clip(format!(
                "The plugin was not reinstalled for {agent}: {cause}"
            )),
        );
    }
    if !success {
        return (
            Outcome::Failed,
            clip(error.unwrap_or_else(|| "`mash init` reported a problem on this machine.".into())),
        );
    }
    let names: Vec<&str> = agents.iter().map(|(name, _)| name.as_str()).collect();
    (
        Outcome::Succeeded,
        format!("The plugin is current for {}.", names.join(", ")),
    )
}

fn name(agent: &serde_json::Value) -> String {
    serde_json::from_value::<Agent>(agent.clone()).map_or_else(
        |_| agent.as_str().unwrap_or("an agent").to_owned(),
        |agent| agent.display_name().to_owned(),
    )
}

/// The last structured error `mash init` wrote to stderr, where progress
/// lines also arrive.
fn last_error(errors: &str) -> Option<String> {
    errors.lines().rev().find_map(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|value| value["error"]["message"].as_str().map(str::to_owned))
    })
}

fn clip(message: String) -> String {
    message.chars().take(DETAIL).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_reinstall_is_judged_by_each_agents_plugin_step() {
        let report = |plugin: &str| {
            format!(
                r#"{{"agents":[{{"agent":"claude-code","mcp":{{"state":"already-present"}},"plugin":{plugin}}}]}}"#
            )
        };
        let (outcome, message) = judge(
            true,
            &report(r#"{"state":"done","command":"claude plugin install mash@mash"}"#),
            "",
        );
        assert_eq!(outcome, Outcome::Succeeded);
        assert!(message.contains("Claude Code"), "{message}");
        let (outcome, message) = judge(
            true,
            &report(
                r#"{"state":"failed","command":"claude plugin install mash@mash","message":"offline"}"#,
            ),
            "",
        );
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.contains("Claude Code: offline"), "{message}");
        let (outcome, message) = judge(
            false,
            &report(r#"{"state":"already-present"}"#),
            "{\"error\":{\"message\":\"stale registration\"}}\n",
        );
        assert_eq!(outcome, Outcome::Failed);
        assert_eq!(message, "stale registration");
    }

    #[test]
    fn plugin_reinstall_that_found_no_agent_or_wrote_no_report_fails() {
        let (outcome, message) = judge(true, r#"{"agents":[]}"#, "");
        assert_eq!(outcome, Outcome::Failed);
        assert!(
            message.contains("PATH this dashboard runs with"),
            "{message}"
        );
        let (outcome, message) = judge(
            false,
            "",
            "progress\n{\"error\":{\"message\":\"no config\"}}\n",
        );
        assert_eq!(outcome, Outcome::Failed);
        assert_eq!(message, "no config");
        assert_eq!(judge(false, "not json", "").0, Outcome::Failed);
    }

    #[test]
    fn plugin_reinstall_runs_the_binary_a_deploy_put_in_place() {
        let directory = tempfile::tempdir().unwrap();
        let binary = directory.path().join("mash");
        assert!(installed(Ok(binary.clone())).is_err());
        std::fs::write(&binary, "").unwrap();
        assert_eq!(installed(Ok(binary.clone())).unwrap(), binary);
        let replaced = PathBuf::from(format!("{} (deleted)", binary.display()));
        assert_eq!(installed(Ok(replaced)).unwrap(), binary);
        assert!(installed(Err(std::io::Error::other("none"))).is_err());
    }
}
