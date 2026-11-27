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
mod context;
mod downloads;
mod engine;
mod frame;
mod go;
mod links;
mod lockstep;
mod login;
mod menubar;
mod navigation;
mod print;
mod size;
mod stop;
mod update;

/// The Windows installer's requests for the `mash` command (cli/windows.rs).
#[cfg(windows)]
pub use cli::installer_request;
use engine::Engine;
use navigation::{Navigation, navigation};
pub(crate) use navigation::{navigate, open_outside};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, RunEvent, Theme, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_deep_link::DeepLinkExt;

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
    // Something that replaced the app without its installer (a copied
    // folder, a repair) may have left the `mash` command behind.
    #[cfg(windows)]
    cli::keep_in_step();
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
        .plugin(frame::remembered())
        .plugin(login::plugin())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state.clone())
        .manage(downloads::Downloads::default())
        .invoke_handler(tauri::generate_handler![
            retry_engine,
            engine_log,
            update::banner::update_banner,
            update::banner::update_install,
            update::banner::update_later,
            context::context_menu
        ])
        .on_menu_event(|app, event| menubar::choose(app, event.id().as_ref()))
        .setup(move |app| {
            update::prepare(app.handle());
            frame::dark_menus();
            open_window(app.handle())?;
            menubar::install(app.handle())?;
            listen_for_links(app.handle());
            start_engine(app.handle().clone(), state.clone());
            update::start(app.handle());
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
        } => {
            show(app);
        }
        _ => {}
    });
}

/// What runs in the app's windows before each page loads, the dashboard's
/// included: the flag that says the page is in the app, and how to name the
/// app's own `mash` in a command.
pub(crate) fn window_script() -> String {
    let (page, command, shell) = (
        include_str!("window.js"),
        cli::page_script(),
        frame::shell_script(),
    );
    format!("{page}\n{command}\n{shell}\n")
}

/// A menu choice the menu bar's own items do not cover: the Go menu, a
/// right-click menu, Open at Login, or an update's.
pub(crate) fn other_choice(app: &AppHandle, id: &str) {
    if !(go::choose(app, id) || context::choose(app, id) || login::choose(app, id)) {
        update::choose(app, id);
    }
}

/// The window, first showing the app's loading page.
fn open_window(app: &AppHandle) -> tauri::Result<()> {
    let navigation = app.clone();
    let popups = app.clone();
    let downloads = app.clone();
    let (width, height) = size::opening(app);
    let builder = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("index.html".into()))
        .title("Mash")
        .inner_size(width, height)
        .min_inner_size(size::MIN.0, size::MIN.1)
        .center()
        // Mash is dark only, so the title bar is too. On macOS a window's theme
        // sets the whole app's appearance, so the panel and notices are dark too.
        .theme(Some(Theme::Dark))
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
        .on_download(move |_webview, event| downloads::handle(&downloads, event));
    frame::fit(builder).build()?;
    let window = app
        .get_webview_window(WINDOW)
        .ok_or(tauri::Error::WindowNotFound)?;
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

/// The dashboard's address, once the engine is ready.
pub(crate) fn dashboard(app: &AppHandle) -> Option<Url> {
    lock(&app.state::<State>()).dashboard.clone()
}

/// Show the page a mash:// link names, or keep it until the engine is ready.
/// mash://quick shows the search panel instead, for launchers and shortcuts.
pub(crate) fn follow(app: &AppHandle, link: &str) {
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

/// Bring the window forward, opening it again if it was closed. None when
/// there is no window to show.
pub(crate) fn show(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    let window = match app.get_webview_window(WINDOW) {
        Some(window) => window,
        None => {
            if let Err(error) = open_window(app) {
                eprintln!("Mash couldn’t open its window: {error}");
                return None;
            }
            let dashboard = lock(&app.state::<State>()).dashboard.clone();
            if let Some(dashboard) = dashboard {
                show_route(app, &dashboard, "#/");
            }
            app.get_webview_window(WINDOW)?
        }
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Some(window)
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
    std::thread::spawn(move || {
        update::before_engine(&app, &log);
        engine_thread(&app, &state, &log, pid_file);
    });
}

fn engine_thread(app: &AppHandle, state: &State, log: &engine::Log, pid_file: Option<PathBuf>) {
    match Engine::start(log, pid_file) {
        Ok(engine) => {
            let dashboard = engine.address.clone();
            let route = {
                let mut shared = lock(state);
                shared.dashboard = Some(dashboard.clone());
                shared.engine = Some(engine);
                shared.starting = false;
                shared.waiting.take()
            };
            show_route(app, &dashboard, route.as_deref().unwrap_or("#/"));
            if let Err(error) = menubar::create(app, &dashboard) {
                eprintln!("Mash couldn’t prepare its search panel: {error}");
            }
            update::engine_started(app);
            watch_engine(app, state);
        }
        Err(error) => {
            lock(state).starting = false;
            update::engine_failed(app);
            show_error(app, &error);
        }
    }
}

/// Stop the engine before an update replaces it. The panel shows one of its
/// pages, so that closes too.
pub(crate) fn stop_engine(app: &AppHandle) {
    let state = app.state::<State>();
    let engine = {
        let mut shared = lock(&state);
        shared.dashboard = None;
        shared.engine.take()
    };
    menubar::close(app);
    if let Some(mut engine) = engine {
        engine.stop();
    }
}

/// Start the engine again after an update that did not install.
pub(crate) fn restart_engine(app: &AppHandle) {
    let state = app.state::<State>().inner().clone();
    start_engine(app.clone(), state);
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
    use super::page;
    use tauri::Url;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
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
