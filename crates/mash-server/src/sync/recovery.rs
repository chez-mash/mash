//! Background work that finishes or retires transfers: expiring incomplete
//! operations and recovering publications a crash or failed apply left behind.
use super::{
    App, AppError, BackfillCommit, BackfillPreparation, CatalogHandle, CommitInput, Duration,
    EXPIRY_SWEEP, INCOMPLETE_RETENTION, Identity, apply_attachment, apply_manifest, catalog,
    commit_publication, epoch_millis, parse_start, prepare_backfill, silver_store, wire,
    write_events,
};

/// Periodic reclamation of abandoned receiving transfers. Runs independently of
/// connections and restarts; the first sweep happens immediately.
pub(crate) fn spawn_expiry(
    app: App,
    period: Duration,
    retention: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            match sweep_expired(&app, retention).await {
                Ok(0) => {}
                Ok(expired) => {
                    tracing::info!(expired, "expired abandoned synchronization transfers");
                }
                Err(error) => {
                    tracing::warn!(
                        kind = ?error.error.kind,
                        message = %error.error.message,
                        "synchronization expiry sweep failed"
                    );
                }
            }
        }
    })
}

/// Production sweep interval and retention used by the server.
pub(crate) fn spawn_periodic_expiry(app: App) -> tokio::task::JoinHandle<()> {
    spawn_expiry(app, EXPIRY_SWEEP, INCOMPLETE_RETENTION)
}

/// How often completed-but-unapplied transfers are retried without a client.
pub(super) const APPLY_RETRY: Duration = Duration::from_secs(30);

/// Retry durable apply work so a queued row never waits for a restart or a
/// reconnect. Idempotent with startup recovery and with a client resending
/// `FinishUpload`.
pub(crate) fn spawn_apply_recovery(app: App) -> tokio::task::JoinHandle<()> {
    spawn_apply_retry(app, APPLY_RETRY)
}

/// Apply retry with an explicit period, used by tests.
pub(crate) fn spawn_apply_retry(app: App, period: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Startup recovery already covered the initial tick.
        interval.tick().await;
        loop {
            interval.tick().await;
            match recover(&app).await {
                Ok(0) => {}
                Ok(resumed) => {
                    tracing::info!(resumed, "retried durable synchronization work");
                }
                Err(error) => {
                    tracing::warn!(
                        kind = ?error.error.kind,
                        message = %error.error.message,
                        "synchronization apply retry failed"
                    );
                }
            }
        }
    })
}

pub(super) async fn sweep_expired(app: &App, retention: Duration) -> Result<usize, AppError> {
    let handle: CatalogHandle = app.catalog.clone();
    let cutoff =
        epoch_millis().saturating_sub(i64::try_from(retention.as_millis()).unwrap_or(i64::MAX));
    catalog(&handle, move |catalog| catalog.expire_incomplete(cutoff)).await
}

/// Resume operations that reached `queued_for_apply`, `applying` or
/// `views_pending` before the process stopped. Runs before the listener serves
/// requests, so a client resume cannot race the same operation.
///
/// A views failure leaves the row in `views_pending`; a retry re-applies the
/// retained pieces. Silver duplicate detection makes that a views-only repair,
/// and the receipt on the silver head proves whether history committed.
pub(crate) async fn recover(app: &App) -> Result<usize, AppError> {
    let handle: CatalogHandle = app.catalog.clone();
    let cutoff = epoch_millis()
        .saturating_sub(i64::try_from(INCOMPLETE_RETENTION.as_millis()).unwrap_or(i64::MAX));
    let expired = catalog(&handle, move |catalog| catalog.expire_incomplete(cutoff)).await?;
    if expired > 0 {
        tracing::info!(expired, "expired abandoned synchronization transfers");
    }
    let pending = catalog(&handle, crate::shared::sync::catalog::Catalog::recoverable).await?;
    let mut resumed = 0usize;
    for row in pending {
        let Some(identity) = app
            .identities
            .values()
            .find(|identity| identity.group_id == row.group_id && identity.node_id == row.node_id)
            .cloned()
        else {
            tracing::warn!(
                operation = %row.operation_id,
                "durable synchronization operation belongs to an unconfigured node"
            );
            continue;
        };
        let loaded = catalog(&handle, {
            let operation_id = row.operation_id.clone();
            let group = row.group_id.clone();
            let node = row.node_id.clone();
            move |catalog| catalog.recover_pieces(&operation_id, &group, &node)
        })
        .await;
        let (operation, pieces) = match loaded {
            Ok(loaded) => loaded,
            Err(error) => {
                tracing::warn!(
                    operation = %row.operation_id,
                    message = %error.error.message,
                    "durable synchronization operation has no retained pieces"
                );
                continue;
            }
        };
        let result = match operation.kind.as_str() {
            "publication" => {
                recover_publication(app, &identity, &handle, &operation, &pieces).await
            }
            "backfill_manifest" => apply_manifest(app, &identity, &handle, &operation, &pieces)
                .await
                .map(|_| ()),
            "attachment" => apply_attachment(app, &identity, &handle, &operation, &pieces)
                .await
                .map(|_| ()),
            other => {
                tracing::warn!(
                    operation = %row.operation_id,
                    kind = other,
                    "durable synchronization operation has an unknown kind"
                );
                continue;
            }
        };
        match result {
            Ok(()) => {
                resumed = resumed.saturating_add(1);
                tracing::info!(
                    operation = %row.operation_id,
                    kind = %row.kind,
                    "resumed durable synchronization operation"
                );
            }
            Err(error) => {
                tracing::warn!(
                    operation = %row.operation_id,
                    kind = %row.kind,
                    message = %error.error.message,
                    "durable synchronization operation could not be resumed"
                );
            }
        }
    }
    Ok(resumed)
}

pub(super) async fn recover_publication(
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation: &catalog::Operation,
    pieces: &[wire::Piece],
) -> Result<(), AppError> {
    let max_decoded = usize::try_from(app.store.max_node_bytes()).unwrap_or(usize::MAX);
    let events = wire::decode_publication(pieces, max_decoded)
        .map_err(|_| AppError::internal("retained publication bytes are invalid"))?;
    if wire::record_count(&events) != operation.record_count {
        return Err(AppError::internal(
            "retained publication record count does not match its manifest",
        ));
    }
    let (start, backfill_id) = parse_start(&operation.start_json)?;
    let (start, events, backfill) = match backfill_id {
        Some(backfill_id) => {
            match prepare_backfill(app, identity, handle, &backfill_id, start, events).await? {
                BackfillPreparation::Superseded { members, revision } => {
                    let group = identity.group_id.clone();
                    let node = identity.node_id.clone();
                    let covered = catalog(handle, {
                        let backfill_id = backfill_id.clone();
                        move |catalog| catalog.cover(&backfill_id, &group, &node, &members)
                    })
                    .await?;
                    let _ = covered;
                    let receipt =
                        silver_store::receipt_digest(&operation.manifest_digest, revision);
                    let operation_id = operation.operation_id.clone();
                    let _ = catalog(handle, move |catalog| {
                        catalog.published(&operation_id, revision, &receipt)
                    })
                    .await;
                    return Ok(());
                }
                BackfillPreparation::Apply {
                    start,
                    events,
                    members,
                } => (
                    start,
                    events,
                    Some(BackfillCommit {
                        backfill_id,
                        members,
                    }),
                ),
            }
        }
        None => (start, events, None),
    };
    let file = write_events(start, &events)?;
    match commit_publication(
        app,
        identity,
        handle,
        operation,
        &events,
        CommitInput {
            file,
            backfill,
            silver: None,
        },
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(failure) => Err(failure.error),
    }
}
