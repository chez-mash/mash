//! One read-only examination that names what is wrong on this machine and
//! the command that repairs it. `status` reports state; this explains it.
//! Nothing here writes, migrates, downloads or starts anything.
use super::inspection;
use super::models::{ArtifactState, StatusResponse};
use crate::config::{IndexDestination, ResolvedConfig};
use crate::provider::Provider;
use crate::runtime;
use crate::silver::certificate::Completeness;
use crate::storage::{SourceCount, Storage};
use crate::views::qdrant::Reachability;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// Below this, Claude Code's own cleanup removes transcripts Mash would
/// otherwise still hold a year on.
const RETENTION_DAYS: u64 = 365;
/// Claude Code's retention when its settings name none.
const DEFAULT_RETENTION_DAYS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Check {
    Configuration,
    Index,
    History,
    SearchService,
    Search,
    Models,
    Watcher,
    Processes,
    SharedCopy,
    Integrations,
    Retention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Ok,
    /// Works, with something worth knowing or doing.
    Warning,
    /// Something Mash is for does not work until this is repaired.
    Problem,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Finding {
    pub(super) check: Check,
    pub(super) state: State,
    pub(super) summary: String,
    /// Why, when the summary alone does not say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) cause: Option<String>,
    /// A command to run, or an instruction when no single command does it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) repair: Option<String>,
}

impl Finding {
    fn ok(check: Check, summary: impl Into<String>) -> Self {
        Self {
            check,
            state: State::Ok,
            summary: summary.into(),
            cause: None,
            repair: None,
        }
    }

    fn found(check: Check, state: State, summary: impl Into<String>) -> Self {
        Self {
            state,
            ..Self::ok(check, summary)
        }
    }

    fn cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    fn repair(mut self, repair: impl Into<String>) -> Self {
        self.repair = Some(repair.into());
        self
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Response {
    /// No problem was found. Warnings do not make a machine unhealthy.
    healthy: bool,
    problems: usize,
    warnings: usize,
    findings: Vec<Finding>,
}

impl Response {
    fn new(findings: Vec<Finding>) -> Self {
        let count = |state| findings.iter().filter(|f| f.state == state).count();
        let problems = count(State::Problem);
        Self {
            healthy: problems == 0,
            problems,
            warnings: count(State::Warning),
            findings,
        }
    }

    pub fn healthy(&self) -> bool {
        self.healthy
    }

    /// The report as lines a person reads, problems first.
    pub(super) fn text(&self) -> Result<String, serde_json::Error> {
        let mut out = if self.healthy && self.warnings == 0 {
            "Mash doctor: nothing needs attention.\n".to_owned()
        } else {
            format!(
                "Mash doctor: {} problems, {} warnings.\n",
                self.problems, self.warnings
            )
        };
        for state in [State::Problem, State::Warning, State::Ok] {
            for finding in self.findings.iter().filter(|f| f.state == state) {
                let label = serde_json::to_value(finding.state)?;
                let check = serde_json::to_value(finding.check)?;
                let _ = writeln!(
                    out,
                    "[{}] {}: {}",
                    label.as_str().unwrap_or_default(),
                    check.as_str().unwrap_or_default(),
                    finding.summary
                );
                if let Some(cause) = &finding.cause {
                    let _ = writeln!(out, "  Why: {cause}");
                }
                if let Some(repair) = &finding.repair {
                    let _ = writeln!(out, "  Repair: {repair}");
                }
            }
        }
        Ok(out)
    }
}

pub(super) async fn examine(
    configuration: &ResolvedConfig,
    database: &Path,
    models: &Path,
) -> Response {
    let mut findings = vec![configured(configuration)];
    let shared = shared_target(configuration).is_some_and(|shared| shared.token().is_some());
    match inspection::status(configuration, database, models, shared, None).await {
        Ok(status) => {
            let sources = source_states(configuration, database, &status).await;
            findings.push(index(database, &status));
            findings.extend(history(configuration, &status, &sources));
            findings.push(search_service(&status));
            findings.push(search(&status));
            findings.extend(models_installed(configuration, &status));
            findings.push(watcher(database, &status));
            findings.push(shared_copy(configuration, &status));
            findings.push(integrations(&status));
        }
        Err(error) => {
            // The index could not be read at all; say why and stop there.
            let mut finding = Finding::found(
                Check::Index,
                State::Problem,
                "The index cannot be read by this release.",
            )
            .cause(error.error.message.clone());
            if let Some(action) = error.error.recommended_action {
                finding = finding.repair(command(action));
            }
            findings.push(finding);
        }
    }
    let paths = super::Paths {
        db: Some(database.to_path_buf()),
        config: Some(configuration.path.clone()),
        models_dir: Some(models.to_path_buf()),
    };
    findings.push(
        crate::blocking(move || Ok(processes(&runtime::inspect(&paths))))
            .await
            .unwrap_or_else(|_| {
                Finding::ok(Check::Processes, "Running processes were not inspected.")
            }),
    );
    if configuration.providers.contains_key(&Provider::ClaudeCode) {
        findings.extend(
            crate::blocking(|| Ok(retention(claude_settings().as_deref())))
                .await
                .ok()
                .flatten(),
        );
    }
    Response::new(findings)
}

const fn command(action: crate::Action) -> &'static str {
    match action {
        crate::Action::Init => "mash init",
        crate::Action::Index => "mash index",
        crate::Action::ModelsInstall => "mash models install",
        crate::Action::StartQdrant => "mash qdrant install",
    }
}

fn shared_target(configuration: &ResolvedConfig) -> Option<&crate::config::SharedConfig> {
    (configuration.destination == IndexDestination::Both)
        .then_some(configuration.shared.as_ref())
        .flatten()
}

fn configured(configuration: &ResolvedConfig) -> Finding {
    let path = configuration.path.display();
    if configuration.loaded {
        Finding::ok(
            Check::Configuration,
            format!("Configuration loaded from {path}."),
        )
    } else {
        Finding::found(
            Check::Configuration,
            State::Warning,
            format!("No configuration file at {path}; built-in defaults are in use."),
        )
        .repair("mash init")
    }
}

fn index(database: &Path, status: &StatusResponse) -> Finding {
    if database.is_file() {
        return Finding::ok(
            Check::Index,
            format!(
                "{} chats and {} messages are indexed in {}.",
                status.chats,
                status.messages,
                database.display()
            ),
        );
    }
    // A release that moved Mash's data finds nothing where it now looks.
    let earlier = crate::platform::earlier_homes()
        .into_iter()
        .find(|home| home.join("mash.sqlite3").is_file());
    let home = database.parent().unwrap_or(database).display();
    match earlier {
        Some(earlier) => Finding::found(
            Check::Index,
            State::Problem,
            format!("No index at {}, but an earlier release left one in {}.", database.display(), earlier.display()),
        )
        .cause(format!("This release keeps its data in {home} and does not read the earlier location."))
        .repair(format!(
            "Copy `mash.sqlite3`, `mash.sqlite3.attachments`, `config.toml` and `models` from {} into {home}, then run `mash index`.",
            earlier.display()
        )),
        None => Finding::found(Check::Index, State::Warning, "Nothing is indexed yet.")
            .repair(status.recommended_action.map_or("mash index", command)),
    }
}

/// Which sources stand between silver and a complete certificate.
#[derive(Default)]
struct SourceStates {
    awaiting: Vec<SourceCount>,
    rejected: Vec<SourceCount>,
}

async fn source_states(
    configuration: &ResolvedConfig,
    database: &Path,
    status: &StatusResponse,
) -> SourceStates {
    if status
        .silver
        .as_ref()
        .is_none_or(crate::silver::certificate::Certificate::is_complete)
    {
        return SourceStates::default();
    }
    let database = database.to_path_buf();
    let since_days = configuration.since_days;
    crate::blocking(move || {
        let cutoff =
            mash_silver::discovery::cutoff_modified_ns(std::time::SystemTime::now(), since_days)?;
        let storage = Storage::open_silver(&database)?;
        Ok(SourceStates {
            awaiting: storage.sources_awaiting_reading(cutoff)?,
            rejected: storage.rejected_sources()?,
        })
    })
    .await
    .unwrap_or_default()
}

fn listed(counts: &[SourceCount]) -> String {
    let mut text = String::new();
    for count in counts {
        if !text.is_empty() {
            text.push_str(", ");
        }
        let _ = write!(text, "{} {}", count.provider, count.sources);
    }
    text
}

fn history(
    configuration: &ResolvedConfig,
    status: &StatusResponse,
    sources: &SourceStates,
) -> Option<Finding> {
    let certificate = status.silver.as_ref()?;
    Some(match certificate.completeness {
        Completeness::Complete => Finding::ok(
            Check::History,
            "Every source in the history range was read by this release.",
        ),
        Completeness::Pending => Finding::found(
            Check::History,
            State::Warning,
            "An indexing pass is running, or one was interrupted.",
        )
        .repair("mash index"),
        Completeness::Incomplete => Finding::found(
            Check::History,
            State::Warning,
            format!(
                "{} sources could not be read ({}); everything else is indexed.",
                certificate.rejected_sources,
                listed(&sources.rejected)
            ),
        )
        .cause("A source is rejected when its format is one this release does not read, or its file is damaged.")
        .repair(match sources.rejected.first() {
            Some(first) => format!(
                "mash index --provider {} (its progress names each rejected source and why)",
                first.provider
            ),
            None => "mash index".to_owned(),
        }),
        Completeness::Unverified => unread(configuration, sources),
    })
}

/// Sources an earlier reader wrote and nothing has read since. Search is not
/// served from history this release cannot vouch for, so this stops search.
fn unread(configuration: &ResolvedConfig, sources: &SourceStates) -> Finding {
    let waiting: u64 = sources.awaiting.iter().map(|count| count.sources).sum();
    if waiting == 0 {
        return Finding::found(
            Check::History,
            State::Problem,
            "The index was last certified by an earlier release.",
        )
        .cause("A provider's scan has not been repeated since the upgrade.")
        .repair("mash index");
    }
    let stranded: Vec<&SourceCount> = sources
        .awaiting
        .iter()
        .filter(|count| count.outside_range > 0)
        .collect();
    let finding = Finding::found(
        Check::History,
        State::Problem,
        format!(
            "{waiting} sources were last read by an earlier reader and not since ({}).",
            listed(&sources.awaiting)
        ),
    );
    if stranded.is_empty() {
        return finding
            .cause("An upgrade or `mash index --reread` marked them, and the pass that reads them has not finished.")
            .repair("mash index");
    }
    let outside: u64 = stranded.iter().map(|count| count.outside_range).sum();
    let range = configuration.since_days.map_or_else(
        || "the history range".to_owned(),
        |days| format!("the {days}-day history range"),
    );
    let commands = stranded
        .iter()
        .map(|count| format!("mash index --provider {} --all-history", count.provider))
        .collect::<Vec<_>>()
        .join(" && ");
    finding
        .cause(format!(
            "{outside} of them are older than {range}, so an ordinary pass never reaches them."
        ))
        .repair(commands)
}

fn search_service(status: &StatusResponse) -> Finding {
    let service = &status.local_qdrant;
    let url = &service.url;
    let managed = service.installed.is_some();
    match service.state {
        Reachability::Reachable => {
            Finding::ok(Check::SearchService, format!("Qdrant answers at {url}."))
        }
        Reachability::Unreachable => {
            let finding = Finding::found(
                Check::SearchService,
                State::Problem,
                format!("Qdrant does not answer at {url}; indexing and search need it."),
            )
            .repair("mash qdrant install");
            match &service.message {
                Some(message) => finding.cause(message.clone()),
                None => finding,
            }
        }
        Reachability::Rejected => {
            let finding = Finding::found(
                Check::SearchService,
                State::Problem,
                format!("The Qdrant at {url} cannot serve this release."),
            )
            .repair(if managed {
                "mash qdrant install (upgrades the Qdrant Mash installed and keeps its data)"
            } else {
                "Upgrade that Qdrant to 1.20 or later on its existing storage. Mash did not install it, so `mash qdrant install` will not replace it."
            });
            match &service.message {
                Some(message) => finding.cause(message.clone()),
                None => finding,
            }
        }
    }
}

fn search(status: &StatusResponse) -> Finding {
    if status.ready {
        return Finding::ok(
            Check::Search,
            format!(
                "Search is ready over {} messages of dialogue.",
                status.searchable_messages
            ),
        );
    }
    let problem = |summary: &str| Finding::found(Check::Search, State::Problem, summary);
    if status.local_qdrant.state != Reachability::Reachable {
        return problem("Search cannot be served.")
            .cause("Qdrant must answer first; see search-service.");
    }
    if let Some(error) = &status.local_views_error {
        return problem("Search cannot be served.").cause(error.clone());
    }
    match status
        .silver
        .as_ref()
        .map(|certificate| certificate.completeness)
    {
        None => Finding::found(
            Check::Search,
            State::Warning,
            "There is nothing to search yet.",
        )
        .repair(status.recommended_action.map_or("mash index", command)),
        Some(Completeness::Unverified) => problem("Search is refused.").cause(
            "Mash never serves search from history it cannot certify; repair history first.",
        ),
        Some(_)
            if status.local_artifacts.lexical != ArtifactState::Ready
                || status.local_artifacts.dense != ArtifactState::Ready =>
        {
            let watching = status
                .watcher
                .as_ref()
                .is_some_and(|watcher| watcher.running == Some(true));
            if watching {
                Finding::found(
                    Check::Search,
                    State::Warning,
                    "Search is behind the stored history.",
                )
                .cause("New messages were stored and the running watcher has not refreshed search yet; that takes about a minute.")
            } else {
                problem("Search is behind the stored history and nothing is refreshing it.")
                    .repair(status.recommended_action.map_or("mash index", command))
            }
        }
        Some(_) => problem("Search is not ready.")
            .repair(status.recommended_action.map_or("mash index", command)),
    }
}

fn models_installed(configuration: &ResolvedConfig, status: &StatusResponse) -> Option<Finding> {
    if !configuration.views.build.dense_enabled() {
        return None;
    }
    Some(if status.models_installed {
        Finding::ok(Check::Models, "Semantic search models are installed.")
    } else {
        Finding::found(
            Check::Models,
            State::Warning,
            "Semantic search models are not installed; search is by words only.",
        )
        .repair("mash models install")
    })
}

const fn restart_watcher() -> &'static str {
    if cfg!(target_os = "macos") {
        "launchctl kickstart -k gui/$(id -u)/dev.mash.index-watch"
    } else if cfg!(target_os = "linux") {
        "systemctl --user restart mash-index-watch.service"
    } else {
        "Restart the Mash index watcher's scheduled task."
    }
}

fn watcher(database: &Path, status: &StatusResponse) -> Finding {
    // The service watches the default index; another index has no watcher
    // of its own to be missing.
    if status.watcher.is_some() && database != super::default_database_path() {
        return Finding::ok(
            Check::Watcher,
            "The installed watcher service serves the default index, not this one.",
        );
    }
    match status.watcher.as_ref().map(|watcher| watcher.running) {
        None => Finding::ok(
            Check::Watcher,
            "No watcher service is installed; new chats are indexed when `mash index` runs.",
        ),
        Some(Some(false)) => Finding::found(
            Check::Watcher,
            State::Problem,
            "The watcher service is installed but not running, so new chats are not being indexed.",
        )
        .repair(restart_watcher()),
        Some(Some(true)) => Finding::ok(Check::Watcher, "The watcher service is running."),
        Some(None) => Finding::ok(
            Check::Watcher,
            "A watcher service is installed; whether it is running could not be checked.",
        ),
    }
}

/// Long-lived Mash processes left running on a build that has been replaced.
fn processes(inventory: &runtime::Inventory) -> Finding {
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

fn shared_copy(configuration: &ResolvedConfig, status: &StatusResponse) -> Finding {
    let Some(shared) = shared_target(configuration) else {
        return Finding::ok(
            Check::SharedCopy,
            "This machine does not publish to a shared index.",
        );
    };
    if shared.token().is_none() {
        // The machine may be publishing fine; this process cannot tell.
        return Finding::found(
            Check::SharedCopy,
            State::Warning,
            "The shared copy was not checked: its token is not available to this process.",
        )
        .cause(format!(
            "`{}` is not set here. A watcher started by its launcher has it; a plain shell may not.",
            shared.token_env
        ))
        .repair(format!("Export `{}` in this environment, then run `mash doctor` again.", shared.token_env));
    }
    if let Some(error) = &status.shared_error {
        return Finding::found(
            Check::SharedCopy,
            State::Problem,
            format!("The shared index at {} did not answer.", shared.api_url),
        )
        .cause(error.clone());
    }
    let published = status.shared_index.as_ref().and_then(|index| {
        index
            .nodes
            .iter()
            .find(|node| node.node_id == shared.node_id)
    });
    let Some(published) = published else {
        return Finding::found(
            Check::SharedCopy,
            State::Warning,
            "This machine has published nothing to the shared index yet.",
        )
        .repair("mash index");
    };
    let current = published
        .source_silver
        .as_ref()
        .map(|silver| silver.revision)
        == status.silver.as_ref().map(|silver| silver.revision);
    if current {
        return Finding::ok(
            Check::SharedCopy,
            format!("The shared copy is current ({} chats).", published.chats),
        );
    }
    let finding = Finding::found(
        Check::SharedCopy,
        State::Warning,
        "The shared copy is behind this machine's index.",
    )
    .repair("mash index (publishes now and reports why if the server refuses)");
    match status.shared.as_ref().and_then(|shared| shared.error.as_deref()) {
        Some(error) => finding.cause(error.to_owned()),
        None => finding.cause("A publication is pending. The watcher publishes after each pass, and writes a `shared-publication-failed` line to its log when one is refused."),
    }
}

fn integrations(status: &StatusResponse) -> Finding {
    let (attention, next) = status.installation.attention();
    if attention == 0 {
        Finding::ok(Check::Integrations, "Agent integrations need no attention.")
    } else {
        let finding = Finding::found(
            Check::Integrations,
            State::Warning,
            format!("{attention} findings about agent integrations need attention."),
        )
        .repair("mash status --installation");
        match next {
            Some(next) => finding.cause(next.to_owned()),
            None => finding,
        }
    }
}

/// The part of Claude Code's settings that decides how long its transcripts,
/// which are Mash's source for that agent, are kept.
#[derive(Deserialize)]
struct ClaudeSettings {
    #[serde(rename = "cleanupPeriodDays")]
    cleanup_period_days: Option<u64>,
}

fn claude_settings() -> Option<PathBuf> {
    let directory = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| directories::BaseDirs::new().map(|base| base.home_dir().join(".claude")))?;
    directory.is_dir().then(|| directory.join("settings.json"))
}

/// Claude Code deletes old transcripts itself; Mash then drops those chats,
/// because a source that is gone is treated as deleted history.
fn retention(settings: Option<&Path>) -> Option<Finding> {
    let path = settings?;
    let days = std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ClaudeSettings>(&bytes).ok())
        .and_then(|settings| settings.cleanup_period_days)
        .unwrap_or(DEFAULT_RETENTION_DAYS);
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
        .cause("Sessions not started from the desktop app are removed by Claude Code's own cleanup, and Mash then removes those chats from the index and the shared copy.")
        .repair(format!(
            "Set \"cleanupPeriodDays\": 3650 in {}.",
            path.display()
        ))
    })
}
