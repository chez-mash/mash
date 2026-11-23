//! Mash in the menu bar: an icon whose click drops the quick search panel
//! down under it, a native menu on right-click, and a global shortcut that
//! shows the same panel mid-screen.
//!
//! The panel is a page of the dashboard (`#/quick`) in a second, borderless
//! window, so its search, results and links are the dashboard's own; this
//! module only places, shows and hides that window. The panel asks for nothing
//! but to be hidden (capabilities/quick.json); a row it opens arrives as a
//! mash:// link, which the app follows in its main window.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Rect, Url, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The panel window's label.
pub const PANEL: &str = "quick";
/// The panel's size in logical pixels.
const PANEL_SIZE: (f64, f64) = (440.0, 420.0);
/// The shortcut that shows the panel from anywhere. ⌥Space is left to the
/// launchers (Spotlight's neighbours Raycast and Alfred use it by default).
pub const SHORTCUT: &str = "Alt+Shift+Space";

/// Where the panel appears.
pub enum Placement {
    /// Under the menu bar icon at this rectangle.
    Under(Rect),
    /// In the upper middle of the screen, like Spotlight.
    Centre,
}

/// The global shortcut plugin, whose shortcut is registered once the app is
/// running: registering it in the plugin would stop the app from starting
/// when another app already holds the shortcut.
pub fn shortcut_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                toggle(app, Placement::Centre);
            }
        })
        .build()
}

/// The icon, its right-click menu, and the shortcut. Menu choices, from this
/// menu and the app menu alike, arrive at `choose`.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    // Only the panel's shortcut is global. The others work while Mash is the
    // active app, through the app menu that macOS shows; elsewhere there is no
    // app menu, so the tray menu names no shortcut it cannot keep.
    let app_shortcut = |keys: &'static str| cfg!(target_os = "macos").then_some(keys);
    let search = MenuItem::with_id(app, "search", "Search Mash…", true, Some(SHORTCUT))?;
    let open = MenuItem::with_id(app, "open", "Open Mash", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        "Settings…",
        true,
        app_shortcut("CmdOrCtrl+,"),
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Mash", true, app_shortcut("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&search, &open, &separator, &settings])?;
    #[cfg(unix)]
    menu.append(&command_line_item(app)?)?;
    menu.append_items(&[&PredefinedMenuItem::separator(app)?, &quit])?;
    TrayIconBuilder::with_id("mash")
        .icon(tray_icon()?)
        // On macOS a template image takes the menu bar's own colour in light
        // and dark. Other systems draw a tray icon as it is.
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Mash")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle(tray.app_handle(), Placement::Under(rect));
            }
        })
        .build(app)?;
    #[cfg(target_os = "macos")]
    app.set_menu(app_menu(app)?)?;
    if let Err(error) = app.global_shortcut().register(SHORTCUT) {
        eprintln!("Mash couldn’t use {SHORTCUT} as its shortcut: {error}");
    }
    Ok(())
}

/// The tray picture. macOS draws the one-colour scoop (icons/tray.svg) in the
/// menu bar's colour. Windows and Linux bars are light or dark with no way to
/// tint an icon, so there it is the app's own full-colour tile, which reads
/// on both.
fn tray_icon() -> tauri::Result<Image<'static>> {
    if cfg!(target_os = "macos") {
        Image::from_bytes(include_bytes!("../icons/tray.png"))
    } else {
        Image::from_bytes(include_bytes!("../icons/64x64.png"))
    }
}

/// The item that links the app's `mash` into `~/.local/bin` (cli.rs).
#[cfg(unix)]
fn command_line_item(app: &AppHandle) -> tauri::Result<MenuItem<tauri::Wry>> {
    MenuItem::with_id(
        app,
        "command-line",
        "Install Command-Line Tool…",
        true,
        None::<&str>,
    )
}

/// The macOS app menu: the standard one, with Settings… and the command-line
/// tool after About.
#[cfg(target_os = "macos")]
fn app_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::default(app)?;
    let first = menu.items()?.into_iter().next();
    if let Some(about) = first.as_ref().and_then(|item| item.as_submenu()) {
        about.insert_items(
            &[
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?,
                &command_line_item(app)?,
            ],
            1,
        )?;
    }
    Ok(menu)
}

/// Act on a choice from the tray menu or the app menu.
pub fn choose(app: &AppHandle, id: &str) {
    match id {
        "search" => toggle(app, Placement::Centre),
        "open" => crate::open_route(app, "#/"),
        "settings" => crate::open_route(app, "#/settings"),
        "command-line" => crate::cli::install(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

/// The panel window, once the dashboard it shows is running. It starts
/// hidden and hides again whenever it loses focus, like a menu.
pub fn create(app: &AppHandle, dashboard: &Url) -> tauri::Result<()> {
    let page = crate::page(dashboard, "#/quick");
    if let Some(panel) = app.get_webview_window(PANEL) {
        return panel.navigate(page);
    }
    let navigation = app.clone();
    let window = WebviewWindowBuilder::new(app, PANEL, WebviewUrl::External(page))
        .title("Search Mash")
        .inner_size(PANEL_SIZE.0, PANEL_SIZE.1)
        .resizable(false)
        .decorations(false)
        // The page draws the panel's rounded edge and the window around it is
        // clear. Linux desktops without a compositor paint a clear window black,
        // so there the window is opaque and its corners square.
        .transparent(cfg!(any(target_os = "macos", windows)))
        .shadow(true)
        .always_on_top(true)
        // On every desktop (Space), like a menu.
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .visible(false)
        .initialization_script(crate::window_script())
        .on_navigation(move |url| crate::navigate(&navigation, url, true))
        .build()?;
    let panel = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = panel.hide();
        }
    });
    Ok(())
}

/// Show the panel where `placement` says, or hide it if it is already showing.
/// Before the engine is ready there is no panel yet, so the main window, with
/// its loading page, comes forward instead.
pub fn toggle(app: &AppHandle, placement: Placement) {
    let Some(panel) = app.get_webview_window(PANEL) else {
        crate::show(app);
        return;
    };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    if let Some(position) = position(&panel, &placement) {
        let _ = panel.set_position(position);
    }
    let _ = panel.show();
    let _ = panel.set_focus();
}

/// Close the panel when the engine behind it stops, so it never shows a page
/// that can no longer load; the shortcut and the icon then bring the main
/// window forward instead, and the panel returns with the engine.
pub fn close(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL) {
        let _ = panel.destroy();
    }
}

/// Hide the panel, after it has opened something in the main window.
pub fn hide(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL) {
        let _ = panel.hide();
    }
}

fn position(panel: &tauri::WebviewWindow, placement: &Placement) -> Option<PhysicalPosition<i32>> {
    let monitor = match placement {
        Placement::Under(rect) => {
            let point = rect.position.to_physical::<f64>(1.0);
            panel.monitor_from_point(point.x, point.y).ok().flatten()
        }
        Placement::Centre => panel.primary_monitor().ok().flatten(),
    }?;
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let screen = Area {
        x: f64::from(area.position.x),
        y: f64::from(area.position.y),
        width: f64::from(area.size.width),
        height: f64::from(area.size.height),
    };
    let panel_size = (PANEL_SIZE.0 * scale, PANEL_SIZE.1 * scale);
    let (x, y) = match placement {
        Placement::Under(rect) => {
            let at = rect.position.to_physical::<f64>(scale);
            let size = rect.size.to_physical::<f64>(scale);
            let icon = Area {
                x: at.x,
                y: at.y,
                width: size.width,
                height: size.height,
            };
            under_icon(&icon, panel_size, &screen, 6.0 * scale)
        }
        Placement::Centre => upper_middle(panel_size, &screen),
    };
    #[allow(clippy::cast_possible_truncation)]
    Some(PhysicalPosition::new(x.round() as i32, y.round() as i32))
}

/// A rectangle in physical pixels.
struct Area {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Centred under the icon, `gap` below it, kept on the screen's work area.
fn under_icon(icon: &Area, (width, _height): (f64, f64), screen: &Area, gap: f64) -> (f64, f64) {
    let centred = icon.x + icon.width / 2.0 - width / 2.0;
    let x = centred.clamp(
        screen.x + gap,
        (screen.x + screen.width - width - gap).max(screen.x + gap),
    );
    let y = (icon.y + icon.height + gap).max(screen.y + gap);
    (x, y)
}

/// Centred across the screen, a fifth of the way down, where Spotlight sits.
fn upper_middle((width, _height): (f64, f64), screen: &Area) -> (f64, f64) {
    (
        screen.x + (screen.width - width) / 2.0,
        screen.y + screen.height / 5.0,
    )
}

#[cfg(test)]
mod tests {
    use super::{Area, under_icon, upper_middle};

    fn area(x: f64, y: f64, width: f64, height: f64) -> Area {
        Area {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn the_panel_drops_down_centred_under_its_icon() {
        let screen = area(0.0, 50.0, 2880.0, 1700.0);
        let icon = area(2000.0, 0.0, 44.0, 48.0);
        assert_eq!(
            under_icon(&icon, (880.0, 840.0), &screen, 12.0),
            (1582.0, 62.0)
        );
    }

    #[test]
    fn the_panel_stays_on_screen_beside_an_icon_near_the_edge() {
        let screen = area(0.0, 50.0, 2880.0, 1700.0);
        let right = area(2840.0, 0.0, 40.0, 48.0);
        assert_eq!(
            under_icon(&right, (880.0, 840.0), &screen, 12.0).0,
            2880.0 - 880.0 - 12.0
        );
        let left = area(10.0, 0.0, 40.0, 48.0);
        assert_eq!(under_icon(&left, (880.0, 840.0), &screen, 12.0).0, 12.0);
        // A second screen to the right keeps its own edges.
        let second = area(2880.0, 50.0, 1920.0, 1030.0);
        let icon = area(4700.0, 0.0, 40.0, 48.0);
        assert_eq!(
            under_icon(&icon, (880.0, 840.0), &second, 12.0).0,
            2880.0 + 1920.0 - 880.0 - 12.0
        );
    }

    #[test]
    fn the_shortcut_shows_it_in_the_upper_middle_of_the_screen() {
        assert_eq!(
            upper_middle((880.0, 840.0), &area(0.0, 50.0, 2880.0, 1700.0)),
            (1000.0, 390.0)
        );
    }
}
