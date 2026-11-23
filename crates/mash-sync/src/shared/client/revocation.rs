//! A credential the server once accepted and no longer does, and checking a
//! token before it is saved. Only the refusal's kind is read; the server's
//! words are not repeated to the person.
use super::{Endpoint, request_with_token, response};
use crate::shared::Freshness;
use crate::{AppError, config::SharedConfig};

/// Freshness read with a token that is not saved yet, to check a join string
/// from an earlier server before anything is written.
pub async fn freshness_with_token(
    config: &SharedConfig,
    token: &str,
) -> Result<Freshness, AppError> {
    response(
        request_with_token(config, reqwest::Method::GET, Endpoint::Freshness, token)?,
        Endpoint::Freshness,
    )
    .await
}

/// Whether a refusal says the server once accepted this credential and no
/// longer does. Only the kind is read; the server's words are not repeated.
pub(super) async fn revoked(response: reqwest::Response) -> bool {
    let Ok(bytes) = response.bytes().await else {
        return false;
    };
    bytes.len() <= 64 * 1024
        && serde_json::from_slice::<AppError>(&bytes)
            .is_ok_and(|error| error.error.kind == crate::ErrorKind::CredentialRevoked)
}

/// What a machine whose credential was removed or replaced is told.
pub fn revoked_credential() -> AppError {
    AppError::credential_revoked(
        "the shared server no longer accepts this machine's credential: its operator removed this machine or replaced its credential. Join again with a new join string, or stop sharing",
    )
}

/// Why the server refused this machine's credential: removed or replaced by
/// its operator, or never accepted.
pub(super) async fn unauthorized(response: reqwest::Response) -> AppError {
    if revoked(response).await {
        return revoked_credential();
    }
    let mut error = AppError::shared(
        "shared index authorization failed; check node token and group configuration",
    );
    error.error.retryable = false;
    error
}
