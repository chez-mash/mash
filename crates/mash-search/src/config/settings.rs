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
            assistant: change
                .assistant
                .as_ref()
                .map(|assistant| (assistant.enabled, assistant.model.as_str())),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A history location that is absolute on the host running the test, as
    /// configured roots must be.
    fn home(name: &str) -> PathBuf {
        PathBuf::from(if cfg!(windows) { r"C:\h" } else { "/h" }).join(name)
    }

    fn file() -> String {
        format!(
            "version = 1\n\n[providers.codex]\nroots = ['{}']\n\n[index]\nall_history = true\n\n[views.query]\nlimit = 7\n",
            home("codex").display()
        )
    }

    fn detected() -> ProviderRoots {
        ProviderRoots::from([(Provider::ClaudeCode, vec![home("claude")])])
    }

    fn change(text: &str) -> Change {
        Change {
            revision: revision(text),
            agents: BTreeMap::new(),
            since_days: None,
            destination: IndexDestination::Local,
            memory_capture: true,
            hide_credentials: crate::redaction::Settings::default(),
            assistant: None,
        }
    }

    fn setup(text: &str, base_url: &str, key: &str) -> AssistantSetup {
        AssistantSetup {
            revision: revision(text),
            base_url: Some(base_url.into()),
            model: Some("anthropic/claude-haiku-5.5".into()),
            key: key.into(),
        }
    }

    #[test]
    fn the_assistant_can_be_set_up_with_a_key_saved_under_its_variable() {
        let unset = |_: &str| false;
        let text = file();
        let (updated, name) = assistant_revision(
            &text,
            &setup(&text, "https://openrouter.ai/api/v1", " sk-1 "),
            unset,
        )
        .unwrap();
        assert_eq!(name, "MASH_ASSISTANT_KEY");
        let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
        assert_eq!(
            view.assistant,
            Some(Assistant {
                enabled: true,
                model: "anthropic/claude-haiku-5.5".into()
            })
        );
        // The key itself never goes into the file.
        assert!(!updated.contains("sk-1"));
        // A file that already names the key's variable keeps that name.
        let named = format!(
            "{}\n[assistant]\nbase_url = 'https://x.test/v1'\ntoken_env = 'KEY'\nmodel = 'a/b'\nenabled = false\n",
            file()
        );
        let (_, name) = assistant_revision(
            &named,
            &setup(&named, "https://openrouter.ai/api/v1", "k"),
            unset,
        )
        .unwrap();
        assert_eq!(name, "KEY");
        // A new key alone keeps the configured service and model, and turns Mashy on.
        let key_only = AssistantSetup {
            revision: revision(&named),
            base_url: None,
            model: None,
            key: "k2".into(),
        };
        let (updated, _) = assistant_revision(&named, &key_only, unset).unwrap();
        assert!(updated.contains("https://x.test/v1") && updated.contains("a/b"));
        let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
        assert_eq!(
            view.assistant.map(|assistant| assistant.enabled),
            Some(true)
        );
        // Without a configured service, a key alone is not enough.
        let bare = AssistantSetup {
            revision: revision(&text),
            base_url: None,
            model: None,
            key: "k".into(),
        };
        assert!(assistant_revision(&text, &bare, unset).is_err());
    }

    #[test]
    fn setting_the_assistant_up_refuses_what_would_not_work() {
        let unset = |_: &str| false;
        let text = file();
        let good = "https://openrouter.ai/api/v1";
        for key in ["", "  ", "two\nlines"] {
            assert!(assistant_revision(&text, &setup(&text, good, key), unset).is_err());
        }
        // A plain-HTTP service away from this machine would send the key in the clear.
        assert!(
            assistant_revision(&text, &setup(&text, "http://example.com/v1", "k"), unset).is_err()
        );
        // A variable set for the dashboard would be used instead of the saved key.
        assert!(assistant_revision(&text, &setup(&text, good, "k"), |_: &str| true).is_err());
        // An edit made elsewhere in between is not overwritten.
        let mut stale = setup(&text, good, "k");
        stale.revision = "old".into();
        assert!(assistant_revision(&text, &stale, unset).is_err());
    }

    #[test]
    fn the_assistant_is_switched_and_given_a_model_only_where_one_is_configured() {
        let with = format!(
            "{}\n[assistant]\nbase_url = 'https://openrouter.ai/api/v1'\ntoken_env = 'KEY'\nmodel = 'a/b'\n",
            file()
        );
        let view = describe(Path::new("/c.toml"), &with, &detected()).unwrap();
        assert_eq!(
            view.assistant,
            Some(Assistant {
                enabled: true,
                model: "a/b".into()
            })
        );
        let mut request = change(&with);
        request.assistant = Some(Assistant {
            enabled: false,
            model: "c/d".into(),
        });
        let updated = revise(&with, &request, &detected()).unwrap();
        let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
        assert_eq!(view.assistant, request.assistant);
        // The service and the key's name are kept as written.
        assert!(updated.contains("openrouter.ai") && updated.contains("KEY"));
        // A change that leaves the assistant out leaves it as it is.
        let kept = revise(&updated, &change(&updated), &detected()).unwrap();
        let view = describe(Path::new("/c.toml"), &kept, &detected()).unwrap();
        assert_eq!(view.assistant, request.assistant);
        // An empty model name is refused, and so is an assistant the file lacks.
        let mut unnamed = change(&with);
        unnamed.assistant = Some(Assistant {
            enabled: true,
            model: " ".into(),
        });
        assert!(revise(&with, &unnamed, &detected()).is_err());
        let file = &file();
        assert!(
            describe(Path::new("/c.toml"), file, &detected())
                .unwrap()
                .assistant
                .is_none()
        );
        let mut absent = change(file);
        absent.assistant = request.assistant.clone();
        assert!(revise(file, &absent, &detected()).is_err());
    }

    #[test]
    fn a_change_edits_only_what_it_names_and_keeps_the_rest_of_the_file() {
        let file = &file();
        let mut request = change(file);
        request.agents = BTreeMap::from([(Provider::ClaudeCode, true)]);
        request.since_days = Some(30);
        request.memory_capture = false;
        request.hide_credentials.publication = false;
        let updated = revise(file, &request, &detected()).unwrap();
        let view = describe(Path::new("/c.toml"), &updated, &detected()).unwrap();
        assert!(view.agents[&Provider::Codex].indexed);
        assert!(view.agents[&Provider::ClaudeCode].indexed);
        assert_eq!(view.agents[&Provider::ClaudeCode].roots, [home("claude")]);
        assert_eq!(view.since_days, Some(30));
        assert!(!view.memory_capture);
        assert!(view.hide_credentials.output && !view.hide_credentials.publication);
        // A setting the page does not offer survives the rewrite.
        assert!(updated.contains("limit = 7"));
    }

    #[test]
    fn an_agent_with_no_history_here_is_offered_nothing_and_cannot_be_turned_on() {
        let file = &file();
        let view = describe(Path::new("/c.toml"), file, &detected()).unwrap();
        assert!(!view.agents.contains_key(&Provider::Cursor));
        assert!(!view.agents[&Provider::ClaudeCode].indexed);
        let mut request = change(file);
        request.agents = BTreeMap::from([(Provider::Cursor, true)]);
        let error = revise(file, &request, &detected()).unwrap_err();
        assert!(error.error.message.contains("no cursor history"));
    }

    #[test]
    fn stale_or_unusable_changes_leave_the_file_alone() {
        let file = &file();
        let mut stale = change(file);
        stale.revision = revision("something else");
        assert!(
            revise(file, &stale, &detected())
                .unwrap_err()
                .error
                .message
                .contains("changed since")
        );
        let mut none = change(file);
        none.agents = BTreeMap::from([(Provider::Codex, false)]);
        assert!(
            revise(file, &none, &detected())
                .unwrap_err()
                .error
                .message
                .contains("at least one")
        );
        // History cannot be sent to a shared server that is not configured.
        let mut both = change(file);
        both.destination = IndexDestination::Both;
        assert!(revise(file, &both, &detected()).is_err());
        let mut days = change(file);
        days.since_days = Some(0);
        assert!(revise(file, &days, &detected()).is_err());
    }

    #[test]
    fn applying_keeps_the_previous_file_beside_the_new_one() {
        let file = &file();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        std::fs::write(&path, file).unwrap();
        let mut request = change(file);
        request.since_days = Some(7);
        // Detection reads this machine; the change names no agent, so it is unused.
        let view = apply(&path, &request).unwrap();
        assert_eq!(view.since_days, Some(7));
        assert_ne!(view.revision, revision(file));
        assert_eq!(
            std::fs::read_to_string(root.path().join("config.toml.previous")).unwrap(),
            *file
        );
    }
}
