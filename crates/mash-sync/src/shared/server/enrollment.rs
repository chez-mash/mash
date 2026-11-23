//! Machines added to a server with a one-time join string.
//!
//! The server keeps these machines in a file beside its configuration
//! (`server.toml.nodes`), never in the operator's own file. Each entry holds
//! hashes only: of the machine's token once it has joined, and of the one-time
//! join secret until then. A join string carries that secret, not a token. The
//! joining machine makes its own token and sends only its hash when it redeems
//! the secret, so no token ever crosses the network or rests on the server.
//!
//! A join secret works once and for [`JOIN_LIFETIME`] after it was made. Both
//! limits are decided here, on the server, against the time the server itself
//! recorded when it made the secret, so a joining machine's clock never
//! matters. Credentials the operator removed or replaced are remembered by
//! hash, so a machine that still uses one is told so instead of being treated
//! as a stranger.
//!
//! The `mash-server --add-node` command, the dashboard on the server machine
//! and the running server all change this file through [`update`], which
//! holds a lock beside it and replaces the file in one rename. A running
//! server rereads it and applies the change without a restart.
use crate::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod joining;
pub use joining::{JoinStatus, check, join_string_for, mint, redeem, reissue};
#[cfg(test)]
mod tests;

/// How long a join string can be redeemed after it was made.
pub const JOIN_LIFETIME: u64 = 24 * 60 * 60;
/// Removed and replaced credentials remembered so their holder hears why.
const REMEMBERED_REVOCATIONS: usize = 256;
/// Bound on the file, as for the server configuration.
const MAX_BYTES: usize = 256 * 1024;

/// The file of added machines.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Added {
    /// The address written into join strings, recorded when the first one
    /// was made, so later ones need not be told it again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(default)]
    pub nodes: Vec<Entry>,
    /// Credentials this server no longer accepts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revoked: Vec<Revoked>,
    /// Keys that may manage machines from any dashboard; see `server::admin`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admins: Vec<super::admin::AdminKey>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit: Vec<super::admin::Action>,
}

/// One added machine.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub group_id: String,
    pub node_id: String,
    /// BLAKE3 of the machine's token, once it has joined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_hash: Option<String>,
    /// Never valid here; read only to refuse it clearly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env: Option<String>,
    /// When the current credential (its join string) was made, in seconds
    /// since 1970 by this server's clock. Absent for machines added before
    /// join strings expired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    /// BLAKE3 of the one-time join secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_expires_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<u64>,
    /// Removed: the machine can no longer publish or search, and what it
    /// published stays on the server and searchable until it is deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_at: Option<u64>,
    /// The operator asked the running server to delete what this removed
    /// machine published; the server drops the entry once that is done.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_requested_at: Option<u64>,
}

/// A credential that was removed or replaced.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Revoked {
    pub group_id: String,
    pub node_id: String,
    pub hash: String,
    pub revoked_at: u64,
}

/// Where an added machine stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum State {
    /// Its join string can still be redeemed.
    Waiting {
        expires_at: u64,
    },
    /// Its join string was never redeemed and no longer can be.
    Expired,
    Joined,
    /// Its credential was removed; what it published is kept.
    Removed {
        delete_requested: bool,
    },
}

impl Entry {
    pub fn state(&self, now: u64) -> State {
        if let Some(_removed) = self.removed_at {
            return State::Removed {
                delete_requested: self.delete_requested_at.is_some(),
            };
        }
        if self.token_hash.is_some() {
            return State::Joined;
        }
        match self.join_expires_at {
            Some(expires_at) if now < expires_at => State::Waiting { expires_at },
            _ => State::Expired,
        }
    }

    /// A short, non-secret label for the current credential: the end of the
    /// hash the server keeps, never of the secret itself.
    pub fn fingerprint(&self) -> Option<String> {
        let hash = self.token_hash.as_ref().or(self.join_hash.as_ref())?;
        Some(
            hash.chars()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
        )
    }

    fn revoke(&mut self, revoked: &mut Vec<Revoked>, now: u64) {
        for hash in [self.token_hash.take(), self.join_hash.take()]
            .into_iter()
            .flatten()
        {
            revoked.push(Revoked {
                group_id: self.group_id.clone(),
                node_id: self.node_id.clone(),
                hash,
                revoked_at: now,
            });
        }
        let excess = revoked.len().saturating_sub(REMEMBERED_REVOCATIONS);
        revoked.drain(..excess);
        self.join_expires_at = None;
        self.joined_at = None;
    }
}

/// The added-machines file of a server configuration.
pub fn nodes_path(configuration: &Path) -> PathBuf {
    let mut name = configuration.as_os_str().to_owned();
    name.push(".nodes");
    name.into()
}

fn lock_path(configuration: &Path) -> PathBuf {
    let mut name = configuration.as_os_str().to_owned();
    name.push(".nodes.lock");
    name.into()
}

/// Seconds since 1970 by this machine's clock.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Read the file; a missing one has no machines.
pub fn read(configuration: &Path) -> Result<Added, AppError> {
    let text = match std::fs::read_to_string(nodes_path(configuration)) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Added::default()),
        Err(_) => {
            return Err(AppError::configuration(
                "could not read the added-nodes file",
            ));
        }
    };
    parse(&text)
}

/// Parse and validate the file's text.
pub fn parse(text: &str) -> Result<Added, AppError> {
    if text.len() > MAX_BYTES {
        return Err(AppError::configuration(
            "the added-nodes file exceeds 262144 bytes",
        ));
    }
    let added: Added =
        toml::from_str(text).map_err(|_| AppError::configuration("invalid added-nodes file"))?;
    super::admin::validate(&added)?;
    for entry in &added.nodes {
        if entry.token_env.is_some() {
            return Err(AppError::configuration(
                "the added-nodes file holds token hashes only",
            ));
        }
        crate::config::validate_identifier(&entry.group_id, "group_id")?;
        crate::config::validate_identifier(&entry.node_id, "node_id")?;
        for hash in [&entry.token_hash, &entry.join_hash].into_iter().flatten() {
            blake3::Hash::from_hex(hash).map_err(|_| {
                AppError::configuration("token_hash must be 64 hexadecimal characters")
            })?;
        }
        if entry.token_hash.is_none() && entry.join_hash.is_none() && entry.removed_at.is_none() {
            return Err(AppError::configuration(
                "each added node needs a token hash or a join hash",
            ));
        }
    }
    Ok(added)
}

/// Change the file under its lock and replace it in one rename, so a reader
/// never sees half of it and two writers never lose each other's change.
pub fn update<T>(
    configuration: &Path,
    change: impl FnOnce(&mut Added) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let failed = || AppError::configuration("could not write the added-nodes file");
    let lock = lock_path(configuration);
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(false).write(true);
    crate::platform::private_file(&mut options);
    let file = options.open(&lock).map_err(|_| failed())?;
    let _held = mash_model::file_lock::FileLock::exclusive(file).map_err(|_| failed())?;
    let mut added = read(configuration)?;
    let result = change(&mut added)?;
    let text = toml::to_string(&added).map_err(|_| AppError::internal("encode added nodes"))?;
    let path = nodes_path(configuration);
    let directory = path.parent().ok_or_else(failed)?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".nodes")
        .tempfile_in(directory)
        .map_err(|_| failed())?;
    {
        use std::io::Write;
        temporary.write_all(text.as_bytes()).map_err(|_| failed())?;
        temporary.as_file().sync_all().map_err(|_| failed())?;
    }
    // tempfile creates the file readable only by this user.
    temporary.persist(&path).map_err(|_| failed())?;
    Ok(result)
}

/// A fresh secret: 32 random bytes as hexadecimal.
pub fn secret() -> Result<String, AppError> {
    let mut random = [0_u8; 32];
    getrandom::fill(&mut random).map_err(|_| AppError::internal("create a join secret"))?;
    Ok(random.iter().fold(String::new(), |mut hex, byte| {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
        hex
    }))
}

fn hash(secret: &str) -> String {
    blake3::hash(secret.as_bytes()).to_hex().to_string()
}

/// Why a machine name cannot be added.
fn taken(entry: &Entry, group_id: &str) -> AppError {
    let mut error = AppError::conflict(format!(
        "{group_id} already has a machine called {}; pick another name, or replace its credential if it is the same machine",
        entry.node_id
    ));
    error.error.retryable = false;
    error
}

/// Add a machine waiting to join, and return the secret for its join string.
/// `configured` names the machines in the operator's own configuration, which
/// are taken too.
pub fn add(
    added: &mut Added,
    configured: &[(String, String)],
    group_id: &str,
    node_id: &str,
    now: u64,
) -> Result<String, AppError> {
    crate::config::validate_identifier(group_id, "group_id")?;
    crate::config::validate_identifier(node_id, "node_id")?;
    if let Some(entry) = added
        .nodes
        .iter()
        .find(|entry| entry.group_id == group_id && entry.node_id == node_id)
    {
        return Err(taken(entry, group_id));
    }
    if configured
        .iter()
        .any(|(group, node)| group == group_id && node == node_id)
    {
        return Err(AppError::conflict(format!(
            "{group_id} already has a machine called {node_id} in the server configuration; pick another name"
        )));
    }
    let secret = secret()?;
    added.nodes.push(Entry {
        group_id: group_id.to_owned(),
        node_id: node_id.to_owned(),
        token_hash: None,
        token_env: None,
        created_at: Some(now),
        join_hash: Some(hash(&secret)),
        join_expires_at: Some(now + JOIN_LIFETIME),
        joined_at: None,
        removed_at: None,
        delete_requested_at: None,
    });
    Ok(secret)
}

fn find<'a>(
    added: &'a mut Added,
    group_id: &str,
    node_id: &str,
) -> Result<&'a mut Entry, AppError> {
    added
        .nodes
        .iter_mut()
        .find(|entry| entry.group_id == group_id && entry.node_id == node_id)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "{group_id}/{node_id} is not a machine added to this server"
            ))
        })
}

/// Replace a machine's credential: the old token and any unused join string
/// stop working now, and the returned secret is for a new join string. What
/// the machine published stays. A removed machine comes back this way.
pub fn replace(
    added: &mut Added,
    group_id: &str,
    node_id: &str,
    now: u64,
) -> Result<String, AppError> {
    let Added { nodes, revoked, .. } = added;
    let entry = nodes
        .iter_mut()
        .find(|entry| entry.group_id == group_id && entry.node_id == node_id)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "{group_id}/{node_id} is not a machine added to this server"
            ))
        })?;
    if entry.delete_requested_at.is_some() {
        return Err(AppError::conflict(format!(
            "{node_id}'s published chats are being deleted; add it again once that is done"
        )));
    }
    entry.revoke(revoked, now);
    let secret = secret()?;
    entry.created_at = Some(now);
    entry.join_hash = Some(hash(&secret));
    entry.join_expires_at = Some(now + JOIN_LIFETIME);
    entry.removed_at = None;
    Ok(secret)
}

/// Remove a machine: its credential stops working now. What it published
/// stays on the server, and searchable, until it is deleted.
pub fn remove(added: &mut Added, group_id: &str, node_id: &str, now: u64) -> Result<(), AppError> {
    let Added { nodes, revoked, .. } = added;
    let entry = nodes
        .iter_mut()
        .find(|entry| entry.group_id == group_id && entry.node_id == node_id)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "{group_id}/{node_id} is not a machine added to this server"
            ))
        })?;
    if entry.removed_at.is_none() {
        entry.revoke(revoked, now);
        entry.removed_at = Some(now);
    }
    Ok(())
}

/// Ask the running server to delete everything a removed machine published.
/// Only a removed machine qualifies, so no credential can publish again
/// while the deletion runs.
pub fn request_delete(
    added: &mut Added,
    group_id: &str,
    node_id: &str,
    now: u64,
) -> Result<(), AppError> {
    let entry = find(added, group_id, node_id)?;
    if entry.removed_at.is_none() {
        return Err(AppError::conflict(format!(
            "remove {node_id} before deleting what it published"
        )));
    }
    entry.delete_requested_at.get_or_insert(now);
    Ok(())
}

/// Forget a machine whose published history the server has deleted.
pub fn forget(added: &mut Added, group_id: &str, node_id: &str) {
    added
        .nodes
        .retain(|entry| !(entry.group_id == group_id && entry.node_id == node_id));
}

/// Whether `hash` is a credential the operator removed or replaced.
pub fn revoked(added: &Added, hash: &blake3::Hash) -> bool {
    let hex = hash.to_hex();
    added
        .revoked
        .iter()
        .any(|revoked| revoked.hash == hex.as_str())
}
