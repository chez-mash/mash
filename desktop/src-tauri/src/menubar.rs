//! Mash in the menu bar: an icon whose click drops the quick search panel
//! down under it, a native menu on right-click, and a global shortcut that
//! drops the same panel from the same place, like a menu bar extra.
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
    AppHandle, Manager, Monitor, PhysicalPosition, Rect, Url, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The panel window's label.
pub const PANEL: &str = "quick";
/// The panel's size in logical pixels: room for the search field, five
/// chats and a footer, about the size of a menu bar extra.
const PANEL_SIZE: (f64, f64) = (380.0, 305.0);
/// The space between the panel and the menu bar or the screen's edge, in
/// logical pixels.
const GAP: f64 = 6.0;
/// The shortcut that shows the panel from anywhere. ⌥Space is left to the
/// launchers (Spotlight's neighbours Raycast and Alfred use it by default).
pub const SHORTCUT: &str = "Alt+Shift+Space";

/// Where the panel appears. It always drops down from the menu bar icon.
pub enum Placement {
    /// Under the icon at this rectangle, which was just clicked.
    Under(Rect),
    /// Under the icon wherever it is now, for the shortcut and the menus.
    /// When the icon is on no screen (a menu bar manager such as Ice or
    /// Bartender has hidden it, or the system does not say where it is), the
    /// panel drops down from the top right of the screen with the pointer,
    /// where menu bar icons live.
    MenuBar,
}

/// The global shortcut plugin, whose shortcut is registered once the app is
/// running: registering it in the plugin would stop the app from starting
/// when another app already holds the shortcut.
pub fn shortcut_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                toggle(app, Placement::MenuBar);
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
    menu.append(&command_line_item(app)?)?;
    crate::update::add_menu(app, crate::update::Holder::Menu(menu.clone()), None)?;
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

/// The item that puts the app's `mash` command in place, or removes it
/// (cli.rs).
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
        crate::update::add_menu(app, crate::update::Holder::Submenu(about.clone()), Some(4))?;
    }
    Ok(menu)
}

/// Act on a choice from the tray menu or the app menu.
pub fn choose(app: &AppHandle, id: &str) {
    match id {
        "search" => toggle(app, Placement::MenuBar),
        "open" => crate::open_route(app, "#/"),
        "settings" => crate::open_route(app, "#/settings"),
        "command-line" => crate::cli::install(app),
        "quit" => app.exit(0),
        id => {
            crate::update::choose(app, id);
        }
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
    let builder = WebviewWindowBuilder::new(app, PANEL, WebviewUrl::External(page))
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
        // Mash is dark only; on macOS the main window's theme already sets it.
        .theme(Some(tauri::Theme::Dark))
        .initialization_script(crate::window_script())
        .initialization_script(PANEL_SCRIPT)
        .on_navigation(move |url| crate::navigate(&navigation, url, true));
    // On macOS the panel is drawn on the system's popover material, which
    // blurs what is behind it like a menu bar extra, dark as the app is; the
    // page leaves its background translucent to let it through.
    #[cfg(target_os = "macos")]
    let builder = builder.effects(tauri::utils::config::WindowEffectsConfig {
        effects: vec![tauri::window::Effect::Popover],
        state: Some(tauri::window::EffectState::Active),
        radius: Some(PANEL_RADIUS),
        ..Default::default()
    });
    let window = builder.build()?;
    #[cfg(target_os = "macos")]
    over_full_screen(&window);
    let panel = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = panel.hide();
        }
    });
    Ok(())
}

/// Let the panel show over an app in full screen. macOS gives a full-screen
/// app a Space of its own, which shows only windows marked as auxiliary to
/// full screen; on every other desktop is not enough, and without the mark
/// the panel opened on a desktop the reader could not see. This is what
/// macOS's own menu bar extras do.
#[cfg(target_os = "macos")]
fn over_full_screen(window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        let Ok(pointer) = target.ns_window() else {
            return;
        };
        // SAFETY: Tauri hands out the window's live NSWindow, and this runs
        // on the main thread, where AppKit may be used.
        let ns_window = unsafe { &*pointer.cast::<NSWindow>() };
        ns_window.setCollectionBehavior(
            ns_window.collectionBehavior()
                | NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
    });
}

/// The corner radius of the panel's material, matching the page's
/// `--radius-7` (QuickSearch.svelte).
#[cfg(target_os = "macos")]
const PANEL_RADIUS: f64 = 12.0;

/// Tells the panel's page that the window behind it is the system material
/// (QuickSearch.svelte makes its background translucent only then).
const PANEL_SCRIPT: &str = if cfg!(target_os = "macos") {
    "window.__MASH_PANEL_MATERIAL__ = true;"
} else {
    ""
};

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
    if let Some(position) = position(app, &panel, &placement) {
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

/// Where the panel goes: centred under the menu bar icon, on the screen that
/// shows the icon, kept inside that screen's work area (the screen less its
/// menu bar and dock or taskbar).
///
/// Every rectangle here is in physical pixels in the desktop's one coordinate
/// space, as both the tray and the monitors report them, so a menu bar on a
/// second screen with its own scale finds its own screen and its own scale.
fn position(
    app: &AppHandle,
    panel: &tauri::WebviewWindow,
    placement: &Placement,
) -> Option<PhysicalPosition<i32>> {
    let icon = match placement {
        Placement::Under(rect) => Some(*rect),
        Placement::MenuBar => app
            .tray_by_id("mash")
            .and_then(|tray| tray.rect().ok().flatten()),
    };
    let monitors = panel.available_monitors().ok()?;
    let primary_scale = panel
        .primary_monitor()
        .ok()
        .flatten()
        .map_or(1.0, |monitor| monitor.scale_factor());
    // The tray reports physical pixels; a logical rectangle is read at the
    // main screen's scale, the best guess available.
    let icon = icon.map(|rect| {
        let at = rect.position.to_physical::<f64>(primary_scale);
        let size = rect.size.to_physical::<f64>(primary_scale);
        Area {
            x: at.x,
            y: at.y,
            width: size.width,
            height: size.height,
        }
    });
    let showing = icon.as_ref().and_then(|icon| {
        let (x, y) = (icon.x + icon.width / 2.0, icon.y + icon.height / 2.0);
        monitors
            .iter()
            .find(|monitor| screen(monitor).contains(x, y))
    });
    let (monitor, icon) = match showing {
        Some(monitor) => (monitor.clone(), icon),
        None => {
            let cursor = app.cursor_position().ok();
            let pointed = cursor.and_then(|at| panel.monitor_from_point(at.x, at.y).ok().flatten());
            (
                pointed.or_else(|| panel.primary_monitor().ok().flatten())?,
                None,
            )
        }
    };
    let scale = monitor.scale_factor();
    let area = work_area(&monitor);
    let size = (PANEL_SIZE.0 * scale, PANEL_SIZE.1 * scale);
    let (x, y) = match icon {
        Some(icon) => under_icon(&icon, size, &area, GAP * scale),
        None => top_right(size, &area, GAP * scale),
    };
    #[allow(clippy::cast_possible_truncation)]
    Some(PhysicalPosition::new(x.round() as i32, y.round() as i32))
}

fn screen(monitor: &Monitor) -> Area {
    Area {
        x: f64::from(monitor.position().x),
        y: f64::from(monitor.position().y),
        width: f64::from(monitor.size().width),
        height: f64::from(monitor.size().height),
    }
}

fn work_area(monitor: &Monitor) -> Area {
    let area = monitor.work_area();
    Area {
        x: f64::from(area.position.x),
        y: f64::from(area.position.y),
        width: f64::from(area.size.width),
        height: f64::from(area.size.height),
    }
}

/// A rectangle in physical pixels.
struct Area {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Area {
    fn contains(&self, x: f64, y: f64) -> bool {
        (self.x..self.x + self.width).contains(&x) && (self.y..self.y + self.height).contains(&y)
    }
}

/// Centred under the icon, `gap` below it, kept inside the screen's work
/// area. A taskbar at the bottom of the screen (Windows, some Linux desktops)
/// puts its icons below the work area, so there the panel rises above the
/// icon instead.
fn under_icon(icon: &Area, (width, height): (f64, f64), screen: &Area, gap: f64) -> (f64, f64) {
    let centred = icon.x + icon.width / 2.0 - width / 2.0;
    let x = clamp(
        centred,
        screen.x + gap,
        screen.x + screen.width - width - gap,
    );
    let below = icon.y + icon.height + gap;
    let y = if below + height > screen.y + screen.height {
        icon.y - height - gap
    } else {
        below
    };
    let y = clamp(y, screen.y + gap, screen.y + screen.height - height - gap);
    (x, y)
}

/// The top right of the screen's work area, where menu bar icons live, for
/// when the icon itself cannot be found.
fn top_right((width, _height): (f64, f64), screen: &Area, gap: f64) -> (f64, f64) {
    (
        (screen.x + screen.width - width - gap).max(screen.x + gap),
        screen.y + gap,
    )
}

/// `value` kept between `low` and `high`, preferring `low` when they cross
/// (a panel larger than the screen keeps its top left corner on it).
fn clamp(value: f64, low: f64, high: f64) -> f64 {
    value.min(high).max(low)
}

#[cfg(test)]
mod tests {
    use super::{Area, top_right, under_icon};

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
    fn a_bottom_taskbar_puts_the_panel_above_its_icon() {
        let screen = area(0.0, 0.0, 1920.0, 1040.0);
        let icon = area(1800.0, 1050.0, 32.0, 30.0);
        assert_eq!(
            under_icon(&icon, (760.0, 660.0), &screen, 12.0),
            (1920.0 - 760.0 - 12.0, 1040.0 - 660.0 - 12.0)
        );
    }

    #[test]
    fn without_its_icon_the_panel_drops_from_the_top_right() {
        // Mash's icon hidden by a menu bar manager: a 1168x755 point Retina
        // screen whose work area starts under a 24-point menu bar.
        assert_eq!(
            top_right((760.0, 660.0), &area(0.0, 48.0, 2336.0, 1364.0), 12.0),
            (2336.0 - 760.0 - 12.0, 60.0)
        );
    }
}
