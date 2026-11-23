//! A shared server names itself as the owner of each group's Qdrant views, so
//! `mash qdrant prune` on the same Qdrant never takes them for leftovers.
use crate::AppError;
use crate::views::qdrant::{self as qdrant_index, ownership::Owner};

/// Mark every group this server keeps. Only a server that may write its views
/// does so; a read-only start leaves Qdrant untouched.
pub(super) async fn mark(
    views: &qdrant_index::Store,
    groups: &[String],
    writable: bool,
) -> Result<(), AppError> {
    if !writable {
        return Ok(());
    }
    let host = qdrant_index::ownership::host_name();
    for group in groups {
        views
            .record_owner(group, &Owner::server(group, host.clone()))
            .await?;
    }
    Ok(())
}
