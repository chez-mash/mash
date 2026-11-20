//! Attachment transfers: begin, apply and finish one attachment's bytes.
use super::{
    App, AppError, AttachmentStart, AttachmentStored, Availability, CatalogHandle, Identity,
    Metadata, OwnedBegin, Sink, begin_operation, catalog, reject, send, server_frame, wire,
};

pub(super) async fn handle_begin_attachment(
    sink: &mut Sink,
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation_id: &str,
    begin: wire::BeginAttachment,
) -> Result<(), AppError> {
    let Some(attachment) = begin.attachment else {
        return reject(
            sink,
            operation_id,
            "begin",
            "invalid_attachment",
            "attachment metadata is required",
        )
        .await;
    };
    let Ok(metadata) = Metadata::try_from(attachment) else {
        return reject(
            sink,
            operation_id,
            "begin",
            "invalid_attachment",
            "attachment metadata is invalid",
        )
        .await;
    };
    if begin.message_id.is_empty()
        || metadata.id.is_empty()
        || metadata.availability != Availability::Retained
        || metadata.digest.as_deref().is_none_or(|digest| {
            digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        })
        || metadata.bytes.is_none()
        || begin.decoded_bytes != metadata.bytes.unwrap_or_default()
        || begin.decoded_bytes
            > app
                .store
                .max_node_bytes()
                .min(crate::files::MAX_BYTES as u64)
        || begin.manifest_digest.len() != 64
        || !begin.manifest_digest.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return reject(
            sink,
            operation_id,
            "begin",
            "invalid_attachment",
            "attachment metadata or size is invalid",
        )
        .await;
    }
    let start_json = serde_json::json!({
        "message_id": begin.message_id,
        "attachment": metadata,
    })
    .to_string();
    let owned = OwnedBegin {
        operation_id: operation_id.to_owned(),
        group_id: identity.group_id.clone(),
        node_id: identity.node_id.clone(),
        kind: "attachment".into(),
        manifest_digest: begin.manifest_digest,
        decoded_bytes: begin.decoded_bytes,
        record_count: 1,
        start_json,
    };
    begin_operation(sink, handle, operation_id, owned).await
}

/// Authorize and durably store attachment bytes. Shared by the socket handler
/// and recovery.
pub(super) async fn apply_attachment(
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation: &catalog::Operation,
    pieces: &[wire::Piece],
) -> Result<String, AppError> {
    let start: AttachmentStart = serde_json::from_str(&operation.start_json)
        .map_err(|_| AppError::internal("attachment metadata is not valid"))?;
    let max_decoded = crate::files::MAX_BYTES
        .min(usize::try_from(app.store.max_node_bytes()).unwrap_or(usize::MAX));
    let bytes = wire::decode_bytes(pieces, max_decoded)
        .map_err(|_| AppError::usage("attachment bytes were not valid"))?;
    let digest = start
        .attachment
        .digest
        .clone()
        .ok_or_else(|| AppError::usage("attachment digest is missing"))?;
    let request = crate::files::Request {
        node_id: identity.node_id.clone(),
        message_id: start.message_id.clone(),
        attachment_id: Some(start.attachment.id.clone()),
    };
    let known = crate::attachments::known(app, identity, &request)
        .await
        .map_err(|crate::ApiError(_, error)| error)?;
    if !known.iter().any(|metadata| metadata == &start.attachment) {
        return Err(AppError::usage(
            "upload is not linked to this node's published message",
        ));
    }
    let root = crate::attachments::directory(app, identity, &identity.node_id)
        .ok_or_else(|| AppError::usage("shared attachment storage is disabled"))?;
    let metadata = start.attachment;
    let write_metadata = metadata.clone();
    crate::blocking(move || crate::files::write_bytes(&root, &write_metadata, &bytes)).await?;
    let operation_id = operation.operation_id.clone();
    let receipt = digest.clone();
    let _ = catalog(handle, move |catalog| {
        catalog.completed(&operation_id, &receipt)
    })
    .await;
    Ok(digest)
}

pub(super) async fn finish_attachment(
    sink: &mut Sink,
    app: &App,
    identity: &Identity,
    handle: &CatalogHandle,
    operation_id: &str,
    operation: &catalog::Operation,
    pieces: &[wire::Piece],
) -> Result<(), AppError> {
    match apply_attachment(app, identity, handle, operation, pieces).await {
        Ok(digest) => {
            send(
                sink,
                operation_id,
                server_frame::Body::AttachmentStored(AttachmentStored { digest }),
            )
            .await
        }
        Err(error) => {
            reject(
                sink,
                operation_id,
                "applying",
                "attachment_rejected",
                &error.error.message,
            )
            .await
        }
    }
}
