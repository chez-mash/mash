//! `mash uninstall`: remove what Mash created on this machine while running:
//! its user services, its agent connections, and its index, models and
//! configuration. Without `apply` it reports that plan and changes nothing.
//! The executable stays, for whatever installed it to remove, and so does
//! anything in Mash's directory that these commands did not create.
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::installation::Agent;
use super::models::Paths;
use crate::AppError;

const WATCHER_TASK: &str = "mash-index-watch";

#[derive(Debug, Clone, Copy)]
pub struct Request {
    /// Carry the plan out; otherwise only report it.
    pub apply: bool,
    /// Keep the index, its search data, the configuration and saved tokens.
    pub keep_history: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "Uninstall")]
pub struct Response {
    /// False for a plan: nothing was changed.
    applied: bool,
    services: Vec<Service>,
    agents: Vec<Connection>,
    paths: Vec<Item>,
    /// What this command leaves alone, and why.
    kept: Vec<Kept>,
    binary: Binary,
    next: Vec<String>,
}

impl Response {
    /// A step failed, so something Mash created is still here: exit 1.
    pub(super) fn incomplete(&self) -> bool {
        let failed = |outcome: &Outcome| matches!(outcome, Outcome::Failed { .. });
        self.services.iter().any(|service| failed(&service.outcome))
            || self.paths.iter().any(|item| failed(&item.outcome))
            || self.agents.iter().any(|agent| {
                [&agent.mcp, &agent.plugin]
                    .into_iter()
                    .flatten()
                    .any(|step| failed(&step.outcome))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallOutcome")]
#[serde(tag = "state", rename_all = "kebab-case")]
enum Outcome {
    /// Would be removed; nothing has changed.
    Planned,
    Removed,
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallServiceKind")]
#[serde(rename_all = "kebab-case")]
enum ServiceKind {
    /// The search service `mash qdrant install` set up.
    Qdrant,
    /// The dashboard `mash dash remote` keeps running.
    Dashboard,
    /// The `mash index --watch` job `mash-fleet` installed.
    Watcher,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallService")]
struct Service {
    kind: ServiceKind,
    /// The name its service manager knows it by.
    name: String,
    outcome: Outcome,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallAgent")]
struct Connection {
    agent: Agent,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp: Option<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugin: Option<Step>,
}

/// One of the agent's own commands that disconnects Mash.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallStep")]
struct Step {
    command: &'static str,
    outcome: Outcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallContent")]
#[serde(rename_all = "kebab-case")]
enum Content {
    Configuration,
    /// The index: Mash's copy of your chat history, with its attachments.
    Index,
    Models,
    /// Qdrant's program and data; `mash index` rebuilds the data from the index.
    SearchData,
    /// Node tokens saved by `mash init --join`.
    Tokens,
    /// The dashboard service's token and log.
    Dashboard,
    /// What the agent plugin's hooks remember between sessions.
    HookState,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallPath")]
struct Item {
    content: Content,
    path: PathBuf,
    /// Size on disk, with every file that belongs to it.
    bytes: u64,
    outcome: Outcome,
    /// Everything removed with `path`: it and the files beside it that
    /// carry its name.
    #[serde(skip)]
    members: Vec<PathBuf>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallKept")]
struct Kept {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
    reason: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "UninstallBinary")]
struct Binary {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
    /// How to remove the executable once this command has finished.
    remove: String,
}

/// Where this installation keeps its files.
struct Target {
    home: PathBuf,
    config: PathBuf,
    database: PathBuf,
    models: PathBuf,
    /// The user's home directory, which nothing here may delete or contain.
    user: PathBuf,
}

impl Target {
    fn resolve(paths: &Paths) -> Result<Self, AppError> {
        let home = crate::platform::mash_home()
            .ok_or_else(|| AppError::configuration("cannot find the home directory"))?;
        let user = directories::BaseDirs::new()
            .ok_or_else(|| AppError::configuration("cannot find the home directory"))?
            .home_dir()
            .to_path_buf();
        let absolute = |path: &Path| std::path::absolute(path).map_err(AppError::io);
        Ok(Self {
            config: match &paths.config {
                Some(path) => absolute(path)?,
                None => home.join("config.toml"),
            },
            database: match &paths.db {
                Some(path) => absolute(path)?,
                None => home.join("mash.sqlite3"),
            },
            models: match &paths.models_dir {
                Some(path) => absolute(path)?,
                None => home.join("models"),
            },
            home,
            user,
        })
    }
}

pub(super) async fn run(paths: Paths, request: Request) -> Result<Response, AppError> {
    let target = Target::resolve(&paths)?;
    let config = target.config.clone();
    // A configuration that no longer loads must not block removing it.
    let shared = crate::blocking(move || Ok(crate::config::load(Some(&config)).ok()))
        .await?
        .and_then(|configuration| configuration.shared);
    let running = crate::blocking(move || Ok(crate::runtime::inspect(&paths).processes.len()))
        .await
        .unwrap_or(0);

    let qdrant = super::qdrant::Host::system().ok();
    let installed = qdrant.as_ref().and_then(super::qdrant::installed);
    let mut services = Vec::new();
    let mut search_data = Vec::new();
    if let Some((service, directories)) = installed {
        search_data = directories;
        let outcome = match (&qdrant, request.apply) {
            (Some(host), true) => {
                outcome(super::qdrant::uninstall(host, !request.keep_history).await)
            }
            _ => Outcome::Planned,
        };
        if let Some(name) = service {
            services.push(Service {
                kind: ServiceKind::Qdrant,
                name,
                outcome,
            });
        }
    }
    if let Some((layout, port)) = crate::remote::installed() {
        services.push(Service {
            kind: ServiceKind::Dashboard,
            name: layout.service().to_owned(),
            outcome: if request.apply {
                outcome(match port {
                    Some(port) => crate::remote::uninstall(&layout, port).await,
                    None => crate::remote::remove_service(&layout).await,
                })
            } else {
                Outcome::Planned
            },
        });
    }
    services.extend(watcher(request.apply).await);

    let agents = agents(request.apply).await;

    let (paths, mut kept) =
        crate::blocking(move || Ok(files(&target, &search_data, request))).await?;
    kept.push(Kept {
        path: None,
        reason: "Your agents' own chat histories: Mash only ever read them.".to_owned(),
    });
    if let Some(shared) = shared {
        kept.push(Kept {
            path: None,
            reason: format!(
                "History this machine already published to {} as node `{}`: it lives on that server.",
                shared.api_url, shared.node_id
            ),
        });
    }

    let binary = binary(std::env::current_exe().ok().as_deref());
    let mut response = Response {
        applied: request.apply,
        services,
        agents,
        paths,
        kept,
        binary,
        next: Vec::new(),
    };
    response.next = next(&response, request, running);
    Ok(response)
}

fn outcome<T>(result: Result<T, AppError>) -> Outcome {
    match result {
        Ok(_) => Outcome::Removed,
        Err(error) => Outcome::Failed {
            message: error.error.message,
        },
    }
}

/// The watcher job, where this machine's service manager can show one.
async fn watcher(apply: bool) -> Option<Service> {
    if cfg!(windows) {
        // A scheduled task has no definition file; ask the task scheduler.
        let query = format!(
            "if (Get-ScheduledTask -TaskName '{WATCHER_TASK}' -ErrorAction SilentlyContinue) {{ 'present' }}"
        );
        if !powershell(&query).await.ok()?.contains("present") {
            return None;
        }
        let remove = format!(
            "Stop-ScheduledTask -TaskName '{WATCHER_TASK}' -ErrorAction SilentlyContinue; Unregister-ScheduledTask -TaskName '{WATCHER_TASK}' -Confirm:$false -ErrorAction Stop"
        );
        return Some(Service {
            kind: ServiceKind::Watcher,
            name: WATCHER_TASK.to_owned(),
            outcome: if apply {
                outcome(powershell(&remove).await)
            } else {
                Outcome::Planned
            },
        });
    }
    let layout = crate::remote::watcher_layout(crate::watch::installed_definition()?)?;
    Some(Service {
        kind: ServiceKind::Watcher,
        name: layout.service().to_owned(),
        outcome: if apply {
            outcome(crate::remote::remove_service(&layout).await)
        } else {
            Outcome::Planned
        },
    })
}

async fn powershell(script: &str) -> Result<String, AppError> {
    crate::remote::command(
        "powershell.exe",
        &[
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ],
    )
    .await
}

/// Each agent whose user settings still name Mash, disconnected through the
/// agent's own CLI: the `undo` commands `mash init` lists.
async fn agents(apply: bool) -> Vec<Connection> {
    let mut connections = Vec::new();
    for agent in [Agent::ClaudeCode, Agent::Codex] {
        let (mcp, plugin) = crate::blocking(move || Ok(super::installation::connection(agent)))
            .await
            .unwrap_or((false, false));
        if !mcp && !plugin {
            continue;
        }
        let mut connection = Connection {
            agent,
            mcp: None,
            plugin: None,
        };
        if mcp {
            connection.mcp = Some(step(agent, agent.remove_mcp(), apply).await);
        }
        if plugin {
            connection.plugin = Some(step(agent, agent.remove_plugin(), apply).await);
        }
        connections.push(connection);
    }
    connections
}

async fn step(agent: Agent, command: &'static str, apply: bool) -> Step {
    let outcome = if apply {
        match super::setup::which(agent.program()) {
            Some(program) => {
                let arguments: Vec<&str> = command.split_whitespace().skip(1).collect();
                match super::setup::run(&program, &arguments).await {
                    Ok(()) => Outcome::Removed,
                    Err(message) => Outcome::Failed { message },
                }
            }
            None => Outcome::Failed {
                message: format!(
                    "`{}` was not found on PATH, so its settings still name Mash; run the command where it is installed",
                    agent.program()
                ),
            },
        }
    } else {
        Outcome::Planned
    };
    Step { command, outcome }
}

/// The files and directories to remove, removed when `request.apply`, and
/// what stays in Mash's directory.
fn files(target: &Target, search_data: &[PathBuf], request: Request) -> (Vec<Item>, Vec<Kept>) {
    let home = &target.home;
    let mut history = vec![
        (Content::Configuration, configuration(&target.config)),
        (Content::Index, family(&target.database)),
        (Content::Tokens, vec![home.join("node.env")]),
    ];
    let mut items = vec![
        (Content::Models, vec![target.models.clone()]),
        (
            Content::Dashboard,
            vec![home.join("dashboard.env"), home.join("dashboard.log")],
        ),
        (Content::HookState, vec![home.join("recall")]),
    ];
    let mut kept = Vec::new();
    let keep = |kept: &mut Vec<Kept>, members: &[PathBuf]| {
        if let Some(path) = members.iter().find(|path| path.exists()) {
            kept.push(Kept {
                path: Some(path.clone()),
                reason: "Kept by --keep-history.".to_owned(),
            });
        }
    };
    if request.keep_history {
        for (_, members) in &history {
            keep(&mut kept, members);
        }
        // `qdrant uninstall` without storage deletion leaves only the data.
        keep(&mut kept, search_data);
    } else {
        items.append(&mut history);
        items.extend(
            search_data
                .iter()
                .map(|directory| (Content::SearchData, vec![directory.clone()])),
        );
    }

    let mut removed = Vec::new();
    let mut listed = Vec::new();
    for (content, members) in items {
        let members: Vec<PathBuf> = members.into_iter().filter(|path| present(path)).collect();
        let Some(path) = members.first().cloned() else {
            continue;
        };
        let mut item = Item {
            content,
            path,
            bytes: members.iter().map(|path| size(path)).sum(),
            outcome: Outcome::Planned,
            members,
        };
        if request.apply {
            item.outcome = match item
                .members
                .iter()
                .try_for_each(|path| remove(path, &target.user))
            {
                Ok(()) => Outcome::Removed,
                Err(message) => Outcome::Failed { message },
            };
        }
        removed.extend(item.members.iter().cloned());
        listed.push(item);
    }

    // Whatever else is in Mash's directory was not created by these commands
    // (a shared server's data, a backup made by hand), so it stays.
    let mut others: Vec<PathBuf> = std::fs::read_dir(home)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            !removed.contains(path) && !kept.iter().any(|k| k.path.as_ref() == Some(path))
        })
        .filter(|path| !request.keep_history || !related(path, target, search_data))
        .collect();
    others.sort();
    if others.is_empty() && request.apply && !request.keep_history {
        // Only an empty directory goes; a failed step above leaves it.
        let _ = std::fs::remove_dir(home);
    }
    kept.extend(others.into_iter().map(|path| Kept {
        path: Some(path),
        reason: "Not created by Mash's local commands, so it is left in place.".to_owned(),
    }));
    (listed, kept)
}

/// Whether `path` belongs to something `--keep-history` keeps.
fn related(path: &Path, target: &Target, search_data: &[PathBuf]) -> bool {
    configuration(&target.config).contains(&path.to_path_buf())
        || family(&target.database).contains(&path.to_path_buf())
        || search_data.iter().any(|directory| directory == path)
        || path == target.home.join("node.env")
}

/// The configuration and the copy `mash init --force` keeps of the one it
/// replaced. A backup made by hand beside them is not Mash's to delete.
fn configuration(path: &Path) -> Vec<PathBuf> {
    let mut previous = path.as_os_str().to_owned();
    previous.push(".previous");
    vec![path.to_path_buf(), previous.into()]
}

/// A database and everything beside it that carries its name: its
/// write-ahead log, attachments, locks and runtime records.
fn family(path: &Path) -> Vec<PathBuf> {
    let mut members = vec![path.to_path_buf()];
    let (Some(parent), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
    else {
        return members;
    };
    let mut beside: Vec<PathBuf> = std::fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| {
            entry.file_name().to_str().is_some_and(|other| {
                other
                    .strip_prefix(name)
                    .is_some_and(|rest| rest.starts_with(['.', '-']))
            })
        })
        .map(|entry| entry.path())
        .collect();
    beside.sort();
    members.append(&mut beside);
    members
}

fn present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Bytes under `path`, without following links.
fn size(path: &Path) -> u64 {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| size(&entry.path()))
        .sum()
}

/// Delete one file or directory tree. A path that is, or holds, the user's
/// home directory is refused whatever a flag named.
fn remove(path: &Path, user: &Path) -> Result<(), String> {
    if !path.is_absolute() || user.starts_with(path) {
        return Err(format!(
            "refusing to delete {}: it is not a location Mash owns",
            path.display()
        ));
    }
    let result = match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot delete {}: {error}", path.display())),
    }
}

/// How to remove the executable, judged from where it is installed.
fn binary(executable: Option<&Path>) -> Binary {
    let Some(path) = executable else {
        return Binary {
            path: None,
            remove: "Remove the mash executable with whatever installed it.".to_owned(),
        };
    };
    let text = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let remove = if ["/cellar/", "/homebrew/", "/linuxbrew/"]
        .iter()
        .any(|part| text.contains(part))
    {
        "brew uninstall mash".to_owned()
    } else if text.contains("/scoop/") {
        "scoop uninstall mash".to_owned()
    } else if text.contains("/.cargo/bin/") {
        "cargo uninstall mash".to_owned()
    } else if text == "/usr/bin/mash" {
        "sudo apt remove mash-cli".to_owned()
    } else {
        // The install script puts `mash-server` beside `mash`.
        let server = path.with_file_name(format!("mash-server{}", std::env::consts::EXE_SUFFIX));
        if server.is_file() {
            format!("Delete {} and {}", path.display(), server.display())
        } else {
            format!("Delete {}", path.display())
        }
    };
    Binary {
        path: Some(path.to_path_buf()),
        remove,
    }
}

fn next(response: &Response, request: Request, running: usize) -> Vec<String> {
    let mut next = Vec::new();
    if !request.apply {
        next.push(
            "Nothing was changed. Run `mash uninstall --yes` to remove what is listed.".to_owned(),
        );
        if !request.keep_history
            && response
                .paths
                .iter()
                .any(|item| item.content == Content::Index)
        {
            next.push("To keep a copy of your indexed history first, run `mash export DIRECTORY`; to keep it in place, add --keep-history.".to_owned());
        }
        return next;
    }
    if response.incomplete() {
        next.push("Some steps failed and are marked `failed`; fix what each message names, then run `mash uninstall --yes` again.".to_owned());
    }
    if !response.agents.is_empty() {
        next.push("Restart your agents so they stop loading Mash.".to_owned());
    }
    if running > 0 {
        next.push(format!(
            "{running} Mash process(es) started before this ran, such as an agent's MCP server or a watcher in a terminal; each ends when the agent or terminal that started it closes."
        ));
    }
    next.push(format!(
        "Mash's executable is still installed. To remove it: {}",
        response.binary.remove
    ));
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(root: &Path) -> Target {
        let home = root.join("user/.mash");
        Target {
            config: home.join("config.toml"),
            database: home.join("mash.sqlite3"),
            models: home.join("models"),
            home,
            user: root.join("user"),
        }
    }

    fn populate(target: &Target) {
        let home = &target.home;
        std::fs::create_dir_all(home.join("models/embedding")).unwrap();
        std::fs::create_dir_all(home.join("mash.sqlite3.attachments")).unwrap();
        std::fs::create_dir_all(home.join("shared-silver")).unwrap();
        std::fs::create_dir_all(home.join("recall")).unwrap();
        for (name, bytes) in [
            ("config.toml", 10),
            ("config.toml.previous", 5),
            ("config.toml.bak", 4),
            ("mash.sqlite3", 100),
            ("mash.sqlite3-wal", 20),
            ("mash.sqlite3.attachments/a", 3),
            ("models/embedding/model.onnx", 40),
            ("node.env", 7),
            ("dashboard.env", 2),
            ("notes.txt", 1),
        ] {
            std::fs::write(home.join(name), vec![0; bytes]).unwrap();
        }
    }

    const PLAN: Request = Request {
        apply: false,
        keep_history: false,
    };

    #[test]
    fn a_plan_lists_sizes_and_changes_nothing() {
        let root = tempfile::tempdir().unwrap();
        let target = target(root.path());
        populate(&target);
        let (items, kept) = files(&target, &[], PLAN);
        let bytes = |content| {
            items
                .iter()
                .find(|item| item.content == content)
                .map(|item| item.bytes)
        };
        assert_eq!(bytes(Content::Index), Some(123));
        assert_eq!(bytes(Content::Configuration), Some(15));
        assert_eq!(bytes(Content::Models), Some(40));
        assert!(items.iter().all(|item| item.outcome == Outcome::Planned));
        // What Mash's local commands did not create is named and left.
        let left: Vec<_> = kept.iter().filter_map(|kept| kept.path.clone()).collect();
        assert_eq!(
            left,
            [
                target.home.join("config.toml.bak"),
                target.home.join("notes.txt"),
                target.home.join("shared-silver")
            ]
        );
        assert!(target.database.exists() && target.models.exists());
    }

    #[test]
    fn applying_removes_what_mash_created_and_leaves_the_rest() {
        let root = tempfile::tempdir().unwrap();
        let target = target(root.path());
        populate(&target);
        let apply = Request {
            apply: true,
            keep_history: false,
        };
        let (items, _) = files(&target, &[], apply);
        assert!(items.iter().all(|item| item.outcome == Outcome::Removed));
        let mut left: Vec<_> = std::fs::read_dir(&target.home)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        left.sort();
        assert_eq!(left, ["config.toml.bak", "notes.txt", "shared-silver"]);

        // With nothing foreign in it, the directory itself goes too.
        populate(&target);
        std::fs::remove_file(target.home.join("config.toml.bak")).unwrap();
        std::fs::remove_file(target.home.join("notes.txt")).unwrap();
        std::fs::remove_dir(target.home.join("shared-silver")).unwrap();
        files(&target, &[], apply);
        assert!(!target.home.exists());
        assert!(target.user.exists());
    }

    #[test]
    fn keep_history_keeps_the_index_configuration_tokens_and_search_data() {
        let root = tempfile::tempdir().unwrap();
        let target = target(root.path());
        populate(&target);
        let storage = target.home.join("qdrant");
        std::fs::create_dir_all(storage.join("storage")).unwrap();
        let (items, kept) = files(
            &target,
            std::slice::from_ref(&storage),
            Request {
                apply: true,
                keep_history: true,
            },
        );
        let removed: Vec<_> = items.iter().map(|item| item.content).collect();
        assert_eq!(
            removed,
            [Content::Models, Content::Dashboard, Content::HookState]
        );
        for name in [
            "config.toml",
            "config.toml.previous",
            "mash.sqlite3",
            "mash.sqlite3-wal",
            "mash.sqlite3.attachments/a",
            "node.env",
            "qdrant/storage",
        ] {
            assert!(target.home.join(name).exists(), "{name}");
        }
        assert!(!target.models.exists());
        let reasons = kept
            .iter()
            .filter(|kept| kept.reason.contains("--keep-history"))
            .count();
        assert_eq!(reasons, 4, "{kept:?}");
    }

    #[test]
    fn a_path_holding_the_home_directory_is_never_deleted() {
        let root = tempfile::tempdir().unwrap();
        let mut target = target(root.path());
        populate(&target);
        // `--models-dir` naming the home directory, or anything above it.
        for models in [target.user.clone(), root.path().to_path_buf()] {
            target.models = models;
            let (items, _) = files(
                &target,
                &[],
                Request {
                    apply: true,
                    keep_history: true,
                },
            );
            let models = items
                .iter()
                .find(|item| item.content == Content::Models)
                .unwrap();
            assert!(
                matches!(&models.outcome, Outcome::Failed { message } if message.contains("refusing")),
                "{models:?}"
            );
        }
        assert!(target.home.join("config.toml").exists());
        assert!(remove(Path::new("relative"), &target.user).is_err());
    }

    #[test]
    fn the_binary_is_removed_by_whatever_installed_it() {
        for (path, expected) in [
            (
                "/opt/homebrew/Cellar/mash/0.1.0/bin/mash",
                "brew uninstall mash",
            ),
            ("/home/linuxbrew/.linuxbrew/bin/mash", "brew uninstall mash"),
            (
                "C:\\Users\\me\\scoop\\apps\\mash\\current\\mash.exe",
                "scoop uninstall mash",
            ),
            ("/home/me/.cargo/bin/mash", "cargo uninstall mash"),
            ("/usr/bin/mash", "sudo apt remove mash-cli"),
            (
                "/home/me/.local/bin/mash",
                "Delete /home/me/.local/bin/mash",
            ),
        ] {
            assert_eq!(binary(Some(Path::new(path))).remove, expected, "{path}");
        }
        let root = tempfile::tempdir().unwrap();
        let mash = root.path().join("mash");
        let server = root
            .path()
            .join(format!("mash-server{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&server, "").unwrap();
        assert_eq!(
            binary(Some(&mash)).remove,
            format!("Delete {} and {}", mash.display(), server.display())
        );
    }

    #[test]
    fn a_failed_step_makes_the_run_incomplete() {
        let mut response = Response {
            applied: true,
            services: vec![],
            agents: vec![Connection {
                agent: Agent::Codex,
                mcp: Some(Step {
                    command: Agent::Codex.remove_mcp(),
                    outcome: Outcome::Removed,
                }),
                plugin: None,
            }],
            paths: vec![],
            kept: vec![],
            binary: binary(None),
            next: vec![],
        };
        assert!(!response.incomplete());
        response.agents[0].plugin = Some(Step {
            command: Agent::Codex.remove_plugin(),
            outcome: Outcome::Failed {
                message: "offline".into(),
            },
        });
        assert!(response.incomplete());
        let apply = Request {
            apply: true,
            keep_history: false,
        };
        let steps = next(&response, apply, 2);
        assert!(steps[0].contains("failed"), "{steps:?}");
        assert!(steps.iter().any(|step| step.contains("2 Mash process")));
        assert!(steps.last().unwrap().contains("still installed"));
    }
}
