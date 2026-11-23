//! The settings a person changes from the dashboard: which agents are
//! indexed, how far back, where history goes, memory capture and credential
//! masking. Every other setting in the file is kept as written, and history
//! locations come from this machine's own detection, never from the request.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::IndexDestination;
use super::document::{Edit, edit, parse_document, with_assistant};
use crate::AppError;
use crate::provider::{Provider, ProviderRoots};

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct View {
    path: PathBuf,
    /// Names the file's contents. A change states the contents it was made
    /// against, so an edit made elsewhere in between is not overwritten.
    revision: String,
    agents: BTreeMap<Provider, Agent>,
    since_days: Option<u32>,
    destination: IndexDestination,
    /// Whether a shared server is configured, so history can go to it.
    shared: bool,
    /// Agents this machine keeps off the shared server.
    unshared: Vec<Provider>,
    memory_capture: bool,
    hide_credentials: crate::redaction::Settings,
    /// The dashboard assistant, when the file configures a service for it.
    assistant: Option<Assistant>,
}

/// The assistant's switch and model. Its service and key stay in the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "AssistantSettings")]
#[serde(deny_unknown_fields)]
pub struct Assistant {
    enabled: bool,
    model: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "AgentSettings")]
struct Agent {
    indexed: bool,
    /// The configured locations, or the ones found on this machine.
    roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Change {
    revision: String,
    agents: BTreeMap<Provider, bool>,
    since_days: Option<u32>,
    destination: IndexDestination,
    memory_capture: bool,
    hide_credentials: crate::redaction::Settings,
    /// Agents kept off the shared server. Left out, the list stays as it is;
    /// a change makes the next publication replace everything sent before.
    #[serde(default)]
    unshared: Option<Vec<Provider>>,
    /// Left out, the assistant's settings stay as they are.
    #[serde(default)]
    assistant: Option<Assistant>,
}

fn read(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path).map_err(|_| {
        AppError::configuration(format!(
            "no readable configuration at {}; run `mash init` first",
            path.display()
        ))
    })
}

fn revision(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

pub fn view(path: &Path) -> Result<View, AppError> {
    describe(path, &read(path)?, &super::init::detect()?)
}

fn describe(path: &Path, text: &str, detected: &ProviderRoots) -> Result<View, AppError> {
    let document = parse_document(text.as_bytes())?;
    let agents = Provider::ALL
        .iter()
        .filter_map(|provider| {
            let configured = document.providers.get(provider);
            let roots = configured.or_else(|| detected.get(provider))?.clone();
            Some((
                *provider,
                Agent {
                    indexed: configured.is_some(),
                    roots,
                },
            ))
        })
        .collect();
    Ok(View {
        path: path.to_owned(),
        revision: revision(text),
        agents,
        since_days: document.since_days,
        destination: document.destination,
        shared: document.shared.is_some(),
        unshared: document
            .shared
            .as_ref()
            .map_or_else(Default::default, |shared| shared.unshared.clone()),
        memory_capture: document.records.enabled,
        hide_credentials: document.redaction,
        assistant: document.assistant.map(|assistant| Assistant {
            enabled: assistant.enabled,
            model: assistant.model,
        }),
    })
}

/// Rewrite the file with `change`, keeping the previous file beside it, and
/// return the settings as they now stand.
pub fn apply(path: &Path, change: &Change) -> Result<View, AppError> {
    let detected = super::init::detect()?;
    let text = read(path)?;
    let updated = revise(&text, change, &detected)?;
    super::init::write(path, &updated)?;
    describe(path, &updated, &detected)
}

/// What the dashboard sends to set Mashy up: a service, its API key and a
/// model. The key is saved, never returned.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "AssistantSetup")]
#[serde(deny_unknown_fields)]
pub struct AssistantSetup {
    revision: String,
    /// The service's API base, such as `https://openrouter.ai/api/v1`. Left
    /// out, the service already configured is kept, so only the key changes.
    #[serde(default)]
    base_url: Option<String>,
    /// Left out, the model already configured is kept.
    #[serde(default)]
    model: Option<String>,
    key: String,
}

/// The variable a key entered in the dashboard is saved under, unless the
/// file already names one for the assistant.
const ASSISTANT_KEY: &str = "MASH_ASSISTANT_KEY";

/// Point the assistant at a service and save its key where Mash keeps saved
/// credentials (`~/.mash/node.env`, readable only by this user), so Mashy
/// works without editing the file or the dashboard's environment.
pub fn set_up_assistant(path: &Path, setup: &AssistantSetup) -> Result<View, AppError> {
    let detected = super::init::detect()?;
    let text = read(path)?;
    let set_in_environment = |name: &str| std::env::var(name).is_ok_and(|value| !value.is_empty());
    let (updated, token_env) = assistant_revision(&text, setup, set_in_environment)?;
    let saved = super::saved_tokens()
        .ok_or_else(|| AppError::configuration("no home directory to save the key in"))?;
    super::save_token(&saved, &token_env, setup.key.trim())?;
    super::init::write(path, &updated)?;
    describe(path, &updated, &detected)
}

/// The file with the assistant set up as `setup` asks, and the variable its
/// key is saved under. `set_in_environment` says whether a variable is set
/// for the dashboard.
fn assistant_revision(
    text: &str,
    setup: &AssistantSetup,
    set_in_environment: impl Fn(&str) -> bool,
) -> Result<(String, String), AppError> {
    if revision(text) != setup.revision {
        return Err(AppError::configuration(
            "the configuration file changed since these settings were loaded; reload them and try again",
        ));
    }
    let key = setup.key.trim();
    if key.is_empty() || key.len() > 4096 || key.chars().any(char::is_control) {
        return Err(AppError::configuration("enter the service's API key"));
    }
    let configured = parse_document(text.as_bytes())?.assistant;
    let token_env = configured.as_ref().map_or_else(
        || ASSISTANT_KEY.to_owned(),
        |assistant| assistant.token_env.clone(),
    );
    // A key in the dashboard's environment is used before a saved one, so a
    // key saved here would be silently ignored.
    if set_in_environment(&token_env) {
        return Err(AppError::configuration(format!(
            "{token_env} is set for the dashboard and is used instead of a saved key; change it there"
        )));
    }
    let chosen = |given: Option<&String>, kept: Option<&String>, what: &str| {
        given
            .map(|value| value.trim().to_owned())
            .or_else(|| kept.cloned())
            .ok_or_else(|| AppError::configuration(format!("choose the assistant's {what}")))
    };
    let assistant = super::Assistant {
        base_url: chosen(
            setup.base_url.as_ref(),
            configured.as_ref().map(|assistant| &assistant.base_url),
            "service",
        )?,
        token_env: token_env.clone(),
        model: chosen(
            setup.model.as_ref(),
            configured.as_ref().map(|assistant| &assistant.model),
            "model",
        )?,
        enabled: true,
    };
    assistant.validate()?;
    Ok((with_assistant(text, assistant)?, token_env))
}

fn revise(text: &str, change: &Change, detected: &ProviderRoots) -> Result<String, AppError> {
    if revision(text) != change.revision {
        return Err(AppError::configuration(
            "the configuration file changed since these settings were loaded; reload them and try again",
        ));
    }
    let mut providers = parse_document(text.as_bytes())?.providers;
    for (provider, indexed) in &change.agents {
        if !indexed {
            providers.remove(provider);
        } else if !providers.contains_key(provider) {
            let roots = detected.get(provider).ok_or_else(|| {
                AppError::configuration(format!("no {provider} history was found on this machine"))
            })?;
            providers.insert(*provider, roots.clone());
        }
    }
    if providers.is_empty() {
        return Err(AppError::configuration(
            "keep at least one agent's history indexed",
        ));
    }
    edit(
        text,
        &Edit {
            providers: &providers,
            since_days: change.since_days,
            destination: change.destination,
            memory_capture: change.memory_capture,
            redaction: change.hide_credentials,
            unshared: change.unshared.as_deref(),
            assistant: change
                .assistant
                .as_ref()
                .map(|assistant| (assistant.enabled, assistant.model.as_str())),
        },
    )
}

#[cfg(test)]
mod tests;
