//! Open at Login, a tick in the menu bar menu as Mac menu bar apps place it.
//! The app registers itself with the system through `auto-launch`: a launch
//! agent on macOS, the Run key on Windows, an autostart entry on Linux. A
//! start at login passes `--at-login`, and then the app opens into the menu
//! bar alone, without its window, as menu bar apps start; any later start
//! opens the window as usual.
//!
//! On macOS the launch agent names the app's bundle identifier, so the
//! system lists it under Mash, with the app's icon, in Login Items and in its
//! background-activity notice. Without it macOS files the agent under the
//! developer's name instead.

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Manager, Wry};

const ID: &str = "open-at-login";
const AT_LOGIN: &str = "--at-login";

/// The system's record of opening the app at login.
struct Login(AutoLaunch);

/// Build the launcher for this copy of the app and keep it in the app's
/// state. A launcher already set up is written again, so it names this copy
/// and carries what the current version adds to it.
pub fn install(app: &AppHandle) {
    let launcher = match launcher(app) {
        Ok(launcher) => launcher,
        Err(error) => {
            eprintln!("Mash can’t offer Open at Login: {error}");
            return;
        }
    };
    if launcher.is_enabled().unwrap_or(false)
        && let Err(error) = launcher.enable()
    {
        eprintln!("Mash couldn’t refresh how it opens at login: {error}");
    }
    app.manage(Login(launcher));
}

fn launcher(app: &AppHandle) -> Result<AutoLaunch, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(&app.package_info().name)
        .set_args(&[AT_LOGIN]);
    #[cfg(target_os = "macos")]
    {
        let executable = executable
            .canonicalize()
            .map_err(|error| error.to_string())?;
        builder
            .set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent)
            .set_bundle_identifiers(&[app.config().identifier.as_str()])
            .set_app_path(&executable.display().to_string());
    }
    #[cfg(target_os = "linux")]
    {
        // An AppImage runs from a temporary mount; log in starts the file.
        let path = app.env().appimage.unwrap_or(executable);
        builder.set_app_path(&path.display().to_string());
    }
    #[cfg(windows)]
    builder.set_app_path(&executable.display().to_string());
    builder.build().map_err(|error| error.to_string())
}

/// Whether the system started this run at login.
pub fn at_login() -> bool {
    std::env::args().any(|argument| argument == AT_LOGIN)
}

/// The menu item, ticked while the app is set to open at login, and greyed
/// out when the system's launcher could not be set up.
pub fn item(app: &AppHandle) -> tauri::Result<CheckMenuItem<Wry>> {
    let login = app.try_state::<Login>();
    let on = login
        .as_ref()
        .is_some_and(|login| login.0.is_enabled().unwrap_or(false));
    CheckMenuItem::with_id(app, ID, "Open at Login", login.is_some(), on, None::<&str>)
}

/// Turns opening at login on or off; false when `id` is not this item.
pub fn choose(app: &AppHandle, id: &str) -> bool {
    if id != ID {
        return false;
    }
    let Some(login) = app.try_state::<Login>() else {
        return true;
    };
    let launch = &login.0;
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
