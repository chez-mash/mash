//! Admin keys: the credential that lets a dashboard on any machine manage a
//! server's machines, and the record of what each key did.
//!
//! An admin key is separate from every machine's token. A machine's token
//! lets it publish and search its group and nothing else; only an admin key
//! adds, removes or replaces machines, or deletes what one published. The
//! server keeps admin keys in the added-machines file as BLAKE3 hashes, like
//! join secrets, so the file never holds one. A key is shown once, when it is
//! made, and starts with `mash-admin.` so it cannot be mistaken for a join
//! string or a token.
use super::enrollment::Added;
use crate::AppError;
use serde::{Deserialize, Serialize};

const PREFIX: &str = "mash-admin.";
/// How many admin actions the server remembers.
const REMEMBERED_ACTIONS: usize = 200;
/// How many admin keys one server may hold.
const MAX_KEYS: usize = 32;

/// One admin key, by name.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdminKey {
    pub name: String,
    /// BLAKE3 of the whole key, prefix included.
    pub hash: String,
    pub created_at: u64,
}

/// One admin action, kept so the operator can see who changed what.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub at: u64,
    /// The admin key's name.
    pub key: String,
    /// The machine the request said it came from; told by the caller, so it
    /// names, it does not prove.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// `add`, `replace`, `remove` or `delete`.
    pub action: String,
    /// `GROUP/NODE`.
    pub target: String,
}

fn hash(key: &str) -> String {
    blake3::hash(key.as_bytes()).to_hex().to_string()
}

/// Whether text has the shape of an admin key, before any lookup.
pub fn well_formed(key: &str) -> bool {
    key.strip_prefix(PREFIX)
        .is_some_and(|rest| rest.len() == 64 && rest.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// Make a new admin key named `name`, replacing one of the same name, and
/// return it. It is shown once; the file keeps only its hash.
pub fn create(added: &mut Added, name: &str, now: u64) -> Result<String, AppError> {
    crate::config::validate_identifier(name, "admin key name")?;
    let key = format!("{PREFIX}{}", super::enrollment::secret()?);
    added.admins.retain(|admin| admin.name != name);
    if added.admins.len() >= MAX_KEYS {
        return Err(AppError::usage(format!(
            "this server already has {MAX_KEYS} admin keys; revoke one first"
        )));
    }
    added.admins.push(AdminKey {
        name: name.to_owned(),
        hash: hash(&key),
        created_at: now,
    });
    Ok(key)
}

/// Stop accepting the admin key named `name`.
pub fn revoke(added: &mut Added, name: &str) -> Result<(), AppError> {
    let before = added.admins.len();
    added.admins.retain(|admin| admin.name != name);
    if added.admins.len() == before {
        return Err(AppError::usage(format!(
            "this server has no admin key named {name}"
        )));
    }
    Ok(())
}

/// The name of the admin key `key` is, if the server accepts it.
pub fn name_of<'a>(added: &'a Added, key: &str) -> Option<&'a str> {
    if !well_formed(key) {
        return None;
    }
    let wanted = hash(key);
    added
        .admins
        .iter()
        .find(|admin| constant_time_eq(admin.hash.as_bytes(), wanted.as_bytes()))
        .map(|admin| admin.name.as_str())
}

/// The accepted admin keys as hashes, for the running server's gate.
pub fn hashes(added: &Added) -> Vec<(blake3::Hash, String)> {
    added
        .admins
        .iter()
        .filter_map(|admin| {
            Some((
                blake3::Hash::from_hex(&admin.hash).ok()?,
                admin.name.clone(),
            ))
        })
        .collect()
}

/// Remember an admin action, dropping the oldest beyond what is kept.
pub fn record(added: &mut Added, action: Action) {
    added.audit.push(action);
    let excess = added.audit.len().saturating_sub(REMEMBERED_ACTIONS);
    added.audit.drain(..excess);
}

/// Check the admin entries of a file being read.
pub fn validate(added: &Added) -> Result<(), AppError> {
    for admin in &added.admins {
        crate::config::validate_identifier(&admin.name, "admin key name")?;
        blake3::Hash::from_hex(&admin.hash).map_err(|_| {
            AppError::configuration("an admin key hash must be 64 hexadecimal characters")
        })?;
    }
    Ok(())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && left.iter().zip(right).fold(0, |acc, (a, b)| acc | (a ^ b)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_shown_once_kept_as_a_hash_and_can_be_revoked() {
        let mut added = Added::default();
        let key = create(&mut added, "laptop", 10).unwrap();
        assert!(well_formed(&key));
        let text = toml::to_string(&added).unwrap();
        assert!(!text.contains(&key), "the file never holds the key");
        assert!(!text.contains(key.trim_start_matches(PREFIX)));
        assert_eq!(name_of(&added, &key), Some("laptop"));
        // A new key with the same name replaces the old one.
        let rotated = create(&mut added, "laptop", 11).unwrap();
        assert_eq!(name_of(&added, &key), None);
        assert_eq!(name_of(&added, &rotated), Some("laptop"));
        revoke(&mut added, "laptop").unwrap();
        assert_eq!(name_of(&added, &rotated), None);
        assert!(revoke(&mut added, "laptop").is_err());
    }

    #[test]
    fn tokens_join_secrets_and_lookalikes_are_not_admin_keys() {
        let mut added = Added::default();
        let key = create(&mut added, "ops", 1).unwrap();
        let secret = key.trim_start_matches(PREFIX).to_owned();
        for candidate in [
            secret.clone(),
            format!("mash-join.{secret}"),
            format!("{PREFIX}{}", &secret[..63]),
            format!("{PREFIX}{secret}0"),
            format!(
                "{PREFIX}{}",
                secret.to_uppercase().replace(char::is_numeric, "z")
            ),
            String::new(),
        ] {
            assert_eq!(name_of(&added, &candidate), None, "{candidate}");
        }
    }

    #[test]
    fn the_audit_keeps_the_most_recent_actions() {
        let mut added = Added::default();
        for at in 0..(REMEMBERED_ACTIONS as u64 + 5) {
            record(
                &mut added,
                Action {
                    at,
                    key: "ops".into(),
                    from: None,
                    action: "add".into(),
                    target: "g/n".into(),
                },
            );
        }
        assert_eq!(added.audit.len(), REMEMBERED_ACTIONS);
        assert_eq!(added.audit[0].at, 5);
    }
}
