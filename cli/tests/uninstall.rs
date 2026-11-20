//! `mash uninstall` against a redirected home. Unix only: Windows resolves
//! Mash's directories through the Known Folders API, not HOME.
#![cfg(unix)]

use super::*;

/// A home where Mash is set up and all three agents are connected, with stand-in
/// agent CLIs that log their calls. `plugin` is the exit status of their
/// plugin commands.
fn fixture(plugin: i32) -> (tempfile::TempDir, Vec<(&'static str, String)>) {
    let root = tempfile::tempdir().unwrap();
    let home = root.path();
    let mash = home.join(".mash");
    std::fs::create_dir_all(mash.join("models/embedding")).unwrap();
    std::fs::create_dir_all(mash.join("mash.sqlite3.attachments")).unwrap();
    for (name, text) in [
        // A configuration that no longer loads must not block removing it.
        (".mash/config.toml", "not = [valid"),
        (".mash/mash.sqlite3", "index"),
        (".mash/mash.sqlite3-wal", "log"),
        (".mash/models/embedding/model.onnx", "model"),
        (".mash/node.env", "MASH_NODE_TOKEN=secret\n"),
        (
            ".claude.json",
            r#"{"mcpServers":{"mash":{"command":"mash","args":["mcp"]}}}"#,
        ),
        (
            ".claude/plugins/installed_plugins.json",
            r#"{"plugins":{"mash@mash":[{"scope":"user","installPath":"/plugins/mash"}]}}"#,
        ),
        (
            ".codex/config.toml",
            "[mcp_servers.mash]\ncommand = \"mash\"\nargs = [\"mcp\"]\n\n[plugins.\"mash@mash\"]\nenabled = true\n",
        ),
        (
            ".copilot/mcp-config.json",
            r#"{"mcpServers":{"mash":{"tools":["*"],"type":"local","command":"mash","args":["mcp"]}}}"#,
        ),
        (
            ".copilot/config.json",
            "// This file is managed automatically.\n{\"installedPlugins\":[{\"name\":\"mash\",\"marketplace\":\"mash\",\"cache_path\":\"/plugins/mash\",\"enabled\":true}]}\n",
        ),
        (".claude/projects/p/chat.jsonl", "the agent's own history"),
    ] {
        let path = home.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let bin = home.join("bin");
    std::fs::create_dir(&bin).unwrap();
    for agent in ["claude", "codex", "copilot"] {
        use std::os::unix::fs::PermissionsExt;
        let script = bin.join(agent);
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho \"{agent} $*\" >> '{}'\n[ \"$1\" = plugin ] && {{ echo offline >&2; exit {plugin}; }}\nexit 0\n",
                home.join("calls.log").display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let env = vec![
        ("HOME", home.display().to_string()),
        (
            "XDG_CONFIG_HOME",
            home.join(".config").display().to_string(),
        ),
        ("PATH", format!("{}:/usr/bin:/bin", bin.display())),
    ];
    (root, env)
}

fn uninstall(env: &[(&str, String)], arguments: &[&str]) -> (Option<i32>, Value) {
    let output = Command::cargo_bin("mash")
        .unwrap()
        .arg("uninstall")
        .args(arguments)
        .env_clear()
        .envs(env.iter().map(|(key, value)| (*key, value.as_str())))
        .output()
        .unwrap();
    let value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&output.stderr)));
    (output.status.code(), value)
}

#[test]
fn uninstall_plans_first_then_removes_what_mash_created() {
    let (root, env) = fixture(0);
    let home = root.path();

    let (code, plan) = uninstall(&env, &[]);
    assert_eq!(code, Some(0), "{plan}");
    assert_eq!(plan["applied"], false);
    let contents: Vec<_> = plan["paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["content"].as_str().unwrap(),
                item["outcome"]["state"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        contents,
        [
            ("models", "planned"),
            ("configuration", "planned"),
            ("index", "planned"),
            ("tokens", "planned")
        ],
        "{plan}"
    );
    assert_eq!(plan["paths"][2]["bytes"], 8, "{plan}");
    assert_eq!(plan["agents"][0]["agent"], "claude-code");
    assert_eq!(
        plan["agents"][1]["plugin"]["command"],
        "codex plugin remove mash@mash"
    );
    assert_eq!(plan["agents"][2]["agent"], "copilot");
    assert_eq!(
        plan["agents"][2]["mcp"]["command"],
        "copilot mcp remove mash"
    );
    assert!(
        plan["next"][0]
            .as_str()
            .unwrap()
            .contains("mash uninstall --yes"),
        "{plan}"
    );
    // A plan changes nothing and asks the agents nothing.
    assert!(home.join(".mash/mash.sqlite3").exists());
    assert!(!home.join("calls.log").exists());

    let (code, done) = uninstall(&env, &["--yes"]);
    assert_eq!(code, Some(0), "{done}");
    assert_eq!(done["applied"], true);
    assert!(!home.join(".mash").exists(), "{done}");
    let calls = std::fs::read_to_string(home.join("calls.log")).unwrap();
    for command in [
        "claude mcp remove --scope user mash",
        "claude plugin uninstall mash@mash",
        "codex mcp remove mash",
        "codex plugin remove mash@mash",
        "copilot mcp remove mash",
        "copilot plugin uninstall mash@mash",
    ] {
        assert!(calls.lines().any(|line| line == command), "{calls}");
    }
    // The agents' own histories were only ever read.
    assert!(home.join(".claude/projects/p/chat.jsonl").exists());
    assert!(
        done["next"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .as_str()
            .unwrap()
            .contains("still installed"),
        "{done}"
    );
}

#[test]
fn uninstall_keeps_history_on_request_and_exits_1_when_a_step_fails() {
    let (root, env) = fixture(1);
    let home = root.path();
    let (code, done) = uninstall(&env, &["--yes", "--keep-history"]);
    assert_eq!(code, Some(1), "{done}");
    assert_eq!(done["agents"][0]["mcp"]["outcome"]["state"], "removed");
    assert_eq!(done["agents"][0]["plugin"]["outcome"]["state"], "failed");
    assert_eq!(done["agents"][0]["plugin"]["outcome"]["message"], "offline");
    for kept in [
        "config.toml",
        "mash.sqlite3",
        "mash.sqlite3-wal",
        "node.env",
    ] {
        assert!(home.join(".mash").join(kept).exists(), "{kept}");
    }
    assert!(!home.join(".mash/models").exists());
    assert!(
        done["next"][0].as_str().unwrap().contains("failed"),
        "{done}"
    );
}
