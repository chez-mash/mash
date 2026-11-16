//! Which folder a chat belongs to, from the editor's per-workspace state.
//! Older composer roots do not record it; each workspace's own database
//! lists the composers opened there. Best effort: a workspace that cannot
//! be read names no folder.
use super::models::{WorkspaceComposers, WorkspaceFolder};
use crate::sqlite_source;
use std::path::Path;

/// Workspaces looked through before giving up.
const LIMIT: usize = 4096;
const RECORD_BYTES: i64 = 8 * 1024 * 1024;

/// `store` is `User/globalStorage/state.vscdb`; workspaces sit beside it in
/// `User/workspaceStorage/<id>/` with a `workspace.json` naming the folder.
pub fn folder(store: &Path, composer: &str) -> Option<String> {
    let workspaces = store.parent()?.parent()?.join("workspaceStorage");
    let mut found = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(workspaces).ok()?.take(LIMIT).flatten() {
        let directory = entry.path();
        if lists(&directory.join("state.vscdb"), composer)
            && let Some(folder) = named(&directory.join("workspace.json"))
        {
            found.insert(folder);
        }
    }
    // A chat listed under two folders belongs to neither for certain.
    (found.len() == 1)
        .then(|| found.into_iter().next())
        .flatten()
}

fn lists(database: &Path, composer: &str) -> bool {
    if !database.is_file() {
        return false;
    }
    let Ok(connection) = sqlite_source::open(database) else {
        return false;
    };
    let value: Option<String> = connection
        .query_row(
            "SELECT CASE WHEN length(CAST(value AS BLOB))<=?1 THEN CAST(value AS TEXT) END
               FROM ItemTable WHERE key='composer.composerData'",
            [RECORD_BYTES],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    value
        .and_then(|value| serde_json::from_str::<WorkspaceComposers>(&value).ok())
        .is_some_and(|listed| {
            listed
                .all_composers
                .iter()
                .any(|listed| listed.composer_id.as_deref() == Some(composer))
        })
}

fn named(file: &Path) -> Option<String> {
    let size = std::fs::metadata(file).ok()?.len();
    if size > 64 * 1024 {
        return None;
    }
    let workspace: WorkspaceFolder = serde_json::from_slice(&std::fs::read(file).ok()?).ok()?;
    crate::platform::file_uri_path(&workspace.folder?)
}
