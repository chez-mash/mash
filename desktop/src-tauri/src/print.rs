//! The window a chat is printed from, for the dashboard's PDF export.
//!
//! In a browser the dashboard opens an empty tab, writes the chat into it and
//! calls the tab's `print()`, or its `close()` if the export fails. A webview
//! does neither for a page (WKWebView ignores both), so the app gives that tab
//! a real window of its own, and the dashboard, inside the app, sends the tab
//! to `mash://print` or `mash://close` instead. The app prints the window
//! natively, where Save as PDF is one of the choices, and the window stays
//! open with the chat until the reader closes it.

use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::webview::{NewWindowFeatures, NewWindowResponse};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

/// The window's size when the page does not ask for one, in logical pixels.
const SIZE: (f64, f64) = (820.0, 900.0);

/// What the dashboard asks of the window it writes a chat into.
#[derive(Debug, PartialEq, Eq)]
enum Request {
    Print,
    Close,
}

fn request(url: &Url) -> Option<Request> {
    if url.scheme() != "mash" {
        return None;
    }
    match url.as_str().trim_end_matches('/') {
        "mash://print" => Some(Request::Print),
        "mash://close" => Some(Request::Close),
        _ => None,
    }
}

/// Open the empty window the dashboard writes a chat into, as the new window
/// it asked for.
pub fn open(
    app: &AppHandle,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<tauri::Wry> {
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    let label = format!("print-{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let navigation = app.clone();
    let outside = app.clone();
    let printing = label.clone();
    let built = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .inner_size(SIZE.0, SIZE.1)
        .window_features(features)
        .title("Print")
        .on_document_title_changed(|window, title| {
            let _ = window.set_title(&title);
        })
        .on_navigation(move |url| {
            let Some(request) = request(url) else {
                return crate::navigate(&navigation, url, false);
            };
            if let Some(window) = navigation.get_webview_window(&printing) {
                let _ = match request {
                    Request::Print => window.print(),
                    Request::Close => window.close(),
                };
            }
            false
        })
        .on_new_window(move |url, _features| {
            crate::open_outside(&outside, &url);
            NewWindowResponse::Deny
        })
        .build();
    match built {
        Ok(window) => NewWindowResponse::Create { window },
        Err(error) => {
            eprintln!("Mash couldn’t open a window to print from: {error}");
            NewWindowResponse::Deny
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Request, request};
    use tauri::Url;

    #[test]
    fn the_dashboard_asks_the_window_to_print_or_close() {
        let ask = |link: &str| request(&Url::parse(link).unwrap());
        assert_eq!(ask("mash://print"), Some(Request::Print));
        assert_eq!(ask("mash://print/"), Some(Request::Print));
        assert_eq!(ask("mash://close"), Some(Request::Close));
        for link in ["mash://printer", "mash://chat/print", "https://print/"] {
            assert_eq!(ask(link), None, "{link}");
        }
    }
}
