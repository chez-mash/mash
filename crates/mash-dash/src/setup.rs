//! First-run setup from the browser: what `mash init` asks in a terminal.
//!
//! The page chooses among typed options only: which found histories to index,
//! how far back, and which installed agents to connect. History locations
//! come from this machine, agents are connected by their own CLIs through the
//! same core as `mash init`, and the first index runs as a separate
//! `mash index` process, never inside the dashboard.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_app::core::installation::Agent;
use mash_app::core::{self, InitPlan, InitRequest, Operation, Request};
use mash_app::{config, semantic, views};
use mash_model::AppError;
use mash_model::provider::Provider;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use super::Dash;

/// What the setup page shows: the file's choices, what this machine has, and
/// the first index when this dashboard started one.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRun")]
struct Setup {
    #[serde(flatten)]
    file: config::setup::View,
    /// Whether this dashboard accepts setup, as for settings.
    editable: bool,
    integrations: Vec<Integration>,
    qdrant: Qdrant,
    models_installed: bool,
    index: IndexRun,
}

/// An agent Mash can connect itself to through the agent's own CLI.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunIntegration")]
struct Integration {
    agent: Agent,
    /// Whether the agent's CLI is on this machine's `PATH`.
    installed: bool,
    /// Whether the configuration records Mash as connected to it.
    connected: bool,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunQdrant")]
struct Qdrant {
    url: String,
    state: views::qdrant::Reachability,
    reachable: bool,
    /// Why Mash could not use the configured search service.
    message: Option<String>,
}

/// The `mash index` this dashboard started after setup, if any.
#[derive(Clone, Default, serde::Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunIndex")]
#[serde(tag = "state", rename_all = "kebab-case")]
enum IndexRun {
    #[default]
    Idle,
    Running,
    /// Not started, and why.
    Skipped {
        reason: String,
    },
    Finished {
        ok: bool,
        chats: Option<u64>,
        messages: Option<u64>,
        /// What went wrong, or why search by meaning is not current.
        message: Option<String>,
    },
}

/// One dashboard runs per process, so one first index at a time.
fn index_run() -> &'static Mutex<IndexRun> {
    static RUN: OnceLock<Mutex<IndexRun>> = OnceLock::new();
    RUN.get_or_init(Mutex::default)
}

fn set_index(run: IndexRun) {
    *index_run()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = run;
}

fn index_state() -> IndexRun {
    index_run()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunRequest")]
#[serde(deny_unknown_fields)]
struct Choice {
    revision: String,
    agents: BTreeMap<Provider, bool>,
    since_days: Option<u32>,
    /// The installed agents to connect Mash to.
    connect: Vec<Agent>,
    /// Start the first index once setup is saved.
    index: bool,
}

/// What setup changed, and the page's state afterwards.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunDone")]
struct Done {
    report: core::InitReport,
    setup: Setup,
}

pub(super) fn routes(router: aide::axum::ApiRouter<Arc<Dash>>) -> aide::axum::ApiRouter<Arc<Dash>> {
    use super::contract::{changes, reads};
    router.api_route(
        "/api/services/setup",
        aide::axum::routing::get_with(
            view,
            reads::<Setup>(
                "/api/services/setup",
                "What first-run setup would choose on this machine.",
            ),
        )
        .post_with(
            apply,
            changes::<Done>(
                "/api/services/setup",
                "Save the configuration, connect agents and start the first index.",
            ),
        ),
    )
}

fn failure(status: StatusCode, error: &AppError) -> Response {
    (
        status,
        axum::Json(serde_json::json!({ "error": error.error })),
    )
        .into_response()
}

fn editable(state: &Dash) -> bool {
    state.loopback || state.services.managed()
}

async fn describe(state: &Dash) -> Result<Setup, AppError> {
    let paths = state.paths.clone();
    let (file, current, models_installed) = tokio::task::spawn_blocking(move || {
        let path = super::configuration_path(&paths)?;
        let models = paths
            .models_dir
            .clone()
            .unwrap_or_else(core::default_models_path);
        Ok::<_, AppError>((
            config::setup::view(&path)?,
            config::setup::current(&path)?,
            semantic::Models::new(models).is_installed(),
        ))
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("read setup")))?;
    let observed = views::qdrant::reachability(&current.qdrant).await;
    let integrations = [
        (Agent::ClaudeCode, "claude", config::Agent::ClaudeCode),
        (Agent::Codex, "codex", config::Agent::Codex),
        (Agent::Copilot, "copilot", config::Agent::Copilot),
    ]
    .into_iter()
    .map(|(agent, program, recorded)| Integration {
        agent,
        installed: core::setup::which(program).is_some(),
        connected: current.connected_agents.contains(&recorded),
    })
    .collect();
    Ok(Setup {
        file,
        editable: editable(state),
        integrations,
        qdrant: Qdrant {
            url: observed.url,
            state: observed.state,
            reachable: observed.state == views::qdrant::Reachability::Reachable,
            message: observed.message,
        },
        models_installed,
        index: index_state(),
    })
}

async fn view(State(state): State<Arc<Dash>>) -> Response {
    match describe(&state).await {
        Ok(setup) => axum::Json(setup).into_response(),
        Err(error) => failure(StatusCode::BAD_REQUEST, &error),
    }
}

/// Saves the configuration and connects the chosen agents, as `mash init`
/// does, then starts the first index as its own process when asked. Accepted
/// where settings changes are: on the machine itself, or from a dashboard
/// started with a services configuration.
async fn apply(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<Choice>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !editable(&state) {
        return failure(
            StatusCode::FORBIDDEN,
            &AppError::configuration(
                "setup runs on the machine itself, or from a dashboard started with a services configuration",
            ),
        );
    }
    let Ok(axum::Json(choice)) = input else {
        return failure(
            StatusCode::BAD_REQUEST,
            &AppError::configuration("invalid setup choices"),
        );
    };
    match run(&state, choice).await {
        Ok(done) => axum::Json(done).into_response(),
        Err(error) => failure(StatusCode::BAD_REQUEST, &error),
    }
}

async fn run(state: &Dash, choice: Choice) -> Result<Done, AppError> {
    let paths = state.paths.clone();
    let chosen = config::setup::Choice {
        revision: choice.revision,
        agents: choice.agents,
        since_days: choice.since_days,
    };
    let (choices, previous) = tokio::task::spawn_blocking(move || {
        config::setup::plan(&super::configuration_path(&paths)?, &chosen)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("plan setup")))?;
    let connect = !choice.connect.is_empty();
    let request = InitRequest {
        force: false,
        agents: connect,
        // The dashboard never indexes in its own process.
        index: false,
        marketplace: core::MARKETPLACE.to_owned(),
        plan: Some(InitPlan { choices, previous }),
        selected_agents: connect.then_some(choice.connect),
        backfill_memory: None,
    };
    let response = core::execute(Request {
        paths: state.paths.clone(),
        operation: Operation::Init(request),
    })
    .await?;
    let core::Response::Init(response) = response else {
        return Err(AppError::internal("setup answered with something else"));
    };
    if choice.index {
        start_index(state).await;
    }
    let indexing = matches!(index_state(), IndexRun::Running | IndexRun::Finished { .. });
    Ok(Done {
        report: response.into_report(choice.index && indexing),
        setup: describe(state).await?,
    })
}

/// Starts `mash index` from this executable, unless one started here is
/// still running or Qdrant does not answer.
async fn start_index(state: &Dash) {
    if matches!(index_state(), IndexRun::Running) {
        return;
    }
    let paths = state.paths.clone();
    let qdrant = tokio::task::spawn_blocking(move || {
        config::setup::current(&super::configuration_path(&paths)?).map(|current| current.qdrant)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::internal("read setup")));
    let observed = match qdrant {
        Ok(qdrant) => views::qdrant::reachability(&qdrant).await,
        Err(error) => {
            return set_index(IndexRun::Skipped {
                reason: error.error.message,
            });
        }
    };
    if observed.state != views::qdrant::Reachability::Reachable {
        return set_index(IndexRun::Skipped {
            reason: observed.message.unwrap_or_else(|| {
                "the local Qdrant service did not answer; start it, then run `mash index`"
                    .to_owned()
            }),
        });
    }
    let Ok(executable) = std::env::current_exe() else {
        return set_index(IndexRun::Skipped {
            reason: "could not find the Mash executable; run `mash index`".to_owned(),
        });
    };
    let mut command = tokio::process::Command::new(executable);
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
        .arg("index")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    match command.spawn() {
        Ok(child) => {
            set_index(IndexRun::Running);
            tokio::spawn(async move { set_index(finished(child).await) });
        }
        Err(error) => set_index(IndexRun::Skipped {
            reason: format!("could not start `mash index`: {error}"),
        }),
    }
}

/// Wait for the index and read what it reported: its result on stdout, or
/// the last error line on stderr, where progress events also arrive.
async fn finished(mut child: tokio::process::Child) -> IndexRun {
    let errors = child.stderr.take().map(|stderr| {
        tokio::spawn(async move {
            let mut last = None;
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.contains("\"error\"") {
                    last = Some(line);
                }
            }
            last
        })
    });
    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut output).await;
    }
    let ok = child.wait().await.is_ok_and(|status| status.success());
    let error = match errors {
        Some(errors) => errors.await.ok().flatten(),
        None => None,
    };
    let report = serde_json::from_str::<serde_json::Value>(&output).unwrap_or_default();
    let message = |value: &serde_json::Value| {
        value["error"]["message"]
            .as_str()
            .or_else(|| value["embedding_error"].as_str())
            .map(str::to_owned)
    };
    IndexRun::Finished {
        ok,
        chats: report["indexed_chats"].as_u64(),
        messages: report["indexed_messages"].as_u64(),
        message: message(&report).or_else(|| {
            error
                .and_then(|line| serde_json::from_str::<serde_json::Value>(&line).ok())
                .and_then(|value| message(&value))
        }),
    }
}
