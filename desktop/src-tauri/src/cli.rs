//! The `mash` command line from inside the app.
//!
//! The app carries its own `mash`, but nothing puts it on the reader's PATH,
//! so a page that says "run mash index" names the app's own binary instead
//! (`command`, shown by the dashboard in place of `mash`). "Install
//! Command-Line Tool…" links that binary into `~/.local/bin` on macOS and
//! Linux. It never replaces a `mash` that is already there, says what it did,
//! and the same item removes the link again.

use crate::engine;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// The command a reader types for this app's `mash`, quoted for their shell.
pub fn command(program: &Path) -> String {
    let text = program.to_string_lossy();
    let plain = text
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "/\\._-+:".contains(character));
    if plain {
        text.into_owned()
    } else if cfg!(windows) {
        // PowerShell runs a quoted path only after the call operator.
        format!("& \"{text}\"")
    } else {
        format!("'{}'", text.replace('\'', r"'\''"))
    }
}

/// The page script line that tells the dashboard how to name `mash`.
pub fn page_script() -> String {
    let command = engine::locate().map(|program| command(&program)).ok();
    format!(
        "window.__MASH_DESKTOP_COMMAND__ = {};",
        serde_json::to_string(&command).unwrap_or_else(|_| "null".into())
    )
}

/// What installing the command-line tool would do on this machine.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    /// Make the link.
    Link,
    /// The link is there and points at this app: offer to remove it.
    Linked,
    /// Something else is called `mash` there; it is left alone.
    Taken(String),
}

fn plan(link: &Path, program: &Path) -> Plan {
    let Ok(metadata) = std::fs::symlink_metadata(link) else {
        return Plan::Link;
    };
    if metadata.file_type().is_symlink() {
        match std::fs::read_link(link) {
            Ok(target) if same_file(&target, program) => Plan::Linked,
            Ok(target) => Plan::Taken(format!("a link to {}", target.display())),
            Err(_) => Plan::Taken("a link Mash couldn’t read".into()),
        }
    } else {
        Plan::Taken("a separate copy of mash, such as one the install script put there".into())
    }
}

fn same_file(one: &Path, other: &Path) -> bool {
    one == other
        || matches!(
            (std::fs::canonicalize(one), std::fs::canonicalize(other)),
            (Ok(one), Ok(other)) if one == other
        )
}

/// The folder the link goes in.
fn bin_folder() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("bin"))
}

/// Whether the app's own PATH holds `folder`. A terminal's PATH may differ,
/// on macOS especially, where apps get a short default one.
fn on_path(folder: &Path) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|entry| entry == folder))
}

/// Link the app's `mash` into `~/.local/bin`, or offer to remove that link,
/// and say what happened.
pub fn install(app: &AppHandle) {
    let program = match engine::locate() {
        Ok(program) => std::fs::canonicalize(&program).unwrap_or(program),
        Err(error) => return tell(app, MessageDialogKind::Error, &error),
    };
    // An AppImage runs from a folder that changes on every start, so a link
    // into it would stop working once the app quits.
    if std::env::var_os("APPIMAGE").is_some() {
        return tell(
            app,
            MessageDialogKind::Info,
            "This copy of Mash runs from an AppImage, whose files move each time it starts, \
             so a link to its mash would stop working. Install the Mash command-line tool \
             from getmash.io instead.",
        );
    }
    // A package that installs the app into a folder on PATH (such as
    // /usr/bin for the .deb) has already put mash there.
    if program.parent().is_some_and(on_path) {
        return tell(
            app,
            MessageDialogKind::Info,
            &format!(
                "mash is already on your PATH, at {}. Open a terminal and run mash.",
                program.display()
            ),
        );
    }
    let Some(folder) = bin_folder() else {
        return tell(
            app,
            MessageDialogKind::Error,
            "Mash couldn’t find your home folder.",
        );
    };
    let link = folder.join("mash");
    match plan(&link, &program) {
        Plan::Link => tell(
            app,
            MessageDialogKind::Info,
            &make_link(&folder, &link, &program),
        ),
        Plan::Taken(what) => tell(
            app,
            MessageDialogKind::Info,
            &format!(
                "{} is already {what}. Mash left it as it is. To use this app's mash instead, \
                 remove that file and choose Install Command-Line Tool… again.",
                link.display()
            ),
        ),
        Plan::Linked => offer_removal(app, link, &program),
    }
}

#[cfg(unix)]
fn make_link(folder: &Path, link: &Path, program: &Path) -> String {
    if let Err(error) =
        std::fs::create_dir_all(folder).and_then(|()| std::os::unix::fs::symlink(program, link))
    {
        return format!("Mash couldn’t create {}: {error}.", link.display());
    }
    let reach = if on_path(folder) {
        "Open a new terminal and run mash.".to_owned()
    } else {
        format!(
            "Open a new terminal and run mash. If it isn’t found, add {} to your PATH, \
             for example with export PATH=\"$HOME/.local/bin:$PATH\" in your shell’s profile.",
            folder.display()
        )
    };
    format!(
        "{} now runs the mash inside this app ({}). {reach} To remove it, choose \
         Install Command-Line Tool… again.",
        link.display(),
        program.display()
    )
}

#[cfg(not(unix))]
fn make_link(_folder: &Path, _link: &Path, _program: &Path) -> String {
    "Installing the command-line tool from the app is available on macOS and Linux.".into()
}

fn offer_removal(app: &AppHandle, link: PathBuf, program: &Path) {
    let handle = app.clone();
    app.dialog()
        .message(format!(
            "{} already runs the mash inside this app ({}). Remove it?",
            link.display(),
            program.display()
        ))
        .title("Command-Line Tool")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Remove".into(),
            "Keep".into(),
        ))
        .show(move |remove| {
            if !remove {
                return;
            }
            let said = match std::fs::remove_file(&link) {
                Ok(()) => format!("Removed {}.", link.display()),
                Err(error) => format!("Mash couldn’t remove {}: {error}.", link.display()),
            };
            tell(&handle, MessageDialogKind::Info, &said);
        });
}

fn tell(app: &AppHandle, kind: MessageDialogKind, message: &str) {
    app.dialog()
        .message(message)
        .title("Command-Line Tool")
        .kind(kind)
        .show(|_| {});
}

#[cfg(test)]
mod tests {
    use super::{Plan, command, plan};
    use std::path::Path;

    #[test]
    fn names_the_bundled_program_ready_to_paste() {
        assert_eq!(
            command(Path::new("/Applications/Mash.app/Contents/MacOS/mash")),
            "/Applications/Mash.app/Contents/MacOS/mash"
        );
        let spaced = command(Path::new("/Users/sam/My Apps/Mash.app/Contents/MacOS/mash"));
        if cfg!(windows) {
            assert!(spaced.starts_with("& \""), "{spaced}");
        } else {
            assert_eq!(spaced, "'/Users/sam/My Apps/Mash.app/Contents/MacOS/mash'");
            assert_eq!(command(Path::new("/tmp/it's/mash")), r"'/tmp/it'\''s/mash'");
        }
    }

    #[cfg(unix)]
    #[test]
    fn never_replaces_another_mash() {
        let folder = std::env::temp_dir().join(format!("mash-cli-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let program = folder.join("bundled-mash");
        std::fs::write(&program, "").unwrap();
        let link = folder.join("mash");
        assert_eq!(plan(&link, &program), Plan::Link);
        std::os::unix::fs::symlink(&program, &link).unwrap();
        assert_eq!(plan(&link, &program), Plan::Linked);
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("/somewhere/else/mash", &link).unwrap();
        assert!(matches!(plan(&link, &program), Plan::Taken(_)));
        std::fs::remove_file(&link).unwrap();
        std::fs::write(&link, "").unwrap();
        assert!(matches!(plan(&link, &program), Plan::Taken(_)));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
