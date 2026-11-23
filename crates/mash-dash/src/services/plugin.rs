//! Reinstalling the agent plugins on this machine, as the setup warning's
//! manual step `mash init --yes --no-index` does, by running that same setup
//! code inside the dashboard rather than as a command.
//!
//! The request arrives through the service action route, so it carries that
//! route's token, origin and forwarding rules, and a machine only ever runs it
//! for itself. Agents are found where a shell would find them, not only on
//! the dashboard's own PATH, which a desktop app or service manager keeps
//! short. Setup speaks for the build running it (it judges the plugin
//! against that build and registers this executable), so it is refused
//! while the installed Mash is another release than the dashboard runs: a
//! deploy that replaced the file must restart the dashboard first. Runs are
//! kept in memory only: repeating one is harmless, because setup checks each
//! agent before it changes anything.
use super::model::{Action, now};
use super::operations::{ActionRequest, Operation, Outcome};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_app::core::installation::Agent;
use std::ffi::OsString;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// The one target a reinstall names: every agent's plugin on the machine.
pub(super) const TARGET: &str = "agent-plugins";
/// Each agent CLI call inside setup has its own two-minute limit.
const LIMIT: Duration = Duration::from_mins(10);
const KEPT: usize = 10;
const DETAIL: usize = 300;

/// The reinstalls this dashboard started, newest first, and the executable
/// it started from.
pub(super) struct Runs {
    runs: Mutex<Vec<Operation>>,
    started: mash_app::runtime::StartedExecutable,
}

impl Default for Runs {
    fn default() -> Self {
        Self {
            runs: Mutex::default(),
            started: mash_app::runtime::StartedExecutable::now(),
        }
    }
}

impl Runs {
    pub(super) fn history(&self) -> Vec<Operation> {
        self.runs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn finish(&self, id: &str, outcome: Outcome, message: String) {
        let mut runs = self.runs.lock().unwrap_or_else(PoisonError::into_inner);
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
    if state.services.plugin_runs.started.replaced() {
        return Err(REPLACED.into());
    }
    let mut runs = state
        .services
        .plugin_runs
        .runs
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
    let operation = Operation {
        request,
        requested_at: now(),
        completed_at: None,
        outcome: Outcome::Pending,
        message: "Reinstalling the plugin, as `mash init --yes --no-index` does.".into(),
    };
    runs.insert(0, operation.clone());
    runs.truncate(KEPT);
    drop(runs);
    let state = state.clone();
    let id = operation.request.request_id.clone();
    tokio::spawn(async move {
        let paths = state.paths.clone();
        let agent_path = state.services.agent_path.clone();
        let (outcome, message) = reinstall_here(paths, agent_path).await;
        state.services.plugin_runs.finish(&id, outcome, message);
    });
    Ok(operation)
}

/// A deploy replaces the executable while the dashboard keeps running.
const REPLACED: &str = "The Mash installed on this machine was replaced after this dashboard started, so the dashboard would set up the older release it still runs. Restart the dashboard on this machine, then reinstall.";

/// Run `mash init --yes --no-index`'s setup in this process: an existing
/// configuration is kept, and each agent found gets its MCP registration and
/// plugin checked and, where needed, put right.
async fn reinstall_here(
    paths: mash_app::core::Paths,
    agent_path: Option<OsString>,
) -> (Outcome, String) {
    use mash_app::core::{self, InitRequest, Operation as Run, Request, Response};
    let request = InitRequest {
        force: false,
        agents: true,
        index: false,
        marketplace: core::MARKETPLACE.to_owned(),
        plan: None,
        selected_agents: None,
        backfill_memory: None,
        agent_path,
    };
    let run = core::execute(Request {
        paths,
        operation: Run::Init(request),
    });
    match tokio::time::timeout(LIMIT, run).await {
        Ok(Ok(Response::Init(response))) => serde_json::to_value(&response).map_or_else(
            |_| (Outcome::Failed, "Setup ran but its report could not be read.".into()),
            |report| judge(&report),
        ),
        Ok(Ok(_)) => (Outcome::Failed, "Setup answered with something else.".into()),
        Ok(Err(error)) => (Outcome::Failed, clip(&error.error.message)),
        // Dropping the run stops it, and the agent CLI it was waiting on.
        Err(_) => (
            Outcome::Unknown,
            "Setup did not finish within 10 minutes and was stopped. Check the agent setup again before retrying.".into(),
        ),
    }
}

/// What setup reported, judged by each agent's steps: the plugin, and an
/// MCP registration that names another executable and could not be replaced.
fn judge(report: &serde_json::Value) -> (Outcome, String) {
    let agents: Vec<(String, &serde_json::Value)> = report["agents"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|agent| (name(&agent["agent"]), agent))
        .collect();
    if agents.is_empty() {
        return (Outcome::Failed, NO_AGENT.into());
    }
    for (agent, steps) in &agents {
        let plugin = &steps["plugin"];
        if plugin["state"] == "failed" {
            let cause = plugin["message"].as_str().unwrap_or("no reason given");
            return (
                Outcome::Failed,
                clip(&format!(
                    "The plugin was not reinstalled for {agent}: {cause}"
                )),
            );
        }
        let mcp = &steps["mcp"];
        if matches!(mcp["state"].as_str(), Some("failed" | "stale")) {
            let cause = mcp["message"].as_str().unwrap_or("no reason given");
            return (
                Outcome::Failed,
                clip(&format!(
                    "Mash could not be registered with {agent}: {cause}"
                )),
            );
        }
    }
    let names: Vec<&str> = agents.iter().map(|(name, _)| name.as_str()).collect();
    (
        Outcome::Succeeded,
        format!("The plugin is current for {}.", names.join(", ")),
    )
}

const NO_AGENT: &str = "No Claude Code, Codex or GitHub Copilot CLI was found on this machine: not on the dashboard's PATH, the login shell's PATH or in the folders their installers use. Their plugins are installed through those CLIs, so there was nothing to reinstall.";

fn name(agent: &serde_json::Value) -> String {
    serde_json::from_value::<Agent>(agent.clone()).map_or_else(
        |_| agent.as_str().unwrap_or("an agent").to_owned(),
        |agent| agent.display_name().to_owned(),
    )
}

fn clip(message: &str) -> String {
    message.chars().take(DETAIL).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(mcp: &str, plugin: &str) -> serde_json::Value {
        serde_json::from_str(&format!(
            r#"{{"agents":[{{"agent":"claude-code","mcp":{mcp},"plugin":{plugin}}}]}}"#
        ))
        .unwrap()
    }

    #[test]
    fn plugin_reinstall_is_judged_by_each_agents_steps() {
        let present = r#"{"state":"already-present"}"#;
        let (outcome, message) = judge(&report(
            present,
            r#"{"state":"done","command":"claude plugin install mash@mash"}"#,
        ));
        assert_eq!(outcome, Outcome::Succeeded);
        assert!(message.contains("Claude Code"), "{message}");
        let (outcome, message) = judge(&report(
            present,
            r#"{"state":"failed","command":"claude plugin install mash@mash","message":"offline"}"#,
        ));
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.contains("Claude Code: offline"), "{message}");
        let (outcome, message) = judge(&report(
            r#"{"state":"stale","stale":"/old/mash","command":"claude mcp add","message":"locked"}"#,
            present,
        ));
        assert_eq!(outcome, Outcome::Failed);
        assert!(
            message.contains("registered with Claude Code: locked"),
            "{message}"
        );
    }

    #[test]
    fn plugin_reinstall_that_found_no_agent_says_where_it_looked() {
        let (outcome, message) = judge(&serde_json::json!({"agents": []}));
        assert_eq!(outcome, Outcome::Failed);
        assert!(message.contains("login shell's PATH"), "{message}");
        assert_eq!(message.matches("instead").count(), 0, "{message}");
    }
}
