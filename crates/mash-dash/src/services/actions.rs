//! The service action route and the history it leaves, on this machine or
//! forwarded to the peer the request names.
//!
//! A forwarded action is trusted only when the peer echoes the exact request
//! back; otherwise its outcome is reported as unverified rather than retried.
//! Locally, a plugin reinstall runs from the Mash executable on disk and is
//! remembered in memory, while a supervisor action is guarded by a durable
//! receipt, so the history merges both, newest first.
use super::{MachineSelection, decode, error, model, operations, plugin};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

pub(super) async fn action(
    State(state): State<Arc<super::super::Dash>>,
    headers: HeaderMap,
    input: Result<axum::Json<operations::ActionRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Ok(axum::Json(request)) = input else {
        return error(StatusCode::BAD_REQUEST, "Invalid service action request");
    };
    if !operations::valid_id(&request.request_id) {
        return error(StatusCode::BAD_REQUEST, "Invalid operation request ID");
    }
    if request.machine != state.services.local_id() {
        if headers.contains_key("x-mash-service-forwarded") {
            return error(StatusCode::CONFLICT, "Remote machine identity mismatch");
        }
        let Some(peer) = state
            .services
            .peers
            .iter()
            .find(|p| p.id == request.machine)
        else {
            return error(StatusCode::NOT_FOUND, "Unknown machine");
        };
        let mut call = state
            .services
            .client
            .post(format!("{}/api/services/action", peer.url))
            .header("x-mash-service-forwarded", "1");
        if let Some(token) = &peer.token {
            call = call.bearer_auth(token);
        }
        let result = call.json(&request).send().await;
        return match result {
            Ok(response) => match decode::<operations::Operation>(response).await {
                Ok(operation) if operation.request == request => {
                    axum::Json(operation).into_response()
                }
                _ => error(
                    StatusCode::BAD_GATEWAY,
                    "Remote action outcome could not be verified. Check operation history and status; do not retry blindly.",
                ),
            },
            Err(_) => error(
                StatusCode::BAD_GATEWAY,
                "Remote action outcome is unknown. Check operation history and status; do not retry blindly.",
            ),
        };
    }
    if request.action == model::Action::ReinstallPlugin {
        return plugin::reinstall(&state, request);
    }
    let Some(service) = state
        .services
        .config
        .as_ref()
        .and_then(|c| {
            c.services
                .iter()
                .find(|s| format!("managed-{}", s.id) == request.service)
        })
        .cloned()
    else {
        return error(
            StatusCode::NOT_FOUND,
            "This service is not a configured control target",
        );
    };
    let Some(directory) = state.services.directory.clone() else {
        return error(StatusCode::FORBIDDEN, "Service control is not configured");
    };
    // The operation survives a disconnected browser request. Its durable receipt
    // makes an interrupted server process distinguishable from a safe retry.
    match tokio::spawn(operations::execute(directory, service, request)).await {
        Ok(Ok(operation)) => axum::Json(operation).into_response(),
        Ok(Err(message)) => error(StatusCode::CONFLICT, &message),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Operation worker stopped; check history before acting again",
        ),
    }
}

pub(super) async fn history(
    State(state): State<Arc<super::super::Dash>>,
    headers: HeaderMap,
    Query(selection): Query<MachineSelection>,
) -> Response {
    if selection.machine != state.services.local_id() {
        if headers.contains_key("x-mash-service-forwarded") {
            return error(StatusCode::CONFLICT, "Remote machine identity mismatch");
        }
        let Some(peer) = state
            .services
            .peers
            .iter()
            .find(|p| p.id == selection.machine)
        else {
            return error(StatusCode::NOT_FOUND, "Unknown machine");
        };
        return match state
            .services
            .remote::<Vec<operations::Operation>>(
                peer,
                "/api/services/operations",
                &[("machine", &selection.machine)],
            )
            .await
        {
            Ok(records) => axum::Json(records).into_response(),
            Err(message) => error(StatusCode::BAD_GATEWAY, &message),
        };
    }
    let mut records = state.services.plugin_runs.history();
    if let Some(directory) = state.services.directory.clone() {
        match tokio::task::spawn_blocking(move || operations::history(&directory)).await {
            Ok(Ok(receipts)) => records.extend(receipts),
            _ => {
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not read operation history",
                );
            }
        }
    }
    records.sort_by_key(|record| std::cmp::Reverse(record.requested_at));
    axum::Json(records).into_response()
}
