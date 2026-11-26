// No console window behind the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The installer starts the app to put the `mash` command in place or take
    // it away; that is all such a start does, before anything else of the
    // app (a second instance would otherwise hand its arguments over).
    #[cfg(windows)]
    if let Some(status) = mash_desktop::installer_request(std::env::args().skip(1)) {
        std::process::exit(status);
    }
    mash_desktop::run();
}
