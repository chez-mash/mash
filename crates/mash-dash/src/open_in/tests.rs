use std::path::{Path, PathBuf};

use mash_model::provider::Provider;

use super::AppLink;
use super::links::{self, App, Machine};
use super::registry::{registers_protocol, scheme_keys};

const ID: &str = "586f44fa-8aaa-4587-b69f-b763d3279ef1";
const SUBAGENT: &str = "50cf5623ea898883ead39971";

/// A machine whose apps live only in the test's own Applications folder.
fn machine(home: &Path) -> Machine {
    let folders = [home.join("Applications")];
    Machine {
        registered: Box::new(move |app: &App| links::bundle_registers(&folders, app)),
        claude_sessions: vec![claude_sessions(home)],
        copilot_db: home.join(".copilot/data.db"),
    }
}

fn claude_sessions(home: &Path) -> PathBuf {
    home.join("Library/Application Support/Claude/claude-code-sessions")
}

fn link(agent: Provider, id: &str, home: &Path) -> Option<AppLink> {
    links::app_link(agent, id, None, &machine(home))
}

fn app(home: &Path, name: &str, scheme: &str) {
    let contents = home.join("Applications").join(name).join("Contents");
    std::fs::create_dir_all(&contents).unwrap();
    let plist = format!(
        "<plist><dict><key>CFBundleURLSchemes</key><array><string>{scheme}</string></array></dict></plist>"
    );
    std::fs::write(contents.join("Info.plist"), plist).unwrap();
}

fn claude_session(root: &Path, file: &str, cli: &str) {
    let folder = root.join("acct/org");
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
            url: format!("codex://threads/{ID}"),
            adds_session: false,
        })
    );
    // Mash's IDs for subagent chats are not thread IDs.
    assert_eq!(link(Provider::Codex, SUBAGENT, home.path()), None);
}

#[test]
fn an_app_that_does_not_register_the_scheme_is_not_used() {
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "ChatGPT.app", "chatgpt");
    app(home.path(), "Claude.app", "claude-desktop");
    app(home.path(), "GitHub Copilot.app", "github-mobile");
    assert_eq!(link(Provider::Codex, ID, home.path()), None);
    assert_eq!(link(Provider::ClaudeCode, ID, home.path()), None);
    copilot_workspaces(home.path(), &[(ID, false)]);
    let chat = links::copilot_chat_id(ID);
    assert_eq!(link(Provider::Copilot, &chat, home.path()), None);
}

#[test]
fn claude_sessions_open_and_say_whether_the_app_adds_them() {
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "Claude.app", "claude");
    let adds = |adds_session| AppLink {
        app: "Claude",
        url: format!("claude://resume?session={ID}"),
        adds_session,
    };
    // A CLI session the app has never seen is imported as a new one.
    assert_eq!(
        link(Provider::ClaudeCode, ID, home.path()),
        Some(adds(true))
    );
    // A file that mentions the ID elsewhere does not make the app its owner.
    let other = "11111111-2222-3333-4444-555555555555";
    let root = claude_sessions(home.path());
    claude_session(&root, "local_a.json", other);
    let mention = serde_json::json!({"cliSessionId": other, "note": ID});
    std::fs::write(root.join("acct/org/local_b.json"), mention.to_string()).unwrap();
    // Nor does a file that is not one of the app's session files.
    claude_session(&root, "remote_c.json", ID);
    assert_eq!(
        link(Provider::ClaudeCode, ID, home.path()),
        Some(adds(true))
    );
    claude_session(&root, "local_d.json", ID);
    assert_eq!(
        link(Provider::ClaudeCode, ID, home.path()),
        Some(adds(false))
    );
    // Subagent chats have no session of their own to open.
    assert_eq!(link(Provider::ClaudeCode, SUBAGENT, home.path()), None);
}

#[test]
fn the_claude_app_owns_a_session_filed_in_any_of_its_folders() {
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "Claude.app", "claude");
    let package = home
        .path()
        .join("Packages/Claude/LocalCache/Roaming/Claude");
    let mut machine = machine(home.path());
    machine.claude_sessions.push(package.clone());
    claude_session(&package, "local_a.json", ID);
    let link = links::app_link(Provider::ClaudeCode, ID, None, &machine).unwrap();
    assert!(!link.adds_session);
}

#[test]
fn the_claude_link_carries_the_chat_folder_encoded() {
    assert_eq!(
        links::claude_url(ID, Some("/Users/me/My Work/a&b#c+d/日本")),
        format!(
            "claude://resume?session={ID}&cwd=%2FUsers%2Fme%2FMy%20Work%2Fa%26b%23c%2Bd%2F%E6%97%A5%E6%9C%AC"
        )
    );
    assert_eq!(
        links::claude_url(ID, Some(r"C:\work\mash")),
        format!("claude://resume?session={ID}&cwd=C%3A%5Cwork%5Cmash")
    );
    // Without a recorded folder the link names the session alone.
    for none in [None, Some("")] {
        assert_eq!(
            links::claude_url(ID, none),
            format!("claude://resume?session={ID}")
        );
    }
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "Claude.app", "claude");
    let link = links::app_link(Provider::ClaudeCode, ID, Some("/w"), &machine(home.path()));
    assert_eq!(
        link.unwrap().url,
        format!("claude://resume?session={ID}&cwd=%2Fw")
    );
}

#[test]
fn unreserved_characters_stay_as_written() {
    assert_eq!(links::query_value("Az09-._~"), "Az09-._~");
    assert_eq!(links::query_value("a b"), "a%20b");
}

fn copilot_workspaces(home: &Path, sessions: &[(&str, bool)]) {
    let folder = home.join(".copilot");
    std::fs::create_dir_all(&folder).unwrap();
    let connection = rusqlite::Connection::open(folder.join("data.db")).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS workspaces (id TEXT PRIMARY KEY, session_id TEXT, archived_at TEXT);
             DELETE FROM workspaces;",
        )
        .unwrap();
    for (index, (session, archived)) in sessions.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO workspaces (id, session_id, archived_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    format!("w{index}"),
                    session,
                    archived.then_some("2026-11-01T00:00:00Z")
                ],
            )
            .unwrap();
    }
}

#[test]
fn copilot_sessions_open_only_in_a_workspace_of_the_app() {
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "GitHub Copilot.app", "ghapp");
    let chat = links::copilot_chat_id(ID);
    // Without the app's database, no session is the app's.
    assert_eq!(link(Provider::Copilot, &chat, home.path()), None);
    let other = "11111111-2222-3333-4444-555555555555";
    copilot_workspaces(home.path(), &[(other, false)]);
    // A session outside the app's workspaces would be set up anew: no link.
    assert_eq!(link(Provider::Copilot, &chat, home.path()), None);
    copilot_workspaces(home.path(), &[(other, false), (ID, false)]);
    assert_eq!(
        link(Provider::Copilot, &chat, home.path()),
        Some(AppLink {
            app: "Copilot",
            url: format!("ghapp://sessions/{ID}"),
            adds_session: false,
        })
    );
    // The raw session ID is not Mash's chat ID.
    assert_eq!(link(Provider::Copilot, ID, home.path()), None);
    // A VS Code Copilot Chat session hashes differently, so it never matches.
    let vscode = mash_silver::parsers::stable_id("session", &["copilot-chat", ID]);
    assert_eq!(link(Provider::Copilot, &vscode, home.path()), None);
    // An archived workspace is not one the app lists.
    copilot_workspaces(home.path(), &[(ID, true)]);
    assert_eq!(link(Provider::Copilot, &chat, home.path()), None);
}

#[test]
fn a_copilot_database_without_workspaces_offers_nothing() {
    let home = tempfile::tempdir().unwrap();
    app(home.path(), "GitHub Copilot.app", "ghapp");
    let folder = home.path().join(".copilot");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("data.db"), "not a database").unwrap();
    let chat = links::copilot_chat_id(ID);
    assert_eq!(link(Provider::Copilot, &chat, home.path()), None);
}

#[test]
fn agents_whose_apps_have_no_chat_link_get_none() {
    let home = tempfile::tempdir().unwrap();
    for (bundle, scheme) in [
        ("Cursor.app", "cursor"),
        ("OpenCode.app", "opencode"),
        ("Antigravity.app", "antigravity"),
    ] {
        app(home.path(), bundle, scheme);
    }
    for agent in [Provider::Cursor, Provider::Opencode, Provider::Antigravity] {
        assert_eq!(link(agent, ID, home.path()), None, "{agent:?}");
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
#[tokio::test]
async fn other_systems_offer_no_link() {
    let params = super::OpenInParams {
        agent: Provider::Codex,
        id: ID.to_owned(),
        cwd: None,
    };
    assert_eq!(super::find(params).await, None);
}

/// `reg query HKEY_CLASSES_ROOT\vscode /s`, as Windows prints it for an
/// installer's registration.
const INSTALLED: &str = r#"
HKEY_CLASSES_ROOT\vscode
    URL Protocol    REG_SZ
    (Default)    REG_SZ    URL:vscode

HKEY_CLASSES_ROOT\vscode\shell

HKEY_CLASSES_ROOT\vscode\shell\open

HKEY_CLASSES_ROOT\vscode\shell\open\command
    (Default)    REG_SZ    "C:\Program Files\Microsoft VS Code\Code.exe" --open-url -- "%1"

"#;

/// The same for a packaged app (`ms-settings`), which delegates the launch.
const PACKAGED: &str = r"
HKEY_CLASSES_ROOT\ms-settings
    (Default)    REG_SZ    URL:ms-settings
    URL Protocol    REG_SZ

HKEY_CLASSES_ROOT\ms-settings\Application
    AppUserModelId    REG_SZ    windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel

HKEY_CLASSES_ROOT\ms-settings\Shell\Open
    ContractId    REG_SZ    Windows.Protocol
    DesiredInitialViewState    REG_DWORD    0x0

HKEY_CLASSES_ROOT\ms-settings\Shell\Open\Command
    DelegateExecute    REG_SZ    {4ed3a719-cea8-4bd9-910d-e252f997afc2}

";

#[test]
fn a_windows_scheme_is_registered_with_a_protocol_and_an_open_command() {
    assert!(registers_protocol(r"HKEY_CLASSES_ROOT\vscode", INSTALLED));
    // `reg` ends an empty value's line with the separator.
    let padded = INSTALLED.replace("REG_SZ\n", "REG_SZ    \n");
    assert!(registers_protocol(r"HKEY_CLASSES_ROOT\vscode", &padded));
    assert!(registers_protocol(
        r"HKEY_CLASSES_ROOT\ms-settings",
        PACKAGED
    ));
    let user = INSTALLED.replace(
        r"HKEY_CLASSES_ROOT\vscode",
        r"HKEY_CURRENT_USER\Software\Classes\codex",
    );
    assert!(registers_protocol(
        r"HKEY_CURRENT_USER\Software\Classes\codex",
        &user
    ));
    assert_eq!(
        scheme_keys("claude"),
        [
            r"HKEY_CURRENT_USER\Software\Classes\claude".to_owned(),
            r"HKEY_CLASSES_ROOT\claude".to_owned(),
        ]
    );
}

#[test]
fn a_windows_class_without_a_protocol_or_command_is_not_a_scheme() {
    let key = r"HKEY_CLASSES_ROOT\vscode";
    // A file type has an open command but is no URL protocol.
    let file_type = INSTALLED.replace("    URL Protocol    REG_SZ\n", "");
    assert!(!registers_protocol(key, &file_type));
    // A protocol with nothing to run opens nothing.
    let no_command = INSTALLED.replace(
        "    (Default)    REG_SZ    \"C:\\Program Files\\Microsoft VS Code\\Code.exe\" --open-url -- \"%1\"\n",
        "    (Default)    REG_SZ    \n",
    );
    assert!(!registers_protocol(key, &no_command));
    // A command key of some other scheme does not count.
    assert!(!registers_protocol(r"HKEY_CLASSES_ROOT\claude", INSTALLED));
    assert!(!registers_protocol(key, ""));
}
