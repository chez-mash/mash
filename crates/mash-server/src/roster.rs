//! Which tokens the running server accepts, and keeping that current.
//!
//! The server starts with the machines in its configuration and in the
//! added-machines file beside it. While it runs it rereads that file every
//! second, so a machine added, removed or given a new credential by
//! `--add-node` or the dashboard on this machine takes effect without a
//! restart. A machine in a group the server does not serve yet still needs
//! one: a new group's search collections are set up only at startup.
//!
//! A removed machine whose history the operator asked to delete is retired
//! here, inside the server that owns the history directory. Its credential
//! was revoked first, so nothing new can arrive for it; the deletion also
//! waits until no connection that machine opened earlier is still open.
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use mash_app::shared::server::{Identity, configuration, enrollment, status};
use mash_model::AppError;

use super::App;

/// The accepted tokens, swapped whole when the file changes.
#[derive(Default)]
pub(crate) struct Roster {
    pub(crate) tokens: HashMap<blake3::Hash, Identity>,
    /// Credentials the operator removed or replaced.
    pub(crate) revoked: HashSet<blake3::Hash>,
    /// Admin keys, by hash, with their names. A machine's token is never one.
    pub(crate) admins: HashMap<blake3::Hash, String>,
}

#[derive(Clone, Default)]
pub(crate) struct Nodes {
    current: Arc<RwLock<Arc<Roster>>>,
}

impl Nodes {
    pub(crate) fn fixed(tokens: HashMap<blake3::Hash, Identity>) -> Self {
        let nodes = Self::default();
        nodes.set(Roster {
            tokens,
            revoked: HashSet::new(),
            admins: HashMap::new(),
        });
        nodes
    }

    pub(crate) fn current(&self) -> Arc<Roster> {
        self.current
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set(&self, roster: Roster) {
        *self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::new(roster);
    }

    /// Whether a machine still holds a working token. A connection opened
    /// before its credential was removed checks this before each step.
    pub(crate) fn authorized(&self, identity: &Identity) -> bool {
        self.current()
            .tokens
            .values()
            .any(|known| known.group_id == identity.group_id && known.node_id == identity.node_id)
    }
}

/// What the server needs to follow its added-machines file.
pub(crate) struct Live {
    pub(crate) configuration: PathBuf,
    listen: String,
    pub(crate) started_at: u64,
    /// The file's text as last applied.
    applied: tokio::sync::Mutex<Option<String>>,
    /// Synchronization connections open per machine.
    sessions: Mutex<HashMap<(String, String), usize>>,
}

/// Counts one open connection until dropped.
pub(crate) struct Session {
    live: Arc<Live>,
    key: (String, String),
}

impl Drop for Session {
    fn drop(&mut self) {
        let mut sessions = self
            .live
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(count) = sessions.get_mut(&self.key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                sessions.remove(&self.key);
            }
        }
    }
}

impl Live {
    pub(crate) fn new(configuration: PathBuf, listen: String) -> Arc<Self> {
        Arc::new(Self {
            configuration,
            listen,
            started_at: enrollment::now(),
            applied: tokio::sync::Mutex::new(None),
            sessions: Mutex::default(),
        })
    }

    pub(crate) fn open(self: &Arc<Self>, identity: &Identity) -> Session {
        let key = (identity.group_id.clone(), identity.node_id.clone());
        *self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(key.clone())
            .or_default() += 1;
        Session {
            live: self.clone(),
            key,
        }
    }

    fn connected(&self, group_id: &str, node_id: &str) -> bool {
        self.sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&(group_id.to_owned(), node_id.to_owned()))
    }
}

/// Reread the file and apply it if it changed, or when `force` is set.
/// A file that does not apply leaves the accepted tokens as they were.
pub(crate) async fn refresh(app: &App, live: &Live, force: bool) -> Result<(), AppError> {
    let mut applied = live.applied.lock().await;
    let path = live.configuration.clone();
    let text = mash_app::blocking(move || Ok(status::nodes_text(&path))).await?;
    if !force && applied.as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    let config = configuration::load(&live.configuration).await?;
    let mut tokens = configuration::identities(&config)?;
    let mut restart_needed = Vec::new();
    for member in configuration::members(&config) {
        if app.store.serves(&member.group_id) {
            app.store.admit(&member).await?;
        } else {
            restart_needed.push(format!("{}/{}", member.group_id, member.node_id));
            tokens.retain(|_, identity| {
                !(identity.group_id == member.group_id && identity.node_id == member.node_id)
            });
        }
    }
    let revoked = configuration::added(&config)
        .revoked
        .iter()
        .filter_map(|revoked| blake3::Hash::from_hex(&revoked.hash).ok())
        .collect();
    let admins = mash_app::shared::server::admin::hashes(configuration::added(&config))
        .into_iter()
        .collect();
    app.nodes.set(Roster {
        tokens,
        revoked,
        admins,
    });
    *applied = Some(text);
    drop(applied);
    for entry in &configuration::added(&config).nodes {
        if entry.delete_requested_at.is_some() {
            delete(app, live, &entry.group_id, &entry.node_id).await?;
        }
    }
    write_status(app, live, &config, restart_needed, None).await
}

/// Delete what a removed machine published, then forget the machine.
async fn delete(app: &App, live: &Live, group_id: &str, node_id: &str) -> Result<(), AppError> {
    if live.connected(group_id, node_id) {
        return Ok(());
    }
    let identity = configuration::member(group_id, node_id);
    super::retire_node(
        &app.store,
        &app.catalog,
        app.attachments_dir.as_deref(),
        &identity,
    )
    .await?;
    let path = live.configuration.clone();
    let (group, node) = (group_id.to_owned(), node_id.to_owned());
    mash_app::blocking(move || {
        enrollment::update(&path, |added| {
            enrollment::forget(added, &group, &node);
            Ok(())
        })
    })
    .await?;
    tracing::info!(
        group = group_id,
        node = node_id,
        "deleted a removed machine's history"
    );
    Ok(())
}

/// What each member of a served group has published and is sending now,
/// and the groups served, for the status file and the status route alike.
pub(crate) async fn node_statuses(
    app: &App,
    config: &configuration::Configuration,
) -> (Vec<status::NodeStatus>, HashSet<String>) {
    let mut nodes = Vec::new();
    let mut groups = HashSet::new();
    for member in configuration::members(config) {
        if !app.store.serves(&member.group_id) {
            continue;
        }
        groups.insert(member.group_id.clone());
        let published = app.store.published(&member).await.ok().flatten();
        let (group, node) = (member.group_id.clone(), member.node_id.clone());
        let receiving = super::sync::catalog(&app.catalog, move |catalog| {
            catalog.latest_backfill(&group, &node)
        })
        .await
        .ok()
        .flatten()
        .filter(|backfill| backfill.state != "complete" && backfill.total_chats > 0)
        .map(|backfill| status::Receiving {
            covered: backfill.covered,
            total: backfill.total_chats,
        });
        let delete_waiting = configuration::added(config).nodes.iter().any(|entry| {
            entry.group_id == member.group_id
                && entry.node_id == member.node_id
                && entry.delete_requested_at.is_some()
        });
        nodes.push(status::NodeStatus {
            group_id: member.group_id,
            node_id: member.node_id,
            published,
            receiving,
            delete_waiting,
        });
    }
    (nodes, groups)
}

async fn write_status(
    app: &App,
    live: &Live,
    config: &configuration::Configuration,
    restart_needed: Vec<String>,
    problem: Option<String>,
) -> Result<(), AppError> {
    let (nodes, groups) = node_statuses(app, config).await;
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort();
    let applied = live.applied.lock().await.clone().unwrap_or_default();
    let report = status::ServerStatus {
        listen: live.listen.clone(),
        started_at: live.started_at,
        updated_at: enrollment::now(),
        applied: status::applied(&applied),
        groups,
        restart_needed,
        nodes,
        problem,
    };
    let path = live.configuration.clone();
    mash_app::blocking(move || status::write(&path, &report)).await
}

/// Follow the file for as long as the server runs: apply changes within a
/// second, and rewrite the status on a heartbeat.
pub(crate) fn spawn_watch(app: App, live: Arc<Live>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticks = 0_u64;
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            ticks += 1;
            let heartbeat = ticks.is_multiple_of(status::HEARTBEAT_SECONDS);
            if let Err(error) = refresh(&app, &live, heartbeat).await
                && heartbeat
            {
                tracing::warn!(message = %error.error.message, "could not apply the added-machines file");
                // Still say the server runs, and why the file did not apply.
                let report = status::ServerStatus {
                    listen: live.listen.clone(),
                    started_at: live.started_at,
                    updated_at: enrollment::now(),
                    problem: Some(error.error.message),
                    ..status::read(&live.configuration).unwrap_or_default()
                };
                let path = live.configuration.clone();
                let _ = mash_app::blocking(move || status::write(&path, &report)).await;
            }
        }
    })
}
