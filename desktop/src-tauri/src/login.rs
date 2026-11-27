//! Open at Login, a tick in the menu bar menu as Mac menu bar apps place it.
//! Tauri's autostart plugin registers the app with the system: a launch
//! agent on macOS, the Run key on Windows, an autostart entry on Linux. A
//! start at login passes `--at-login`, and then the app opens into the menu
//! bar alone, without its window, as menu bar apps start; any later start
//! opens the window as usual.

use tauri::menu::CheckMenuItem;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

const ID: &str = "open-at-login";
const AT_LOGIN: &str = "--at-login";

/// The plugin, telling the system to start the app with `--at-login`.
pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![AT_LOGIN]))
}

/// Whether the system started this run at login.
pub fn at_login() -> bool {
    std::env::args().any(|argument| argument == AT_LOGIN)
}

/// The menu item, ticked while the app is set to open at login.
pub fn item(app: &AppHandle) -> tauri::Result<CheckMenuItem<Wry>> {
    let on = app.autolaunch().is_enabled().unwrap_or(false);
    CheckMenuItem::with_id(app, ID, "Open at Login", true, on, None::<&str>)
}

/// Turns opening at login on or off; false when `id` is not this item.
pub fn choose(app: &AppHandle, id: &str) -> bool {
    if id != ID {
        return false;
    }
    let launch = app.autolaunch();
    let wanted = !launch.is_enabled().unwrap_or(false);
    let changed = if wanted {
        launch.enable()
    } else {
        launch.disable()
    };
    if let Err(error) = changed {
        eprintln!("Mash couldn’t change whether it opens at login: {error}");
    }
    true
}
