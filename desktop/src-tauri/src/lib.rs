//! The Mash desktop app: one window showing the Mash dashboard, served by the
//! `mash` engine that ships inside the app.
//!
//! The window loads the dashboard over loopback exactly as a browser would, so
//! every page, theme and behaviour comes from the dashboard itself. The app
//! adds only what a browser tab cannot: it starts and stops the engine, opens
//! mash:// links, keeps a single window, lives in the menu bar with a quick
//! search panel (menubar.rs), and sends any address that is not the dashboard
//! to the reader's browser.

mod engine;
mod links;
mod menubar;

use engine::Engine;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{AppHandle, Manager, RunEvent, Theme, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

const WINDOW: &str = "main";

/// What the app knows about its engine and the links waiting for it.
#[derive(Default)]
struct Shared {
    engine: Option<Engine>,
    /// The dashboard's address once the engine is ready.
    dashboard: Option<Url>,
    /// The route a mash:// link asked for before the dashboard was ready.
    waiting: Option<String>,
}

type State = Arc<Mutex<Shared>>;

fn lock(state: &State) -> std::sync::MutexGuard<'_, Shared> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn run() {
    let state = State::default();
    let builder = tauri::Builder::default();
    // A second launch hands over to the running app and quits. One that
    // carries a mash:// link leaves the window to the link, which the
    // deep-link plugin delivers (mash://quick shows only the search panel);
    // a plain relaunch brings the main window forward.
    #[cfg(any(target_os = "linux", target_os = "macos", windows))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(
        |app, arguments, _directory| {
            if !arguments
                .iter()
                .any(|argument| links::route(argument).is_some())
            {
                show(app);
            }
        },
    ));
    let app = builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(menubar::shortcut_plugin())
        .manage(state.clone())
        .setup(move |app| {
            open_window(app.handle())?;
            menubar::install(app.handle())?;
            listen_for_links(app.handle());
            start_engine(app.handle().clone(), state.clone());
            Ok(())
        })
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => {
            eprintln!("Mash couldn’t open: {error}");
            std::process::exit(1);
        }
    };
    app.run(|app, event| match event {
        RunEvent::Exit => {
            let state = app.state::<State>();
            if let Some(mut engine) = lock(&state).engine.take() {
                engine.stop();
            }
        }
        // Clicking the Dock icon brings back a window closed into the menu bar.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => show(app),
        _ => {}
    });
}

/// The window, first showing the app's loading page.
fn open_window(app: &AppHandle) -> tauri::Result<()> {
    let navigation = app.clone();
    let popups = app.clone();
    let downloads = app.clone();
    let (width, height) = window_size(app);
    WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("index.html".into()))
        .title("Mash")
        .inner_size(width, height)
        .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
        .center()
        // Mash has one theme, the light one, so the title bar is light too.
        .theme(Some(Theme::Light))
        .initialization_script(include_str!("window.js"))
        .on_navigation(move |url| navigate(&navigation, url, false))
        .on_new_window(move |url, _features| {
            // An empty window is the one the dashboard writes a chat into for
            // printing to PDF; it holds no address of its own.
            if url.scheme() == "about" {
                return NewWindowResponse::Allow;
            }
            open_outside(&popups, &url);
            NewWindowResponse::Deny
        })
        .on_download(move |_webview, event| {
            if let DownloadEvent::Requested { destination, .. } = event
                && let Ok(folder) = downloads.path().download_dir()
            {
                *destination = unused_path(&folder, destination);
            }
            true
        })
        .build()?;
    // Closing the window leaves Mash in the menu bar, as a menu bar app does;
    // Quit in its menu ends it. Linux desktops do not all show a tray icon,
    // so there closing the window quits.
    #[cfg(any(target_os = "macos", windows))]
    {
        let window = app
            .get_webview_window(WINDOW)
            .ok_or(tauri::Error::WindowNotFound)?;
        let hidden = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hidden.hide();
            }
        });
    }
    Ok(())
}

/// The window's size when the screen has room, and its smallest size, in
/// logical pixels.
const PREFERRED_SIZE: (f64, f64) = (1280.0, 820.0);
const MIN_SIZE: (f64, f64) = (420.0, 480.0);

/// The window's opening size: the preferred size, shrunk to fit the main
/// screen's work area (the screen less its taskbar or dock).
fn window_size(app: &AppHandle) -> (f64, f64) {
    let Ok(Some(monitor)) = app.primary_monitor() else {
        return PREFERRED_SIZE;
    };
    let area = monitor
        .work_area()
        .size
        .to_logical::<f64>(monitor.scale_factor());
    fit(area.width, area.height)
}

/// The preferred size, shrunk to leave a margin and room for the title bar
/// inside a work area of `width` by `height`, but never below the minimum.
fn fit(width: f64, height: f64) -> (f64, f64) {
    const TITLE_BAR: f64 = 40.0;
    let room =
        |preferred: f64, available: f64, minimum: f64| preferred.min(available * 0.94).max(minimum);
    (
        room(PREFERRED_SIZE.0, width, MIN_SIZE.0),
        room(PREFERRED_SIZE.1, height - TITLE_BAR, MIN_SIZE.1),
    )
}

/// What a window does with an address it is asked to load.
#[derive(Debug, PartialEq, Eq)]
enum Navigation {
    /// Load it: the app's own page, or a page of the dashboard.
    Load,
    /// A mash:// link, which the app follows in its main window.
    Follow,
    /// Anything else, which belongs in the reader's browser.
    Outside,
}

fn navigation(url: &Url, dashboard: Option<&Url>) -> Navigation {
    let local_page = url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost");
    let same_origin = dashboard.is_some_and(|dashboard| dashboard.origin() == url.origin());
    let in_page = matches!(url.scheme(), "about" | "blob");
    if local_page || same_origin || in_page {
        Navigation::Load
    } else if url.scheme() == "mash" {
        Navigation::Follow
    } else {
        Navigation::Outside
    }
}

/// Whether a window (the panel, when `from_panel`) may load `url` itself.
/// A mash:// link is how the dashboard asks for a page of the main window, so
/// the panel opens its rows and menu that way and then gets out of the way.
pub(crate) fn navigate(app: &AppHandle, url: &Url, from_panel: bool) -> bool {
    let dashboard = lock(&app.state::<State>()).dashboard.clone();
    match navigation(url, dashboard.as_ref()) {
        Navigation::Load => true,
        Navigation::Follow => {
            if from_panel {
                menubar::hide(app);
            }
            follow(app, url.as_str());
            false
        }
        Navigation::Outside => {
            open_outside(app, url);
            false
        }
    }
}

/// Open a web or mail address in the program the reader uses for it.
fn open_outside(app: &AppHandle, url: &Url) {
    if matches!(url.scheme(), "http" | "https" | "mailto") {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
    }
}

/// A file in `folder` named like the download, numbered so nothing is replaced.
fn unused_path(folder: &Path, suggested: &Path) -> PathBuf {
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

/// Follow mash:// links, both the one that launched the app and later ones.
fn listen_for_links(app: &AppHandle) {
    // Installed apps register mash:// when installed; this also covers an app
    // run without installing, such as an AppImage or a development build.
    #[cfg(any(target_os = "linux", windows))]
    let _ = app.deep_link().register_all();
    let handle = app.clone();
    app.deep_link().on_open_url(move |event| {
        for url in event.urls() {
            follow(&handle, url.as_str());
        }
    });
    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for url in urls {
            follow(app, url.as_str());
        }
    }
}

/// Show the page a mash:// link names, or keep it until the engine is ready.
/// mash://quick shows the search panel instead, for launchers and shortcuts.
fn follow(app: &AppHandle, link: &str) {
    let Some(route) = links::route(link) else {
        return;
    };
    if route == "#/quick" {
        menubar::toggle(app, menubar::Placement::Centre);
        return;
    }
    open_route(app, &route);
}

/// Show a dashboard route in the main window, or keep it until the engine is
/// ready.
pub(crate) fn open_route(app: &AppHandle, route: &str) {
    let route = route.to_owned();
    let state = app.state::<State>();
    let mut shared = lock(&state);
    match shared.dashboard.clone() {
        Some(dashboard) => {
            drop(shared);
            show_route(app, &dashboard, &route);
        }
        None => shared.waiting = Some(route),
    }
    show(app);
}

/// The dashboard page for a route such as `#/chat/abc?node=laptop`.
pub(crate) fn page(dashboard: &Url, route: &str) -> Url {
    let mut page = dashboard.clone();
    page.set_fragment(Some(route.trim_start_matches('#')));
    page
}

fn show_route(app: &AppHandle, dashboard: &Url, route: &str) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.navigate(page(dashboard, route));
    }
}

/// Bring the window forward.
pub(crate) fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Start the engine away from the window's thread, then open the dashboard,
/// and report on the loading page if it fails to start or later stops.
fn start_engine(app: AppHandle, state: State) {
    std::thread::spawn(move || match Engine::start() {
        Ok(engine) => {
            let dashboard = engine.address.clone();
            let route = {
                let mut shared = lock(&state);
                shared.dashboard = Some(dashboard.clone());
                shared.engine = Some(engine);
                shared.waiting.take()
            };
            show_route(&app, &dashboard, route.as_deref().unwrap_or("#/"));
            if let Err(error) = menubar::create(&app, &dashboard) {
                eprintln!("Mash couldn’t prepare its search panel: {error}");
            }
            watch_engine(&app, &state);
        }
        Err(error) => show_error(&app, &error),
    });
}

/// Wait for the engine to stop on its own, which only happens when it fails.
fn watch_engine(app: &AppHandle, state: &State) {
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let mut shared = lock(state);
        let Some(engine) = shared.engine.as_mut() else {
            return; // The app is quitting.
        };
        if engine.stopped() {
            shared.engine = None;
            shared.dashboard = None;
            drop(shared);
            show_error(app, "The Mash engine stopped.");
            return;
        }
    }
}

/// Show `message` on the app's loading page.
fn show_error(app: &AppHandle, message: &str) {
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    let Ok(mut local) = window.url() else {
        return;
    };
    // The loading page's address; the window may be showing the dashboard.
    if navigation(&local, None) != Navigation::Load {
        local = local_page();
    }
    local.set_path("/index.html");
    local.set_fragment(None);
    local
        .query_pairs_mut()
        .clear()
        .append_pair("error", message);
    let _ = window.navigate(local);
}

/// The address the app serves its own page from on this platform.
fn local_page() -> Url {
    let address = if cfg!(windows) {
        "http://tauri.localhost/"
    } else {
        "tauri://localhost/"
    };
    Url::parse(address).expect("a fixed, valid address")
}

#[cfg(test)]
mod tests {
    use super::{Navigation, fit, navigation, page, unused_path};
    use tauri::Url;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn only_the_dashboard_and_the_app_page_load_in_the_window() {
        let dashboard = url("http://127.0.0.1:47290/");
        let stays_in_app =
            |url: &Url, dashboard: Option<&Url>| navigation(url, dashboard) == Navigation::Load;
        for inside in [
            "http://127.0.0.1:47290/#/chat/abc",
            "http://127.0.0.1:47290/api/export",
            "tauri://localhost/index.html",
            "http://tauri.localhost/index.html",
            "about:blank",
            "blob:http://127.0.0.1:47290/5f1c",
        ] {
            assert!(stays_in_app(&url(inside), Some(&dashboard)), "{inside}");
        }
        for outside in [
            "https://getmash.io/agents/",
            "http://127.0.0.1:8790/",
            "http://localhost:47290/",
            "mailto:someone@example.com",
        ] {
            assert!(!stays_in_app(&url(outside), Some(&dashboard)), "{outside}");
        }
        assert!(
            !stays_in_app(&dashboard, None),
            "before the engine is ready"
        );
        // A mash:// link from the dashboard opens a page of the main window.
        for link in [
            "mash://chat/abc?message=m",
            "mash://settings",
            "mash://quick",
        ] {
            assert_eq!(
                navigation(&url(link), Some(&dashboard)),
                Navigation::Follow,
                "{link}"
            );
        }
        assert_eq!(
            navigation(&url("https://getmash.io/"), Some(&dashboard)),
            Navigation::Outside
        );
    }

    #[test]
    fn the_window_fits_the_screen() {
        assert_eq!(
            fit(2560.0, 1400.0),
            (1280.0, 820.0),
            "a large screen gets the preferred size"
        );
        let (width, height) = fit(1280.0, 760.0);
        assert!(
            width < 1280.0 && height + 40.0 < 760.0,
            "{width}x{height} fits 1280x760"
        );
        assert_eq!(fit(300.0, 300.0), (420.0, 480.0), "never below the minimum");
    }

    #[test]
    fn a_route_becomes_the_dashboard_page() {
        let dashboard = url("http://127.0.0.1:47290/");
        assert_eq!(
            page(&dashboard, "#/chat/abc?node=laptop").as_str(),
            "http://127.0.0.1:47290/#/chat/abc?node=laptop"
        );
        assert_eq!(page(&dashboard, "#/").as_str(), "http://127.0.0.1:47290/#/");
    }

    #[test]
    fn the_app_has_the_version_of_the_mash_it_bundles() {
        // The app builds outside the Mash workspace, so its version is
        // restated in its own manifest; this keeps the two together.
        let workspace = include_str!("../../../Cargo.toml");
        let version = workspace
            .split("[workspace.package]")
            .nth(1)
            .and_then(|section| {
                section
                    .lines()
                    .find_map(|line| line.strip_prefix("version = "))
            })
            .map(|value| value.trim().trim_matches('"'));
        assert_eq!(version, Some(env!("CARGO_PKG_VERSION")));
    }

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
