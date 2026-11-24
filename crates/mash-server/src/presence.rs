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
use mash_app::shared::server::report::{CheckIn, HEARTBEAT_SECONDS, Heartbeat, Os};
use mash_app::shared::server::{Identity, enrollment};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use super::App;

/// A machine's last check-in: when, and the system it said it runs.
#[derive(Clone, Copy)]
pub(crate) struct Seen {
    pub(crate) at: u64,
    pub(crate) os: Option<Os>,
}

#[derive(Clone, Default)]
pub(crate) struct Presence(Arc<Mutex<HashMap<(String, String), Seen>>>);

impl Presence {
    fn beat(&self, id: &Identity, seen: Seen) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((id.group_id.clone(), id.node_id.clone()), seen);
    }

    /// `node` of `group`'s last check-in, if it has checked in since the server started.
    pub(crate) fn last_seen(&self, group: &str, node: &str) -> Option<Seen> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&(group.to_owned(), node.to_owned()))
            .copied()
    }
}

/// `POST /v1/heartbeat`: the calling machine is running, on the system it
/// names. It changes nothing but what the server last heard from it.
pub(crate) async fn heartbeat(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    Json(check_in): Json<CheckIn>,
) -> Json<Heartbeat> {
    let now = enrollment::now();
    app.presence.beat(
        &id,
        Seen {
            at: now,
            os: check_in.os,
        },
    );
    Json(Heartbeat {
        now,
        every_seconds: HEARTBEAT_SECONDS,
    })
}
