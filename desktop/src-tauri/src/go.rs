//! The Go menu of the Mac app: moving between the dashboard's pages with
//! the keys a Mac app uses, ⌘[ and ⌘] for back and forward, ⌘F for search
//! and ⌘1 to ⌘5 for the main pages, as Finder's Go menu does. Each item acts
//! on the main window's page: a page change goes through `open_route`, back,
//! forward and find run in the page itself. On Windows and Linux the system's
//! own keys (Alt+Left, Ctrl+F) reach the page without a menu.

use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Wry;
#[cfg(target_os = "macos")]
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};

/// What an item does to the dashboard.
enum Act {
    /// Runs in the page: history and the search field's event
    /// (`FIND_EVENT` in dashboard/src/lib/findField.ts).
    Page(&'static str),
    /// Opens one of the dashboard's pages.
    Route(&'static str),
}

/// The items, in menu order: id, title, keys and what it does. An empty id is
/// a separator.
const ITEMS: &[(&str, &str, &str, Act)] = &[
    (
        "go-back",
        "Back",
        "CmdOrCtrl+[",
        Act::Page("history.back()"),
    ),
    (
        "go-forward",
        "Forward",
        "CmdOrCtrl+]",
        Act::Page("history.forward()"),
    ),
    ("", "", "", Act::Page("")),
    (
        "go-find",
        "Search Chats",
        "CmdOrCtrl+F",
        Act::Page("dispatchEvent(new Event(\"mash:find\"))"),
    ),
    ("", "", "", Act::Page("")),
    ("go-overview", "Overview", "CmdOrCtrl+1", Act::Route("#/")),
    ("go-chats", "Chats", "CmdOrCtrl+2", Act::Route("#/search")),
    (
        "go-memory",
        "Memory",
        "CmdOrCtrl+3",
        Act::Route("#/records?"),
    ),
    (
        "go-machines",
        "Machines",
        "CmdOrCtrl+4",
        Act::Route("#/machines"),
    ),
    ("go-agents", "Agents", "CmdOrCtrl+5", Act::Route("#/agents")),
];

/// Adds the Go menu to the app's menus, after View, as macOS apps place it.
#[cfg(target_os = "macos")]
pub fn with_go_menu(app: &AppHandle, menu: Menu<Wry>) -> tauri::Result<Menu<Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for (id, title, keys, _) in ITEMS {
        if id.is_empty() {
            items.push(Box::new(PredefinedMenuItem::separator(app)?));
        } else {
            items.push(Box::new(MenuItem::with_id(
                app,
                *id,
                *title,
                true,
                Some(*keys),
            )?));
        }
    }
    let items: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    let go = Submenu::with_items(app, "Go", true, &items)?;
    // After View: App, File, Edit, View, then Go, Window and Help.
    let at = menu.items()?.len().min(4);
    menu.insert(&go, at)?;
    Ok(menu)
}

/// Acts on a Go menu choice; false when `id` is not one of its items.
pub fn choose(app: &AppHandle, id: &str) -> bool {
    let Some((_, _, _, act)) = ITEMS
        .iter()
        .find(|(item, ..)| !item.is_empty() && *item == id)
    else {
        return false;
    };
    match act {
        Act::Route(route) => crate::open_route(app, route),
        Act::Page(script) => {
            if let Some(window) = crate::show(app) {
                let _ = window.eval(*script);
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_its_own_id_and_keys() {
        let named: Vec<_> = ITEMS.iter().filter(|(id, ..)| !id.is_empty()).collect();
        for (index, (id, title, keys, _)) in named.iter().enumerate() {
            assert!(id.starts_with("go-"), "{id}");
            assert!(!title.is_empty() && keys.starts_with("CmdOrCtrl+"), "{id}");
            for (other, _, other_keys, _) in &named[index + 1..] {
                assert_ne!(id, other);
                assert_ne!(keys, other_keys, "{id} and {other} share keys");
            }
        }
    }
}
