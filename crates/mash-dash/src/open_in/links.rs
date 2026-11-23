//! The link for one chat, and the evidence that its app opens it. Each route
//! was read from the app's own link handler and tried against the app:
//!
//! - Codex: the `ChatGPT` app (where Codex Desktop lives) registers `codex://`
//!   and opens `codex://threads/<thread id>` at that thread. Codex Desktop
//!   reads the same thread store as the Codex CLI, so a CLI thread opens too.
//! - Claude Code: the Claude app registers `claude://` and opens
//!   `claude://resume?session=<CLI session id>` at the app session that owns
//!   that transcript. For a session it does not own, the same link imports
//!   the CLI session as a new app session, so the link says whether it adds
//!   the session, from the app's own session files. The chat's folder rides
//!   along as `cwd`.
//! - Copilot: the GitHub Copilot app registers `ghapp://` and opens
//!   `ghapp://sessions/<session id>` at the workspace that holds that Copilot
//!   session. For any other session the app asks to set it up, and adds its
//!   project, so a link is made only for a session in one of the app's own
//!   workspaces. Mash's ID for a Copilot chat is a hash of the session ID, so
//!   the session is found by hashing the app's ones the same way. VS Code
//!   Copilot Chat sessions hash differently and never match.
//!
//! The Cursor, Antigravity, `OpenCode` and VS Code apps were examined too; none
//! has a built-in link to one particular chat, so they get none.
use std::path::{Path, PathBuf};

use mash_model::provider::Provider;

use super::AppLink;

/// A desktop app that opens one agent's sessions.
pub(super) struct App {
    pub(super) name: &'static str,
    pub(super) scheme: &'static str,
    /// Its macOS bundle names.
    #[cfg(any(target_os = "macos", test))]
    pub(super) bundles: &'static [&'static str],
}

pub(super) const CODEX: App = App {
    name: "Codex",
    scheme: "codex",
    #[cfg(any(target_os = "macos", test))]
    bundles: &["ChatGPT.app", "Codex.app"],
};
pub(super) const CLAUDE: App = App {
    name: "Claude",
    scheme: "claude",
    #[cfg(any(target_os = "macos", test))]
    bundles: &["Claude.app"],
};
pub(super) const COPILOT: App = App {
    name: "Copilot",
    scheme: "ghapp",
    #[cfg(any(target_os = "macos", test))]
    bundles: &["GitHub Copilot.app"],
};

/// Where this computer keeps what the links depend on.
pub(super) struct Machine {
    /// Whether an installed app registers the app's scheme.
    pub(super) registered: Box<dyn Fn(&App) -> bool + Send>,
    /// Folders the Claude app may keep its session files in; each holds them
    /// two folders down (account, then organisation).
    pub(super) claude_sessions: Vec<PathBuf>,
    /// The Copilot app's database, which lists its workspaces.
    pub(super) copilot_db: PathBuf,
}

impl Machine {
    #[cfg(target_os = "macos")]
    pub(super) fn this() -> Option<Self> {
        let home = PathBuf::from(std::env::var_os("HOME")?);
        let folders = [PathBuf::from("/Applications"), home.join("Applications")];
        Some(Self {
            registered: Box::new(move |app| bundle_registers(&folders, app)),
            claude_sessions: vec![
                home.join("Library/Application Support/Claude/claude-code-sessions"),
            ],
            copilot_db: home.join(".copilot/data.db"),
        })
    }

    /// The Claude app keeps its data in `%APPDATA%\Claude`, or, installed as
    /// a package, in that package's own copy of the roaming folder; the app
    /// looks in the same places for its data.
    #[cfg(windows)]
    pub(super) fn this() -> Option<Self> {
        const PACKAGES: [&str; 2] = ["Claude_pzs8sxrjxfjjc", "AnthropicPBC.Claude_fnn82j28hfe8t"];
        let profile = PathBuf::from(std::env::var_os("USERPROFILE")?);
        let sessions = PathBuf::from("Claude").join("claude-code-sessions");
        let mut claude_sessions = Vec::new();
        if let Some(roaming) = std::env::var_os("APPDATA") {
            claude_sessions.push(PathBuf::from(roaming).join(&sessions));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let packages = PathBuf::from(local).join("Packages");
            claude_sessions.extend(PACKAGES.iter().map(|package| {
                packages
                    .join(package)
                    .join("LocalCache")
                    .join("Roaming")
                    .join(&sessions)
            }));
        }
        Some(Self {
            registered: Box::new(|app| super::registry::registered(app.scheme)),
            claude_sessions,
            copilot_db: profile.join(".copilot").join("data.db"),
        })
    }
}

/// The link that opens a chat in its agent's app, if an installed app does.
pub(super) fn app_link(
    agent: Provider,
    id: &str,
    cwd: Option<&str>,
    machine: &Machine,
) -> Option<AppLink> {
    let registered = |app: &App| (machine.registered)(app);
    match agent {
        // Both apps accept only a UUID; Mash's own IDs for subagent chats
        // are not one and have no app session.
        Provider::Codex if is_uuid(id) && registered(&CODEX) => Some(AppLink {
            app: CODEX.name,
            url: format!("codex://threads/{id}"),
            adds_session: false,
        }),
        Provider::ClaudeCode if is_uuid(id) && registered(&CLAUDE) => Some(AppLink {
            app: CLAUDE.name,
            url: claude_url(id, cwd),
            adds_session: !claude_app_owns(&machine.claude_sessions, id),
        }),
        Provider::Copilot if registered(&COPILOT) => copilot_app_session(&machine.copilot_db, id)
            .map(|session| AppLink {
                app: COPILOT.name,
                url: format!("ghapp://sessions/{session}"),
                adds_session: false,
            }),
        _ => None,
    }
}

pub(super) fn claude_url(id: &str, cwd: Option<&str>) -> String {
    match cwd.filter(|cwd| !cwd.is_empty()) {
        Some(cwd) => format!("claude://resume?session={id}&cwd={}", query_value(cwd)),
        None => format!("claude://resume?session={id}"),
    }
}

/// Text as one value of a URL's query: every byte but the unreserved ones
/// percent-encoded, so spaces, `&`, `#`, `+`, backslashes and non-ASCII
/// letters all arrive as written.
pub(super) fn query_value(text: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

pub(super) fn is_uuid(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.iter().map(|group| group.len()).eq([8, 4, 4, 4, 12])
        && groups
            .iter()
            .all(|group| group.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Whether an app bundle of one of the app's names in one of `folders`
/// registers its scheme in its (XML) Info.plist.
#[cfg(any(target_os = "macos", test))]
pub(super) fn bundle_registers(folders: &[PathBuf], app: &App) -> bool {
    let registered = format!("<string>{}</string>", app.scheme);
    folders
        .iter()
        .flat_map(|folder| app.bundles.iter().map(move |name| folder.join(name)))
        .filter_map(|bundle| std::fs::read_to_string(bundle.join("Contents/Info.plist")).ok())
        .any(|plist| plist.contains(&registered))
}

/// Whether one of the Claude app's session files has this CLI session as its
/// transcript.
fn claude_app_owns(roots: &[PathBuf], id: &str) -> bool {
    let children = |folder: &Path| -> Vec<PathBuf> {
        std::fs::read_dir(folder).map_or_else(
            |_| Vec::new(),
            |entries| {
                entries
                    .filter_map(|entry| Some(entry.ok()?.path()))
                    .collect()
            },
        )
    };
    roots
        .iter()
        .flat_map(|root| children(root))
        .flat_map(|account| children(&account))
        .flat_map(|organisation| children(&organisation))
        .filter(|file| is_claude_session_file(file))
        .any(|file| names_transcript(&file, id))
}

fn is_claude_session_file(file: &Path) -> bool {
    let named = file
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("local_"));
    named
        && file
            .extension()
            .is_some_and(|extension| extension == "json")
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

/// The Copilot session behind Mash's chat `id`, when one of the Copilot
/// app's workspaces holds it. An archived workspace is left out: the app
/// lists only the others as its own.
fn copilot_app_session(database: &Path, id: &str) -> Option<String> {
    use rusqlite::OpenFlags;
    if !database.is_file() {
        return None;
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let connection = rusqlite::Connection::open_with_flags(database, flags).ok()?;
    let mut sessions = connection
        .prepare(
            "SELECT session_id FROM workspaces \
             WHERE session_id IS NOT NULL AND archived_at IS NULL",
        )
        .ok()?;
    let rows = sessions.query_map([], |row| row.get::<_, String>(0)).ok()?;
    rows.filter_map(Result::ok)
        .find(|session| is_uuid(session) && copilot_chat_id(session) == id)
}

/// Mash's ID for the chat of a Copilot session, as the indexer makes it.
pub(super) fn copilot_chat_id(session: &str) -> String {
    mash_silver::parsers::stable_id("session", &[Provider::Copilot.as_str(), session])
}
