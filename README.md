<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/mash-logo-dark.svg">
    <img src="assets/mash-logo-light.svg" alt="Mash logo" width="280">
  </picture>
</p>

# Mash

Search the chats your coding agents already saved, find the message behind a
decision, and pick up work in another agent. Mash is open source and self-hosted.
By default, indexing and search stay on your machine and require no account.
[How Mash handles your history](docs/privacy.md)

[Try the demo without installing](https://getmash.io/demo/#/search?q=checkout) · [Quick start](#quick-start) · [Docs](https://getmash.io/)

[![Supported agents: Claude Code, Codex, GitHub Copilot, Cursor, OpenCode, Antigravity](assets/supported-agents.svg)](docs/supported-agents.md)

[![Illustration of Mash search results alongside the original conversation about a checkout outage](assets/dashboard-demo.png)](https://getmash.io/demo/#/chat/checkout-outage-review?node=build-box&message=checkout-outage-review-1&shared=true)

<span id="why-mash"></span>

## Highlights

- **Pick up work in another agent.** Ask an agent connected over MCP to find
  the decision you made, cite the original message, and tell you what’s still open.
  [Follow a Claude Code to Codex handoff](docs/pick-up-work.md#follow-a-sample-handoff)
- **Multi-agent search.** Search by word or exact phrase across chats that
  Claude Code, Codex, GitHub Copilot, Cursor, OpenCode, and Antigravity already
  saved. [Supported histories](docs/supported-agents.md)
- **Multi-machine search.** Find a chat from your workstation on your laptop
  through an optional Mash server you run. Each machine keeps a local index
  it can search offline. [Set up sharing](docs/shared-index.md)
- **Jump straight to the original message.** Read the surrounding conversation
  and tool output in the dashboard.
  [Open the checkout outage chat](https://getmash.io/demo/#/chat/checkout-outage-review?node=build-box&message=checkout-outage-review-1&shared=true)
- **Save decisions with a source.** Ask a connected agent to remember a
  decision, then follow its supporting quote back to the chat.
  [Open the saved sign-in decision](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
- **Export a whole conversation.** Save it from the dashboard as Markdown,
  HTML, or PDF. [Export a transcript](docs/dashboard.md#save-a-transcript)
- **Mashy, an optional dashboard assistant.** Bring your own model service and
  API key to ask questions about your history and get links to chats and
  saved decisions. [Configure Mashy](docs/configuration.md#the-dashboard-assistant)
- **GPU acceleration for semantic search.** Optional embeddings use DirectML
  on supported Windows GPUs, Core ML on Apple silicon, or the CPU when local
  acceleration is unavailable. Indexing can also use a configured remote worker.
  [Embedding inference](docs/configuration.md#embedding-inference)

## Quick start

### Install it yourself

Run the command for your system to download Mash and start the browser
setup. You’ll need at least one saved chat from a [supported agent](docs/supported-agents.md).
See [what setup changes](docs/installation.md#what-setup-changes) and
[how to verify the download](docs/downloads.md#verify-the-checksum).

**macOS (Apple silicon) or Linux (x86-64 or ARM64)**

Linux and WSL require a [systemd user session](docs/local-search.md#linux-without-a-systemd-user-session).

```bash
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh -s -- --dashboard
```

**Windows x64 (PowerShell)**

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1))) -Dashboard
```

Open the link the installer shows you and leave the terminal open. Follow the
steps in your browser, then choose **Set up and index**. You can skip connecting
agents or clear any preselected agents. [See the dashboard setup steps](docs/dashboard.md#set-up-mash-from-the-dashboard)

<span id="try-it"></span>

### Find your first chat

When setup says search is ready, open **chats**. Type a word you remember, such
as an error or function name, and press Enter. Open a result to see the original
message in its conversation.

To reopen the dashboard later, run `mash dash` and open the address it prints.
If your terminal can’t find `mash`, use the full path shown during setup or
[add Mash to your PATH](docs/troubleshooting.md#mash-command-not-found).
[Find and read a chat](docs/dashboard.md#find-and-read-a-chat) ·
[Search from the terminal](docs/search.md#your-first-search) ·
[Troubleshoot search](docs/troubleshooting.md#search-returns-no-results)

### Ask your agent

Paste this into Claude Code, Codex, or another coding agent.

> Read https://getmash.io/setup.md and set up Mash on this machine.
> Review the choices with me first.

If you already have the [Mash plugin](docs/plugin.md#start-with-the-plugin),
ask your agent to use its `setup` skill. If you installed Mash yourself, you
can [connect an agent later](docs/agents.md#connect-claude-code-codex-or-copilot).

<span id="build-from-source"></span>

For package managers, source builds, or terminal setup, see the
[installation guide](docs/installation.md). To try Mash with less history,
follow the [local trial guide](docs/privacy.md#try-a-small-local-setup).

<!-- quick-start-end -->

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
[Linux CI](https://github.com/mashium/mash/actions/workflows/ci.yml) · [Windows CI](https://github.com/mashium/mash/actions/workflows/windows.yml) · [Docs checks](https://github.com/mashium/mash/actions/workflows/docs.yml)

## Documentation and contributions

[Browse the guides](docs/README.md), [get help with a problem](docs/troubleshooting.md),
or [report a security concern privately](SECURITY.md#report-a-vulnerability).
[Architecture overview](docs/architecture.md) ·
[Download and verify a release archive](docs/downloads.md).
Before replacing an existing installation, read the [upgrade guide](docs/upgrades.md#upgrade-an-existing-installation).
To fix a guide, improve a translation, or send a code change, see [Contributing](CONTRIBUTING.md#where-to-start).

[Español](docs/es/index.md) · [Français](docs/fr/index.md) · [简体中文](docs/zh-cn/index.md) · [日本語](docs/ja/index.md)

## Licence

Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at your option.
