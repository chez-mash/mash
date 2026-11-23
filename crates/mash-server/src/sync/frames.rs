//! Sending frames to a connected machine, and the refusal one gets when its
//! credential stopped working while it was connected.
use super::{AppError, Message, Sink};
use crate::shared::sync::wire::{self, CommitState, Failed, ServerFrame, server_frame};
use futures_util::SinkExt;

pub(super) async fn send(
    sink: &mut Sink,
    operation_id: &str,
    body: server_frame::Body,
) -> Result<(), AppError> {
    let frame = ServerFrame {
        operation_id: operation_id.to_owned(),
        body: Some(body),
    };
    let bytes = wire::encode_server(&frame)?;
    sink.send(Message::Binary(bytes.into()))
        .await
        .map_err(|_| AppError::shared("synchronization connection failed"))
}

/// Tell a machine removed or given a new credential while connected that it
/// must join again; nothing it sends afterwards is applied.
pub(super) async fn refuse_revoked(sink: &mut Sink, operation_id: &str) -> Result<(), AppError> {
    send(
        sink,
        operation_id,
        server_frame::Body::Failed(Failed {
            phase: "receiving".into(),
            code: "credential_revoked".into(),
            commit_state: CommitState::Unknown as i32,
            retryable: false,
            recovery_action: "join again with a new join string".into(),
            message: "this server no longer accepts this machine's credential".into(),
            kind: crate::ErrorKind::CredentialRevoked.name(),
        }),
    )
    .await
}
