//! The `mash` command on Windows: the registry, the installer's requests and
//! the menu item. What the command is, and why, is in `path_command`.
//!
//! The installer puts the command in place by default and takes it away on
//! uninstall, by starting the app with a request (hooks.nsh). "Install
//! Command-Line Tool…" removes it and puts it back; a removal is remembered,
//! so installing a newer version over this one leaves it off. Only the
//! user's own PATH is written (`HKCU\Environment`), which needs no
//! administrator, and only by adding or removing this one folder: a PATH
//! that cannot be read as text is never written.

use super::path_command::{self, FOLDER, IDENTIFIER, Request};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE, Type};

/// Where a user's own environment is kept, and the system's.
const USER: &str = "Environment";
const SYSTEM: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";
/// "The system cannot find the file specified": the value is not there.
const NOT_FOUND: i32 = 0x8007_0002_u32.cast_signed();

/// The app's `mash`, and the command that is a copy of it.
struct Layout {
    engine: PathBuf,
    folder: PathBuf,
    command: PathBuf,
}

/// The layout around this copy of the app. The paths are the ones Windows
/// reports, never canonical ones, whose `\\?\` form PATH does not take.
fn layout() -> Result<Layout, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let directory = executable.parent().unwrap_or(Path::new("."));
    let engine = directory.join("mash.exe");
    if !engine.is_file() {
        return Err(format!(
            "The Mash engine is missing from this app ({}).",
            engine.display()
        ));
    }
    let folder = directory.join(FOLDER);
    if folder.to_string_lossy().contains(';') {
        return Err(format!(
            "Mash is installed in {}, and a folder with a semicolon in its name cannot go on \
             PATH.",
            directory.display()
        ));
    }
    Ok(Layout {
        engine,
        command: folder.join("mash.exe"),
        folder,
    })
}

/// The file whose presence says the person removed the command, so an
/// installer does not put it back. It is in the app's own settings folder,
/// which outlives an update.
fn removed_mark() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|roaming| {
        PathBuf::from(roaming)
            .join(IDENTIFIER)
            .join("command-line-tool-removed")
    })
}

fn remember_removal(removed: bool) {
    let Some(mark) = removed_mark() else {
        return;
    };
    if removed {
        if let Some(folder) = mark.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let _ = std::fs::write(
            &mark,
            "The mash command was removed with Install Command-Line Tool… in the Mash app, \
             so installing the app again leaves it off.\n",
        );
    } else {
        let _ = std::fs::remove_file(&mark);
    }
}

/// The user's own PATH as the registry holds it: the text, and whether it
/// is the kind whose variables Windows expands.
struct UserPath {
    key: Key,
    text: String,
    expands: bool,
}

fn user_path() -> Result<UserPath, String> {
    fn failed(error: impl std::fmt::Display) -> String {
        format!("Mash couldn’t read your PATH: {error}.")
    }
    let key = CURRENT_USER.create(USER).map_err(failed)?;
    let value = match key.get_value("Path") {
        Ok(value) => value,
        Err(error) if error.code().0 == NOT_FOUND => {
            return Ok(UserPath {
                key,
                text: String::new(),
                expands: true,
            });
        }
        Err(error) => return Err(failed(error)),
    };
    let expands = match value.ty() {
        Type::ExpandString => true,
        Type::String => false,
        _ => return Err("Your PATH is not text in the registry, so Mash left it alone.".into()),
    };
    let text = String::try_from(value).map_err(failed)?;
    Ok(UserPath { key, text, expands })
}

/// Rewrite the user's PATH with `change`, which answers nothing when there
/// is nothing to change, and tell running programs. Whether it changed.
fn edit_user_path(change: impl Fn(&str) -> Option<String>) -> Result<bool, String> {
    let path = user_path()?;
    let Some(text) = change(&path.text) else {
        return Ok(false);
    };
    let written = if path.expands {
        path.key.set_expand_string("Path", &text)
    } else {
        path.key.set_string("Path", &text)
    };
    written.map_err(|error| format!("Mash couldn’t change your PATH: {error}."))?;
    announce();
    Ok(true)
}

/// Tell every top-level window that the environment changed, which is how
/// Explorer learns of it and hands the new PATH to terminals opened after.
fn announce() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };
    let what: Vec<u16> = "Environment".encode_utf16().chain(Some(0)).collect();
    // SAFETY: `what` is a NUL-terminated string that outlives the call, the
    // call copies nothing back, and a window that does not answer is given
    // up on after the timeout.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            what.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }
}

/// Another `mash` that a new terminal finds before the app's: one in the
/// system's PATH or earlier in the user's.
fn other_first(layout: &Layout) -> Option<PathBuf> {
    let variable = |name: &str| std::env::var(name).ok();
    let system = LOCAL_MACHINE
        .open(SYSTEM)
        .and_then(|key| key.get_string("Path"))
        .unwrap_or_default();
    let mut listed = path_command::folders(&system, variable);
    listed.extend(path_command::folders(&user_path().ok()?.text, variable));
    path_command::other_before(&listed, &layout.folder)
}

/// Copy the app's `mash` into place, if the copy there is not it already,
/// and put its folder on PATH.
fn put(layout: &Layout) -> Result<(), String> {
    if !path_command::in_step(&layout.engine, &layout.command) {
        path_command::place(&layout.engine, &layout.command).map_err(|error| {
            format!(
                "Mash couldn’t create {}: {error}.",
                layout.command.display()
            )
        })?;
    }
    super::runtime::carry(&layout.engine, &layout.command).map_err(|error| {
        format!(
            "Mash couldn’t put the Visual C++ runtime beside {}: {error}.",
            layout.command.display()
        )
    })?;
    let folder = layout.folder.to_string_lossy();
    edit_user_path(|path| path_command::with_folder(path, &folder))?;
    Ok(())
}

/// Take the command away and its folder off PATH.
fn take_away(layout: &Layout) -> Result<(), String> {
    let folder = layout.folder.to_string_lossy();
    // PATH first: should the file be held open, nothing points at it.
    edit_user_path(|path| path_command::without_folder(path, &folder))?;
    super::runtime::drop(&layout.command);
    path_command::remove(&layout.command).map_err(|error| {
        format!(
            "Mash couldn’t remove {}: {error}.",
            layout.command.display()
        )
    })
}

/// The command's file, which services and dashboards that follow the app's
/// version may name.
pub fn command_file() -> Option<PathBuf> {
    layout().ok().map(|layout| layout.command)
}

fn is_installed(layout: &Layout) -> bool {
    layout.command.is_file()
        && user_path().is_ok_and(|path| {
            path_command::has_folder(&path.text, &layout.folder.to_string_lossy())
        })
}

/// Do what the installer asked, if `arguments` are its request, and answer
/// with the status to exit with. Nothing is shown: an installer may be
/// running silently.
pub fn installer_request(arguments: impl Iterator<Item = String>) -> Option<i32> {
    let request = path_command::request(arguments)?;
    let done = layout().and_then(|layout| match request {
        Request::Installed if removed_mark().is_some_and(|mark| mark.exists()) => Ok(()),
        Request::Installed => put(&layout),
        Request::Uninstalling => take_away(&layout),
    });
    Some(i32::from(done.is_err()))
}

/// At launch: bring the command up to the app's `mash` when something
/// replaced the app without asking (it is otherwise already in step).
pub fn keep_in_step() {
    if let Ok(layout) = layout()
        && layout.command.is_file()
        && !path_command::in_step(&layout.engine, &layout.command)
    {
        let _ = path_command::place(&layout.engine, &layout.command);
    }
}

/// "Install Command-Line Tool…": put the command in place, or offer to
/// remove it when it is there, and say what happened.
pub fn install(app: &AppHandle) {
    let layout = match layout() {
        Ok(layout) => layout,
        Err(why) => return super::tell(app, MessageDialogKind::Error, &why),
    };
    if is_installed(&layout) {
        return offer_removal(app, layout);
    }
    let said = match put(&layout) {
        Ok(()) => {
            remember_removal(false);
            path_command::installed(&layout.command, other_first(&layout).as_deref())
        }
        Err(why) => why,
    };
    super::tell(app, MessageDialogKind::Info, &said);
}

fn offer_removal(app: &AppHandle, layout: Layout) {
    let handle = app.clone();
    app.dialog()
        .message(path_command::offer(
            &layout.command,
            other_first(&layout).as_deref(),
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
            let said = match take_away(&layout) {
                Ok(()) => {
                    remember_removal(true);
                    path_command::removed(&layout.command)
                }
                Err(why) => why,
            };
            super::tell(&handle, MessageDialogKind::Info, &said);
        });
}
