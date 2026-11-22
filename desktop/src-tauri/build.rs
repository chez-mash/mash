//! The app's loading page draws with the dashboard's own theme and mark, so it
//! copies them from the dashboard rather than keeping a second copy to drift.

use std::path::Path;

fn main() {
    let dashboard = Path::new("../../dashboard");
    let ui = Path::new("../ui");
    for (from, to) in [
        ("src/theme/tokens.css", "tokens.css"),
        ("public/mash-mark.svg", "mash-mark.svg"),
    ] {
        let from = dashboard.join(from);
        println!("cargo:rerun-if-changed={}", from.display());
        std::fs::copy(&from, ui.join(to)).unwrap_or_else(|error| {
            panic!("copy {} into the app's page: {error}", from.display())
        });
    }
    tauri_build::build();
}
