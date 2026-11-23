//! `mash qdrant prune`: find Mash's views groups in the configured Qdrant that
//! no index uses any more, and remove them only when asked.
//!
//! A group's two collections are named by a hash, so ownership is decided
//! only from evidence, never guessed:
//! - the indexes this command is given (this machine's, plus `--db`) are in
//!   use, and so is every group whose owner marker names a database that
//!   exists or a shared server's group;
//! - a group whose marker names a database that is gone, in a folder that
//!   still exists, is a leftover and may be removed;
//! - a marker naming a database in a folder that cannot be read (an
//!   unmounted disk, say) and every group without a marker are kept, unless
//!   the person names the group explicitly.
//!
//! Any failure to list, count or read a marker stops the command before it
//! deletes anything. Deleting re-reads the marker first, removes the
//! documents and then the state collection, so an interrupted run leaves the
//! marker behind and a later run finishes the job.
use crate::AppError;
use crate::config::ResolvedConfig;
use crate::views::qdrant::{api, ownership};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const PREFIX: &str = "mash_views_";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Request {
    /// Remove what the listing calls leftovers (and the named unknown groups).
    pub apply: bool,
    /// Further indexes whose views are in use, besides this machine's.
    pub databases: Vec<PathBuf>,
    /// Groups without an owner marker to remove anyway, by their hash or a
    /// collection name. Only exact names count.
    pub unknown: Vec<String>,
    /// Tests share one Qdrant, so each removes only its own groups.
    #[cfg(test)]
    pub only: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// An index this command was given uses it, or its marker names an existing database.
    InUse,
    /// A shared server keeps it. This command never removes these.
    SharedServer,
    /// Its marker names a database that no longer exists. Removed with `--yes`.
    Leftover,
    /// Its marker names a database in a folder that cannot be read. Kept.
    OwnerUnreachable,
    /// No marker; older Mash wrote it. Kept unless named with `--include-unknown`.
    Unknown,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Group {
    /// The hash both collection names share.
    pub group: String,
    pub collections: Vec<String>,
    /// Points across its collections, as Qdrant counts them.
    pub points: u64,
    pub class: Class,
    pub owner: Option<ownership::Owner>,
    /// Whether this run removes it (with `--yes`) or would (without).
    pub remove: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Response {
    /// False for a dry run: nothing was deleted.
    pub applied: bool,
    pub groups: Vec<Group>,
    /// Collections deleted by this run, in order.
    pub deleted: Vec<String>,
    /// What to do next.
    pub next: String,
}

/// Where a marker's database stands on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnDisk {
    Present,
    /// Missing from a folder that exists and can be read.
    Gone,
    /// The folder is missing or unreadable, so absence proves nothing.
    Unreachable,
}

fn on_disk(path: &Path) -> OnDisk {
    if path.exists() {
        return OnDisk::Present;
    }
    match path.parent().map(std::fs::read_dir) {
        Some(Ok(_)) => OnDisk::Gone,
        _ => OnDisk::Unreachable,
    }
}

/// One group's class from its evidence. The single decision point for what
/// may be removed.
fn classify(
    group: &str,
    owner: Option<&ownership::Owner>,
    in_use: &BTreeSet<String>,
    disk: impl Fn(&Path) -> OnDisk,
) -> Class {
    if in_use.contains(group) {
        return Class::InUse;
    }
    let Some(owner) = owner else {
        return Class::Unknown;
    };
    match (owner.owner, owner.database.as_deref()) {
        (ownership::OwnerKind::Server, _) => Class::SharedServer,
        (ownership::OwnerKind::Local, Some(database)) => match disk(Path::new(database)) {
            OnDisk::Present => Class::InUse,
            OnDisk::Gone => Class::Leftover,
            OnDisk::Unreachable => Class::OwnerUnreachable,
        },
        // A local marker without its database cannot be checked.
        (ownership::OwnerKind::Local, None) => Class::OwnerUnreachable,
    }
}

/// A group's state collection, where its owner marker lives.
fn state_of(group: &str) -> String {
    format!("{PREFIX}{group}_state")
}

/// The group hash of a Mash views collection name, or `None` for any other collection.
fn group_of(name: &str) -> Option<&str> {
    let rest = name.strip_prefix(PREFIX)?;
    let (hash, kind) = rest.rsplit_once('_')?;
    let valid = hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit());
    (valid && matches!(kind, "messages" | "state")).then_some(hash)
}

/// The views group of an index database, read without opening it for writing.
fn group_for(database: &Path, nodes: &[&str]) -> Result<Vec<String>, AppError> {
    let canonical = std::fs::canonicalize(database)
        .map_err(|_| AppError::configuration(format!("{} cannot be read", database.display())))?;
    let connection = rusqlite::Connection::open_with_flags(
        &canonical,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(AppError::database)?;
    let instance: String = connection
        .query_row(
            "SELECT instance FROM local_database_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| {
            AppError::configuration(format!("{} is not a Mash index", database.display()))
        })?;
    let identity = crate::storage::gold_input::gold_identity(&instance, &canonical);
    Ok(nodes
        .iter()
        .map(|node| {
            let id = crate::views::local::group_id(&identity, node);
            blake3::hash(id.as_bytes()).to_hex().to_string()
        })
        .collect())
}

/// `run`, as the core's response.
pub async fn respond(
    request: Request,
    database: &Path,
    configuration: &ResolvedConfig,
) -> Result<super::Response, AppError> {
    run(request, database, configuration)
        .await
        .map(super::Response::QdrantPrune)
}

pub async fn run(
    request: Request,
    database: &Path,
    configuration: &ResolvedConfig,
) -> Result<Response, AppError> {
    let key = crate::views::local::key(configuration)?;
    let client = api::Client::new(&configuration.qdrant.url, key.as_deref(), 1024 * 1024)?;
    // This machine's index under its own node; other indexes may have been
    // built under any node, so both readings protect them.
    let node = crate::views::local::node(configuration);
    let mut in_use = BTreeSet::new();
    in_use.extend(group_for(database, &[node])?);
    for other in &request.databases {
        in_use.extend(group_for(other, &[node, "local"])?);
    }
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for name in client.collection_names().await? {
        if let Some(group) = group_of(&name) {
            groups.entry(group.to_owned()).or_default().push(name);
        }
    }
    let named: BTreeSet<String> = request
        .unknown
        .iter()
        .map(|name| group_of(name).unwrap_or(name).to_owned())
        .collect();
    let mut listed = Vec::new();
    for (group, mut collections) in groups {
        collections.sort();
        let mut points = 0;
        for name in &collections {
            points += client.collection_points(name).await?.unwrap_or(0);
        }
        let state = state_of(&group);
        let owner = if collections.contains(&state) {
            ownership::read_owner(&client, &state).await?
        } else {
            None
        };
        let class = classify(&group, owner.as_ref(), &in_use, on_disk);
        let remove =
            class == Class::Leftover || (class == Class::Unknown && named.contains(&group));
        listed.push(Group {
            group,
            collections,
            points,
            class,
            owner,
            remove,
        });
    }
    if let Some(missing) = named
        .iter()
        .find(|name| !listed.iter().any(|g| &g.group == *name))
    {
        return Err(AppError::usage(format!(
            "{missing} is not a Mash views group in this Qdrant; give its full hash or collection name"
        )));
    }
    let mut deleted = Vec::new();
    if request.apply {
        for group in listed
            .iter()
            .filter(|group| group.remove && within(&request, &group.group))
        {
            remove(&client, group, &in_use, &mut deleted).await?;
        }
    }
    let removable = listed.iter().filter(|group| group.remove).count();
    let next = if request.apply {
        format!("Removed {} collections.", deleted.len())
    } else if removable == 0 {
        "Nothing to remove.".to_owned()
    } else {
        format!("Run `mash qdrant prune --yes` to remove the {removable} groups marked remove.")
    };
    Ok(Response {
        applied: request.apply,
        groups: listed,
        deleted,
        next,
    })
}

#[cfg(test)]
fn within(request: &Request, group: &str) -> bool {
    request
        .only
        .as_ref()
        .is_none_or(|only| only.contains(group))
}
#[cfg(not(test))]
fn within(_: &Request, _: &str) -> bool {
    true
}

/// Delete one group: check its marker again (an index may have been restored
/// since the listing), then the documents, then the state with its marker.
async fn remove(
    client: &api::Client,
    group: &Group,
    in_use: &BTreeSet<String>,
    deleted: &mut Vec<String>,
) -> Result<(), AppError> {
    let state = state_of(&group.group);
    let owner = if group.collections.contains(&state) {
        ownership::read_owner(client, &state).await?
    } else {
        None
    };
    let class = classify(&group.group, owner.as_ref(), in_use, on_disk);
    if class != group.class {
        return Err(AppError::configuration(format!(
            "{} changed owner while pruning; nothing more was deleted",
            group.group
        )));
    }
    let mut order = group.collections.clone();
    order.sort_by_key(|name| name.ends_with("_state"));
    for name in order {
        client.delete_collection(&name).await?;
        deleted.push(name);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
