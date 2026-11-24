//! Reading a shared server's state, and managing its machines with an admin
//! key, from a dashboard on any machine of the group.
//!
//! The status read uses this machine's own token, which can only publish,
//! search and read status. Management sends the admin key saved on this
//! machine (see [`super::admin_key`]) and never the token, so the server can
//! tell the two apart; the request also names this machine so the server's
//! record says where a change came from.
use super::server::report::{
    CheckIn, Done, GroupStatus, Heartbeat, JoinAnswer, MachineRequest, Os, Overview,
};
use crate::{AppError, config::SharedConfig};
use serde::de::DeserializeOwned;
use std::time::Duration;

fn client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| AppError::shared("could not configure HTTP client"))
}

fn bearer(secret: &str) -> Result<reqwest::header::HeaderValue, AppError> {
    let mut value = reqwest::header::HeaderValue::from_str(&format!("Bearer {secret}"))
        .map_err(|_| AppError::shared("invalid credential"))?;
    value.set_sensitive(true);
    Ok(value)
}

async fn answer<T: DeserializeOwned>(
    config: &SharedConfig,
    response: reqwest::Response,
) -> Result<T, AppError> {
    let status = response.status();
    let bytes = response.bytes().await.map_err(|_| {
        AppError::shared(format!(
            "could not reach the shared index at {}",
            config.api_url
        ))
    })?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(AppError::shared("the server's answer was too large"));
    }
    if status.is_success() {
        return serde_json::from_slice(&bytes).map_err(|_| {
            AppError::shared("the server's answer was not understood; it may run an older Mash")
        });
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::shared(format!(
            "{} does not offer this yet; it may run an older Mash than this machine",
            config.api_url
        )));
    }
    // The server's own error says what went wrong: a wrong admin key, a
    // revoked credential, a name already taken. It never holds a secret.
    match serde_json::from_slice::<AppError>(&bytes) {
        Ok(error) => Err(error),
        Err(_) => Err(AppError::shared(format!(
            "{} refused the request",
            config.api_url
        ))),
    }
}

fn unreachable(config: &SharedConfig) -> AppError {
    AppError::shared(format!(
        "could not reach the shared index at {}; check its address and the network",
        config.api_url
    ))
}

/// This machine's group as the server sees it, read with this machine's token.
pub async fn status(config: &SharedConfig) -> Result<GroupStatus, AppError> {
    let token = config.required_token()?;
    let response = client()?
        .get(format!(
            "{}/v1/status",
            config.api_url.trim_end_matches('/')
        ))
        .header(reqwest::header::AUTHORIZATION, bearer(&token)?)
        .header("x-mash-group", &config.group_id)
        .header("x-mash-node", &config.node_id)
        .send()
        .await
        .map_err(|_| unreachable(config))?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(super::client::unauthorized_answer(response).await);
    }
    answer(config, response).await
}

/// Tells the server this machine is running, and on which system, with this
/// machine's token.
pub async fn heartbeat(config: &SharedConfig) -> Result<Heartbeat, AppError> {
    let token = config.required_token()?;
    let response = client()?
        .post(format!(
            "{}/v1/heartbeat",
            config.api_url.trim_end_matches('/')
        ))
        .header(reqwest::header::AUTHORIZATION, bearer(&token)?)
        .header("x-mash-group", &config.group_id)
        .header("x-mash-node", &config.node_id)
        .json(&CheckIn { os: Os::current() })
        .send()
        .await
        .map_err(|_| unreachable(config))?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(super::client::unauthorized_answer(response).await);
    }
    answer(config, response).await
}

fn admin_request(
    config: &SharedConfig,
    key: &str,
    method: reqwest::Method,
    route: &str,
) -> Result<reqwest::RequestBuilder, AppError> {
    Ok(client()?
        .request(
            method,
            format!("{}/v1/admin/{route}", config.api_url.trim_end_matches('/')),
        )
        .header(reqwest::header::AUTHORIZATION, bearer(key)?)
        .header("x-mash-from", &config.node_id))
}

/// Every machine, the recorded actions and the admin keys, read with `key`.
pub async fn overview(config: &SharedConfig, key: &str) -> Result<Overview, AppError> {
    let response = admin_request(config, key, reqwest::Method::GET, "machines")?
        .send()
        .await
        .map_err(|_| unreachable(config))?;
    answer(config, response).await
}

/// What an admin can ask about one machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Add,
    Replace,
    Remove,
    Delete,
}

impl Change {
    fn route(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Replace => "replace",
            Self::Remove => "remove",
            Self::Delete => "delete",
        }
    }
}

/// What a change answered: a new join string, or that it was done.
pub enum Changed {
    Join(JoinAnswer),
    Done(Done),
}

/// Ask the server to change one machine, with `key`.
pub async fn change(
    config: &SharedConfig,
    key: &str,
    change: Change,
    request: &MachineRequest,
) -> Result<Changed, AppError> {
    let response = admin_request(config, key, reqwest::Method::POST, change.route())?
        .json(request)
        .send()
        .await
        .map_err(|_| unreachable(config))?;
    Ok(match change {
        Change::Add | Change::Replace => Changed::Join(answer(config, response).await?),
        Change::Remove | Change::Delete => Changed::Done(answer(config, response).await?),
    })
}
