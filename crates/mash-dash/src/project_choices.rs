//! The reader's choices about projects: which are pinned to the top of the
//! dashboard's project lists, and which are hidden from them.
//!
//! The choices belong to the dashboard, not to a browser: they are kept in a
//! small file beside the index this dashboard reads, so they follow the reader
//! from a laptop to a phone. Like the settings, they can be changed on the
//! machine itself, or from a dashboard started with a services configuration
//! and therefore behind its access token; anyone else who can open the
//! dashboard sees them but cannot change them.
//!
//! Hiding a project only takes it out of the lists of projects; its chats stay
//! in search and in every chat list. The file is replaced whole through a
//! temporary file, so a reader never sees half of it, and is readable only by
//! the user. A missing file is no choices.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mash_model::AppError;
use serde::{Deserialize, Serialize};

use super::Dash;

const PATH: &str = "/api/projects/choices";
/// More than this in either list is no longer a short list of exceptions.
const MOST: usize = 50;
/// Longer than any project path; a guard against a file that is not choices.
const LONGEST: usize = 4096;

/// The pinned and hidden projects, each in the order the reader chose them.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub(super) struct Choices {
    pub(super) pinned: Vec<String>,
    pub(super) hidden: Vec<String>,
}

/// The choices, and whether this dashboard accepts a change to them.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "ProjectChoices")]
pub(super) struct Answer {
    #[serde(flatten)]
    choices: Choices,
    editable: bool,
}

pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::{changes, reads};
    use aide::axum::routing::get_with;
    router.api_route(
        PATH,
        get_with(
            read,
            reads::<Answer>(PATH, "The projects pinned and hidden on this dashboard."),
        )
        .post_with(
            change,
            changes::<Answer>(PATH, "Replace the pinned and hidden projects."),
        ),
    )
}

fn editable(state: &Dash) -> bool {
    state.loopback || state.services.managed()
}

/// The file the choices live in: beside the index this dashboard reads.
fn file(state: &Dash) -> PathBuf {
    let database = state
        .paths
        .db
        .clone()
        .unwrap_or_else(mash_app::core::default_database_path);
    database.with_file_name("dashboard-projects.json")
}

async fn read(State(state): State<Arc<Dash>>) -> Response {
    let path = file(&state);
    let editable = editable(&state);
    answer(
        tokio::task::spawn_blocking(move || load(&path))
            .await
            .unwrap_or_else(|_| Err(AppError::internal("read project choices"))),
        editable,
    )
}

async fn change(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<Choices>, JsonRejection>,
) -> Response {
    if !editable(&state) {
        return failure(
            StatusCode::FORBIDDEN,
            &AppError::configuration(
                "project choices can be changed on the machine itself, or from a dashboard started with a services configuration",
            ),
        );
    }
    let Ok(axum::Json(choices)) = input else {
        return failure(
            StatusCode::BAD_REQUEST,
            &AppError::usage("invalid project choices"),
        );
    };
    let path = file(&state);
    answer(
        tokio::task::spawn_blocking(move || {
            let choices = settle(choices)?;
            save(&path, &choices)?;
            Ok(choices)
        })
        .await
        .unwrap_or_else(|_| Err(AppError::internal("save project choices"))),
        true,
    )
}

fn answer(choices: Result<Choices, AppError>, editable: bool) -> Response {
    match choices {
        Ok(choices) => axum::Json(Answer { choices, editable }).into_response(),
        Err(error) => failure(StatusCode::BAD_REQUEST, &error),
    }
}

fn failure(status: StatusCode, error: &AppError) -> Response {
    (
        status,
        axum::Json(serde_json::json!({ "error": error.error })),
    )
        .into_response()
}

/// Each list with every project once, in its first place; or why the choices
/// cannot be kept. A project is pinned or hidden, never both.
fn settle(choices: Choices) -> Result<Choices, AppError> {
    let pinned = settle_list(choices.pinned)?;
    let hidden = settle_list(choices.hidden)?;
    if pinned.iter().any(|project| hidden.contains(project)) {
        return Err(AppError::usage(
            "a project is either pinned or hidden, not both",
        ));
    }
    Ok(Choices { pinned, hidden })
}

fn settle_list(projects: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut settled: Vec<String> = Vec::new();
    for project in projects {
        if project.trim().is_empty() || project.len() > LONGEST {
            return Err(AppError::usage(
                "a chosen project needs a name of a usual length",
            ));
        }
        if !settled.contains(&project) {
            settled.push(project);
        }
    }
    if settled.len() > MOST {
        return Err(AppError::usage(format!(
            "at most {MOST} projects can be pinned, and at most {MOST} hidden"
        )));
    }
    Ok(settled)
}

fn load(path: &Path) -> Result<Choices, AppError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Choices::default());
        }
        Err(_) => {
            return Err(AppError::configuration(
                "the project choices could not be read",
            ));
        }
    };
    let stored: Choices = serde_json::from_str(&text).map_err(|_| {
        AppError::configuration(format!(
            "{} is not a list of project choices; remove it to start again",
            path.display()
        ))
    })?;
    settle(stored)
}

fn save(path: &Path, choices: &Choices) -> Result<(), AppError> {
    let failed = |_| AppError::configuration("the project choices could not be saved");
    let text =
        serde_json::to_string_pretty(choices).map_err(|_| AppError::internal("encode choices"))?;
    let temporary = path.with_extension("json.tmp");
    mash_model::platform::write_private(&temporary, &text, 0o600).map_err(failed)?;
    std::fs::rename(&temporary, path).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choices(pinned: &[&str], hidden: &[&str]) -> Choices {
        Choices {
            pinned: pinned.iter().map(|&p| p.to_owned()).collect(),
            hidden: hidden.iter().map(|&p| p.to_owned()).collect(),
        }
    }

    #[test]
    fn choices_keep_their_order_once_each_and_survive_a_reload() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dashboard-projects.json");
        assert_eq!(
            load(&path).unwrap(),
            Choices::default(),
            "no file is no choices"
        );
        let settled = settle(choices(&["/srv/b", "/srv/a", "/srv/b"], &["/tmp/x"])).unwrap();
        save(&path, &settled).unwrap();
        assert_eq!(
            load(&path).unwrap(),
            choices(&["/srv/b", "/srv/a"], &["/tmp/x"])
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn what_is_not_a_short_list_of_projects_is_refused() {
        assert!(settle(choices(&[" "], &[])).is_err());
        assert!(settle(choices(&[], &[&"x".repeat(LONGEST + 1)])).is_err());
        let many: Vec<String> = (0..=MOST).map(|n| format!("/p{n}")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        assert!(settle(choices(&[], &many)).is_err());
        assert!(
            settle(choices(&["/srv/a"], &["/srv/a"])).is_err(),
            "pinned and hidden at once"
        );
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dashboard-projects.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(
            load(&path).is_err(),
            "a damaged file is said, not taken for none"
        );
    }
}
