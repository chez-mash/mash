//! Starting the shared index's HTTP service and running it until it is told
//! to stop.
use super::{
    App, AppError, Listening, admission, configuration, join, presence, roster, router, shared,
    store, sync,
};
use futures_util::{FutureExt, future::BoxFuture};
use std::path::Path;
use std::sync::Arc;

/// Explicit operator changes applied before serving.
pub struct Activation {
    pub views: bool,
    pub query: bool,
    pub memory: bool,
}

/// Serves the shared index until the process is told to stop. `models` is
/// used to replay views from silver before serving.
///
/// The future is boxed here so that it is compiled in this crate, once. An
/// `async fn` is compiled where it is first polled, which for this one is the
/// executables' package: that package is rebuilt for every commit and would
/// compile the whole server again each time.
pub fn run<'a>(
    path: &'a Path,
    activation: Activation,
    models: &'a Path,
    replay: bool,
) -> BoxFuture<'a, Result<(), AppError>> {
    serve(path, activation, models, replay).boxed()
}

async fn serve(
    path: &Path,
    activation: Activation,
    models: &Path,
    replay: bool,
) -> Result<(), AppError> {
    let config = configuration::load(path).await?;
    let identities = configuration::identities(&config)?;
    let members = configuration::members(&config);
    let store = store::initialize(
        &config,
        &members,
        activation.views,
        activation.query,
        activation.memory,
    )
    .await?
    .with_response_redaction(crate::redaction::Policy::new(config.redaction.responses));
    if replay {
        for identity in &members {
            let replayed = store.replay(identity, models).await?;
            crate::write_json(&mut std::io::stderr(), &replayed)?;
        }
    }
    let listener = tokio::net::TcpListener::bind(config.listen)
        .await
        .map_err(|_| AppError::configuration("could not bind HTTP listener"))?;
    let address = listener
        .local_addr()
        .map_err(|_| AppError::internal("get HTTP listener address"))?;
    crate::write_json(&mut std::io::stderr(), &Listening { listening: address })?;
    let transfer_dir = config.silver_dir()?.join("sync");
    let catalog = std::sync::Arc::new(std::sync::Mutex::new(sync::open_catalog(&transfer_dir)?));
    let live = roster::Live::new(path.to_path_buf(), address.to_string());
    let app = App {
        activity: shared::activity::Registry::default(),
        mutation_capacity: config.max_concurrent_uploads,
        attachments_dir: config.attachments_dir,
        catalog,
        store,
        nodes: roster::Nodes::fixed(identities),
        live: Some(live.clone()),
        joins: Arc::new(tokio::sync::Semaphore::new(join::CONCURRENT)),
        uploads: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_uploads)),
        waiters: Arc::new(tokio::sync::Semaphore::new(admission::MAX_WAITERS)),
        presence: presence::Presence::default(),
    };
    // The first status says the server runs; a file that does not apply
    // keeps the tokens it started with.
    if let Err(error) = roster::refresh(&app, &live, true).await {
        tracing::warn!(message = %error.error.message, "could not apply the added-machines file");
    }
    let watch = roster::spawn_watch(app.clone(), live);
    // Resume durable transfers before accepting requests. A failure is logged
    // rather than fatal: the endpoint still serves and a client can resume.
    match sync::recover(&app).await {
        Ok(0) => {}
        Ok(resumed) => {
            crate::write_json(
                &mut std::io::stderr(),
                &serde_json::json!({"sync_recovered_operations": resumed}),
            )?;
        }
        Err(error) => {
            tracing::warn!(
                kind = ?error.error.kind,
                message = %error.error.message,
                "durable synchronization recovery failed"
            );
        }
    }
    let expiry = sync::spawn_periodic_expiry(app.clone());
    let apply_retry = sync::spawn_apply_recovery(app.clone());
    let stopping = app.store.clone();
    let served = axum::serve(listener, router(app))
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                () = test_parent_exit() => {}
            }
            // Open event streams would otherwise hold the server open.
            stopping.events().stop();
        })
        .await
        .map_err(|_| AppError::internal("HTTP server failed"));
    expiry.abort();
    apply_retry.abort();
    watch.abort();
    served
}

/// Test servers exit once the test that spawned them does: the test holds the
/// write end of stdin, which the kernel closes however the test ends.
#[cfg(debug_assertions)]
async fn test_parent_exit() {
    if std::env::var_os("MASH_TEST_EXIT_WITH_STDIN").is_none() {
        return std::future::pending().await;
    }
    let mut stdin = tokio::io::stdin();
    let mut buffer = [0; 64];
    while tokio::io::AsyncReadExt::read(&mut stdin, &mut buffer)
        .await
        .is_ok_and(|read| read > 0)
    {}
}

#[cfg(not(debug_assertions))]
async fn test_parent_exit() {
    std::future::pending::<()>().await;
}
