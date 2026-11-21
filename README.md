<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/mash-logo-dark.svg">
    <img src="assets/mash-logo-light.svg" alt="mash" width="360">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/mashium/mash/actions/workflows/ci.yml"><img src="https://github.com/mashium/mash/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/mashium/mash/actions/workflows/windows.yml"><img src="https://github.com/mashium/mash/actions/workflows/windows.yml/badge.svg?branch=main" alt="Windows"></a>
  <a href="https://getmash.io"><img src="https://github.com/mashium/mash/actions/workflows/docs.yml/badge.svg?branch=main" alt="Documentation"></a>
  <a href="#licence"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="MIT or Apache 2.0 licence"></a>
</p>

# mash - The **M**ulti-**A**gent **S**earch **H**istory Tool

**Find past coding-agent chats and see why you made a decision.**

Mash searches the history Claude Code, Codex, GitHub Copilot, Cursor, OpenCode,
and Antigravity already saved on your machine.

Browse those chats in the [browser dashboard](docs/dashboard.md) with `mash dash`,
[search from the terminal](docs/search.md), or [connect a coding agent](docs/agents.md)
so it can search your history for you.

Mash is open source and self-hosted. The default setup keeps indexing and
search on your machine and needs no account. [How Mash handles your history](docs/privacy.md)

[Try the demo without installing](https://getmash.io/demo/) · [Quick start](#quick-start) · [Check the release download](docs/installation.md#check-the-release-download)

[![Mash dashboard showing a sample conversation and the message behind a saved decision](assets/dashboard-demo.png)](https://getmash.io/demo/#/chat/sign-in-tokens?node=laptop)

## Why Mash

- **Find the exact message.** Look up an error, filename, or phrase across agents.
  Open the match in the dashboard with its surrounding messages and tool output.
  If you remember the idea more than the wording, optional local models let
  you search by meaning. [Search guide](docs/search.md)
- **Check why a decision was made.** The plugin saves decisions and preferences
  with a quote from the conversation. Open the source message yourself, or
  ask your agent to find it in a later session. [Conversation records](docs/records.md)
- **Pick up work on another machine.** Your laptop can search chats from your
  workstation through a server you run. Each machine keeps its own local
  index, so local search keeps working offline. [Multi-machine setup](docs/shared-index.md)

## Quick start

[Español](docs/es/quick-start.md) · [Français](docs/fr/quick-start.md) · [简体中文](docs/zh-cn/quick-start.md)

[Try a sample search](https://getmash.io/demo/#/search?q=invoice+exports+queue)
and open a result in its conversation. Set up Mash below to search your own
chats in the browser, in the terminal, or through an agent you connect.

Before running a command, you can [inspect the installer and check the release download](docs/installation.md#check-the-release-download).
To start with one agent and a short history range, follow the [local trial guide](docs/privacy.md#try-a-small-local-setup).

**Ask your agent.** Paste this into Claude Code, Codex, or another coding agent.

> Read https://getmash.io/setup.md and set up Mash on this machine.

If you installed the [Mash plugin](docs/plugin.md#start-with-the-plugin),
you can ask for its `setup` skill.

**Set it up yourself.** Run one of these commands in a terminal for a new
local setup. It installs Mash and the local search service, then asks you to
review which histories to read and which agents to connect before indexing.

On macOS (Apple silicon) or Linux (x86-64 or ARM64):

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --setup
```

On Windows x64, the setup stores search data in a short folder on the system
drive, usually `C:\mash-qdrant\storage`. [See where Mash stores data](docs/privacy.md#where-mash-stores-data).
Run this in PowerShell:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Setup
```

The scripts verify the downloads and install Mash into your `.local/bin`.
Qdrant (about 30 MB) runs under your user and starts at login. Word search
works without downloading any models. Linux and WSL need a systemd user
session for this setup.

The default history range is 90 days. Review the setup and choose **Set up
and index** when you’re ready. If the wizard first asks where to search, choose
**Just this computer (recommended)**. For WSL without systemd, a search service you
already run, or a binary-only install, follow the [installation guide](docs/installation.md).
You can also [review setup in the dashboard](docs/dashboard.md#set-up-mash-from-the-dashboard).

<span id="try-it"></span>

## Find your first chat

When setup says search is ready, start the dashboard and leave this terminal open.

```bash
mash dash
```

1. Open the address it prints in your browser, usually [http://127.0.0.1:8790](http://127.0.0.1:8790).
2. Open **chats**, type a word you remember, such as an error or function name, and press Enter.
3. Open a result to see the original message in its conversation.

If your terminal can’t find `mash`, use the full path shown during setup.
For commands that make `mash` available by name, see [command not found](docs/troubleshooting.md#mash-command-not-found).
If a chat is missing, see [what to check when search returns nothing](docs/troubleshooting.md#search-returns-no-results).
See the [dashboard guide](docs/dashboard.md#find-and-read-a-chat) or
[search from the terminal](docs/search.md#your-first-search).

**Search from an agent.** Restart any agent you connected during setup. For
example, in Codex, ask “Use Mash to find what we decided about authentication
in Claude Code. Show me the message behind the answer.”
[Check an agent connection](docs/agents.md#confirm-the-connection) if it
cannot see Mash.

Keep new chats searchable with [`mash index` or a background watcher](docs/synchronization.md).
You can also [export and back up your history](docs/upgrades.md).

<details>
<summary>Install with a package manager</summary>

| Channel | Command |
| --- | --- |
| Homebrew | `brew install mashium/tap/mash` |
| apt, Debian and Ubuntu | [Add the Mash repository](docs/install-methods.md#apt-debian-or-ubuntu), then `sudo apt install mash-cli` |
| Scoop | `scoop bucket add mashium https://github.com/mashium/mash`, then `scoop install mashium/mash` |
| mise | `mise use -g github:mashium/mash` |
| cargo-binstall | `cargo binstall --git https://github.com/mashium/mash mash` |

</details>

## Privacy

With the default local setup, indexing and search stay on your machine.
Sharing history across your machines is optional.

Mash masks recognizable credentials in results by default, but detection is
best effort. Local history and exports can contain sensitive text;
treat backups like the original chats. Mash never modifies the contents of
the agent’s source history. When an agent searches through Mash, the results
it receives become part of that agent's context. [Privacy and redaction →](docs/privacy.md)

## Tested with

Where each agent's history has been indexed and searched so far:

| Agent | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Claude Code | Daily use | Daily use | Light use |
| Codex | Daily use | Daily use | Light use |
| GitHub Copilot | Light use | Light use | Recorded history only |
| OpenCode | Recorded history only | Light use | Simulated in CI |
| Antigravity | Recorded history only | Light use | Recorded history only |
| Cursor | Recorded history only | Partial | Recorded history only |

- **Daily use:** hundreds of real chats from an installed agent on that system.
- **Light use:** fewer than twenty real chats from an installed agent on that system.
- **Simulated in CI:** every push indexes a history written the way the agent
  writes it on that system, while it is being written.
- **Partial:** chats from Cursor's command-line agent read correctly from a real
  installation. Chats made in the Cursor editor are not read yet when they were
  saved by an older Cursor version.
- **Recorded history only:** the reader is tested on every push against history
  in the agent's format, but no installed copy has been indexed on that system.

The Linux and Windows test suites run on every push; macOS has no automated
tests yet. If an agent's chats are missing or look wrong on your system,
[open an issue](https://github.com/mashium/mash/issues).

## Build from source

Use a stable Rust toolchain on platforms without a prebuilt release or to try
an unreleased revision. [Build Mash from source](docs/install-methods.md#build-from-source)
has instructions for the CLI and the web dashboard.

## Documentation

The guides live at **[getmash.io](https://getmash.io)**: search and filters,
agent integrations, the dashboard, sharing across machines, configuration,
troubleshooting, and the CLI and MCP references.

Upgrading an existing installation? Read [the upgrade guide](docs/upgrades.md)
before replacing it.

## Related projects

- Cross-agent handoffs and recovery of earlier file edits. [Déjà Vu](https://github.com/vshulcz/deja-vu)
- Search across dozens of agents from a terminal interface. [CASS](https://github.com/Dicklesworthstone/coding_agent_session_search)
- Automatic capture and AI summaries for future sessions. [Claude-Mem](https://github.com/thedotmack/claude-mem)

## Licence

Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option.

[Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
