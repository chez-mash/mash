//! The `mash` command line from inside the app.
//!
//! The app carries its own `mash`, but nothing puts it on the reader's PATH,
//! so a page that says "run mash index" names the app's own binary instead
//! (`command`, shown by the dashboard in place of `mash`). "Install
//! Command-Line Tool…" puts a `mash` command in `~/.local/bin` that runs that
//! binary, so the command and the app are one version and one update covers
//! both. It never replaces a `mash` that is already there, says what it did,
//! and the same item removes the command again.
//!
//! On macOS the command is a small script that finds the app and replaces
//! itself with the app's `mash` (`exec`), so arguments, input, output, the
//! exit status and signals are those of `mash` itself; when the app has been
//! moved or removed it says so in one line instead of failing as a dangling
//! link would. On Linux it is a link, and only for an app installed at a
//! fixed place: an AppImage's files move on every start, so there `mash`
//! stays a separate install.

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

/// The first line after the interpreter in the command the app writes, by
/// which the app knows the file is its own.
const MARK: &str = "# mash-desktop-command";

/// The `mash` command for the app at `bundle`: run the `mash` inside it, or
/// inside a Mash.app in one of the two usual places when it has moved.
pub fn wrapper(bundle: &Path) -> String {
    wrapper_looking_in(bundle, "/Applications")
}

/// The command, with the system's applications folder named so a test can
/// stand in its own.
fn wrapper_looking_in(bundle: &Path, applications: &str) -> String {
    let recorded = bundle.to_string_lossy().replace('\'', r"'\''");
    format!(
        r#"#!/bin/sh
{MARK} 1
# Runs the mash inside the Mash app, so this command and the app are always
# the same version. Mash wrote this file (Install Command-Line Tool…); choose
# that again in the app to remove it.
recorded='{recorded}'
for app in "$recorded" "{applications}/Mash.app" "$HOME/Applications/Mash.app"; do
  if [ -x "$app/Contents/MacOS/mash" ]; then
    exec "$app/Contents/MacOS/mash" "$@"
  fi
done
echo "mash: Mash.app isn't installed at $recorded; reinstall it or run the install script (https://getmash.io)." >&2
exit 127
"#
    )
}

/// What installing the command-line tool would do on this machine.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    /// Nothing is there: write the command.
    Install,
    /// The app's own command is there: offer to remove it.
    Installed,
    /// A link to some other `mash`, which the app's command may replace when
    /// asked to be used everywhere.
    OtherLink(PathBuf),
    /// A separate copy of `mash`, such as one the install script put there.
    Separate,
}

fn plan(file: &Path, program: &Path) -> Plan {
    let Ok(metadata) = std::fs::symlink_metadata(file) else {
        return Plan::Install;
    };
    if metadata.file_type().is_symlink() {
        return match std::fs::read_link(file) {
            Ok(target) if same_file(&target, program) => Plan::Installed,
            Ok(target) => Plan::OtherLink(target),
            Err(_) => Plan::OtherLink(PathBuf::from("somewhere Mash couldn’t read")),
        };
    }
    // The app's script is small; a `mash` binary is not read to find out.
    let ours = metadata.len() < 4096
        && std::fs::read_to_string(file).is_ok_and(|text| {
            text.lines()
                .nth(1)
                .is_some_and(|line| line.starts_with(MARK))
        });
    if ours {
        Plan::Installed
    } else {
        Plan::Separate
    }
}

fn same_file(one: &Path, other: &Path) -> bool {
    one == other
        || matches!(
            (std::fs::canonicalize(one), std::fs::canonicalize(other)),
            (Ok(one), Ok(other)) if one == other
        )
}

/// Where the command goes: `~/.local/bin/mash`.
pub fn command_file() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("bin").join("mash"))
}

/// Whether the app's own PATH holds `folder`. A terminal's PATH may differ,
/// on macOS especially, where apps get a short default one.
fn on_path(folder: &Path) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|entry| entry == folder))
}

/// The bundled `mash`, and why the command cannot be installed for this copy
/// of the app, if it cannot.
fn installable() -> Result<PathBuf, String> {
    let program = engine::locate()?;
    let program = std::fs::canonicalize(&program).unwrap_or(program);
    // An AppImage runs from a folder that changes on every start, so nothing
    // may point into it.
    if std::env::var_os("APPIMAGE").is_some() {
        return Err(
            "This copy of Mash runs from an AppImage, whose files move each time it starts, \
             so a command that runs its mash would stop working. Install the Mash \
             command-line tool from getmash.io instead; it updates separately from the app."
                .into(),
        );
    }
    // A package that installs the app into a folder on PATH (such as
    // /usr/bin for the .deb) has already put mash there.
    if program.parent().is_some_and(on_path) {
        return Err(format!(
            "mash is already on your PATH, at {}. Open a terminal and run mash.",
            program.display()
        ));
    }
    Ok(program)
}

/// Install the app's `mash` command in `~/.local/bin`, or offer to remove
/// it, and say what happened.
pub fn install(app: &AppHandle) {
    let program = match installable() {
        Ok(program) => program,
        Err(why) => return tell(app, MessageDialogKind::Info, &why),
    };
    let Some(file) = command_file() else {
        return tell(
            app,
            MessageDialogKind::Error,
            "Mash couldn’t find your home folder.",
        );
    };
    let said = match plan(&file, &program) {
        Plan::Install => write_command(&file, &program),
        Plan::Installed => return offer_removal(app, file, &program),
        Plan::OtherLink(target) => format!(
            "{} is already a link to {}. Mash left it as it is; that mash updates separately \
             from this app. To use this app’s mash instead, remove that file and choose \
             Install Command-Line Tool… again.",
            file.display(),
            target.display()
        ),
        Plan::Separate => format!(
            "{} is already a separate copy of mash, such as one the install script put there. \
             Mash left it as it is; it updates separately from this app. To use this app’s \
             mash instead, remove that file and choose Install Command-Line Tool… again.",
            file.display()
        ),
    };
    tell(app, MessageDialogKind::Info, &said);
}

/// Make `~/.local/bin/mash` run the app's `mash` even when another `mash` is
/// there, for "Use the App’s Mash Everywhere", and say what was done. A link
/// is re-pointed; a separate copy is kept beside it as `mash.separate`,
/// never deleted. Copies elsewhere, such as a package manager's, are not
/// touched.
pub fn install_everywhere() -> String {
    let program = match installable() {
        Ok(program) => program,
        Err(why) => return why,
    };
    let Some(file) = command_file() else {
        return "Mash couldn’t find your home folder.".into();
    };
    let installed = |kept: Option<&Path>| match place(&file, &program) {
        Ok(()) => format!(
            "{} now runs this app’s mash and updates with the app.{}",
            file.display(),
            kept.map_or_else(String::new, |kept| format!(
                " The copy that was there is kept as {}.",
                kept.display()
            ))
        ),
        Err(error) => format!("Mash couldn’t create {}: {error}.", file.display()),
    };
    match plan(&file, &program) {
        Plan::Installed => format!("{} already runs this app’s mash.", file.display()),
        Plan::Install => installed(None),
        Plan::OtherLink(_) => match std::fs::remove_file(&file) {
            Ok(()) => installed(None),
            Err(error) => format!("Mash couldn’t replace {}: {error}.", file.display()),
        },
        Plan::Separate => {
            let kept = file.with_extension("separate");
            match std::fs::rename(&file, &kept) {
                Ok(()) => installed(Some(&kept)),
                Err(error) => format!("Mash couldn’t move {} aside: {error}.", file.display()),
            }
        }
    }
}

/// Write the command and say how to reach it.
#[cfg(unix)]
fn write_command(file: &Path, program: &Path) -> String {
    if let Err(error) = place(file, program) {
        return format!("Mash couldn’t create {}: {error}.", file.display());
    }
    let folder = file.parent().unwrap_or(file);
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
        "{} now runs the mash inside this app ({}), and updates with the app. {reach} \
         To remove it, choose Install Command-Line Tool… again.",
        file.display(),
        program.display()
    )
}

/// The script on macOS, where the app is a bundle the script can find again;
/// a link elsewhere.
#[cfg(unix)]
fn place(file: &Path, program: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(folder) = file.parent() {
        std::fs::create_dir_all(folder)?;
    }
    let bundle = program
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .filter(|bundle| {
            cfg!(target_os = "macos") && bundle.extension().is_some_and(|kind| kind == "app")
        });
    match bundle {
        Some(bundle) => {
            std::fs::write(file, wrapper(bundle))?;
            std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755))
        }
        None => std::os::unix::fs::symlink(program, file),
    }
}

#[cfg(not(unix))]
fn write_command(_file: &Path, _program: &Path) -> String {
    "Installing the command-line tool from the app is available on macOS and Linux.".into()
}

#[cfg(not(unix))]
fn place(_file: &Path, _program: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("not available on this system"))
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
    use super::{Plan, command, plan, wrapper, wrapper_looking_in};
    use std::path::{Path, PathBuf};

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
    fn scratch(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("mash cli {name} {}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[cfg(unix)]
    #[test]
    fn never_replaces_another_mash() {
        let folder = scratch("plan");
        let program = folder.join("bundled-mash");
        std::fs::write(&program, "").unwrap();
        let file = folder.join("mash");
        assert_eq!(plan(&file, &program), Plan::Install);
        std::os::unix::fs::symlink(&program, &file).unwrap();
        assert_eq!(plan(&file, &program), Plan::Installed);
        std::fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink("/somewhere/else/mash", &file).unwrap();
        assert_eq!(
            plan(&file, &program),
            Plan::OtherLink("/somewhere/else/mash".into())
        );
        std::fs::remove_file(&file).unwrap();
        std::fs::write(&file, "\x7fELF a separate copy").unwrap();
        assert_eq!(plan(&file, &program), Plan::Separate);
        std::fs::write(&file, "#!/bin/sh\nexec /opt/homebrew/bin/mash \"$@\"\n").unwrap();
        assert_eq!(
            plan(&file, &program),
            Plan::Separate,
            "someone else's script"
        );
        std::fs::write(&file, wrapper(Path::new("/Applications/Mash.app"))).unwrap();
        assert_eq!(
            plan(&file, &program),
            Plan::Installed,
            "the app's own script"
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// A stand-in app whose `mash` reports how it was run: its process
    /// number, each argument on a line of its own, what it read, and then
    /// exits with status 7.
    #[cfg(unix)]
    fn stand_in(root: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let bundle = root.join("My Apps").join("Mash.app");
        let program = bundle.join("Contents/MacOS/mash");
        std::fs::create_dir_all(program.parent().unwrap()).unwrap();
        std::fs::write(
            &program,
            "#!/bin/sh\necho \"pid $$\"\nfor argument in \"$@\"; do echo \"arg <$argument>\"; done\n\
             echo \"read <$(cat)>\"\necho 'to stderr' >&2\nexit 7\n",
        )
        .unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        bundle
    }

    #[cfg(unix)]
    fn run(
        script: &Path,
        home: &Path,
        arguments: &[&str],
        input: &str,
    ) -> (u32, std::process::Output) {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("/bin/sh")
            .arg(script)
            .args(arguments)
            .env("HOME", home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        (pid, child.wait_with_output().unwrap())
    }

    #[cfg(unix)]
    #[test]
    fn the_command_becomes_the_apps_mash() {
        let root = scratch("wrapper");
        let bundle = stand_in(&root);
        let script = root.join("mash");
        let nowhere = root.join("none").display().to_string();
        std::fs::write(&script, wrapper_looking_in(&bundle, &nowhere)).unwrap();
        let (pid, output) = run(
            &script,
            &root,
            &["search", "two words", "it's", "--flag=$HOME", ""],
            "a line in\n",
        );
        let said = String::from_utf8_lossy(&output.stdout);
        // The same process: a signal sent to the command reaches mash itself.
        assert!(said.contains(&format!("pid {pid}\n")), "{said}");
        assert!(
            said.contains(
                "arg <search>\narg <two words>\narg <it's>\narg <--flag=$HOME>\narg <>\n"
            ),
            "{said}"
        );
        assert!(said.contains("read <a line in>"), "{said}");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "to stderr\n");
        assert_eq!(output.status.code(), Some(7));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_moved_app_is_found_in_the_usual_place_and_a_missing_one_is_named() {
        let root = scratch("moved");
        let bundle = stand_in(&root);
        let script = root.join("mash");
        let nowhere = root.join("none").display().to_string();
        std::fs::write(
            &script,
            wrapper_looking_in(&root.join("it's gone/Mash.app"), &nowhere),
        )
        .unwrap();
        // Not where it was recorded, and in neither usual place.
        let (_, missing) = run(&script, &root, &["status"], "");
        assert_eq!(missing.status.code(), Some(127));
        assert!(missing.stdout.is_empty());
        let said = String::from_utf8_lossy(&missing.stderr);
        assert!(
            said.starts_with("mash: Mash.app isn't installed at ")
                && said.contains("it's gone/Mash.app;"),
            "{said}"
        );
        assert_eq!(said.lines().count(), 1, "{said}");
        // Moved into the home folder's Applications: found again.
        std::fs::create_dir_all(root.join("Applications")).unwrap();
        std::fs::rename(&bundle, root.join("Applications/Mash.app")).unwrap();
        let (_, found) = run(&script, &root, &["status"], "");
        assert_eq!(found.status.code(), Some(7));
        assert!(String::from_utf8_lossy(&found.stdout).contains("arg <status>"));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
