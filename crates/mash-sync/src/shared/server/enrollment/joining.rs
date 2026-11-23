//! A join secret on its way through the server: what it is worth now,
//! redeeming it once, and the join string that carries it.
use super::{Added, Entry, State, add, hash, now, replace, update};
use crate::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// What a join secret is worth right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum JoinStatus {
    /// It can be redeemed.
    Ready,
    /// More than a day old and never redeemed.
    Expired,
    /// Already redeemed, by another machine or an earlier attempt.
    Used,
    /// The operator removed the machine or replaced its credential.
    Revoked,
    /// This server never made it.
    Unknown,
}

/// The entry a join secret belongs to and what it is worth now.
pub fn check<'a>(added: &'a Added, secret: &str, now: u64) -> (JoinStatus, Option<&'a Entry>) {
    let hashed = hash(secret);
    if let Some(entry) = added
        .nodes
        .iter()
        .find(|entry| entry.join_hash.as_deref() == Some(hashed.as_str()))
    {
        let status = match entry.state(now) {
            State::Waiting { .. } => JoinStatus::Ready,
            State::Expired => JoinStatus::Expired,
            State::Joined => JoinStatus::Used,
            State::Removed { .. } => JoinStatus::Revoked,
        };
        return (status, Some(entry));
    }
    if added.revoked.iter().any(|revoked| revoked.hash == hashed) {
        return (JoinStatus::Revoked, None);
    }
    (JoinStatus::Unknown, None)
}

/// Redeem a join secret for the token whose hash the joining machine sent.
/// It succeeds once; repeating it with the same token hash answers the same
/// again, so a machine whose first answer was lost can retry safely.
pub fn redeem(
    added: &mut Added,
    secret: &str,
    token_hash: &str,
    now: u64,
) -> Result<(String, String), JoinStatus> {
    if blake3::Hash::from_hex(token_hash).is_err() {
        return Err(JoinStatus::Unknown);
    }
    let hashed = hash(secret);
    let token_taken = added
        .nodes
        .iter()
        .any(|entry| entry.token_hash.as_deref() == Some(token_hash))
        || added
            .revoked
            .iter()
            .any(|revoked| revoked.hash == token_hash);
    let (status, _) = check(added, secret, now);
    let entry = added
        .nodes
        .iter_mut()
        .find(|entry| entry.join_hash.as_deref() == Some(hashed.as_str()));
    match (status, entry) {
        (JoinStatus::Ready, Some(entry)) if !token_taken => {
            entry.token_hash = Some(token_hash.to_owned());
            entry.joined_at = Some(now);
            entry.join_expires_at = None;
            Ok((entry.group_id.clone(), entry.node_id.clone()))
        }
        (JoinStatus::Used, Some(entry)) if entry.token_hash.as_deref() == Some(token_hash) => {
            Ok((entry.group_id.clone(), entry.node_id.clone()))
        }
        (JoinStatus::Ready, Some(_)) => Err(JoinStatus::Unknown),
        (status, _) => Err(status),
    }
}

/// The join string for a secret, at the address recorded in the file.
pub fn join_string_for(
    added: &Added,
    group_id: &str,
    node_id: &str,
    secret: &str,
) -> Result<String, AppError> {
    let api_url = added.api_url.as_deref().ok_or_else(|| {
        AppError::configuration(
            "this server has no recorded address for join strings; add a machine once with `mash-server --add-node GROUP/NODE --api-url URL`",
        )
    })?;
    crate::shared::join::Join::new(api_url, group_id, node_id, secret).encode()
}

/// Add a machine and return its join string. `api_url`, when given, is the
/// address other machines use for this server and is recorded for later
/// strings; otherwise the recorded one is used. Nothing is written unless
/// the whole string is valid.
pub fn mint(
    configuration: &Path,
    configured: &[(String, String)],
    group_id: &str,
    node_id: &str,
    api_url: Option<&str>,
) -> Result<String, AppError> {
    if let Some(api_url) = api_url {
        let placeholder = "0".repeat(64);
        let probe = crate::shared::join::Join::new(api_url, group_id, node_id, &placeholder);
        crate::shared::join::Join::decode(&probe.encode()?).map_err(|_| {
            AppError::usage(
                "the server address must be the http or https address other machines use for this server",
            )
        })?;
    }
    update(configuration, |added| {
        if let Some(api_url) = api_url {
            added.api_url = Some(api_url.trim_end_matches('/').to_owned());
        }
        if added.api_url.is_none() {
            return join_string_for(added, group_id, node_id, "");
        }
        let secret = add(added, configured, group_id, node_id, now())?;
        join_string_for(added, group_id, node_id, &secret)
    })
}

/// Replace a machine's credential and return its new join string.
pub fn reissue(configuration: &Path, group_id: &str, node_id: &str) -> Result<String, AppError> {
    update(configuration, |added| {
        let secret = replace(added, group_id, node_id, now())?;
        join_string_for(added, group_id, node_id, &secret)
    })
}
