//! What the server tells machines and admins about its machines: the shapes
//! of `GET /v1/status` and the admin routes, shared by the server that sends
//! them and the dashboards that read them. None of them holds a secret except
//! [`JoinAnswer::join`], which only an admin receives, once.
use serde::{Deserialize, Serialize};

/// How a machine stands with the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum MachineState {
    /// Listed in the operator's own configuration file.
    Configured,
    /// Its join string can still be redeemed until `expires_at`.
    Waiting {
        expires_at: u64,
    },
    /// Its join string was never redeemed and no longer can be.
    Expired,
    Joined,
    /// Its credential was removed; what it published stays until deleted.
    Removed {
        delete_requested: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "SharedMachine")]
pub struct MachineStatus {
    pub group_id: String,
    pub node_id: String,
    #[serde(flatten)]
    pub state: MachineState,
    /// When its current credential was made, by the server's clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_created_at: Option<u64>,
    /// The last four characters of the stored hash of its credential: enough
    /// to tell two credentials apart, nothing to sign in with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<super::store::Published>,
    /// A first publish still arriving.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiving: Option<super::status::Receiving>,
    /// Its history is being deleted once its open connections close.
    #[serde(default)]
    pub delete_waiting: bool,
}

/// A group as any of its machines may read it.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "SharedGroupStatus")]
pub struct GroupStatus {
    /// The server's build revision.
    pub build: String,
    /// When the server started, in seconds since 1970 by its clock.
    pub started_at: u64,
    pub now: u64,
    pub group_id: String,
    /// The server needs a restart before it accepts machines of this group.
    pub restart_needed: bool,
    pub machines: Vec<MachineStatus>,
}

/// Every machine in every group, the actions recorded, and the admin keys,
/// for an admin.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "SharedServerOverview")]
pub struct Overview {
    pub build: String,
    pub started_at: u64,
    pub now: u64,
    pub machines: Vec<MachineStatus>,
    /// Newest first.
    pub actions: Vec<super::admin::Action>,
    pub keys: Vec<KeyName>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct KeyName {
    pub name: String,
    pub created_at: u64,
}

/// An admin's request about one machine.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MachineRequest {
    /// Defaults to the server's only group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub node_id: String,
    /// For a deletion: the machine's name typed again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<String>,
}

/// A new join string, shown once.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JoinAnswer {
    /// It holds the machine's one-time secret: pass it privately.
    pub join: String,
    pub expires_in_seconds: u64,
    /// The server must restart before it accepts machines in this group.
    pub restart_needed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Done {
    pub done: bool,
}
