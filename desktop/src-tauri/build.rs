//! The app's loading page draws with the dashboard's own theme, mark and
//! typefaces, so it copies them from the dashboard rather than keeping a
//! second copy to drift.

use std::path::Path;

/// Files the page cannot do without.
const REQUIRED: [(&str, &str); 2] = [
    ("src/theme/tokens.css", "tokens.css"),
    ("public/mash-mark.svg", "mash-mark.svg"),
];

/// The typefaces, from the dashboard's installed packages. Without them (a
/// checkout whose dashboard is not installed) the page uses the system font.
const FONTS: [(&str, &str); 3] = [
    ("geist/files/geist-latin-400-normal.woff2", "sans-400.woff2"),
    ("geist/files/geist-latin-600-normal.woff2", "sans-600.woff2"),
    (
        "geist-mono/files/geist-mono-latin-400-normal.woff2",
        "mono-400.woff2",
    ),
];

fn main() {
    let dashboard = Path::new("../../dashboard");
    let ui = Path::new("../ui");
    for (from, to) in REQUIRED {
        let from = dashboard.join(from);
        println!("cargo:rerun-if-changed={}", from.display());
        std::fs::copy(&from, ui.join(to))
            .unwrap_or_else(|error| panic!("copy {} into the app's page: {error}", from.display()));
    }
    let fonts = dashboard.join("node_modules/@fontsource");
    println!("cargo:rerun-if-changed={}", fonts.display());
    for (from, to) in FONTS {
        let from = fonts.join(from);
        if std::fs::copy(&from, ui.join(to)).is_err() {
            println!(
                "cargo:warning=The loading page will use the system font: {} is missing (run pnpm install in dashboard/).",
                from.display()
            );
        }
    }
    // The app's own commands. Its loading page may start the engine again
    // and read its output (capabilities/default.json); the dashboard in the
    // main window may ask about an update and for it to be installed
    // (capabilities/updates.json) and show a right-click menu
    // (capabilities/frame.json); the panel gets none of them.
    let commands = tauri_build::AppManifest::new().commands(&[
        "retry_engine",
        "engine_log",
        "update_banner",
        "update_install",
        "update_later",
        "context_menu",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(commands))
        .unwrap_or_else(|error| panic!("prepare the app's build: {error:#}"));
}
