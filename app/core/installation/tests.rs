#[cfg(unix)]
use super::plugin::expected_stamp;
use super::*;

fn environment(root: &Path) -> Environment {
    Environment {
        home: root.to_path_buf(),
        codex: root.join(".codex"),
        claude: root.join(".claude"),
        copilot: root.join(".copilot"),
        cwd: root.join("project"),
        path: std::ffi::OsString::new(),
    }
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn installation_cli_only_is_not_broken_but_expected_clients_are() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    let report = collect(Some(&environment), &[], &[]);
    assert_eq!(report.summary.attention, 0);
    assert!(report.clients.iter().all(|client| !client.expected));
    let report = collect(Some(&environment), &[Agent::Codex], &[]);
    assert_eq!(report.clients.len(), 1);
    assert_eq!(report.summary.attention, 2);
}

#[test]
fn installation_partial_setup_warns_and_never_serializes_secrets() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = '/missing/mash'\nargs = ['mcp', 'secret-argument']\n[mcp_servers.mash.env]\nTOKEN = 'secret-token'\n",
    );
    let report = collect(Some(&environment), &[], &[]);
    assert!(report.summary.attention >= 2);
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("secret-argument"));
    assert!(!json.contains("secret-token"));
    assert!(report.clients[0].launch.is_none());
}

#[test]
fn installation_malformed_settings_are_findings_not_errors() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = SECRET_VALUE_INVALID\n",
    );
    let report = collect(Some(&environment), &[Agent::Codex], &[]);
    assert!(report.summary.attention > 0);
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("SECRET_VALUE_INVALID")
    );
    assert!(report.clients[0].findings.iter().any(|finding| matches!(
        finding.component,
        Component::Configuration
    ) && finding.state == State::Invalid));
}

#[test]
fn installation_codex_checks_hook_files_trust_and_versions() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[plugins.'mash@mash-local']\nenabled = true\n",
    );
    let plugin = environment
        .codex
        .join("plugins/cache/mash-local/mash/local");
    write(
        &plugin.join(".codex-plugin/plugin.json"),
        include_str!("../../../plugins/mash/.codex-plugin/plugin.json"),
    );
    write(
        &plugin.join("skills/mash-history/SKILL.md"),
        "mash-history skill",
    );
    write(
        &plugin.join("skills/mash-history/contract.json"),
        include_str!("../../../plugins/mash/skills/mash-history/contract.json"),
    );
    write(
        &plugin.join("hooks/hooks.json"),
        include_str!("../../../plugins/mash/hooks/hooks.json"),
    );
    for script in [
        "session_start.py",
        "prompt_recall.py",
        "file_context.py",
        "record_capture.py",
        "python.sh",
    ] {
        write(&plugin.join("hooks").join(script), "# hook");
    }
    let report = collect(Some(&environment), &[], &[]);
    let findings = &report.clients[0].findings;
    assert_eq!(
        findings
            .iter()
            .filter(|finding| matches!(finding.component, Component::HookTrust)
                && finding.state == State::Missing)
            .count(),
        3,
        "every bundled hook needs its own Codex approval"
    );
    assert!(
        findings
            .iter()
            .any(|finding| matches!(finding.component, Component::Plugin)
                && finding.state == State::Present)
    );
    std::fs::remove_file(plugin.join("hooks/session_start.py")).unwrap();
    write(&plugin.join("skills/mash-history/contract.json"), "{}");
    let report = collect(Some(&environment), &[], &[]);
    assert!(
        report.clients[0]
            .findings
            .iter()
            .any(|finding| matches!(finding.component, Component::Hooks)
                && finding.state == State::Missing)
    );
    assert!(
        report.clients[0]
            .findings
            .iter()
            .any(|finding| matches!(finding.component, Component::Skill)
                && finding.state == State::Outdated)
    );
}

#[test]
fn installation_claude_disabled_hooks_and_mcp_are_reported() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.claude.join("settings.json"),
        r#"{"enabledPlugins":{"mash@mash-local":true},"disableAllHooks":true,"disabledMcpServers":["mash"]}"#,
    );
    let report = collect(Some(&environment), &[], &[]);
    let client = &report.clients[1];
    assert!(client.expected);
    assert!(client.launch.is_none());
    for component in [Component::Hooks, Component::Mcp] {
        assert!(
            client
                .findings
                .iter()
                .any(|finding| std::mem::discriminant(&finding.component)
                    == std::mem::discriminant(&component)
                    && finding.state == State::Disabled)
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn installation_probe_reports_failed_handshake_without_leaking_stderr() {
    let root = tempfile::tempdir().unwrap();
    let mut environment = environment(root.path());
    environment.cwd = root.path().to_path_buf();
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = '/bin/sh'\nargs = ['-c', 'echo SECRET >&2; exit 1']\n",
    );
    let mut report = collect(Some(&environment), &[Agent::Codex], &[]);
    probe::check(&mut report.clients[0], &"0".repeat(64)).await;
    assert!(
        report.clients[0]
            .findings
            .iter()
            .any(|finding| matches!(finding.component, Component::Probe)
                && finding.state == State::Failed)
    );
    assert!(!serde_json::to_string(&report).unwrap().contains("SECRET"));
}

#[cfg(unix)]
#[test]
fn installation_project_override_does_not_probe_user_registration() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = '/bin/true'\n",
    );
    write(
        &environment.cwd.join(".codex/config.toml"),
        "[mcp_servers.mash]\nenabled = false\n",
    );
    let report = collect(Some(&environment), &[Agent::Codex], &[]);
    assert!(report.clients[0].launch.is_none());
    assert!(report.summary.unverified > 0);
}

#[cfg(unix)]
#[test]
fn installation_complete_claude_user_setup_has_no_attention_findings() {
    let root = tempfile::tempdir().unwrap();
    let (environment, _) = claude_fixture(root.path());
    let report = collect(Some(&environment), &[Agent::ClaudeCode], &[]);
    assert_eq!(report.summary.attention, 0, "{report:?}");
    assert!(report.clients[0].launch.is_some());
    assert!(report.clients[0].expected);
}

#[cfg(unix)]
fn claude_fixture(root: &Path) -> (Environment, PathBuf) {
    let mut environment = environment(root);
    let bin = root.join("bin");
    std::fs::create_dir(&bin).unwrap();
    std::os::unix::fs::symlink("/bin/true", bin.join("claude")).unwrap();
    environment.path =
        std::env::join_paths([bin.as_path(), Path::new("/usr/bin"), Path::new("/bin")]).unwrap();
    write(
        &environment.claude.join("settings.json"),
        r#"{"enabledPlugins":{"mash@mash-local":true}}"#,
    );
    write(
        &environment.home.join(".claude.json"),
        r#"{"mcpServers":{"mash":{"command":"/bin/true"}}}"#,
    );
    let plugin = root.join("plugin");
    let registry =
        serde_json::json!({"plugins":{"mash@mash-local":[{"scope":"user","installPath":plugin}]}});
    write(
        &environment.claude.join("plugins/installed_plugins.json"),
        &registry.to_string(),
    );
    write_claude_plugin(&plugin);
    (environment, plugin)
}

#[cfg(unix)]
#[test]
fn installation_plugin_version_and_contract_choose_the_component_to_update() {
    let directory = tempfile::tempdir().unwrap();
    let (environment, plugin) = claude_fixture(directory.path());
    let expected = expected_stamp();
    let matching = expected.contract_fingerprint.as_str();
    let bundled = expected.plugin_version.as_str();
    let cases = [
        ("0.0.0", "0.0.0", matching, Some("Update the Mash plugin")),
        ("10.0.0", "10.0.0", matching, None),
        ("10.0.0", "10.0.0", "different", Some("Update the Mash CLI")),
        (
            bundled,
            bundled,
            "different",
            Some("Install the Mash plugin paired"),
        ),
        (
            "10.0.0",
            bundled,
            matching,
            Some("Reinstall the Mash plugin"),
        ),
        (
            "SECRET-invalid-version",
            bundled,
            matching,
            Some("Reinstall the Mash plugin"),
        ),
    ];
    for (installed, stamped, fingerprint, action) in cases {
        write(
            &plugin.join(".claude-plugin/plugin.json"),
            &serde_json::json!({"name": "mash", "version": installed}).to_string(),
        );
        write(
            &plugin.join("skills/mash-history/contract.json"),
            &serde_json::json!({
                "plugin_version": stamped,
                "contract_fingerprint": fingerprint,
            })
            .to_string(),
        );
        for agents in [&[][..], &[Agent::ClaudeCode][..]] {
            let report = collect(Some(&environment), agents, &[]);
            let client = report
                .summary
                .clients
                .iter()
                .find(|client| client.agent == Agent::ClaudeCode)
                .unwrap();
            if let Some(action) = action {
                assert_eq!(client.connection, Connection::McpOnly, "{report:?}");
                assert!(report.agents_need_attention(), "{report:?}");
                assert!(
                    report
                        .summary
                        .next_action
                        .as_deref()
                        .unwrap()
                        .contains(action),
                    "{report:?}"
                );
            } else {
                assert_eq!(client.connection, Connection::Full, "{report:?}");
                assert!(!report.agents_need_attention(), "{report:?}");
                assert!(report.summary.next_action.is_none(), "{report:?}");
                assert!(
                    serde_json::to_string(&report)
                        .unwrap()
                        .contains("same tool contract")
                );
            }
            assert!(
                !serde_json::to_string(&report)
                    .unwrap()
                    .contains("SECRET-invalid-version")
            );
        }
    }
}

/// A complete plugin tree like the one the passing user setup installs.
#[cfg(unix)]
fn write_claude_plugin(plugin: &Path) {
    write(
        &plugin.join(".claude-plugin/plugin.json"),
        include_str!("../../../plugins/mash/.claude-plugin/plugin.json"),
    );
    write(
        &plugin.join("skills/mash-history/contract.json"),
        include_str!("../../../plugins/mash/skills/mash-history/contract.json"),
    );
    write(&plugin.join("skills/mash-history/SKILL.md"), "mash-history");
    write(
        &plugin.join("hooks/hooks.json"),
        include_str!("../../../plugins/mash/hooks/hooks.json"),
    );
    for name in [
        "session_start.py",
        "prompt_recall.py",
        "file_context.py",
        "record_capture.py",
        "recall_common.py",
        "record_recall.py",
        "python.sh",
    ] {
        write(&plugin.join("hooks").join(name), "# script");
    }
}

#[cfg(unix)]
#[test]
fn installation_claude_directory_marketplace_is_inspected_in_place() {
    let root = tempfile::tempdir().unwrap();
    let mut environment = environment(root.path());
    environment.path = "/usr/bin:/bin".into();
    write(
        &environment.claude.join("settings.json"),
        r#"{"enabledPlugins":{"mash@mash-local":true}}"#,
    );
    write(
        &environment.home.join(".claude.json"),
        r#"{"mcpServers":{"mash":{"command":"/bin/true"}}}"#,
    );
    // The registry still names the cache copy from the first installation.
    let stale = root.path().join("cache/mash/0.1.15");
    write(
        &stale.join(".claude-plugin/plugin.json"),
        r#"{"name":"mash","version":"0.1.15"}"#,
    );
    let registry =
        serde_json::json!({"plugins":{"mash@mash-local":[{"scope":"user","installPath":stale}]}});
    write(
        &environment.claude.join("plugins/installed_plugins.json"),
        &registry.to_string(),
    );
    let checkout = root.path().join("checkout");
    write_claude_plugin(&checkout.join("plugins/mash"));
    let marketplaces = environment.claude.join("plugins/known_marketplaces.json");
    let known = |kind: &str, catalog_source: serde_json::Value| {
        write(
            &checkout.join(".claude-plugin/marketplace.json"),
            &serde_json::json!({"name":"mash","plugins":[{"name":"mash","source":catalog_source}]})
                .to_string(),
        );
        write(
            &marketplaces,
            &serde_json::json!({"mash-local":{"source":{"source":kind,"path":checkout},"installLocation":checkout}})
                .to_string(),
        );
        collect(Some(&environment), &[Agent::ClaudeCode], &[])
            .summary
            .attention
    };
    assert_eq!(known("directory", "./plugins/mash".into()), 0);
    // Only a directory marketplace is read in place, and only a path inside it.
    assert_ne!(known("github", "./plugins/mash".into()), 0);
    assert_ne!(known("directory", "../plugins/mash".into()), 0);
    assert_ne!(
        known(
            "directory",
            serde_json::json!({"source":"github","repo":"chez-mash/mash"})
        ),
        0
    );
}

#[cfg(unix)]
#[tokio::test]
async fn installation_probe_skips_watch_and_preserves_disabled_registration() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    for config in [
        "[mcp_servers.mash]\ncommand = '/bin/true'\nargs = ['mcp', '--watch']\n",
        "[mcp_servers.mash]\ncommand = '/bin/true'\nenabled = false\n",
    ] {
        write(&environment.codex.join("config.toml"), config);
        let mut report = collect(Some(&environment), &[Agent::Codex], &[]);
        if config.contains("--watch") {
            assert!(report.clients[0].findings.iter().any(|finding| matches!(
                finding.component,
                Component::Mcp
            ) && finding.state
                == State::Outdated));
        }
        probe::check(&mut report.clients[0], &"0".repeat(64)).await;
        assert!(report.clients[0].findings.iter().any(|finding| matches!(
            finding.component,
            Component::Probe
        ) && finding.state
            == State::Unverified));
    }
}

#[test]
fn installation_multiple_codex_copies_are_ambiguous() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[plugins.'mash@mash-local']\nenabled = true\n",
    );
    for version in ["0.0.1", "0.0.2"] {
        std::fs::create_dir_all(
            environment
                .codex
                .join("plugins/cache/mash-local/mash")
                .join(version),
        )
        .unwrap();
    }
    let report = collect(Some(&environment), &[Agent::Codex], &[]);
    assert!(report.clients[0].findings.iter().any(|finding| matches!(
        finding.component,
        Component::Plugin
    ) && finding.state
        == State::Unverified));
}

/// An environment whose PATH has discovery-only `claude`, `codex` and
/// `copilot` files.
#[cfg(unix)]
fn with_agent_clis(root: &Path) -> Environment {
    let mut environment = environment(root);
    let bin = root.join("bin");
    for agent in ["claude", "codex", "copilot"] {
        write(&bin.join(agent), "discovery-only test fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(bin.join(agent), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
    }
    environment.path = bin.into_os_string();
    environment
}

#[cfg(unix)]
fn finding(client: &Client, component: Component) -> &Finding {
    client
        .findings
        .iter()
        .find(|finding| finding.component == component)
        .unwrap()
}

// The stand-in agent CLIs are Unix executables.
#[cfg(unix)]
#[test]
fn installation_expects_recorded_agents_whose_registration_disappeared() {
    let root = tempfile::tempdir().unwrap();
    let environment = with_agent_clis(root.path());
    std::fs::create_dir_all(&environment.claude).unwrap();
    // Without a record, a missing registration is valid CLI-only use.
    let unused = collect(Some(&environment), &[], &[]);
    assert!(!unused.agents_need_attention());
    let configuration = finding(&unused.clients[1], Component::Configuration);
    assert_eq!(
        configuration.remediation.as_deref(),
        Some("Run `mash init` and select this agent.")
    );
    let report = collect(Some(&environment), &[], &[Agent::ClaudeCode]);
    let client = &report.clients[1];
    assert!(client.expected && client.expectation == Expectation::Recorded);
    let mcp = finding(client, Component::Mcp);
    assert_eq!(mcp.state, State::Missing);
    assert!(mcp.message.contains("`mash init` connected this agent"));
    let remedy = mcp.remediation.as_deref().unwrap();
    assert!(
        remedy.starts_with("Run `claude mcp add --scope user mash -- "),
        "{remedy}"
    );
    let summary = &report.summary.clients[1];
    assert_eq!(summary.connection, Connection::None);
    assert_eq!(summary.next_action.as_deref(), Some(remedy));
    assert_eq!(summary.missing, [Component::Plugin, Component::Mcp]);
    assert!(report.agents_need_attention());
    // The unrecorded Codex client stays optional.
    assert_eq!(report.summary.clients[0].next_action, None);
}

#[cfg(unix)]
#[tokio::test]
async fn installation_names_a_missing_registered_executable_and_its_replacement() {
    let root = tempfile::tempdir().unwrap();
    let environment = with_agent_clis(root.path());
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = '/old/mash'\nargs = ['mcp']\n",
    );
    assert_eq!(
        clients::registration(&environment, Agent::Codex),
        Registration::Stale {
            command: "/old/mash".into()
        }
    );
    let mut report = collect(Some(&environment), &[], &[]);
    let mcp = finding(&report.clients[0], Component::Mcp);
    assert_eq!(mcp.state, State::Missing);
    assert!(mcp.message.contains("/old/mash"), "{}", mcp.message);
    let remedy = mcp.remediation.as_deref().unwrap();
    assert!(
        remedy.starts_with("Run `codex mcp remove mash && codex mcp add mash -- "),
        "{remedy}"
    );
    probe::check(&mut report.clients[0], &"0".repeat(64)).await;
    assert_eq!(
        finding(&report.clients[0], Component::Probe).message,
        "Skipped: the registered MCP executable does not exist."
    );
    // A registration that runs this binary is current; another binary is stale.
    let current = std::env::current_exe().unwrap();
    write(
        &environment.codex.join("config.toml"),
        &format!(
            "[mcp_servers.mash]\ncommand = {}\nargs = ['mcp']\n",
            toml::Value::String(current.to_string_lossy().into_owned())
        ),
    );
    assert_eq!(
        clients::registration(&environment, Agent::Codex),
        Registration::Current
    );
    write(
        &environment.codex.join("config.toml"),
        "[mcp_servers.mash]\ncommand = '/bin/sh'\nargs = ['mcp']\n",
    );
    assert_eq!(
        clients::registration(&environment, Agent::Codex),
        Registration::Stale {
            command: "/bin/sh".into()
        }
    );
    std::fs::remove_file(environment.codex.join("config.toml")).unwrap();
    assert_eq!(
        clients::registration(&environment, Agent::Codex),
        Registration::Unknown
    );
}

#[cfg(unix)]
#[test]
fn installation_mcp_without_the_plugin_is_mcp_only_with_the_plugin_commands() {
    let root = tempfile::tempdir().unwrap();
    let environment = with_agent_clis(root.path());
    write(
        &environment.home.join(".claude.json"),
        r#"{"mcpServers":{"mash":{"command":"/bin/true","args":["mcp"]}}}"#,
    );
    let report = collect(Some(&environment), &[], &[]);
    let summary = &report.summary.clients[1];
    assert_eq!(summary.connection, Connection::McpOnly);
    let action = summary.next_action.as_deref().unwrap();
    assert!(
        action.contains("The Mash MCP tools work without the plugin"),
        "{action}"
    );
    assert!(
        action.contains(
            "claude plugin marketplace add chez-mash/mash && claude plugin install mash@mash"
        ),
        "{action}"
    );
    assert!(!action.contains("mash init"), "{action}");
    assert!(report.agents_need_attention());
}

/// Copilot's user settings as its own CLI leaves them: the registration with
/// `"type": "local"`, and `config.json` behind its comment header, naming
/// the installed copy `plugin`.
#[cfg(unix)]
fn write_copilot_settings(environment: &Environment, plugin: &Path, enabled: bool) {
    write(
        &environment.copilot.join("mcp-config.json"),
        r#"{"mcpServers":{"mash":{"tools":["*"],"type":"local","command":"/bin/true","args":["mcp"]}}}"#,
    );
    let config = serde_json::json!({
        "copilot_tokens": {"https://github.com:someone": "SECRET_COPILOT_TOKEN"},
        "installedPlugins": [
            {"name": "other", "marketplace": "elsewhere", "enabled": true},
            {
                "name": "mash",
                "marketplace": "mash",
                "version": expected_stamp().plugin_version,
                "cache_path": plugin,
                "installed_source": {"source": "github", "repo": "chez-mash/mash"},
                "enabled": enabled
            }
        ]
    });
    write(
        &environment.copilot.join("config.json"),
        &format!(
            "// User settings belong in settings.json.\n// This file is managed automatically.\n{config}\n"
        ),
    );
}

/// The installed plugin copy, built from the repository's own files.
#[cfg(unix)]
fn write_copilot_plugin(plugin: &Path) {
    write(
        &plugin.join(".github/plugin/plugin.json"),
        include_str!("../../../plugins/mash/.github/plugin/plugin.json"),
    );
    write(
        &plugin.join("skills/mash-history/contract.json"),
        include_str!("../../../plugins/mash/skills/mash-history/contract.json"),
    );
    write(&plugin.join("skills/mash-history/SKILL.md"), "mash-history");
    write(
        &plugin.join("hooks/copilot.json"),
        include_str!("../../../plugins/mash/hooks/copilot.json"),
    );
    for name in [
        "session_start.py",
        "prompt_recall.py",
        "file_context.py",
        "recall_common.py",
        "record_recall.py",
        "python.sh",
    ] {
        write(&plugin.join("hooks").join(name), "# script");
    }
}

#[cfg(unix)]
fn states(client: &Client, component: Component) -> Vec<State> {
    client
        .findings
        .iter()
        .filter(|finding| finding.component == component)
        .map(|finding| finding.state)
        .collect()
}

#[cfg(unix)]
#[test]
fn installation_complete_copilot_user_setup_has_no_attention_findings() {
    let root = tempfile::tempdir().unwrap();
    let mut environment = environment(root.path());
    environment.path = "/usr/bin:/bin".into();
    let plugin = environment.copilot.join("installed-plugins/mash/mash");
    write_copilot_settings(&environment, &plugin, true);
    write_copilot_plugin(&plugin);
    // Unnamed, Copilot is expected because its own settings name Mash.
    let report = collect(Some(&environment), &[], &[]);
    assert_eq!(report.summary.attention, 0, "{report:?}");
    let client = &report.clients[2];
    assert_eq!(client.agent, Agent::Copilot);
    assert!(client.expected && client.expectation == Expectation::Configured);
    assert!(client.launch.is_some());
    assert_eq!(states(client, Component::Plugin), [State::Present]);
    assert_eq!(states(client, Component::Skill), [State::Present]);
    assert_eq!(states(client, Component::Hooks), [State::Present; 3]);
    assert_eq!(states(client, Component::Mcp), [State::Present]);
    assert_eq!(report.summary.clients[2].connection, Connection::Full);
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("SECRET_COPILOT_TOKEN")
    );
    assert_eq!(
        clients::connection(&environment, Agent::Copilot),
        (true, true)
    );
    assert_eq!(
        clients::edited_files(&environment, Agent::Copilot),
        [
            environment.copilot.join("mcp-config.json"),
            environment.copilot.join("config.json"),
            environment.copilot.join("installed-plugins"),
        ]
    );

    // A hook whose command was edited, and a script that is gone.
    let hooks = plugin.join("hooks/copilot.json");
    let text = std::fs::read_to_string(&hooks).unwrap();
    write(
        &hooks,
        &text.replace("prompt_recall.py\\\" 2>", "other.py\\\" 2>"),
    );
    std::fs::remove_file(plugin.join("hooks/file_context.py")).unwrap();
    let report = collect(Some(&environment), &[Agent::Copilot], &[]);
    assert_eq!(
        states(&report.clients[0], Component::Hooks),
        [State::Present, State::Missing, State::Missing]
    );
    assert_eq!(report.summary.clients[0].connection, Connection::McpOnly);
}

#[cfg(unix)]
#[test]
fn installation_copilot_without_the_plugin_is_mcp_only_with_its_plugin_commands() {
    let root = tempfile::tempdir().unwrap();
    let environment = with_agent_clis(root.path());
    write(
        &environment.copilot.join("mcp-config.json"),
        r#"{"mcpServers":{"mash":{"tools":["*"],"type":"local","command":"/bin/true","args":["mcp"]}}}"#,
    );
    // Other plugins are installed, and the file still has its comment header.
    write(
        &environment.copilot.join("config.json"),
        "// This file is managed automatically.\n{\"installedPlugins\":[{\"name\":\"other\",\"enabled\":true}]}",
    );
    let report = collect(Some(&environment), &[], &[]);
    let client = &report.clients[2];
    assert!(client.expected);
    assert_eq!(states(client, Component::Plugin), [State::Missing]);
    assert_eq!(states(client, Component::Mcp), [State::Present]);
    assert_eq!(states(client, Component::Configuration), Vec::new());
    let summary = &report.summary.clients[2];
    assert_eq!(summary.connection, Connection::McpOnly);
    assert_eq!(summary.missing, [Component::Plugin]);
    let action = summary.next_action.as_deref().unwrap();
    assert!(
        action.contains(
            "copilot plugin marketplace add chez-mash/mash && copilot plugin install mash@mash"
        ),
        "{action}"
    );
    assert_eq!(
        clients::connection(&environment, Agent::Copilot),
        (true, false)
    );
}

#[cfg(unix)]
#[test]
fn installation_copilot_disabled_plugin_is_reported() {
    let root = tempfile::tempdir().unwrap();
    let mut environment = environment(root.path());
    environment.path = "/usr/bin:/bin".into();
    let plugin = environment.copilot.join("installed-plugins/mash/mash");
    write_copilot_settings(&environment, &plugin, false);
    write_copilot_plugin(&plugin);
    let report = collect(Some(&environment), &[], &[]);
    let client = &report.clients[2];
    // The MCP registration alone still makes the integration expected.
    assert!(client.expected);
    assert_eq!(states(client, Component::Plugin), [State::Disabled]);
    assert_eq!(states(client, Component::Hooks), Vec::new());
    assert_eq!(report.summary.clients[2].connection, Connection::McpOnly);
    assert!(report.agents_need_attention());
}

#[test]
fn installation_copilot_unreadable_or_malformed_config_is_a_finding() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    let config = environment.copilot.join("config.json");
    for content in [
        "// This file is managed automatically.\n{\"installedPlugins\": SECRET_VALUE_INVALID",
        "// only a comment",
        "",
        "{\"installedPlugins\":{\"name\":\"mash\"}}",
    ] {
        write(&config, content);
        let report = collect(Some(&environment), &[Agent::Copilot], &[]);
        assert!(
            report.clients[0].findings.iter().any(|finding| {
                finding.component == Component::Configuration
                    && finding.state == State::Invalid
                    && finding.path.as_deref() == Some(config.as_path())
            }),
            "{content}: {report:?}"
        );
        assert!(report.summary.attention > 0);
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains("SECRET_VALUE_INVALID")
        );
        assert_eq!(
            clients::connection(&environment, Agent::Copilot),
            (false, false)
        );
    }
    // A directory in the file's place cannot be read at all.
    std::fs::remove_file(&config).unwrap();
    std::fs::create_dir(&config).unwrap();
    let report = collect(Some(&environment), &[Agent::Copilot], &[]);
    assert!(report.clients[0].findings.iter().any(|finding| {
        finding.component == Component::Configuration && finding.state == State::Invalid
    }));
}

#[cfg(unix)]
#[test]
fn installation_copilot_local_transport_is_an_inspectable_registration() {
    let root = tempfile::tempdir().unwrap();
    let environment = environment(root.path());
    let path = environment.copilot.join("mcp-config.json");
    let register = |command: &str, kind: &str| {
        write(
            &path,
            &serde_json::json!({"mcpServers":{"mash":{"tools":["*"],"type":kind,"command":command,"args":["mcp"]}}})
                .to_string(),
        );
    };
    assert_eq!(
        clients::registration(&environment, Agent::Copilot),
        Registration::Unknown
    );
    register("/old/mash", "local");
    assert_eq!(
        clients::registration(&environment, Agent::Copilot),
        Registration::Stale {
            command: "/old/mash".into()
        }
    );
    let current = std::env::current_exe().unwrap();
    register(&current.to_string_lossy(), "local");
    assert_eq!(
        clients::registration(&environment, Agent::Copilot),
        Registration::Current
    );
    // A remote transport stays uninspectable, and `local` means stdio for
    // Copilot only.
    register("/old/mash", "http");
    assert_eq!(
        clients::registration(&environment, Agent::Copilot),
        Registration::Unknown
    );
    write(
        &environment.home.join(".claude.json"),
        r#"{"mcpServers":{"mash":{"type":"local","command":"/old/mash"}}}"#,
    );
    assert_eq!(
        clients::registration(&environment, Agent::ClaudeCode),
        Registration::Unknown
    );
}
