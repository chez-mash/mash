//! Right-click menus for the dashboard in the main window. The page decides
//! what a menu holds, the same items it would draw itself in a browser
//! (dashboard/src/lib/contextMenu.svelte.ts); the app shows them as the
//! system's own menu at the pointer and tells the page which one was chosen
//! by its place in the list. Only titles and the place cross: the page keeps
//! what each item does.

use serde::Deserialize;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};

/// One line of a menu: a title, or a separator when it has none.
#[derive(Deserialize)]
pub struct Line {
    title: Option<String>,
    #[serde(default = "enabled")]
    enabled: bool,
}

fn enabled() -> bool {
    true
}

/// A menu item's id: this prefix and the item's place in the page's list.
const PREFIX: &str = "context:";
/// A menu is short; anything past this is the page's mistake and left out.
const MOST: usize = 24;

/// Shows `lines` as a menu at the pointer in the window that asked.
#[tauri::command]
pub fn context_menu(window: WebviewWindow, lines: Vec<Line>) -> Result<(), String> {
    let app = window.app_handle();
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for (place, line) in lines.iter().take(MOST).enumerate() {
        let item: Box<dyn IsMenuItem<Wry>> = match &line.title {
            Some(title) => {
                let title: String = title.chars().take(80).collect();
                Box::new(
                    MenuItem::with_id(
                        app,
                        format!("{PREFIX}{place}"),
                        title,
                        line.enabled,
                        None::<&str>,
                    )
                    .map_err(|error| error.to_string())?,
                )
            }
            None => {
                Box::new(PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?)
            }
        };
        items.push(item);
    }
    let items: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    let menu = Menu::with_items(app, &items).map_err(|error| error.to_string())?;
    window.popup_menu(&menu).map_err(|error| error.to_string())
}

/// Tells the page which item of its menu was chosen; false when `id` is not
/// a context menu's item.
pub fn choose(app: &AppHandle, id: &str) -> bool {
    let Some(place) = id
        .strip_prefix(PREFIX)
        .and_then(|place| place.parse::<usize>().ok())
    else {
        return false;
    };
    if let Some(window) = app.get_webview_window(crate::WINDOW) {
        let _ = window.eval(format!(
            "dispatchEvent(new CustomEvent(\"mash:context\", {{ detail: {place} }}))"
        ));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_without_a_title_is_a_separator_and_lines_are_enabled_unless_said() {
        let lines: Vec<Line> =
            serde_json::from_str(r#"[{"title":"Open"},{},{"title":"Copy","enabled":false}]"#)
                .unwrap();
        assert_eq!(lines[0].title.as_deref(), Some("Open"));
        assert!(lines[0].enabled);
        assert!(lines[1].title.is_none());
        assert!(!lines[2].enabled);
    }
}
