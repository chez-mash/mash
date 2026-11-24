//! When each machine last checked in.
//!
//! A machine's watcher checks in every minute while it runs, whether or not it
//! has anything to publish, so a recent check-in is evidence that the machine
//! is on and its watcher is running. Publication alone cannot say that: a
//! machine with nothing new to publish is silent.
//!
//! The times are kept in memory only. They are evidence of the last few
//! minutes, which a restart loses anyway, and keeping them out of the store
//! spares it a write a minute per machine. After a restart a machine is
//! unreported until its next check-in. Only authenticated machines check in,
//! so the map holds at most one entry per machine the server accepts.
use axum::Json;
use axum::extract::{Extension, State};
use mash_app::shared::server::report::{HEARTBEAT_SECONDS, Heartbeat};
use mash_app::shared::server::{Identity, enrollment};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use super::App;

#[derive(Clone, Default)]
pub(crate) struct Presence(Arc<Mutex<HashMap<(String, String), u64>>>);

impl Presence {
    fn beat(&self, id: &Identity, now: u64) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((id.group_id.clone(), id.node_id.clone()), now);
    }

    /// When `node` of `group` last checked in, if it has since the server started.
    pub(crate) fn last_seen(&self, group: &str, node: &str) -> Option<u64> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(group.to_owned(), node.to_owned()))
            .copied()
    }
}

/// `POST /v1/heartbeat`: the calling machine is running. It changes nothing
/// but the time the server last heard from it.
pub(crate) async fn heartbeat(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
) -> Json<Heartbeat> {
    let now = enrollment::now();
    app.presence.beat(&id, now);
    Json(Heartbeat {
        now,
        every_seconds: HEARTBEAT_SECONDS,
    })
}
