//! Chat catalogue routes: a filtered page for `list_chats`, and the whole
//! catalogue a dashboard folds and filters itself.
use axum::Json;
use axum::extract::{Extension, State, rejection::JsonRejection};
use axum::http::HeaderMap;

use super::{ApiError, App, Identity, json, protocol, store};

/// Whether the client reads subagent markers in catalogue rows. Older clients
/// reject a row with fields they do not know, so they get rows without them.
fn reads_markers(headers: &HeaderMap) -> bool {
    headers
        .get(protocol::SUBAGENT_MARKERS_HEADER)
        .is_some_and(|value| value == "1")
}

pub(super) async fn list_chats(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    headers: HeaderMap,
    input: Result<Json<crate::chat_catalog::Request>, JsonRejection>,
) -> Result<Json<crate::chat_catalog::Page>, ApiError> {
    let input = json(input)?;
    input.validate()?;
    let mut page = app.store.list_chats(&id, input).await?;
    if !reads_markers(&headers) {
        store::without_subagents(&mut page.chats);
    }
    for thread in &mut page.chats {
        thread.metadata.redact(app.store.responses());
    }
    Ok(Json(page))
}

pub(super) async fn catalog(
    State(app): State<App>,
    Extension(id): Extension<Identity>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::chat_catalog::Chat>>, ApiError> {
    let mut chats = app
        .store
        .catalog(
            &id,
            &crate::chat_catalog::Request {
                exclude_self: true,
                ..Default::default()
            },
        )
        .await?;
    chats.sort_by(|a, b| {
        b.metadata
            .updated_at
            .cmp(&a.metadata.updated_at)
            .then_with(|| a.node_id.cmp(&b.node_id))
            .then_with(|| a.chat_id.cmp(&b.chat_id))
    });
    // One extra row lets the dashboard report an incomplete catalogue.
    chats.truncate(10_001);
    if !reads_markers(&headers) {
        store::without_subagents(&mut chats);
    }
    for chat in &mut chats {
        // The dashboard is this endpoint's only reader and shows one line of
        // each title and no provenance, so neither is masked or sent.
        chat.metadata.enrichment = None;
        if let Some(title) = &mut chat.metadata.title {
            crate::chat_catalog::headline(title);
        }
        chat.metadata.redact(app.store.responses());
    }
    Ok(Json(chats))
}
