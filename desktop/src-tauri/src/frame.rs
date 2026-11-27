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

/// Makes the system's menus dark on Windows, as the rest of the app is: the
/// right-click menus (context.rs) and the tray menu. Windows draws an app's
/// popup menus light unless the app asks for the dark ones, and the way to
/// ask is uxtheme's SetPreferredAppMode(ForceDark) followed by
/// FlushMenuThemes. They are exported by ordinal only (135 and 136) and not
/// documented, but have held since Windows 10 1903 and are what Electron's
/// `nativeTheme.themeSource = "dark"` calls. Where either is missing the menus
/// simply stay light. Seen light on GitHub's Windows runner before this
/// (.github/workflows/windows-shots.yml).
#[cfg(windows)]
pub fn dark_menus() {
    use windows_sys::Win32::System::LibraryLoader::{
        GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
    };
    type Proc = unsafe extern "system" fn() -> isize;
    const FORCE_DARK: i32 = 2;
    let name: Vec<u16> = "uxtheme.dll\0".encode_utf16().collect();
    // SAFETY: uxtheme.dll is loaded from System32 by a NUL-terminated name,
    // and each entry point is called only if found, with the signature it
    // has had since Windows 10 1903: `fn(i32) -> i32` and `fn()`.
    unsafe {
        let module = LoadLibraryExW(
            name.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_SYSTEM32,
        );
        if module.is_null() {
            return;
        }
        if let Some(set) = GetProcAddress(module, 135 as *const u8) {
            let set = std::mem::transmute::<Proc, unsafe extern "system" fn(i32) -> i32>(set);
            set(FORCE_DARK);
        }
        if let Some(flush) = GetProcAddress(module, 136 as *const u8) {
            let flush = std::mem::transmute::<Proc, unsafe extern "system" fn()>(flush);
            flush();
        }
    }
}

/// Elsewhere the system's menus follow the app's dark window theme already.
#[cfg(not(windows))]
pub fn dark_menus() {}

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
