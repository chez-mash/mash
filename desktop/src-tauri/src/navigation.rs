//! Which addresses a window may load. The app's windows show its own page
//! and the dashboard's origin and nothing else: a mash:// link is followed in
//! the main window, a web, mail or agent-app address goes to the program the
//! reader uses for it, and every other scheme is dropped.

use tauri::{AppHandle, Url};
use tauri_plugin_opener::OpenerExt;

/// What a window does with an address it is asked to load.
#[derive(Debug, PartialEq, Eq)]
pub enum Navigation {
    /// Load it: the app's own page, or a page of the dashboard.
    Load,
    /// A mash:// link, which the app follows in its main window.
    Follow,
    /// Anything else, which belongs in the reader's browser.
    Outside,
}

pub fn navigation(url: &Url, dashboard: Option<&Url>) -> Navigation {
    let local_page = url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost");
    let same_origin = dashboard.is_some_and(|dashboard| dashboard.origin() == url.origin());
    let in_page = matches!(url.scheme(), "about" | "blob");
    if local_page || same_origin || in_page {
        Navigation::Load
    } else if url.scheme() == "mash" {
        Navigation::Follow
    } else {
        Navigation::Outside
    }
}

/// Whether a window (the panel, when `from_panel`) may load `url` itself.
/// A mash:// link is how the dashboard asks for a page of the main window, so
/// the panel opens its rows and menu that way and then gets out of the way.
pub fn navigate(app: &AppHandle, url: &Url, from_panel: bool) -> bool {
    let dashboard = crate::dashboard(app);
    match navigation(url, dashboard.as_ref()) {
        Navigation::Load => true,
        Navigation::Follow => {
            if from_panel {
                crate::menubar::hide(app);
            }
            crate::follow(app, url.as_str());
            false
        }
        Navigation::Outside => {
            open_outside(app, url);
            false
        }
    }
}

/// Open a web or mail address in the program the reader uses for it, and a
/// chat's link into its agent's app (the dashboard's Open in Codex, Claude or
/// Copilot). The opener hands the whole link to the system as one value (on
/// Windows through `Start-Process`, never `cmd /c start`), so a link's `&` and
/// percent-encoded query arrive intact.
pub fn open_outside(app: &AppHandle, url: &Url) {
    if opens_outside(url) {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
    }
}

fn opens_outside(url: &Url) -> bool {
    matches!(
        url.scheme(),
        "http" | "https" | "mailto" | "codex" | "claude" | "ghapp"
    )
}

#[cfg(test)]
mod tests {
    use super::{Navigation, navigation, opens_outside};
    use tauri::Url;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn only_the_dashboard_and_the_app_page_load_in_the_window() {
        let dashboard = url("http://127.0.0.1:47290/");
        let stays_in_app =
            |url: &Url, dashboard: Option<&Url>| navigation(url, dashboard) == Navigation::Load;
        for inside in [
            "http://127.0.0.1:47290/#/chat/abc",
            "http://127.0.0.1:47290/api/export",
            "tauri://localhost/index.html",
            "http://tauri.localhost/index.html",
            "about:blank",
            "blob:http://127.0.0.1:47290/5f1c",
        ] {
            assert!(stays_in_app(&url(inside), Some(&dashboard)), "{inside}");
        }
        for outside in [
            "https://getmash.io/agents/",
            "http://127.0.0.1:8790/",
            "http://localhost:47290/",
            "mailto:someone@example.com",
        ] {
            assert!(!stays_in_app(&url(outside), Some(&dashboard)), "{outside}");
        }
        // Web, mail and agent-app links go to the system; other schemes do not.
        for system in [
            "https://getmash.io/",
            "mailto:someone@example.com",
            "codex://threads/01a1b649-827e-7692-a5b1-0de415a69df5",
            "claude://resume?session=586f44fa-8aaa-4587-b69f-b763d3279ef1&cwd=%2Fwork%2Fa%20b",
            "ghapp://sessions/1e792353-1ec1-4f5a-a285-c73b23bdd829",
        ] {
            assert!(opens_outside(&url(system)), "{system}");
        }
        for refused in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "vscode://file/x",
        ] {
            assert!(!opens_outside(&url(refused)), "{refused}");
        }
        assert!(
            !stays_in_app(&dashboard, None),
            "before the engine is ready"
        );
        // A mash:// link from the dashboard opens a page of the main window.
        for link in [
            "mash://chat/abc?message=m",
            "mash://settings",
            "mash://quick",
        ] {
            assert_eq!(
                navigation(&url(link), Some(&dashboard)),
                Navigation::Follow,
                "{link}"
            );
        }
        assert_eq!(
            navigation(&url("https://getmash.io/"), Some(&dashboard)),
            Navigation::Outside
        );
    }
}
