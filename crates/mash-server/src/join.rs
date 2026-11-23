//! Join strings on the wire: checking one before anything is saved, and
//! redeeming it once. These are the only routes without a bearer token; the
//! one-time secret in the body is the credential. Neither route repeats the
//! secret or logs it, and neither says anything about a group to someone
//! whose secret cannot be redeemed.
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use mash_app::shared::join::{CheckAnswer, CheckRequest, Published, RedeemAnswer, RedeemRequest};
use mash_app::shared::server::{configuration, enrollment};
use mash_model::AppError;

use super::{ApiError, App, json};

/// At most this many join requests are answered at once.
pub(crate) const CONCURRENT: usize = 4;

fn not_offered() -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        AppError::usage("this server does not take join strings"),
    )
}

fn valid_secret(secret: &str) -> Result<(), ApiError> {
    if secret.len() == 64 && secret.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(AppError::usage("invalid join request").into())
    }
}

fn admitted(app: &App) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
    app.joins.clone().try_acquire_owned().map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::shared("the server is answering other join requests; try again"),
        )
    })
}

/// What a join string is worth, without using it.
pub(crate) async fn check(
    State(app): State<App>,
    input: Result<Json<CheckRequest>, JsonRejection>,
) -> Result<Json<CheckAnswer>, ApiError> {
    let CheckRequest { secret } = json(input)?;
    valid_secret(&secret)?;
    let live = app.live.clone().ok_or_else(not_offered)?;
    let _permit = admitted(&app)?;
    let path = live.configuration.clone();
    let config = configuration::load(&path).await?;
    let now = enrollment::now();
    let (status, entry) = enrollment::check(configuration::added(&config), &secret, now);
    let mut answer = CheckAnswer {
        status,
        expires_in_seconds: None,
        machines: Vec::new(),
        published: None,
    };
    let Some(entry) = entry.filter(|_| status == enrollment::JoinStatus::Ready) else {
        return Ok(Json(answer));
    };
    answer.expires_in_seconds = entry.join_expires_at.map(|at| at.saturating_sub(now));
    answer.machines = configuration::members(&config)
        .into_iter()
        .filter(|member| member.group_id == entry.group_id && member.node_id != entry.node_id)
        .filter(|member| {
            // A removed machine is no longer part of the group to the reader.
            !configuration::added(&config).nodes.iter().any(|added| {
                added.group_id == member.group_id
                    && added.node_id == member.node_id
                    && added.removed_at.is_some()
            })
        })
        .map(|member| member.node_id)
        .collect();
    if app.store.serves(&entry.group_id) {
        let identity = configuration::member(&entry.group_id, &entry.node_id);
        answer.published = app
            .store
            .published(&identity)
            .await?
            .map(|published| Published {
                instance: published.instance,
                chats: published.chats,
            });
    }
    Ok(Json(answer))
}

/// Redeem a join string for the token whose hash the machine sent.
pub(crate) async fn redeem(
    State(app): State<App>,
    input: Result<Json<RedeemRequest>, JsonRejection>,
) -> Result<Json<RedeemAnswer>, ApiError> {
    let RedeemRequest { secret, token_hash } = json(input)?;
    valid_secret(&secret)?;
    if blake3::Hash::from_hex(&token_hash).is_err() {
        return Err(AppError::usage("invalid join request").into());
    }
    let live = app.live.clone().ok_or_else(not_offered)?;
    let _permit = admitted(&app)?;
    let path = live.configuration.clone();
    // A machine in a group this server does not serve yet cannot publish
    // until a restart, so its string is not used up before then.
    let now = enrollment::now();
    let probe = enrollment::read(&path)?;
    if let (enrollment::JoinStatus::Ready, Some(entry)) = enrollment::check(&probe, &secret, now)
        && !app.store.serves(&entry.group_id)
    {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::shared("the server must restart before machines can join this group"),
        ));
    }
    let outcome = mash_app::blocking(move || {
        enrollment::update(&path, |added| {
            Ok(enrollment::redeem(added, &secret, &token_hash, now))
        })
    })
    .await?;
    let status = match outcome {
        Ok((group, node)) => {
            // Accept the new token now rather than at the next reread.
            super::roster::refresh(&app, &live, true).await?;
            tracing::info!(group = %group, node = %node, "a machine joined with a join string");
            enrollment::JoinStatus::Ready
        }
        Err(status) => status,
    };
    Ok(Json(RedeemAnswer { status }))
}
