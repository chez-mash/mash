//! A join string: everything a new machine needs to publish to and search a
//! shared index, made by the server for one machine and pasted into the
//! dashboard's Join a shared index or `mash init --join`.
//!
//! A current string carries a one-time secret, not a token. The joining
//! machine makes its own token and redeems the secret with the token's hash
//! (see `server::enrollment`), so the string works once and only for a day.
//! Strings made by earlier servers carry the token itself and still join.
//! Either way the string is a secret: errors never echo it, and it travels
//! only in request bodies, never in an address.
use crate::AppError;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

mod client;
pub use client::{
    CheckAnswer, CheckRequest, Published, RedeemAnswer, RedeemRequest, check, conflicting_owner,
    join, local_instance, redeem, refusal,
};

const PREFIX: &str = "mash-join.";

/// What the string lets a machine do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// The node's token itself, from a server older than single-use strings.
    Token(String),
    /// A one-time secret to redeem for a token this machine makes.
    Secret(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join {
    pub api_url: String,
    pub group_id: String,
    pub node_id: String,
    pub credential: Credential,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    /// Format version; a newer server's string is refused, not guessed at.
    v: u8,
    api_url: String,
    group_id: String,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secret: Option<String>,
}

fn printable(value: &str) -> bool {
    (32..=4096).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_graphic())
}

impl Join {
    /// A current string, carrying a one-time secret.
    pub fn new(api_url: &str, group_id: &str, node_id: &str, secret: &str) -> Self {
        Self {
            api_url: api_url.trim_end_matches('/').to_owned(),
            group_id: group_id.to_owned(),
            node_id: node_id.to_owned(),
            credential: Credential::Secret(secret.to_owned()),
        }
    }

    pub fn encode(&self) -> Result<String, AppError> {
        let (v, token, secret) = match &self.credential {
            Credential::Token(token) => (1, Some(token.clone()), None),
            Credential::Secret(secret) => (2, None, Some(secret.clone())),
        };
        let wire = Wire {
            v,
            api_url: self.api_url.clone(),
            group_id: self.group_id.clone(),
            node_id: self.node_id.clone(),
            token,
            secret,
        };
        let json =
            serde_json::to_vec(&wire).map_err(|_| AppError::internal("encode join string"))?;
        Ok(format!("{PREFIX}{}", URL_SAFE_NO_PAD.encode(json)))
    }

    /// Parse and validate a pasted string. Errors never echo it: it is a secret.
    pub fn decode(text: &str) -> Result<Self, AppError> {
        let invalid = || {
            AppError::usage(
                "that is not a complete Mash join string; copy the whole `mash-join.…` value the server made",
            )
        };
        let encoded = text.trim().strip_prefix(PREFIX).ok_or_else(invalid)?;
        let json = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| invalid())?;
        let wire: Wire = serde_json::from_slice(&json).map_err(|_| invalid())?;
        if wire.v > 2 {
            return Err(AppError::usage(
                "this join string was made by a newer Mash; upgrade this machine first",
            ));
        }
        crate::config::validate_identifier(&wire.group_id, "group_id").map_err(|_| invalid())?;
        crate::config::validate_identifier(&wire.node_id, "node_id").map_err(|_| invalid())?;
        let url = reqwest::Url::parse(&wire.api_url).map_err(|_| invalid())?;
        if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
            return Err(invalid());
        }
        let credential = match (wire.v, wire.token, wire.secret) {
            (1, Some(token), None) if printable(&token) => Credential::Token(token),
            (2, None, Some(secret))
                if secret.len() == 64 && secret.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
            {
                Credential::Secret(secret)
            }
            _ => return Err(invalid()),
        };
        Ok(Self {
            api_url: wire.api_url.trim_end_matches('/').to_owned(),
            group_id: wire.group_id,
            node_id: wire.node_id,
            credential,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";
    const SECRET: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn a_join_string_round_trips_and_trims_the_address() {
        let join = Join::new("https://mash.example.net/", "personal", "laptop", SECRET);
        let text = join.encode().unwrap();
        assert!(text.starts_with("mash-join."));
        assert!(
            !text.contains(SECRET),
            "the secret is not readable at a glance"
        );
        let decoded = Join::decode(&format!("  {text}\n")).unwrap();
        assert_eq!(decoded, join);
        assert_eq!(decoded.api_url, "https://mash.example.net");
        // A string from an earlier server carries the token itself.
        let old = Join {
            credential: Credential::Token(TOKEN.to_owned()),
            ..join
        };
        assert_eq!(Join::decode(&old.encode().unwrap()).unwrap(), old);
    }

    #[test]
    fn anything_else_is_refused_without_echoing_it() {
        let encode = |value: serde_json::Value| {
            format!("mash-join.{}", URL_SAFE_NO_PAD.encode(value.to_string()))
        };
        let good = serde_json::json!({"v":1,"api_url":"https://h","group_id":"g","node_id":"n","token":TOKEN});
        assert!(Join::decode(&encode(good.clone())).is_ok());
        let mut cases = vec![
            String::new(),
            "mash-join.".to_owned(),
            "mash-join.not base64!".to_owned(),
            encode(serde_json::json!({"v":1})),
        ];
        for (field, value) in [
            ("api_url", serde_json::json!("ftp://h")),
            ("api_url", serde_json::json!("not a url")),
            ("group_id", serde_json::json!("has space")),
            ("node_id", serde_json::json!("")),
            ("token", serde_json::json!("short")),
            ("token", serde_json::json!(format!("{TOKEN} with space"))),
            ("v", serde_json::json!(3)),
            ("v", serde_json::json!(2)),
            ("secret", serde_json::json!(SECRET)),
            ("extra", serde_json::json!(true)),
        ] {
            let mut changed = good.clone();
            changed[field] = value;
            cases.push(encode(changed));
        }
        for case in cases {
            let error = Join::decode(&case).unwrap_err();
            assert_eq!(error.error.kind, crate::ErrorKind::Usage, "{case}");
            assert!(!error.error.message.contains(TOKEN));
            assert!(!error.error.message.contains(SECRET));
        }
    }
}
