//! The server's state as any machine of a group may read it, and the same
//! across every group for an admin.
//!
//! `GET /v1/status` answers with a machine's ordinary token: the server's
//! build and start time, and each machine of the caller's own group with how
//! it stands, when its credential was made, a short fingerprint of it, what
//! it published, and when its watcher last checked in. It holds no secret and changes nothing, so a machine's
//! token, which can only publish and search, can read it. Other groups are
//! never named.
use axum::Json;
use axum::extract::{Extension, State};
use mash_app::shared::server::report::{GroupStatus, MachineState, MachineStatus};
use mash_app::shared::server::{Identity, configuration, enrollment, status};
use mash_model::AppError;

use super::{ApiError, App};

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
        let mut status = machine(&identity, added, &nodes, now);
        if let Some(seen) = app
            .presence
            .last_seen(&identity.group_id, &identity.node_id)
        {
            status.last_seen_at = Some(seen.at);
            status.os = seen.os;
        }
        machines.push(status);
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
        search: None,
        last_seen_at: None,
        os: None,
    }
}

/// The caller's group as the server sees it.
pub(crate) async fn read(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
) -> Result<Json<GroupStatus>, ApiError> {
    let (mut machines, now) = machines(&app, |group| group == id.group_id).await?;
    // The search state is an extra: a read that fails leaves it unreported
    // rather than failing the status of the whole group.
    if let Ok(search) = app.store.search_readiness(&id).await {
        for machine in &mut machines {
            machine.search = search.get(&machine.node_id).cloned();
        }
    }
    Ok(Json(GroupStatus {
        build: mash_app::build_revision().to_owned(),
        started_at: app.live.as_ref().map_or(0, |live| live.started_at),
        now,
        restart_needed: !app.store.serves(&id.group_id),
        group_id: id.group_id,
        machines,
    }))
}
