//! Who owns a views group. Collection names are hashes, so from outside Qdrant
//! nobody can tell which index or server a group belongs to. Each owner
//! therefore keeps one marker point in its group's `_state` collection: a
//! local index names its database file, a shared server names its group.
//! `mash qdrant prune` reads these markers; a group without one is "unknown"
//! and is never removed on a guess.
use super::{AppError, Store, api, collection, point_id};
use serde::{Deserialize, Serialize};

/// The marker's point ID, the same in every `_state` collection.
pub fn owner_id() -> String {
    point_id(&["owner"])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OwnerKind {
    /// A machine's own index; `database` is its SQLite file.
    Local,
    /// A shared server; `group` is the group it serves.
    Server,
}

/// The marker a group's owner writes. `kind` is always "owner", so the point
/// can never be mistaken for a node's views record in the same collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Owner {
    kind: Marker,
    pub owner: OwnerKind,
    /// The index's database, as an absolute path with links resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    /// The shared group a server keeps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The machine that wrote the marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Marker {
    Owner,
}

impl Owner {
    pub fn local(database: &std::path::Path, host: Option<String>) -> Self {
        Self {
            kind: Marker::Owner,
            owner: OwnerKind::Local,
            database: Some(database.to_string_lossy().into_owned()),
            group: None,
            host,
        }
    }
    pub fn server(group: &str, host: Option<String>) -> Self {
        Self {
            kind: Marker::Owner,
            owner: OwnerKind::Server,
            database: None,
            group: Some(group.to_owned()),
            host,
        }
    }
}

impl Store {
    /// Write (or rewrite, unchanged) the group's owner marker.
    pub async fn record_owner(&self, group_id: &str, owner: &Owner) -> Result<(), AppError> {
        self.api
            .upsert(
                &collection(group_id, true),
                &api::Upsert {
                    points: vec![api::Point {
                        id: owner_id(),
                        vector: api::Vectors::default(),
                        payload: owner,
                    }],
                },
            )
            .await
    }
}

/// The owner marker in a `_state` collection, if one was written. A point at
/// the marker's ID that is not a marker is an error, not "no owner".
pub async fn read_owner(api: &api::Client, state: &str) -> Result<Option<Owner>, AppError> {
    let mut points = api
        .retrieve::<serde_json::Value>(state, std::slice::from_ref(&owner_id()))
        .await?;
    let Some(point) = points.pop() else {
        return Ok(None);
    };
    serde_json::from_value(point.payload)
        .map(Some)
        .map_err(|_| AppError::configuration(format!("{state} has an unreadable owner marker")))
}

/// The host name markers record, when the system reports one.
pub fn host_name() -> Option<String> {
    let name = std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .or_else(|| std::env::var("COMPUTERNAME").ok())?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}
