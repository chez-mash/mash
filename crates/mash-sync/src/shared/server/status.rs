//! What a running server tells the dashboard on its own machine, through a
//! file beside its configuration (`server.toml.status`) rather than a network
//! route. Only someone who can read the configuration's directory, the user
//! the server runs as, can read it. It holds no secrets: names, counts and
//! times. The server rewrites it every few seconds, so a file that stopped
//! changing belongs to a server that is not running.
use crate::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// How often a running server rewrites the file.
pub const HEARTBEAT_SECONDS: u64 = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "SharedServerStatus")]
pub struct ServerStatus {
    pub listen: String,
    pub started_at: u64,
    pub updated_at: u64,
    /// BLAKE3 of the added-machines file the server last applied, so a
    /// change can be seen to have taken effect.
    pub applied: String,
    /// Groups the running server serves.
    pub groups: Vec<String>,
    /// Machines added to a group the server does not serve yet; it accepts
    /// them after a restart.
    #[serde(default)]
    pub restart_needed: Vec<String>,
    #[serde(default)]
    pub nodes: Vec<NodeStatus>,
    /// Why the server could not apply the added-machines file, if it could not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "SharedServerNode")]
pub struct NodeStatus {
    pub group_id: String,
    pub node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<super::store::Published>,
    /// A first publish still arriving: chats received of those announced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiving: Option<Receiving>,
    /// A deletion that waits for the machine's open connections to close.
    #[serde(default)]
    pub delete_waiting: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Receiving {
    pub covered: u64,
    pub total: u64,
}

pub fn status_path(configuration: &Path) -> PathBuf {
    let mut name = configuration.as_os_str().to_owned();
    name.push(".status");
    name.into()
}

/// BLAKE3 of an added-machines file's text; an absent file is empty text.
pub fn applied(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

/// The added-machines file's text as it is now.
pub fn nodes_text(configuration: &Path) -> String {
    std::fs::read_to_string(super::enrollment::nodes_path(configuration)).unwrap_or_default()
}

pub fn write(configuration: &Path, status: &ServerStatus) -> Result<(), AppError> {
    let failed = || AppError::configuration("could not write the server status file");
    let path = status_path(configuration);
    let directory = path.parent().ok_or_else(failed)?;
    let text =
        serde_json::to_vec(status).map_err(|_| AppError::internal("encode server status"))?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".status")
        .tempfile_in(directory)
        .map_err(|_| failed())?;
    std::io::Write::write_all(&mut temporary, &text).map_err(|_| failed())?;
    temporary.persist(&path).map_err(|_| failed())?;
    Ok(())
}

/// The status a server last wrote, if it wrote one that reads.
pub fn read(configuration: &Path) -> Option<ServerStatus> {
    let bytes = std::fs::read(status_path(configuration)).ok()?;
    if bytes.len() > 1024 * 1024 {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// Whether the server that wrote `status` is still running: it rewrites the
/// file on a heartbeat, so a stale one means it stopped.
pub fn running(status: &ServerStatus, now: u64) -> bool {
    now.saturating_sub(status.updated_at) <= HEARTBEAT_SECONDS * 3
}
