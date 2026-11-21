# Install Mash

Follow these steps to install Mash, start its search service, and choose
which chats to search. The default setup keeps indexing and search on your
machine. [How Mash handles your history](privacy.md)

To try it first, [search sample chats in the demo](https://getmash.io/demo/).
For a trial with your own chats, [choose one history and try the local dashboard](privacy.md#try-a-small-local-setup).

For one command that installs Mash, starts the search service and opens the
setup review, use the [quick start](../README.md#quick-start).
If Mash is already installed, [start the search service](#start-the-local-search-service)
or [connect your histories](#connect-your-histories) if the service is running.

Already using Mash and moving to a new release? Follow the [upgrade steps](upgrades.md#upgrade-an-existing-installation).

## What setup changes

1. **Install the programs.** The [basic installer](#recommended-release-installers)
   puts `mash` and `mash-server` in your user directory. You can also
   [verify and unpack a release archive](#check-the-release-download).
2. **Start local search.** `mash qdrant install` downloads Qdrant and sets it
   to run under your account at login. Mash limits it to connections from this
   computer and switches off Qdrant telemetry. [Search service files](local-search.md#where-qdrant-stores-data)
3. **Choose what Mash can read.** The interactive `mash init` wizard shows the
   history folders it found before saving settings or indexing your chats.
   Mash reads the original chats in place and saves a searchable copy on this
   computer by default. If you choose to connect an agent, setup updates its
   settings so it can request excerpts from that copy. You can also
   [skip agent connections](#skip-agent-connections) and search yourself.
   [Where Mash stores your data](privacy.md#where-mash-stores-data)

The quick-start command installs the programs and starts Qdrant before it
opens the review. If you cancel there, `mash uninstall` shows what is already
installed and how to remove it. [Preview removal](uninstall.md)

<span id="install-the-cli"></span>

## Get Mash for your system

The release includes the browser dashboard. Once the search service is running,
you can [set up Mash in your browser](#set-up-in-your-browser). You can also
[connect coding agents](agents.md) so they can search your chats for you.

### Recommended release installers

Prebuilt releases cover Linux x86-64 and ARM64, macOS Apple silicon, and
Windows x64. Choose one command for your system. You can
[inspect the installer or verify a download](#check-the-release-download) first.

The Linux binaries need glibc 2.34 or newer, which Ubuntu 22.04, Debian 12,
RHEL 9 and later releases provide. Alpine and other musl systems are not
supported.

#### macOS or Linux

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

WSL uses this command too.

#### Windows x64

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
```

Both scripts verify the release checksum and install to your user's
`.local/bin` directory. They print the next commands with the installed
binary's path so you can run them before adding the directory to `PATH`.
Set `MASH_VERSION` to a published tag to pin a release, or `MASH_INSTALL_DIR`
to choose another directory. The scripts leave your shell profile as it is.
If your terminal cannot find `mash`, see [command not found](troubleshooting.md#mash-command-not-found).

### Package manager alternatives

Use [Homebrew, apt, Scoop, mise or cargo-binstall](install-methods.md#install-with-a-package-manager).
After installing Mash, [start the search service](#start-the-local-search-service).

### Build from source

To search in a browser, [build from source with the dashboard](install-methods.md#include-the-web-dashboard).
For [terminal search and agent tools](install-methods.md#cli-and-agent-tools),
use the shorter Cargo command.

## Start the local search service

On Linux or WSL, the command below needs a systemd user session.
[Check whether your terminal has one and see the alternatives](local-search.md#linux-without-a-systemd-user-session).
If you already run a [compatible local Qdrant](local-search.md#run-qdrant-yourself),
continue to [connect your histories](#connect-your-histories).

On Windows, Mash puts Qdrant's search data in a short folder on the system
drive, usually `C:\mash-qdrant\storage`.
[See Qdrant's files and logs](local-search.md#where-qdrant-stores-data).

```sh
mash qdrant install
```

Mash verifies the Qdrant download (about 30 MB) and starts the service in the
background under your user. Qdrant listens on this machine and starts again
when you log in. On Windows, it requires a desktop sign-in.
If the service does not start, see [how to check the logs and restart it](local-search.md#check-or-restart-the-service).

### Run Qdrant yourself

You can [reuse a local Qdrant 1.20+ instance](local-search.md#run-qdrant-yourself).
If you need to start one, choose the instructions for your environment.

- <span id="optional-qdrant-docker-image"></span>[Qdrant in Docker](local-search.md#optional-qdrant-docker-image)
- <span id="native-windows-without-docker"></span>[Windows without Docker](local-search.md#native-windows-without-docker)
- <span id="native-macos-or-linux-without-docker"></span>[macOS or Linux without Docker](local-search.md#native-macos-or-linux-without-docker)

## Connect your histories

Once the local search service is running, use the browser dashboard or
the terminal wizard to choose which chats Mash reads.

### Set up in your browser

If you installed a release or [built with the dashboard](install-methods.md#include-the-web-dashboard),
run this in a terminal:

```sh
mash dash
```

Keep the terminal open and visit the address it prints. On a fresh install,
the dashboard opens to setup. Check the histories and date range. Leave
agent connections off if you want to search on your own. Choose
**Set up and index**. Once the index is ready, open **chats** and search for
a word you remember.
[See the dashboard setup steps](dashboard.md#set-up-mash-from-the-dashboard)

### Set up in a terminal

Use the terminal wizard for a source build without the dashboard or to
choose a history folder or connect to a shared server.

```sh
mash init
```

On a fresh setup, the terminal wizard shows the history folders it found,
the history range (files updated in the last 90 days), and the agents it can
connect. If asked where to search, choose **Just this computer (recommended)**
to keep the index on this machine. **Set up and index** applies the options in
the review, including any selected agent connections. Mash can read saved
chats even if the agent's command is missing.

Choose **Customize setup** to use a different date range or history folder,
pick specific integrations, connect an existing shared server, or save
settings without indexing. This path also offers an optional
[memory backfill](records.md#backfill-earlier-memory) after indexing. Backfill
uses your signed-in Claude Code or Codex and sends selected chat text to its
model service.

Esc or Ctrl-C cancels before configuration or agent settings change. When
indexing is selected, the wizard checks the local search service's HTTP
endpoint and, for sharing, server authentication. If a connection fails, you
can save settings and finish setup afterward.

If you connected an agent, restart it to load Mash. The history plugin's
hooks need Python 3.9+. [Use Mash with your agent](agents.md)

### Skip agent connections

You can search your chats in the dashboard or terminal without giving a
coding agent access to your Mash search. The wizard offers one of these paths.

- If the first review offers **Set up and index without connecting agents**,
  choose it to index the histories shown.
- If the wizard asks **Which agents should get access to Mash search?**, use
  Space to clear every selection, then press Enter. If asked about memory
  backfill, decline it.
- If the review lists no agents to connect, choose **Set up and index**.

Once search is ready, [open the dashboard](#try-your-first-search). You can
[connect an agent later](agents.md#connect-claude-code-codex-or-copilot).

## Try your first search

If your installation includes the dashboard, run `mash dash` and open the
address it prints. In **chats**, search for a distinctive word from an older
chat and open a result to read the original message.

[Search from the terminal](search.md#your-first-search) or
[ask a connected agent](agents.md#try-it) to find a chat without opening
the dashboard. If an older chat is missing, see how to
[include older history](indexing.md#add-older-history).

Word search needs no models. To add search by meaning, follow the
[optional model setup](indexing.md#enable-semantic-search).

## Setup options

### Without the wizard

`mash init --yes` accepts the detected defaults: sources modified within the
last 90 days, including current sessions of all detected supported agents
(Copilot too), and available agent integrations, without historical memory
backfill. Runs without terminal input or terminal stderr are non-interactive
too. The JSON result lists the selected provider folders.

Without the wizard, `mash init` checks Qdrant before changing anything. If
Qdrant does not answer, it exits with the commands to start it and leaves
configuration and agent settings untouched.

| Flag | Effect |
| --- | --- |
| `--no-index` | Configure without indexing. Skips the Qdrant check and lists starting Qdrant among the next steps. To choose folders yourself, edit the [configuration](configuration.md), then run `mash index`. |
| `--no-agents` | Search from the dashboard or terminal without connecting agents. |
| `--force` | Non-interactive setup keeps existing configuration unless this is passed. With `--yes`, regenerates configuration from detected histories. In the wizard, opens the settings review directly. |

Without `--force`, the wizard offers to keep existing settings or review them.
Reviewing preserves settings outside the wizard and backs up the previous file.

You can read your history before Qdrant runs: run `mash init --no-index`, then
`mash index`. Indexing saves your chats on this machine before it needs Qdrant,
then stops with the commands to start it and says how many chats it saved.
`mash list-chats` and `mash view` read them meanwhile. Once Qdrant runs, run
`mash index` again to make search current.

### Read the result

The terminal wizard ends with a short report and a next step. Once search is
ready, look for the command to start the dashboard or a terminal search,
depending on your build. For the full JSON report, run `mash init --yes` or
redirect the wizard's stdout with `mash init > setup.json`. Prompts and progress
go to stderr.

- `"agents": []` means no agents were connected, even if indexing succeeded.
  Read `next` for the reason and follow [agent setup](agents.md). CLI search
  works without an agent.
- Each connected agent lists the settings `files` its CLI edited and the `undo`
  commands that remove the connection.
- `index_error` means the first index failed after setup was applied (for
  example, Qdrant stopped or its gRPC port is unreachable). The command exits
  with that error, but the configuration, agent registrations and history
  already saved are kept and still listed. Fix the cause, then run `mash index`.
- `written: false` on a rerun means the existing configuration was reused, not
  that setup failed.
- The searchable message count covers user and assistant dialogue only. Tool
  output and reasoning can be read in a chat but are excluded from search, so
  the count can be much smaller.

## Check the release download

Read the [macOS and Linux installer](https://github.com/mashium/mash/blob/main/install.sh)
or [Windows installer](https://github.com/mashium/mash/blob/main/install.ps1)
before you run it. The scripts install into your user directory and need no
administrator rights. [See what setup changes](#what-setup-changes).

The [public release workflow](https://github.com/mashium/mash/actions/workflows/release.yml)
builds the platform archives after checking that Linux and Windows CI passed on
the source commit. See which installers passed for your version in the
[public install checks](https://github.com/mashium/mash/actions/workflows/release-check.yml).

To install without running a downloaded script, get the
archive for your platform and `SHA256SUMS` from the same
[versioned GitHub release](https://github.com/mashium/mash/releases). Compare
the archive's SHA-256 hash to the line for its exact filename before extracting
it. For example, on Linux x86-64:

```sh
sha256sum mash-x86_64-unknown-linux-gnu.tar.gz
```

On macOS, use `shasum -a 256` with the macOS archive. On Windows, use
`Get-FileHash -Algorithm SHA256` with the Windows ZIP in PowerShell. The
checksum confirms the file matches the published hash. If it matches, extract
`mash` and `mash-server` (or the two `.exe` files) to a directory you own, then
continue with [the search service](#start-the-local-search-service).

For releases that include a build attestation, GitHub CLI can also verify
which workflow and source tag produced an archive. Replace `ARCHIVE` with the
file you downloaded and `RELEASE_TAG` with its tag, including the leading `v`.

```sh
gh attestation verify ARCHIVE --repo mashium/mash --signer-workflow mashium/mash/.github/workflows/release.yml --source-ref refs/tags/RELEASE_TAG --deny-self-hosted-runners
```

The release workflow checks the attestations before publishing the platform
archives. Releases through `v0.1.1-alpha` use the checksum check above.

## Uninstall

Preview what Mash would remove with `mash uninstall`. [See what stays on your computer and how to uninstall](uninstall.md).
