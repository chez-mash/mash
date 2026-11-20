//! `mash index --watch`: hold the watcher lease, index on every debounced
//! change until interrupted, and say whether a replacement build should
//! take over.
use super::{AppError, Cli, Response, Write, core};

#[derive(serde::Serialize)]
struct WatchRole {
    event: WatchRoleEvent,
    role: crate::watch::Role,
}

#[derive(serde::Serialize)]
enum WatchRoleEvent {
    #[serde(rename = "watch-role")]
    Role,
}

/// Run `index --watch` for `cli`, whose watch flag the caller has cleared so
/// each pass is an ordinary index request.
pub(super) async fn run(cli: Cli, debounce: u64) -> Result<Response, AppError> {
    let request = cli.into_request()?;
    let paths = request.paths.clone();
    let lease = std::sync::Arc::new(
        crate::blocking(move || {
            crate::runtime::Lease::register(&paths, crate::runtime::Kind::IndexWatch)
        })
        .await?,
    );
    let (stop, shutdown) = tokio::sync::watch::channel(false);
    let monitor = lease.clone();
    let signal = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                _ = interval.tick() => {
                    let lease = monitor.clone();
                    if crate::blocking(move || lease.ensure_current()).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = stop.send(true);
    });
    let role = |role| {
        let _ = crate::write_json(
            &mut std::io::stderr(),
            &WatchRole {
                event: WatchRoleEvent::Role,
                role,
            },
        );
    };
    let admission = lease.clone();
    // Watch collects state continuously; publish out of band so a slow
    // server never blocks the next indexing pass.
    let mut watch_request = request.clone();
    if let crate::core::Request {
        operation: crate::core::Operation::Index(index),
        ..
    } = &mut watch_request
    {
        index.defer_publication = true;
    }
    let result = crate::watch::run(watch_request.clone(), debounce, shutdown, role, move || {
        let request = watch_request.clone();
        let lease = admission.clone();
        async move {
            crate::blocking(move || lease.ensure_current()).await?;
            let response = core::execute(request).await?;
            let retry = response.needs_retry();
            let mut stdout = std::io::stdout();
            crate::write_json(&mut stdout, &response)?;
            stdout
                .flush()
                .map_err(|_| AppError::internal("flush watch output"))?;
            Ok(retry)
        }
    })
    .await;
    signal.abort();
    // The monitor holds the lease; it must be gone before a restart.
    let _ = signal.await;
    result?;
    let executable = lease.executable().to_path_buf();
    if let Err(reason) = crate::blocking(move || lease.ensure_current()).await {
        return Ok(Response::WatchReplaced { reason, executable });
    }
    Ok(Response::WatchStopped { watching: false })
}
