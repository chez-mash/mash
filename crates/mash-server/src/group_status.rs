//! The server's state as any machine of a group may read it, and the same
//! across every group for an admin.
//!
//! `GET /v1/status` answers with a machine's ordinary token: the server's
//! build and start time, and each machine of the caller's own group with how
//! it stands, when its credential was made, a short fingerprint of it, and
//! what it published. It holds no secret and changes nothing, so a machine's
//! token, which can only publish and search, can read it. Other groups are
//! never named.
use axum::Json;
use axum::extract::{Extension, State};
use mash_app::shared::server::{Identity, configuration, enrollment, status, store};
use mash_model::AppError;
use serde::Serialize;

use super::{ApiError, App};

/// How a machine stands with the server.
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub(crate) enum MachineState {
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

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub(crate) struct MachineStatus {
    pub(crate) group_id: String,
    pub(crate) node_id: String,
    #[serde(flatten)]
    pub(crate) state: MachineState,
    /// When its current credential was made, by the server's clock.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) credential_created_at: Option<u64>,
    /// The last four characters of the stored hash of its credential: enough
    /// to tell two credentials apart, nothing to sign in with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) published: Option<store::Published>,
    /// A first publish still arriving.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receiving: Option<status::Receiving>,
    /// Its history is being deleted once its open connections close.
    pub(crate) delete_waiting: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct GroupStatus {
    /// The server's build revision.
    pub(crate) build: String,
    /// When the server started, in seconds since 1970 by its clock.
    pub(crate) started_at: u64,
    pub(crate) now: u64,
    pub(crate) group_id: String,
    /// This server needs a restart before it accepts machines of this group.
    pub(crate) restart_needed: bool,
    pub(crate) machines: Vec<MachineStatus>,
}

/// Every machine the server knows, with its state, for the groups `include`
/// keeps.
pub(crate) async fn machines(
    app: &App,
    include: impl Fn(&str) -> bool,
) -> Result<(Vec<MachineStatus>, u64), AppError> {
    let live = app.live.as_ref().ok_or_else(|| {
        AppError::usage("this server's machines are set in its configuration file only")
    })?;
    let config = configuration::load(&live.configuration).await?;
    let now = enrollment::now();
    let (nodes, _) = super::roster::node_statuses(app, &config).await;
    let added = configuration::added(&config);
    let mut machines = Vec::new();
    for identity in configuration::members(&config) {
        if !include(&identity.group_id) {
            continue;
        }
        machines.push(machine(&identity, added, &nodes, now));
    }
    machines.sort_by(|a, b| (&a.group_id, &a.node_id).cmp(&(&b.group_id, &b.node_id)));
    Ok((machines, now))
}

fn machine(
    identity: &Identity,
    added: &enrollment::Added,
    nodes: &[status::NodeStatus],
    now: u64,
) -> MachineStatus {
    let entry = added
        .nodes
        .iter()
        .find(|entry| entry.group_id == identity.group_id && entry.node_id == identity.node_id);
    let node = nodes
        .iter()
        .find(|node| node.group_id == identity.group_id && node.node_id == identity.node_id);
    MachineStatus {
        group_id: identity.group_id.clone(),
        node_id: identity.node_id.clone(),
        state: entry.map_or(MachineState::Configured, |entry| match entry.state(now) {
            enrollment::State::Waiting { expires_at } => MachineState::Waiting { expires_at },
            enrollment::State::Expired => MachineState::Expired,
            enrollment::State::Joined => MachineState::Joined,
            enrollment::State::Removed { delete_requested } => {
                MachineState::Removed { delete_requested }
            }
        }),
        credential_created_at: entry.and_then(|entry| entry.created_at),
        fingerprint: entry.and_then(enrollment::Entry::fingerprint),
        published: node.and_then(|node| node.published.clone()),
        receiving: node.and_then(|node| node.receiving),
        delete_waiting: node.is_some_and(|node| node.delete_waiting),
    }
}

/// The caller's group as the server sees it.
pub(crate) async fn read(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
) -> Result<Json<GroupStatus>, ApiError> {
    let (machines, now) = machines(&app, |group| group == id.group_id).await?;
    Ok(Json(GroupStatus {
        build: mash_app::BUILD_REVISION.to_owned(),
        started_at: app.live.as_ref().map_or(0, |live| live.started_at),
        now,
        restart_needed: !app.store.serves(&id.group_id),
        group_id: id.group_id,
        machines,
    }))
}
