//! The app as it was before an update, kept until the new version has
//! started once, so a version that cannot start can be gone back from.
//!
//! The updater replaces the installed app in place and deletes what it
//! replaced. Before it runs, the installed app (the `.app` bundle on macOS,
//! the AppImage file on Linux) is copied into the app's data folder. Going
//! back puts that copy where the app is installed and keeps the failed
//! version beside the copy's place until the next update. On Windows the
//! installer owns the installed files, so nothing is kept there.

use std::path::{Path, PathBuf};

/// What the updater replaces: the bundle that holds `executable` on macOS, or
/// the AppImage this run started from on Linux.
pub fn installed(executable: &Path, appimage: Option<PathBuf>) -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        // Mash.app/Contents/MacOS/mash-desktop
        let bundle = executable.parent()?.parent()?.parent()?;
        (bundle.extension()? == "app").then(|| bundle.to_path_buf())
    } else if cfg!(target_os = "linux") {
        appimage.filter(|file| file.is_file())
    } else {
        None
    }
}

/// Copy the installed app into `folder`, replacing any copy already there,
/// and say where the copy is.
pub fn keep(installed: &Path, folder: &Path) -> std::io::Result<PathBuf> {
    let name = installed
        .file_name()
        .ok_or_else(|| std::io::Error::other("the installed app has no name"))?;
    let _ = std::fs::remove_dir_all(folder);
    std::fs::create_dir_all(folder)?;
    let copy = folder.join(name);
    copy_whole(installed, &copy)?;
    Ok(copy)
}

/// Put the kept copy back where the app is installed. The version being
/// replaced is moved aside first and put back if the copy fails, so the
/// install path always holds a whole app.
pub fn restore(copy: &Path, installed: &Path) -> std::io::Result<()> {
    if std::fs::symlink_metadata(copy).is_err() {
        return Err(std::io::Error::other(
            "the previous version is no longer kept",
        ));
    }
    let mut aside = installed.as_os_str().to_owned();
    aside.push(".failed");
    let aside = PathBuf::from(aside);
    remove(&aside);
    std::fs::rename(installed, &aside)?;
    if let Err(error) = copy_whole(copy, installed) {
        remove(installed);
        let _ = std::fs::rename(&aside, installed);
        return Err(error);
    }
    remove(&aside);
    Ok(())
}

/// Forget the kept copy, once the new version has started.
pub fn discard(folder: &Path) {
    remove(folder);
}

fn remove(path: &Path) {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            let _ = std::fs::remove_dir_all(path);
        }
        Ok(_) => {
            let _ = std::fs::remove_file(path);
        }
        Err(_) => {}
    }
}

/// Copy a file or a folder with its links, modes and times as they are. The
/// system's own `cp` does this for an app bundle, whose signature covers its
/// links and whose files must stay executable.
#[cfg(unix)]
fn copy_whole(from: &Path, to: &Path) -> std::io::Result<()> {
    let status = std::process::Command::new("cp")
        .arg("-Rp")
        .arg(from)
        .arg(to)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "copying {} ended with {status}",
            from.display()
        )))
    }
}

#[cfg(not(unix))]
fn copy_whole(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::copy(from, to).map(|_| ())
}

#[cfg(all(test, unix))]
mod tests {
    use super::{discard, installed, keep, restore};
    use std::path::{Path, PathBuf};

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("mash-previous-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn app(root: &Path, version: &str) -> PathBuf {
        let bundle = root.join("Mash.app");
        std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        std::fs::write(bundle.join("Contents/version"), version).unwrap();
        std::fs::write(bundle.join("Contents/MacOS/mash"), "#!/bin/sh\n").unwrap();
        std::os::unix::fs::symlink("MacOS/mash", bundle.join("Contents/link")).unwrap();
        bundle
    }

    fn version(bundle: &Path) -> String {
        std::fs::read_to_string(bundle.join("Contents/version")).unwrap()
    }

    #[test]
    fn the_previous_app_is_kept_whole_and_comes_back() {
        let root = scratch("restore");
        let bundle = app(&root.join("Applications"), "old");
        let kept = root.join("data/previous");
        let copy = keep(&bundle, &kept).unwrap();
        assert_eq!(version(&copy), "old");
        assert!(
            std::fs::symlink_metadata(copy.join("Contents/link"))
                .unwrap()
                .file_type()
                .is_symlink(),
            "links stay links"
        );
        // The update arrives; then the person goes back.
        std::fs::write(bundle.join("Contents/version"), "new").unwrap();
        restore(&copy, &bundle).unwrap();
        assert_eq!(version(&bundle), "old");
        assert!(!root.join("Applications/Mash.app.failed").exists());
        discard(&kept);
        assert!(!kept.exists());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn going_back_without_a_kept_copy_changes_nothing() {
        let root = scratch("missing");
        let bundle = app(&root.join("Applications"), "new");
        assert!(restore(&root.join("data/previous/Mash.app"), &bundle).is_err());
        assert_eq!(version(&bundle), "new");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn keeping_again_replaces_the_older_copy() {
        let root = scratch("again");
        let bundle = app(&root.join("Applications"), "one");
        let kept = root.join("data/previous");
        keep(&bundle, &kept).unwrap();
        std::fs::write(bundle.join("Contents/version"), "two").unwrap();
        let copy = keep(&bundle, &kept).unwrap();
        assert_eq!(version(&copy), "two");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn finds_what_the_updater_replaces() {
        let found = installed(
            Path::new("/Applications/Mash.app/Contents/MacOS/mash-desktop"),
            None,
        );
        if cfg!(target_os = "macos") {
            assert_eq!(found, Some(PathBuf::from("/Applications/Mash.app")));
            // A development build runs from Cargo's folder, not a bundle.
            assert_eq!(
                installed(Path::new("/src/target/debug/mash-desktop"), None),
                None
            );
        } else {
            assert_eq!(found, None, "only an AppImage is replaced on Linux");
        }
    }
}
