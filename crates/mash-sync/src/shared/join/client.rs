//! Asking the server about a join string, and redeeming it. Both requests
//! carry the secret in their body, never in the address, and carry no
//! bearer token: the secret is the only credential this machine has yet.
use super::{Credential, Join};
use crate::AppError;
use crate::shared::server::enrollment::JoinStatus;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckRequest {
    pub secret: String,
}

/// What the server says about a join secret, before anything is saved. The
/// group's machines and the name's history are told only for a string that
/// can still be redeemed.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CheckAnswer {
    pub status: JoinStatus,
    /// Seconds until the string expires, by the server's own clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in_seconds: Option<u64>,
    /// The other machines of the group, by name.
    #[serde(default)]
    pub machines: Vec<String>,
    /// What the server already holds under this machine's name, if anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<Published>,
}

/// History the server already holds under a name.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Published {
    /// The local index that published it; only that index may publish more.
    pub instance: String,
    pub chats: u64,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RedeemRequest {
    pub secret: String,
    /// BLAKE3 of the token the joining machine made for itself.
    pub token_hash: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RedeemAnswer {
    /// `ready` when the machine joined; otherwise why it could not.
    pub status: JoinStatus,
}

fn client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|_| AppError::shared("could not configure HTTP client"))
}

fn unreachable(join: &Join) -> AppError {
    AppError::shared(format!("could not reach {}", join.api_url))
}

async fn post<I: Serialize, O: serde::de::DeserializeOwned>(
    join: &Join,
    route: &str,
    input: &I,
) -> Result<O, AppError> {
    let response = client()?
        .post(format!("{}/v1/{route}", join.api_url))
        .json(input)
        .send()
        .await
        .map_err(|_| unreachable(join))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::shared(format!(
            "{} does not check join strings; it may run an older Mash than this machine",
            join.api_url
        )));
    }
    if !response.status().is_success() {
        return Err(AppError::shared(format!(
            "{} refused the join check; try again later",
            join.api_url
        )));
    }
    let bytes = response.bytes().await.map_err(|_| unreachable(join))?;
    if bytes.len() > 256 * 1024 {
        return Err(AppError::shared("the server's answer was too large"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| AppError::shared("the server's answer was not understood"))
}

/// Ask the server what a join string is worth, without saving or using it.
/// A string from an earlier server carries a token, which is checked by
/// reading the group's freshness with it.
pub async fn check(join: &Join) -> Result<CheckAnswer, AppError> {
    match &join.credential {
        Credential::Secret(secret) => {
            post(
                join,
                "join/check",
                &CheckRequest {
                    secret: secret.clone(),
                },
            )
            .await
        }
        Credential::Token(token) => {
            let config = crate::config::SharedConfig {
                group_id: join.group_id.clone(),
                node_id: join.node_id.clone(),
                api_url: join.api_url.clone(),
                token_env: String::new(),
                publication_redaction: crate::redaction::Policy::default(),
                unshared: Vec::new(),
            };
            let freshness = crate::shared::client::freshness_with_token(&config, token).await;
            match freshness {
                Ok(freshness) => {
                    let mut machines = freshness
                        .nodes
                        .iter()
                        .map(|node| node.node_id.clone())
                        .chain(freshness.unpublished_nodes.clone().unwrap_or_default())
                        .filter(|node| *node != join.node_id)
                        .collect::<Vec<_>>();
                    machines.sort();
                    machines.dedup();
                    let published = freshness
                        .nodes
                        .iter()
                        .find(|node| node.node_id == join.node_id)
                        .map(|node| Published {
                            instance: node.instance.clone(),
                            chats: u64::try_from(node.chats).unwrap_or_default(),
                        });
                    Ok(CheckAnswer {
                        status: JoinStatus::Ready,
                        expires_in_seconds: None,
                        machines,
                        published,
                    })
                }
                Err(error) if error.error.kind == crate::ErrorKind::CredentialRevoked => {
                    Ok(CheckAnswer {
                        status: JoinStatus::Revoked,
                        expires_in_seconds: None,
                        machines: Vec::new(),
                        published: None,
                    })
                }
                Err(error) => Err(error),
            }
        }
    }
}

/// Redeem a join string and return the token this machine will use. For a
/// current string the token is made here and only its hash is sent; the
/// server accepts it once. A refusal comes back as its status.
pub async fn redeem(join: &Join) -> Result<Result<String, JoinStatus>, AppError> {
    let secret = match &join.credential {
        Credential::Token(token) => return Ok(Ok(token.clone())),
        Credential::Secret(secret) => secret.clone(),
    };
    let token = crate::shared::server::enrollment::secret()?;
    let answer: RedeemAnswer = post(
        join,
        "join",
        &RedeemRequest {
            secret,
            token_hash: blake3::hash(token.as_bytes()).to_hex().to_string(),
        },
    )
    .await?;
    Ok(match answer.status {
        JoinStatus::Ready => Ok(token),
        status => Err(status),
    })
}

/// What a refused join string means, in words for the person joining.
pub fn refusal(status: JoinStatus) -> AppError {
    let message = match status {
        JoinStatus::Ready => "the server accepted this join string",
        JoinStatus::Expired => {
            "this join string has expired: each one works for 24 hours. Ask for a new one on the machine that runs the server. Nothing was saved"
        }
        JoinStatus::Used => {
            "this join string was already used: each one works once, on one computer. Ask for a new one on the machine that runs the server. Nothing was saved"
        }
        JoinStatus::Revoked => {
            "the server's operator replaced this join string or removed its machine. Ask for a new one on the machine that runs the server. Nothing was saved"
        }
        JoinStatus::Unknown => {
            "the server did not make this join string, or the machine it was for was deleted. Ask for a new one on the machine that runs the server. Nothing was saved"
        }
    };
    AppError::usage(message)
}

/// The local index that would publish under the name, compared with the
/// one the server already holds history from.
pub fn conflicting_owner(
    join: &Join,
    published: Option<&Published>,
    local: Option<&str>,
) -> Option<AppError> {
    let published = published?;
    if local == Some(published.instance.as_str()) {
        return None;
    }
    Some(AppError::conflict(format!(
        "another computer already publishes as {}: the server holds {} chats from another index under that name. Each join string is for one computer; ask the server's operator for a join string with a new name. If this computer is {} reinstalled, restore its index from a backup to keep the name, or ask the operator to delete {}'s published chats first. Nothing was saved",
        join.node_id, published.chats, join.node_id, join.node_id
    )))
}

/// The instance of the local index at `silver`, which the server ties a
/// name's history to. None when there is no index yet.
pub fn local_instance(silver: &std::path::Path) -> Result<Option<String>, AppError> {
    if !silver.exists() {
        return Ok(None);
    }
    let connection =
        rusqlite::Connection::open_with_flags(silver, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(AppError::database)?;
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = 'shared_publication')",
            [],
            |row| row.get(0),
        )
        .map_err(AppError::database)?;
    if !exists {
        return Ok(None);
    }
    connection
        .query_row(
            "SELECT instance FROM shared_publication WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            error => Err(AppError::database(error)),
        })
}

/// Check a join string, then redeem it: refused strings, and names whose
/// history belongs to another index, come back as errors before anything is
/// used up. `local` is this machine's index instance, if it has one.
pub async fn join(join: &Join, local: Option<&str>) -> Result<String, AppError> {
    let answer = check(join).await?;
    if answer.status != JoinStatus::Ready {
        return Err(refusal(answer.status));
    }
    if let Some(error) = conflicting_owner(join, answer.published.as_ref(), local) {
        return Err(error);
    }
    redeem(join).await?.map_err(refusal)
}
