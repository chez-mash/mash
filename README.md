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

**Find things you’ve already worked out with your coding agents.**

Mash searches the history Claude Code, Codex, GitHub Copilot, Cursor, OpenCode,
and Antigravity already saved on your machine. Find an earlier decision or
fix, then open the original conversation.

Mash is open source and self-hosted. The default setup reads and searches
your chats on your machine and needs no account. If you choose to share an
index across machines, you run the server. [How Mash handles your history](docs/privacy.md)

## Quick start

[Español](docs/es/quick-start.md)

[Try the browser demo](https://getmash.io/demo/) with sample chats, or set up
Mash to search your own history.

**Ask your agent.** Paste this into Claude Code, Codex, or another coding agent.

> Read https://getmash.io/setup.md and set up Mash on this machine.

**Set it up yourself.** The steps below search existing chats on one machine.
You can try word and phrase search before downloading any models.

### 1. Install Mash

macOS (Apple silicon) or Linux (x86-64 or ARM64), including WSL:

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

Windows x64, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

The scripts verify the release checksum and install to your user's
`.local/bin`. Follow the commands the installer prints if `mash` is not on
your `PATH` yet. These installs need no Rust, Docker or administrator rights.

### 2. Start the local search service

```bash
mash qdrant install
```

This downloads Qdrant (about 30 MB), checks its checksum, and starts it on
this machine. It runs under your user and starts again when you log in.
[Search service setup](docs/installation.md#start-the-local-search-service)
covers using your own Qdrant and removing the service.

### 3. Connect your histories

```bash
mash init
```

Review the local setup, including the histories and installed agents Mash
found, then choose **Set up and index**. The default covers the last 90 days.
Choose **Customize setup** to change what Mash reads or connects.
For a browser wizard, run `mash dash` instead and open
[the local dashboard setup](docs/dashboard.md#set-up-mash-from-the-dashboard).
See [setup options and results](docs/installation.md#connect-your-histories)
for unattended setup, custom folders, or help with the setup report.

## Try it

Pick a distinctive word from an older chat, such as an error or function name.
Replace `migration` with that word.

```bash
mash search migration
```

Copy the `lookup.view.id` from a matching result and open it with
`mash view MESSAGE_ID --context 3`. In the [dashboard](docs/dashboard.md),
search for the word and click the result to open the conversation.

Restart any agent you connected, then give it a real task. For example, in
Codex, ask “Use Mash to find what we decided about authentication in Claude
Code. Show me the message behind the answer.”
[Check an agent connection](docs/agents.md#confirm-the-connection) if it
cannot see Mash.

## More ways to use Mash

- [Search by meaning](docs/indexing.md#enable-semantic-search) with optional local models.
- [Pick up new chats](docs/synchronization.md) with `mash index` or opt in to continuous indexing.
- [Save decisions and their sources](docs/records.md) for later recall.
- [Search across your machines](docs/shared-index.md) with a server you host.
- [Export and back up your history](docs/upgrades.md) to keep it portable.

<details>
<summary>Install with a package manager</summary>

| Channel | Command |
| --- | --- |
| Homebrew | `brew install mashium/tap/mash` |
| apt, Debian and Ubuntu | [Add the Mash repository](docs/installation.md#package-manager-alternatives), then `sudo apt install mash-cli` |
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

For platforms without a prebuilt release, or to run an unreleased revision.
You need a stable Rust toolchain; the first build takes several minutes:

```bash
cargo +stable install --locked --git https://github.com/mashium/mash mash
```

This build leaves out the web dashboard’s pages.
[Install Mash](docs/installation.md#install-the-cli) shows how to include them.

## Documentation

The guides live at **[getmash.io](https://getmash.io)**: search and filters,
agent integrations, the dashboard, sharing across machines, configuration,
troubleshooting, and the CLI and MCP references.

Upgrading an existing installation? Read [the upgrade guide](docs/upgrades.md)
before replacing it.

## Licence

Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option.

[Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
