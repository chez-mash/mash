//! The projects the reader pinned, so the dashboard lists them first.
//!
//! Pins belong to the dashboard, not to a browser: they are kept in a small
//! file beside the index this dashboard reads, so they follow the reader from
//! a laptop to a phone. Like the settings, they can be changed on the machine
//! itself, or from a dashboard started with a services configuration and
//! therefore behind its access token; anyone else who can open the dashboard
//! sees them but cannot change them.
//!
//! The file is replaced whole through a temporary file, so a reader never sees
//! half of it, and is readable only by the user. A missing file is no pins.
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

/// More pins than this is no longer a short list to look at first.
const MOST: usize = 50;
/// Longer than any project path; a guard against a file that is not pins.
const LONGEST: usize = 4096;

/// The pinned projects, in the order the reader pinned them.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub(super) struct Pins {
    pub(super) projects: Vec<String>,
    /// Whether this dashboard accepts a change to them.
    #[serde(default)]
    pub(super) editable: bool,
}

/// A new list of pinned projects, replacing the old one.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct PinChange {
    pub(super) projects: Vec<String>,
}

pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::{changes, reads};
    use aide::axum::routing::get_with;
    router.api_route(
        "/api/pins",
        get_with(
            read,
            reads::<Pins>("/api/pins", "The projects pinned on this dashboard."),
        )
        .post_with(
            change,
            changes::<Pins>("/api/pins", "Replace the pinned projects."),
        ),
    )
}

fn editable(state: &Dash) -> bool {
    state.loopback || state.services.managed()
}

/// The file the pins live in: beside the index this dashboard reads.
fn file(state: &Dash) -> PathBuf {
    let database = state
        .paths
        .db
        .clone()
        .unwrap_or_else(mash_app::core::default_database_path);
    database.with_file_name("dashboard-pins.json")
}

async fn read(State(state): State<Arc<Dash>>) -> Response {
    let path = file(&state);
    let editable = editable(&state);
    answer(
        tokio::task::spawn_blocking(move || load(&path))
            .await
            .unwrap_or_else(|_| Err(AppError::internal("read pins"))),
        editable,
    )
}

async fn change(
    State(state): State<Arc<Dash>>,
    input: Result<axum::Json<PinChange>, JsonRejection>,
) -> Response {
    if !editable(&state) {
        return failure(
            StatusCode::FORBIDDEN,
            &AppError::configuration(
                "pins can be changed on the machine itself, or from a dashboard started with a services configuration",
            ),
        );
    }
    let Ok(axum::Json(change)) = input else {
        return failure(StatusCode::BAD_REQUEST, &AppError::usage("invalid pins"));
    };
    let path = file(&state);
    answer(
        tokio::task::spawn_blocking(move || {
            let projects = settle(change.projects)?;
            save(&path, &projects)?;
            Ok(projects)
        })
        .await
        .unwrap_or_else(|_| Err(AppError::internal("save pins"))),
        true,
    )
}

fn answer(projects: Result<Vec<String>, AppError>, editable: bool) -> Response {
    match projects {
        Ok(projects) => axum::Json(Pins { projects, editable }).into_response(),
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

/// The pins as given, each once, in their first place; or why they are not pins.
fn settle(projects: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut settled: Vec<String> = Vec::new();
    for project in projects {
        if project.trim().is_empty() || project.len() > LONGEST {
            return Err(AppError::usage(
                "a pinned project needs a name of a usual length",
            ));
        }
        if !settled.contains(&project) {
            settled.push(project);
        }
    }
    if settled.len() > MOST {
        return Err(AppError::usage(format!(
            "at most {MOST} projects can be pinned"
        )));
    }
    Ok(settled)
}

#[derive(Serialize, Deserialize)]
struct Stored {
    projects: Vec<String>,
}

fn load(path: &Path) -> Result<Vec<String>, AppError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => {
            return Err(AppError::configuration(
                "the pinned projects could not be read",
            ));
        }
    };
    let stored: Stored = serde_json::from_str(&text).map_err(|_| {
        AppError::configuration(format!(
            "{} is not a list of pinned projects; remove it to start again",
            path.display()
        ))
    })?;
    settle(stored.projects)
}

fn save(path: &Path, projects: &[String]) -> Result<(), AppError> {
    let failed = |_| AppError::configuration("the pinned projects could not be saved");
    let text = serde_json::to_string_pretty(&Stored {
        projects: projects.to_vec(),
    })
    .map_err(|_| AppError::internal("encode pins"))?;
    let temporary = path.with_extension("json.tmp");
    mash_model::platform::write_private(&temporary, &text, 0o600).map_err(failed)?;
    std::fs::rename(&temporary, path).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_keep_their_order_once_each_and_survive_a_reload() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dashboard-pins.json");
        assert_eq!(
            load(&path).unwrap(),
            Vec::<String>::new(),
            "no file is no pins"
        );
        let projects = settle(vec!["/srv/b".into(), "/srv/a".into(), "/srv/b".into()]).unwrap();
        save(&path, &projects).unwrap();
        assert_eq!(load(&path).unwrap(), ["/srv/b", "/srv/a"]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn what_is_not_a_short_list_of_projects_is_refused() {
        assert!(settle(vec![" ".into()]).is_err());
        assert!(settle(vec!["x".repeat(LONGEST + 1)]).is_err());
        assert!(settle((0..=MOST).map(|n| format!("/p{n}")).collect()).is_err());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dashboard-pins.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(
            load(&path).is_err(),
            "a damaged file is said, not taken for none"
        );
    }
}
