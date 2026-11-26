//! The main window's size: the preferred size when the screen has room,
//! shrunk to fit a smaller one, and never below the smallest size the
//! dashboard lays out in.

use tauri::AppHandle;

/// The window's size when the screen has room, and its smallest size, in
/// logical pixels.
const PREFERRED: (f64, f64) = (1280.0, 820.0);
pub const MIN: (f64, f64) = (420.0, 480.0);

/// The window's opening size: the preferred size, shrunk to fit the main
/// screen's work area (the screen less its taskbar or dock).
pub fn opening(app: &AppHandle) -> (f64, f64) {
    let Ok(Some(monitor)) = app.primary_monitor() else {
        return PREFERRED;
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
        room(PREFERRED.0, width, MIN.0),
        room(PREFERRED.1, height - TITLE_BAR, MIN.1),
    )
}

#[cfg(test)]
mod tests {
    use super::fit;

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
}
