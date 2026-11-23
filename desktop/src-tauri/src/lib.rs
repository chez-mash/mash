//! The Mash desktop app: one window showing the Mash dashboard, served by the
//! `mash` engine that ships inside the app.
//!
//! The window loads the dashboard over loopback exactly as a browser would, so
//! every page, theme and behaviour comes from the dashboard itself. The app
//! adds only what a browser tab cannot: it starts and stops the engine, opens
//! mash:// links, keeps a single window, lives in the menu bar with a quick
//! search panel (menubar.rs), and sends any address that is not the dashboard
//! to the reader's browser. Around the dashboard it keeps only what a browser
//! would do itself: saving downloads (downloads.rs) and printing a chat to
//! PDF (print.rs), plus a way to run the bundled `mash` (cli.rs).

mod cli;
mod downloads;
mod engine;
mod links;
mod menubar;
mod print;
mod stop;

use engine::Engine;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, RunEvent, Theme, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

pub(crate) const WINDOW: &str = "main";

/// What the app knows about its engine and the links waiting for it.
#[derive(Default)]
struct Shared {
    engine: Option<Engine>,
    /// The dashboard's address once the engine is ready.
    dashboard: Option<Url>,
    /// The route a mash:// link asked for before the dashboard was ready.
    waiting: Option<String>,
    /// Whether an engine is being started.
    starting: bool,
    /// The engine's last lines of output, for the error page.
    log: engine::Log,
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(menubar::shortcut_plugin())
        .manage(state.clone())
        .manage(downloads::Downloads::default())
        .invoke_handler(tauri::generate_handler![retry_engine, engine_log])
        .on_menu_event(|app, event| menubar::choose(app, event.id().as_ref()))
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

/// What runs in the app's windows before each page loads, the dashboard's
/// included: the flag that says the page is in the app, and how to name the
/// app's own `mash` in a command.
pub(crate) fn window_script() -> String {
    format!("{}\n{}\n", include_str!("window.js"), cli::page_script())
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
        // On macOS that is set on this window alone (see light_title_bar).
        .theme((!cfg!(target_os = "macos")).then_some(Theme::Light))
        .initialization_script(window_script())
        .on_navigation(move |url| navigate(&navigation, url, false))
        .on_new_window(move |url, features| {
            // An empty window is the one the dashboard writes a chat into for
            // printing to PDF; it holds no address of its own.
            if url.scheme() == "about" {
                return print::open(&popups, url, features);
            }
            open_outside(&popups, &url);
            NewWindowResponse::Deny
        })
        .on_download(move |_webview, event| downloads::handle(&downloads, event))
        .build()?;
    let window = app
        .get_webview_window(WINDOW)
        .ok_or(tauri::Error::WindowNotFound)?;
    #[cfg(target_os = "macos")]
    light_title_bar(&window);
    let hidden = window.clone();
    let quitting = app.clone();
    window.on_window_event(move |event| match event {
        // Closing the window leaves Mash in the menu bar, as a menu bar app
        // does; Quit in its menu ends it.
        tauri::WindowEvent::CloseRequested { api, .. }
            if cfg!(any(target_os = "macos", windows)) =>
        {
            api.prevent_close();
            let _ = hidden.hide();
        }
        // Linux desktops do not all show a tray icon, so there closing the
        // window quits, rather than leaving Mash running with no window and,
        // perhaps, no icon to bring one back.
        tauri::WindowEvent::Destroyed if cfg!(not(any(target_os = "macos", windows))) => {
            quitting.exit(0);
        }
        _ => {}
    });
    Ok(())
}

/// Give the main window the light appearance, and only that window. Tauri's
/// window theme sets the whole app's appearance on macOS, which would also
/// turn the search panel and the app's notices light in dark mode; the panel
/// follows the system, as macOS's own menu bar extras do.
#[cfg(target_os = "macos")]
fn light_title_bar(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSWindow};
    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        let Ok(pointer) = target.ns_window() else {
            return;
        };
        // SAFETY: Tauri hands out the window's live NSWindow, and this runs
        // on the main thread, where AppKit may be used.
        unsafe {
            let ns_window = &*pointer.cast::<NSWindow>();
            let appearance = NSAppearance::appearanceNamed(NSAppearanceNameAqua);
            ns_window.setAppearance(appearance.as_deref());
        }
    });
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

/// Open a web or mail address in the program the reader uses for it, and a
/// chat's link into its agent's app (the dashboard's Open in Codex or Claude).
pub(crate) fn open_outside(app: &AppHandle, url: &Url) {
    if opens_outside(url) {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
    }
}

fn opens_outside(url: &Url) -> bool {
    matches!(
        url.scheme(),
        "http" | "https" | "mailto" | "codex" | "claude"
    )
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
        menubar::toggle(app, menubar::Placement::MenuBar);
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

/// Bring the window forward, opening it again if it was closed.
pub(crate) fn show(app: &AppHandle) {
    let window = match app.get_webview_window(WINDOW) {
        Some(window) => window,
        None => {
            if let Err(error) = open_window(app) {
                eprintln!("Mash couldn’t open its window: {error}");
                return;
            }
            let dashboard = lock(&app.state::<State>()).dashboard.clone();
            if let Some(dashboard) = dashboard {
                show_route(app, &dashboard, "#/");
            }
            let Some(window) = app.get_webview_window(WINDOW) else {
                return;
            };
            window
        }
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Start the engine away from the window's thread, then open the dashboard,
/// and report on the loading page if it fails to start or later stops.
fn start_engine(app: AppHandle, state: State) {
    let log = {
        let mut shared = lock(&state);
        if shared.starting || shared.engine.is_some() {
            return;
        }
        shared.starting = true;
        shared.log = engine::Log::default();
        shared.log.clone()
    };
    // The engine's process number, so a later start can stop an engine this
    // run leaves behind if it ends without quitting properly.
    let pid_file = app.path().app_data_dir().ok().and_then(|folder| {
        std::fs::create_dir_all(&folder).ok()?;
        Some(folder.join("engine.pid"))
    });
    std::thread::spawn(move || match Engine::start(&log, pid_file) {
        Ok(engine) => {
            let dashboard = engine.address.clone();
            let route = {
                let mut shared = lock(&state);
                shared.dashboard = Some(dashboard.clone());
                shared.engine = Some(engine);
                shared.starting = false;
                shared.waiting.take()
            };
            show_route(&app, &dashboard, route.as_deref().unwrap_or("#/"));
            if let Err(error) = menubar::create(&app, &dashboard) {
                eprintln!("Mash couldn’t prepare its search panel: {error}");
            }
            watch_engine(&app, &state);
        }
        Err(error) => {
            lock(&state).starting = false;
            show_error(&app, &error);
        }
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
        if let Some(status) = engine.stopped() {
            shared.engine = None;
            shared.dashboard = None;
            drop(shared);
            // The panel shows a page of the stopped engine; close it so the
            // icon and shortcut bring the main window and its error forward.
            menubar::close(app);
            show_error(app, &format!("The Mash engine stopped ({status})."));
            return;
        }
    }
}

/// Start the engine again from the error page.
#[tauri::command]
fn retry_engine(app: AppHandle) {
    let state = app.state::<State>().inner().clone();
    start_engine(app, state);
}

/// The engine's last lines of output, for the error page.
#[tauri::command]
fn engine_log(app: AppHandle) -> Vec<String> {
    let log = lock(&app.state::<State>()).log.clone();
    log.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .cloned()
        .collect()
}

/// Show `message` on the app's loading page, and bring the window forward.
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
    show(app);
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
    use super::{Navigation, fit, navigation, opens_outside, page};
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
        // Web, mail and agent-app links go to the system; other schemes do not.
        for system in [
            "https://getmash.io/",
            "mailto:someone@example.com",
            "codex://threads/01a1b649-827e-7692-a5b1-0de415a69df5",
            "claude://resume?session=586f44fa-8aaa-4587-b69f-b763d3279ef1",
        ] {
            assert!(opens_outside(&url(system)), "{system}");
        }
        for refused in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "vscode://file/x",
        ] {
            assert!(!opens_outside(&url(refused)), "{refused}");
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
}
