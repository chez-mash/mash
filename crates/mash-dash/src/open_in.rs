//! Links that open a chat in its agent's own desktop app on this machine.
//!
//! A link is offered only where it was checked against the app itself, and
//! only when nobody but this machine can reach the dashboard, since the link
//! opens an app on the computer that follows it:
//!
//! - Codex: the `ChatGPT` app (where Codex Desktop lives) registers `codex://`
//!   and opens `codex://threads/<thread id>` at that thread. Codex Desktop
//!   reads the same thread store as the Codex CLI, so a CLI thread opens too.
//! - Claude Code: the Claude app registers `claude://` and opens
//!   `claude://resume?session=<CLI session id>` at the app session that owns
//!   that transcript. For a session the app does not own, the same link
//!   imports it as a new app session; that changes the app, so a link is
//!   offered only when one of the app's own session files names the chat.
//!
//! Both apps and their session files are read from their macOS locations;
//! elsewhere nothing is offered.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use mash_model::provider::Provider;

use super::Dash;

pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::reads;
    use aide::axum::routing::get_with;
    router.api_route(
        "/api/open-in",
        get_with(
            open_in,
            reads::<OpenIn>(
                "/api/open-in",
                "The link that opens a chat of this machine in its agent's desktop app, if any.",
            ),
        ),
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct OpenInParams {
    agent: Provider,
    /// The chat's ID, which is the session ID the agent knows it by.
    id: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
struct OpenIn {
    /// Absent when no app on this machine can open the chat.
    link: Option<AppLink>,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
struct AppLink {
    /// The app's name, as a button names it.
    app: &'static str,
    url: String,
}

async fn open_in(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<OpenInParams>,
) -> Response {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let link = match home {
        Some(home) if state.loopback && !state.shared_only && cfg!(target_os = "macos") => {
            let folders = [PathBuf::from("/Applications"), home.join("Applications")];
            tokio::task::spawn_blocking(move || app_link(params.agent, &params.id, &home, &folders))
                .await
                .ok()
                .flatten()
        }
        _ => None,
    };
    axum::Json(OpenIn { link }).into_response()
}

/// The link for one chat, with apps installed in `folders` and the Claude
/// app's session files under `home`.
fn app_link(agent: Provider, id: &str, home: &Path, folders: &[PathBuf]) -> Option<AppLink> {
    // Both apps accept only a UUID here; Mash's own IDs for subagent chats
    // are not one and have no app session.
    if !is_uuid(id) {
        return None;
    }
    match agent {
        Provider::Codex if has_app(folders, &["ChatGPT.app", "Codex.app"], "codex") => {
            Some(AppLink {
                app: "Codex",
                url: format!("codex://threads/{id}"),
            })
        }
        Provider::ClaudeCode
            if has_app(folders, &["Claude.app"], "claude") && claude_app_owns(home, id) =>
        {
            Some(AppLink {
                app: "Claude",
                url: format!("claude://resume?session={id}"),
            })
        }
        _ => None,
    }
}

fn is_uuid(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.iter().map(|group| group.len()).eq([8, 4, 4, 4, 12])
        && groups
            .iter()
            .all(|group| group.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Whether an app bundle of one of these names in one of `folders` registers
/// `scheme` in its (XML) Info.plist.
fn has_app(folders: &[PathBuf], names: &[&str], scheme: &str) -> bool {
    let registered = format!("<string>{scheme}</string>");
    folders
        .iter()
        .flat_map(|folder| names.iter().map(move |name| folder.join(name)))
        .filter_map(|app| std::fs::read_to_string(app.join("Contents/Info.plist")).ok())
        .any(|plist| plist.contains(&registered))
}

/// Whether one of the Claude app's session files has this CLI session as its
/// transcript. The files sit two folders down (account, then organisation).
fn claude_app_owns(home: &Path, id: &str) -> bool {
    let sessions = home.join("Library/Application Support/Claude/claude-code-sessions");
    let children = |folder: &Path| -> Vec<PathBuf> {
        std::fs::read_dir(folder)
            .map(|entries| {
                entries
                    .filter_map(|entry| Some(entry.ok()?.path()))
                    .collect()
            })
            .unwrap_or_default()
    };
    children(&sessions)
        .iter()
        .flat_map(|account| children(account))
        .flat_map(|organisation| children(&organisation))
        .filter(|file| {
            let named = file
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("local_"));
            named
                && file
                    .extension()
                    .is_some_and(|extension| extension == "json")
        })
        .any(|file| names_transcript(&file, id))
}

/// Whether the session file's own `cliSessionId` is `id`. Most files do not
/// mention `id` at all, so only those that do are parsed.
fn names_transcript(file: &Path, id: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(file) else {
        return false;
    };
    text.contains(id)
        && serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|session| session.get("cliSessionId")?.as_str().map(|cli| cli == id))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "586f44fa-8aaa-4587-b69f-b763d3279ef1";

    /// The link with apps only in the test's own Applications folder.
    fn link(agent: Provider, id: &str, home: &Path) -> Option<AppLink> {
        app_link(agent, id, home, &[home.join("Applications")])
    }

    fn app(home: &Path, name: &str, scheme: &str) {
        let contents = home.join("Applications").join(name).join("Contents");
        std::fs::create_dir_all(&contents).unwrap();
        let plist = format!(
            "<plist><dict><key>CFBundleURLSchemes</key><array><string>{scheme}</string></array></dict></plist>"
        );
        std::fs::write(contents.join("Info.plist"), plist).unwrap();
    }

    fn claude_session(home: &Path, file: &str, cli: &str) {
        let folder = home.join("Library/Application Support/Claude/claude-code-sessions/acct/org");
        std::fs::create_dir_all(&folder).unwrap();
        let body = serde_json::json!({"sessionId": "local_x", "cliSessionId": cli, "title": "t"});
        std::fs::write(folder.join(file), body.to_string()).unwrap();
    }

    #[test]
    fn codex_threads_open_in_the_app_that_registers_codex() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(link(Provider::Codex, ID, home.path()), None);
        app(home.path(), "ChatGPT.app", "codex");
        assert_eq!(
            link(Provider::Codex, ID, home.path()),
            Some(AppLink {
                app: "Codex",
                url: format!("codex://threads/{ID}")
            })
        );
        // Mash's IDs for subagent chats are not thread IDs.
        assert_eq!(
            link(Provider::Codex, "50cf5623ea898883ead39971", home.path()),
            None
        );
    }

    #[test]
    fn an_app_that_does_not_register_the_scheme_is_not_used() {
        let home = tempfile::tempdir().unwrap();
        app(home.path(), "ChatGPT.app", "chatgpt");
        assert_eq!(link(Provider::Codex, ID, home.path()), None);
    }

    #[test]
    fn claude_sessions_open_only_when_the_app_owns_the_transcript() {
        let home = tempfile::tempdir().unwrap();
        app(home.path(), "Claude.app", "claude");
        // A CLI session the app has never seen would be imported: no link.
        assert_eq!(link(Provider::ClaudeCode, ID, home.path()), None);
        // A file that mentions the ID elsewhere does not make the app its owner.
        let other = "11111111-2222-3333-4444-555555555555";
        claude_session(home.path(), "local_a.json", other);
        let folder = home
            .path()
            .join("Library/Application Support/Claude/claude-code-sessions/acct/org");
        let mention = serde_json::json!({"cliSessionId": other, "note": ID});
        std::fs::write(folder.join("local_b.json"), mention.to_string()).unwrap();
        assert_eq!(link(Provider::ClaudeCode, ID, home.path()), None);
        claude_session(home.path(), "local_c.json", ID);
        assert_eq!(
            link(Provider::ClaudeCode, ID, home.path()),
            Some(AppLink {
                app: "Claude",
                url: format!("claude://resume?session={ID}")
            })
        );
    }

    #[test]
    fn other_agents_have_no_link() {
        let home = tempfile::tempdir().unwrap();
        app(home.path(), "Cursor.app", "cursor");
        assert_eq!(link(Provider::Cursor, ID, home.path()), None);
    }
}
