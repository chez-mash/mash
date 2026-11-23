//! Writes from MCP clients wait their turn instead of failing: `record` waits
//! for the MCP gate and the local index's writer lock while an index pass
//! holds them, within a patience that fits common client tool timeouts.
use crate::Mash;
use mash_app::core::{self, Operation};

/// How long `record` waits for an index pass to release the local index. A
/// watcher pass holds it for roughly 5 to 25 s; this is nearly twice the
/// longest measured pass, while the wait plus a typical 1.5 s save stays
/// inside common 60 s MCP client tool timeouts (Codex's default).
pub(crate) const RECORD_PATIENCE: std::time::Duration = std::time::Duration::from_secs(45);

impl Mash {
    /// `record` waits up to `patience` for the MCP gate and the local writer
    /// lock instead of failing while an index pass runs, and stops waiting if
    /// the request is cancelled. The runtime check follows admission, so a
    /// runtime superseded during the wait still refuses the write.
    pub(crate) async fn record_within(
        &self,
        input: crate::records::Mutation,
        patience: std::time::Duration,
        cancelled: impl std::future::Future<Output = ()>,
    ) -> Result<core::Response, crate::AppError> {
        use crate::storage::admission;
        let deadline = tokio::time::Instant::now() + patience;
        tokio::pin!(cancelled);
        let _guard = tokio::select! {
            guard = tokio::time::timeout_at(deadline, self.gate.lock()) => {
                guard.map_err(|_| admission::still_busy())?
            }
            () = &mut cancelled => return Err(admission::abandoned()),
        };
        let database = self
            .paths
            .db
            .clone()
            .unwrap_or_else(core::default_database_path);
        let owner = admission::acquire_until(&database, deadline, cancelled).await?;
        self.ensure_current().await?;
        admission::scope(Some(owner), self.dispatch(Operation::RecordMutation(input))).await
    }
}
