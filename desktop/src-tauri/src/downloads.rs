//! Downloads from the dashboard, such as an exported chat. They go to the
//! reader's Downloads folder, numbered so nothing is replaced, and when one
//! finishes the app says where it went and offers to show it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Manager, Url};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;

/// Where each download in progress is being saved, by its address. macOS does
/// not say where a finished download went, so the app remembers.
#[derive(Default)]
pub struct Downloads(Mutex<HashMap<String, PathBuf>>);

/// The word for the system's file browser.
const SHOW: &str = if cfg!(target_os = "macos") {
    "Show in Finder"
} else {
    "Show in Folder"
};

/// Handle a download event from the main window.
pub fn handle(app: &AppHandle, event: DownloadEvent<'_>) -> bool {
    let downloads = app.state::<Downloads>();
    let mut saving = downloads.0.lock().unwrap_or_else(PoisonError::into_inner);
    match event {
        DownloadEvent::Requested { url, destination } => {
            if let Ok(folder) = app.path().download_dir() {
                *destination = unused_path(&folder, destination);
            }
            saving.insert(key(&url), destination.clone());
        }
        DownloadEvent::Finished { url, path, success } => {
            let remembered = saving.remove(&key(&url));
            let path = path
                .filter(|path| !path.as_os_str().is_empty())
                .or(remembered);
            drop(saving);
            finished(app, path, success);
        }
        _ => {}
    }
    true
}

fn key(url: &Url) -> String {
    url.as_str().to_owned()
}

/// Say where a finished download went, with a button that shows it. The
/// notice is a sheet on the main window, where the download came from.
fn finished(app: &AppHandle, path: Option<PathBuf>, success: bool) {
    let notice = |message: String| {
        let notice = app.dialog().message(message).title("Download");
        match app.get_webview_window(crate::WINDOW) {
            Some(window) => notice.parent(&window),
            None => notice,
        }
    };
    let name = path
        .as_deref()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned());
    let (Some(path), Some(name), true) = (path.clone(), name, success) else {
        notice("The download didn’t finish.".into())
            .kind(MessageDialogKind::Warning)
            .show(|_| {});
        return;
    };
    let folder = path
        .parent()
        .map(|folder| folder.display().to_string())
        .unwrap_or_default();
    let handle = app.clone();
    notice(format!("Saved “{name}” in {folder}."))
        .buttons(MessageDialogButtons::OkCancelCustom(
            SHOW.into(),
            "OK".into(),
        ))
        .show(move |show| {
            if show {
                let _ = handle.opener().reveal_item_in_dir(&path);
            }
        });
}

/// A file in `folder` named like the download, numbered so nothing is replaced.
pub fn unused_path(folder: &Path, suggested: &Path) -> PathBuf {
    let name = suggested
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| "download".into());
    let stem = name
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let extension = name
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()));
    let extension = extension.unwrap_or_default();
    let mut path = folder.join(&name);
    for number in 1.. {
        if !path.exists() {
            break;
        }
        path = folder.join(format!("{stem} ({number}){extension}"));
    }
    path
}

#[cfg(test)]
mod tests {
    use super::unused_path;

    #[test]
    fn downloads_never_replace_a_file() {
        let folder = std::env::temp_dir().join(format!("mash-desktop-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let first = unused_path(&folder, "/somewhere/chat.md".as_ref());
        assert_eq!(first, folder.join("chat.md"));
        std::fs::write(&first, "").unwrap();
        assert_eq!(
            unused_path(&folder, "chat.md".as_ref()),
            folder.join("chat (1).md")
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
