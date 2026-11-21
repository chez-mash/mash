//! Explicit model installation and local/shared forgetting.
use super::models::{ForgetResponse, ModelsInstallResponse};
use super::publication_config;
use crate::AppError;
use crate::config::ResolvedConfig;
use crate::semantic::Models;
use crate::shared;
use crate::storage::Storage;
use std::path::Path;

pub(super) async fn forget(
    database: &Path,
    models: &Path,
    configuration: &ResolvedConfig,
    id: String,
) -> Result<ForgetResponse, AppError> {
    let _progress = crate::progress::Session;
    let db = database.to_path_buf();
    let local_id = id.clone();
    let mut result = crate::blocking(move || local_forget(&db, local_id)).await?;
    if result.forgotten
        && let Err(error) = crate::views::local::build(database, models, configuration, false).await
    {
        // Silver and its tombstone are already committed. A later index retries
        // projection; readers reject the old gold certificate in the meantime.
        result.local_views_error = Some(error.error.message);
    }
    if result.forgotten {
        // A search that began before the forget may have kept its words again.
        let _ = std::fs::remove_file(crate::views::qdrant::vocabulary_file(database));
    }
    if let Some(config) = publication_config(configuration) {
        result.shared = Some(
            shared::publish(
                database,
                config,
                Some(shared::publication::Inference {
                    local: Some(configuration),
                    models_dir: models,
                }),
            )
            .await,
        );
    }
    Ok(result)
}

pub(super) async fn install_models(
    models: &Path,
    configuration: &ResolvedConfig,
) -> Result<ModelsInstallResponse, AppError> {
    let models = models.to_path_buf();
    let (recommended_action, next) = model_install_guidance(configuration);
    crate::blocking(move || {
        let summary = Models::new(models.clone()).install()?;
        Ok(ModelsInstallResponse {
            installed: true,
            model_directory: models,
            summary,
            recommended_action,
            next,
        })
    })
    .await
}

fn model_install_guidance(configuration: &ResolvedConfig) -> (Option<crate::Action>, &'static str) {
    if !configuration.loaded {
        (
            Some(crate::Action::Init),
            "Run `mash init` to set up your histories and build search data. After indexing, pass `--intent related-discussion` to search by meaning.",
        )
    } else if configuration.views.build.dense_enabled() {
        (
            Some(crate::Action::Index),
            "Run `mash index` to refresh search data. Pass `--intent related-discussion` to search by meaning.",
        )
    } else {
        (
            None,
            "Your configuration builds word search only. Set `representations = \"lexical-and-dense\"` in `[views.build]` before running `mash index`. See https://getmash.io/indexing/#enable-semantic-search",
        )
    }
}

fn local_forget(database_path: &Path, id: String) -> Result<ForgetResponse, AppError> {
    let mut storage = Storage::open_existing(database_path)?;
    let forgotten = storage.forget(&id)?;
    if forgotten {
        // Kept matching words may include the forgotten text's; searches read
        // them again rather than keep them on disk. The history is already
        // forgotten, so a failure here only leaves words that match nothing.
        let _ = std::fs::remove_file(crate::views::qdrant::vocabulary_file(database_path));
    }
    Ok(ForgetResponse {
        id,
        forgotten,
        local_views_error: None,
        shared: None,
    })
}

#[cfg(test)]
mod tests {
    use super::model_install_guidance;
    use crate::{Action, config, views::config::Representations};

    #[test]
    fn model_install_guidance_matches_the_configuration() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        std::fs::write(&path, "version = 1\nproviders = {}\n").unwrap();
        let mut configuration = config::load(Some(&path)).unwrap();

        let (action, next) = model_install_guidance(&configuration);
        assert_eq!(action, Some(Action::Index));
        assert!(next.contains("--intent related-discussion"));

        configuration.views.build.representations = Representations::LexicalOnly;
        let (action, next) = model_install_guidance(&configuration);
        assert_eq!(action, None);
        assert!(next.contains("`representations = \"lexical-and-dense\"` in `[views.build]`"));
        assert!(next.contains("#enable-semantic-search"));

        configuration.loaded = false;
        let (action, next) = model_install_guidance(&configuration);
        assert_eq!(action, Some(Action::Init));
        assert!(next.contains("mash init"));
    }
}
