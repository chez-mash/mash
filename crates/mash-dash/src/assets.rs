include!(concat!(env!("OUT_DIR"), "/dash_assets.rs"));

/// Whether this build serves a browser dashboard.
#[must_use]
pub fn has_web_dashboard() -> bool {
    ASSETS.iter().any(|(route, _)| *route == "/index.html")
}
