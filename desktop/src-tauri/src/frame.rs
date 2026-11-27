//! The main window's frame on each system. The dashboard inside it is the
//! same web page everywhere; only the frame is the system's. On macOS the
//! title bar is hidden and its traffic lights sit in the dashboard's top bar,
//! over the sidebar's colour, as in Finder and Mail. The page learns which
//! system it is on from `window.__MASH_SHELL__` and fits that frame
//! (dashboard/src/lib/shell.ts); everywhere else the system draws its own
//! title bar, dark like the app. The window opens where the reader last left
//! it, as a native app's does.

use tauri::plugin::TauriPlugin;
use tauri::window::Color;
use tauri::{Manager, Runtime, WebviewWindowBuilder, Wry};
use tauri_plugin_window_state::StateFlags;

/// The dashboard's page ground (`--surface-page`), so a window never shows
/// white before its page has drawn.
pub const GROUND: Color = Color(0x14, 0x13, 0x11, 0xff);

/// Where the traffic lights sit: 20 points in from the edge like Finder's, and
/// centred in the dashboard's 52-point top bar (`#top` in
/// dashboard/src/terminal.css). Tauri's y does not name the buttons' top or
/// centre, so this is measured: in the Mac VM (tools/mac-vm.sh desktop) the
/// buttons' centre lands 2 points above y, and 28 puts it at 26, the bar's
/// middle.
#[cfg(target_os = "macos")]
const LIGHTS: (f64, f64) = (20.0, 28.0);

/// Fits the main window's builder to this system's frame.
pub fn fit<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    // Started at login, the app opens into the menu bar alone (login.rs).
    let builder = builder
        .background_color(GROUND)
        .visible(!crate::login::at_login());
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(LIGHTS.0, LIGHTS.1));
    builder
}

/// Keeps the main window's size, place and whether it was maximized or full
/// screen, and restores them when the app starts; the plugin falls back to
/// the opening size (size.rs) on a first start or when the screen it was on
/// is gone. Whether it was showing is not kept: the app decides that, since
/// a window closed into the menu bar must still open on the next start. The
/// search panel takes its place under the menu bar icon each time, and print
/// windows come and go, so neither is kept.
pub fn remembered() -> TauriPlugin<Wry> {
    tauri_plugin_window_state::Builder::default()
        .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
        .with_denylist(&[crate::menubar::PANEL])
        .map_label(|label| {
            if label.starts_with("print-") {
                "print"
            } else {
                label
            }
        })
        .build()
}

/// Tells the page which system's window it is in.
pub fn shell_script() -> String {
    let shell = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    };
    format!("window.__MASH_SHELL__ = \"{shell}\";")
}
