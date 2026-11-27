//! The main window's frame on each system. The dashboard inside it is the
//! same web page everywhere; only the frame is the system's. On macOS the
//! title bar is hidden and its traffic lights sit in the dashboard's top bar,
//! over the sidebar's colour, as in Finder and Mail. The page learns which
//! system it is on from `window.__MASH_SHELL__` and fits that frame
//! (dashboard/src/lib/shell.ts); everywhere else the system draws its own
//! title bar, dark like the app.

use tauri::window::Color;
use tauri::{Manager, Runtime, WebviewWindowBuilder};

/// The dashboard's page ground (`--surface-page`), so a window never shows
/// white before its page has drawn.
pub const GROUND: Color = Color(0x14, 0x13, 0x11, 0xff);

/// The height of the dashboard's top bar, which the traffic lights are
/// centred in (`#top` in dashboard/src/terminal.css).
#[cfg(target_os = "macos")]
const BAR_HEIGHT: f64 = 52.0;

/// Fits the main window's builder to this system's frame.
pub fn fit<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    let builder = builder.background_color(GROUND);
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        // The buttons are 14 points tall with their shadow: this centres them
        // in the top bar, 20 points in from the edge like Finder's.
        .traffic_light_position(tauri::LogicalPosition::new(20.0, (BAR_HEIGHT - 14.0) / 2.0));
    builder
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
