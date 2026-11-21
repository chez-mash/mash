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
and Antigravity already saved on your machine. [Check support for your agent](docs/supported-agents.md).

Browse those chats in the [browser dashboard](docs/dashboard.md) with `mash dash`,
[search from the terminal](docs/search.md), or [connect a coding agent](docs/agents.md)
so it can search your history for you.

Mash is open source and self-hosted. The default setup keeps indexing and
search on your machine and needs no account. [How Mash handles your history](docs/privacy.md)

[Try the demo without installing](https://getmash.io/demo/) · [Quick start](#quick-start) · [Check the release download](docs/installation.md#check-the-release-download)

[![Mash dashboard showing a sample conversation and the message behind a saved decision](assets/dashboard-demo.png)](https://getmash.io/demo/#/chat/sign-in-tokens?node=laptop)

## Why Mash

- **Find a chat from before you installed Mash.** Search for an error,
  filename, or phrase across agents, or search by meaning with optional local
  models. Open the match with its surrounding messages and tool output.
  [Find the invoice export chat](https://getmash.io/demo/#/search?q=invoice+exports+queue)
- **See why you made a decision.** The plugin can save the decision with a
  quote from the chat. Open the original message in the dashboard or ask
  your agent to look it up later. [Follow a saved decision to its source](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
- **Search across machines you run.** Find chats from your workstation on
  your laptop through your own Mash server. Each computer can still search
  its own history offline. [Multi-machine setup](docs/shared-index.md)

[Compare Mash with other coding-agent history tools](docs/compare.md).

## Quick start

[Español](docs/es/quick-start.md) · [Français](docs/fr/quick-start.md) · [简体中文](docs/zh-cn/quick-start.md)

[Try a sample search](https://getmash.io/demo/#/search?q=invoice+exports+queue)
and open a result in its conversation. Set up Mash below to search your own
chats in the browser, in the terminal, or through an agent you connect.

Before running a command, you can [inspect the installer and check the release download](docs/installation.md#check-the-release-download).
To start with one agent and a short history range, follow the [local trial guide](docs/privacy.md#try-a-small-local-setup).

**Ask your agent.** Paste this into Claude Code, Codex, or another coding agent.

> Read https://getmash.io/setup.md and set up Mash on this machine.

You can start with the dashboard and choose whether to connect agents.

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
For non-English chats, see [word search examples in other languages](docs/search-languages.md).
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

See [how each agent's history has been tested on Linux, macOS, and Windows](docs/supported-agents.md#tested-with).

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

See [how Mash, Déjà Vu, CASS, Claude-Mem and SpecStory fit different workflows](docs/compare.md).

## Licence

Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option.

[Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
