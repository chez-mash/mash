//! A running watcher checks in with the shared server every minute.
//!
//! Publication says nothing while a machine has nothing new to publish, so a
//! dashboard could not tell an idle machine from one that is off. Checking in
//! is that evidence: the server keeps the time it last heard from each
//! machine, and dashboards show a machine as up while the time is recent.
//!
//! The configuration is reread before every check-in, so a machine that joins
//! a shared index starts checking in without a restart and one that leaves
//! stops. A failed check-in is said once, when check-ins start failing, and
//! tried again at the next one; it never stops the watcher.
use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::watch;

use crate::AppError;
use crate::shared::server::report::HEARTBEAT_SECONDS;

/// The server sets the pace, within bounds a misconfigured server cannot push
/// to a flood or to silence.
const FASTEST: Duration = Duration::from_secs(15);
const SLOWEST: Duration = Duration::from_secs(600);

/// Checks in with the shared server configured at `config` until `shutdown`.
pub async fn check_in(config: Option<PathBuf>, shutdown: watch::Receiver<bool>) {
    beat_until(shutdown, move || {
        let config = config.clone();
        async move {
            let shared = crate::blocking(move || crate::config::load(config.as_deref()))
                .await?
                .shared;
            match shared {
                Some(shared) => Ok(Some(
                    crate::shared::manage::heartbeat(&shared)
                        .await?
                        .every_seconds,
                )),
                None => Ok(None),
            }
        }
    })
    .await;
}

/// Calls `beat` now and then at the pace its answers ask for, until
/// `shutdown`. A beat answers the seconds until the next one, or none when
/// there is no server to check in with.
async fn beat_until<F, Fut>(mut shutdown: watch::Receiver<bool>, mut beat: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Option<u64>, AppError>>,
{
    let mut every = Duration::from_secs(HEARTBEAT_SECONDS);
    let mut failing = false;
    loop {
        if *shutdown.borrow() {
            return;
        }
        match beat().await {
            Ok(asked) => {
                failing = false;
                if let Some(seconds) = asked {
                    every = Duration::from_secs(seconds).clamp(FASTEST, SLOWEST);
                }
            }
            Err(error) => {
                if !failing {
                    let _ = crate::write_json(&mut std::io::stderr(), &error);
                }
                failing = true;
            }
        }
        tokio::select! {
            biased;
            result = shutdown.changed() => {
                if result.is_err() || *shutdown.borrow() { return; }
            }
            () = tokio::time::sleep(every) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test(start_paused = true)]
    async fn beats_at_the_servers_pace_within_bounds_and_stop_on_shutdown() {
        let (stop, shutdown) = watch::channel(false);
        let beats = Arc::new(AtomicU32::new(0));
        let counted = beats.clone();
        let task = tokio::spawn(beat_until(shutdown, move || {
            let counted = counted.clone();
            // The server asks for a beat every second: faster than allowed.
            async move {
                counted.fetch_add(1, Ordering::SeqCst);
                Ok(Some(1))
            }
        }));
        tokio::time::sleep(Duration::from_secs(46)).await;
        // At once, then at 15, 30 and 45 seconds: the fastest pace allowed.
        assert_eq!(beats.load(Ordering::SeqCst), 4);
        stop.send(true).unwrap();
        task.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_beat_is_tried_again_and_never_stops_the_loop() {
        let (stop, shutdown) = watch::channel(false);
        let beats = Arc::new(AtomicU32::new(0));
        let counted = beats.clone();
        let task = tokio::spawn(beat_until(shutdown, move || {
            let counted = counted.clone();
            async move {
                counted.fetch_add(1, Ordering::SeqCst);
                Err(AppError::shared("could not reach the shared index"))
            }
        }));
        tokio::time::sleep(Duration::from_secs(HEARTBEAT_SECONDS * 2 + 1)).await;
        assert_eq!(beats.load(Ordering::SeqCst), 3);
        stop.send(true).unwrap();
        task.await.unwrap();
    }
}
