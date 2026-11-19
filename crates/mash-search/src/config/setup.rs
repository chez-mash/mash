//! First-run setup from the dashboard: the choices `mash init` asks for in a
//! terminal, limited to which agents' histories to index and how far back.
//! History locations come from this machine's own detection or its existing
//! configuration, never from the request.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::IndexDestination;
use super::init::{self, Choices};
use crate::AppError;
use crate::provider::{Provider, ProviderRoots};

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunFile")]
pub struct View {
    path: PathBuf,
    /// Names the file's contents, and is empty while there is no file. A
    /// choice states the contents it was made against.
    revision: String,
    /// Whether a configuration file exists already.
    configured: bool,
    /// The histories found on this machine or named by the file.
    histories: BTreeMap<Provider, History>,
    since_days: Option<u32>,
    /// Whether the file publishes to a shared server, which setup keeps.
    shared: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunHistory")]
struct History {
    /// Indexed by the file, or offered for a first setup.
    indexed: bool,
    roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "FirstRunChoice")]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub revision: String,
    /// Which agents' histories to index. An agent left out keeps what the
    /// file says, and is not indexed when there is no file.
    pub agents: BTreeMap<Provider, bool>,
    pub since_days: Option<u32>,
}

/// The file's text, or `None` while it does not exist.
fn read(path: &Path) -> Result<Option<String>, AppError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(AppError::configuration(format!(
            "cannot read the configuration at {}: {error}",
            path.display()
        ))),
    }
}

fn revision(text: Option<&str>) -> String {
    text.map_or_else(String::new, |text| {
        blake3::hash(text.as_bytes()).to_hex().to_string()
    })
}

/// The configuration at `path`, or the built-in defaults while no file is
/// there: setup is what creates it.
pub fn current(path: &Path) -> Result<super::ResolvedConfig, AppError> {
    let path = std::path::absolute(path).map_err(AppError::io)?;
    super::loading::load_from_paths(Ok(path), None)
}

pub fn view(path: &Path) -> Result<View, AppError> {
    let text = read(path)?;
    let current = current(path)?;
    Ok(describe(path, text.as_deref(), &current, &init::detect()?))
}

fn describe(
    path: &Path,
    text: Option<&str>,
    current: &super::ResolvedConfig,
    detected: &ProviderRoots,
) -> View {
    let configured = text.is_some();
    let histories = Provider::ALL
        .iter()
        .filter_map(|provider| {
            let named = configured
                .then(|| current.providers.get(provider))
                .flatten();
            let roots = named.or_else(|| detected.get(provider))?.clone();
            Some((
                *provider,
                History {
                    // A first setup offers everything found.
                    indexed: named.is_some() || !configured,
                    roots,
                },
            ))
        })
        .collect();
    View {
        path: path.to_owned(),
        revision: revision(text),
        configured,
        histories,
        since_days: current.since_days,
        shared: configured && current.destination == IndexDestination::Both,
    }
}

/// The plan `init` writes for `choice`, and the file text it was made
/// against. Nothing is written here.
pub fn plan(path: &Path, choice: &Choice) -> Result<(Choices, Option<String>), AppError> {
    let text = read(path)?;
    let current = current(path)?;
    let choices = choose(text.as_deref(), &current, &init::detect()?, choice)?;
    Ok((choices, text))
}

fn choose(
    text: Option<&str>,
    current: &super::ResolvedConfig,
    detected: &ProviderRoots,
    choice: &Choice,
) -> Result<Choices, AppError> {
    if revision(text) != choice.revision {
        return Err(AppError::configuration(
            "the configuration file changed since setup was opened; reload and try again",
        ));
    }
    let configured = text.is_some();
    let mut providers = ProviderRoots::new();
    for provider in Provider::ALL {
        let named = configured
            .then(|| current.providers.get(provider))
            .flatten();
        if !choice
            .agents
            .get(provider)
            .copied()
            .unwrap_or(named.is_some())
        {
            continue;
        }
        let roots = named.or_else(|| detected.get(provider)).ok_or_else(|| {
            AppError::configuration(format!("no {provider} history was found on this machine"))
        })?;
        providers.insert(*provider, roots.clone());
    }
    if providers.is_empty() {
        return Err(AppError::configuration(
            "choose at least one agent's history to index",
        ));
    }
    let choices = Choices {
        providers,
        since_days: choice.since_days,
        // Setup here never starts or stops publishing to a shared server.
        shared: (configured && current.destination == IndexDestination::Both)
            .then(|| current.shared.clone())
            .flatten(),
        records: current.records.clone(),
    };
    // Validate now, as the terminal wizard does before it confirms.
    let base = match text {
        Some(text) => text.to_owned(),
        None => init::render(&choices.providers, false)?,
    };
    init::configure(&base, &choices)?;
    Ok(choices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(name: &str) -> PathBuf {
        PathBuf::from(if cfg!(windows) { r"C:\h" } else { "/h" }).join(name)
    }

    fn detected() -> ProviderRoots {
        ProviderRoots::from([
            (Provider::ClaudeCode, vec![home("claude")]),
            (Provider::Codex, vec![home("codex")]),
        ])
    }

    fn fresh() -> super::super::ResolvedConfig {
        current(&home("absent/config.toml")).unwrap()
    }

    fn choice(text: Option<&str>, agents: &[(Provider, bool)]) -> Choice {
        Choice {
            revision: revision(text),
            agents: agents.iter().copied().collect(),
            since_days: Some(90),
        }
    }

    #[test]
    fn a_first_setup_offers_what_was_found_and_indexes_only_what_was_chosen() {
        let current = fresh();
        let view = describe(&home("config.toml"), None, &current, &detected());
        assert!(!view.configured && view.revision.is_empty() && !view.shared);
        assert!(view.histories.values().all(|history| history.indexed));
        assert_eq!(view.histories.len(), 2);

        let chosen = choose(
            None,
            &current,
            &detected(),
            &choice(
                None,
                &[(Provider::ClaudeCode, true), (Provider::Codex, false)],
            ),
        )
        .unwrap();
        assert_eq!(
            chosen.providers,
            ProviderRoots::from([(Provider::ClaudeCode, vec![home("claude")])])
        );
        assert_eq!(chosen.since_days, Some(90));
        assert!(chosen.shared.is_none());
    }

    #[test]
    fn setup_refuses_nothing_to_index_unknown_histories_and_a_stale_file() {
        let current = fresh();
        let none = choice(None, &[(Provider::ClaudeCode, false)]);
        assert!(choose(None, &current, &detected(), &none).is_err());
        // The request cannot name a history this machine does not have.
        let absent = choice(None, &[(Provider::Cursor, true)]);
        assert!(choose(None, &current, &detected(), &absent).is_err());
        let stale = Choice {
            revision: "something else".into(),
            ..choice(None, &[(Provider::ClaudeCode, true)])
        };
        assert!(choose(None, &current, &detected(), &stale).is_err());
        let never = Choice {
            since_days: Some(0),
            ..choice(None, &[(Provider::ClaudeCode, true)])
        };
        assert!(choose(None, &current, &detected(), &never).is_err());
    }

    #[test]
    fn an_existing_file_keeps_its_own_locations_and_unmentioned_agents() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let kept = root.path().join("my-codex");
        let text = init::render(
            &ProviderRoots::from([(Provider::Codex, vec![kept.clone()])]),
            false,
        )
        .unwrap();
        std::fs::write(&path, &text).unwrap();
        let current = current(&path).unwrap();
        let view = describe(&path, Some(&text), &current, &detected());
        assert!(view.configured && !view.revision.is_empty());
        assert!(view.histories[&Provider::Codex].indexed);
        assert!(!view.histories[&Provider::ClaudeCode].indexed);

        let chosen = choose(
            Some(&text),
            &current,
            &detected(),
            &choice(Some(&text), &[(Provider::ClaudeCode, true)]),
        )
        .unwrap();
        assert_eq!(chosen.providers[&Provider::Codex], vec![kept]);
        assert_eq!(
            chosen.providers[&Provider::ClaudeCode],
            vec![home("claude")]
        );
    }
}
