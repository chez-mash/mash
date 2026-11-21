# Install Mash

This guide sets up Mash on one machine; indexing and search stay local. Add
[multi-machine sharing](shared-index.md) later to search chats from another
computer.

Already using Mash? Read [upgrades and backups](upgrades.md) before replacing
a release that uses a different database schema.

For a new local setup in a terminal, use the [quick start](../README.md#quick-start).
Its setup option installs the verified release, runs `mash qdrant install`
to prepare the local search service, then opens `mash init` from the installed
path. You review the detected histories and agent connections before indexing.
Use the steps below to install the binaries alone, run your own search
service, or do the steps separately.

## Install the CLI

### Recommended release installers

These need no Rust. Prebuilt binaries cover
Linux x86-64 and ARM64, macOS Apple silicon, and Windows x64. Choose **one**
method.

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
Set `MASH_VERSION` to a published tag to pin a release, or
`MASH_INSTALL_DIR` to choose another
directory. They do not modify your shell profile or download models.

### Check the release download

Read the [macOS and Linux installer](https://github.com/mashium/mash/blob/main/install.sh)
or [Windows installer](https://github.com/mashium/mash/blob/main/install.ps1)
before you run it. The scripts install into your user directory and need no
administrator rights. The one-command setup also starts a local search service
under your user. Mash shows the histories and agent connections for review
before it indexes chats or edits agent settings.

The [public release workflow](https://github.com/mashium/mash/actions/workflows/release.yml)
builds the platform archives after checking that Linux and Windows CI passed on
the source commit. To install without running a downloaded script, get the
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

The release workflow now checks the attestations before publishing the platform
archives. Releases through `v0.1.1-alpha` use the checksum check above.

### Package manager alternatives

**Homebrew, macOS or Linux**

```sh
brew install mashium/tap/mash
```

**apt, Debian or Ubuntu**

```sh
curl -fsSL https://getmash.io/apt/mash-archive-keyring.gpg | sudo tee /usr/share/keyrings/mash-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/mash-archive-keyring.gpg] https://getmash.io/apt stable main" | sudo tee /etc/apt/sources.list.d/mash.list
sudo apt update && sudo apt install mash-cli
```

The package is `mash-cli` because Debian and Ubuntu already have an unrelated
package named `mash`; the command it installs is still `mash`. The two cannot
be installed together, and apt offers to remove the other one.

**Scoop, Windows**

```powershell
scoop bucket add mashium https://github.com/mashium/mash
scoop install mashium/mash
```

**mise**

```sh
mise use -g github:mashium/mash
```

**cargo-binstall**

```sh
cargo binstall --git https://github.com/mashium/mash mash
```

### Build from source

For platforms without a prebuilt release, or to run an unreleased revision,
build with a stable Rust toolchain. The first build takes several minutes:

```bash
cargo +stable install --locked --git https://github.com/mashium/mash mash
mash --version
```

Commands print JSON. `--help` and `--version` print plain text.

If your shell cannot find `mash`, add Cargo's binary directory to `PATH`:
`export PATH="$HOME/.cargo/bin:$PATH"` on macOS/Linux, or
`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` in PowerShell. Add it
to your shell profile or Windows environment settings to keep it.

This installs the CLI and MCP server without the web dashboard's pages:
`mash dash` says the dashboard has not been built. To include it, install [Node.js](https://nodejs.org/) and [pnpm](https://pnpm.io/), then build
from a checkout:

```bash
git clone https://github.com/mashium/mash
cd mash/dashboard
pnpm install --frozen-lockfile
pnpm build
cd ..
cargo +stable install --locked --path .
```

## Start the local search service

```sh
mash qdrant install
```

Indexing and every search use a local [Qdrant](https://qdrant.tech) service.
This command downloads Qdrant 1.20.0 for your platform (about 30 MB) from
[Qdrant's GitHub releases](https://github.com/qdrant/qdrant/releases/tag/v1.20.0),
checks it against a SHA-256 checksum built into Mash, and starts it in the
background as your user. It needs no administrator rights or Docker, listens
only on this machine (`127.0.0.1`, ports 6333 and 6334), and starts again when
you log in. The JSON result has `"outcome": "installed"`.

| Platform | Runs as | Qdrant's output | Data |
| --- | --- | --- | --- |
| Linux | systemd user service `mash-qdrant.service` | `journalctl --user --unit mash-qdrant.service` | `~/.mash/qdrant` |
| macOS | LaunchAgent `dev.mash.qdrant` | `~/.mash/qdrant/qdrant.log` | `~/.mash/qdrant` |
| Windows | logon task `MashQdrant` | `%USERPROFILE%\.mash\qdrant\qdrant.log` | `C:\mash-qdrant`, because Qdrant fails with longer data paths there |

`mash status` shows the installed version and whether the service is running,
under `local_qdrant.installed`.

On Windows the logon task runs only while you are signed in at the desktop.
On a machine you reach only over SSH, or a server nobody signs in to, the
command reports that Qdrant did not become ready; start Qdrant yourself there,
as [native Windows without Docker](#native-windows-without-docker) describes.

Rerun the command at any time. It restarts a stopped service, restores missing
files, and upgrades Qdrant when a newer Mash pins a newer release, without
changing Qdrant's data. The outcome is `current` when nothing needs
doing, and `already_running` (nothing installed) when a compatible Qdrant
already answers on those ports, such as the Docker container below. If another
program holds a port, the command stops and says which.

- **Linux:** the command needs a systemd user session. Where there is none, as
  in some containers and WSL setups, [run Qdrant yourself](#run-qdrant-yourself).
  The service runs while you are logged in; to keep it running on a machine
  nobody logs in to, enable lingering with `loginctl enable-linger` (some
  systems allow this only for administrators).
- **Other ports:** set `url` and `grpc_url` in the
  [Qdrant configuration](configuration.md#local-qdrant) to `127.0.0.1`
  addresses before installing.

`mash init` checks that Qdrant answers, but never downloads or starts it.

### Run Qdrant yourself

Use the [official Docker image](#optional-qdrant-docker-image), start Qdrant
by hand as below, or use an existing Qdrant 1.20+ service. Both the HTTP and
gRPC ports must reach the same instance. For nondefault endpoints, see
[Qdrant configuration](configuration.md#local-qdrant). Mash and the dashboard
still run directly on your machine.

Each Qdrant has its own storage. When you switch between a native Qdrant, the
Docker container and `mash qdrant install`, stop the old one first to free
ports 6333 and 6334, then populate the new one with
`mash index --rebuild-local-views --full`.

### Native Windows without Docker

On Windows x64, download [Qdrant 1.20.0 for Windows](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-pc-windows-msvc.zip)
and extract it into `C:\mash-qdrant` (or another short, writable folder).
Longer paths under your user profile can make Qdrant's payload indexes fail
even when its startup health check succeeds. In PowerShell, run:

```powershell
Set-Location 'C:\mash-qdrant'
$env:QDRANT__SERVICE__HOST = '127.0.0.1'
$env:QDRANT__STORAGE__STORAGE_PATH = 'C:\mash-qdrant\storage'
$env:QDRANT__STORAGE__SNAPSHOTS_PATH = 'C:\mash-qdrant\snapshots'
.\qdrant.exe
```

Keep this window open and use the same settings next time to reuse the data.
In a second terminal, check readiness with
`curl.exe --fail http://127.0.0.1:6333/readyz` before running `mash init`.

### Native macOS or Linux without Docker

Download the archive for your machine from the
[Qdrant 1.20.0 release](https://github.com/qdrant/qdrant/releases/tag/v1.20.0):

| Machine | Archive |
| --- | --- |
| macOS, Apple Silicon | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-apple-darwin.tar.gz) |
| macOS, Intel | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-apple-darwin.tar.gz) |
| Linux, x86_64 | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-x86_64-unknown-linux-musl.tar.gz) |
| Linux, ARM64 | [Download](https://github.com/qdrant/qdrant/releases/download/v1.20.0/qdrant-aarch64-unknown-linux-musl.tar.gz) |

Extract it into a dedicated folder, such as `~/qdrant`, and start it there:

```sh
cd "$HOME/qdrant"
QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant
```

Keep that terminal open and start Qdrant from the same folder next time to
reuse its data. In a second terminal, check readiness with
`curl --fail http://127.0.0.1:6333/readyz` before running `mash init`.

### Optional Qdrant Docker image

Install and start [Docker Desktop](https://www.docker.com/products/docker-desktop/)
on macOS/Windows, or [Docker Engine](https://docs.docker.com/engine/install/)
on Linux. Windows needs Linux containers; Windows Server/Engine-only hosts may
not support them, so use the native Windows instructions above there. Then run
this in Bash or PowerShell:

```sh
docker run -d --name mash-local-qdrant --restart unless-stopped --memory 2g -p 127.0.0.1:6333:6333 -p 127.0.0.1:6334:6334 --mount type=volume,source=mash-qdrant-data,target=/qdrant/storage qdrant/qdrant:v1.20.0
```

Only this machine can reach it. Open
[the readiness check](http://127.0.0.1:6333/readyz) and wait for “all shards
are ready” before running `mash init`.

```sh
docker logs mash-local-qdrant
docker stop mash-local-qdrant
docker start mash-local-qdrant
```

Qdrant restarts with Docker unless you explicitly stopped it. If the container
already exists, use `docker start` rather than `docker run`. Data lives in the
`mash-qdrant-data` volume and survives stopping; delete the volume only to
discard it.

## Connect your histories

```sh
mash init
```

Word search needs no models, and setup never downloads any. For search by
meaning, run `mash models install` before `mash init`; see
[model setup](indexing.md) for download details and enabling it later.

On a fresh setup, the terminal wizard shows the history folders it found,
the history range (files updated in the last 90 days), and the agents it can
connect. Choose **Set up and index** to keep the index on this machine and
apply those choices. Choose **Set up and index without connecting agents**
to use the same histories and search them yourself.

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

Restart your agents after setup. The history plugin's hooks need Python 3.9+;
see [agent setup](agents.md).

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
| `--no-agents` | CLI-only use; connects no agents. |
| `--force` | Non-interactive setup keeps existing configuration unless this is passed. With `--yes`, regenerates configuration from detected histories. In the wizard, opens the settings review directly. |

Without `--force`, the wizard offers to keep existing settings or review them.
Reviewing preserves settings outside the wizard and backs up the previous file.

You can read your history before Qdrant runs: run `mash init --no-index`, then
`mash index`. Indexing saves your chats on this machine before it needs Qdrant,
then stops with the commands to start it and says how many chats it saved.
`mash list-chats` and `mash view` read them meanwhile. Once Qdrant runs, run
`mash index` again to make search current.

### Read the result

The terminal wizard ends with a short report and a next step. For the full
JSON report, run `mash init --yes` or redirect the wizard's stdout with
`mash init > setup.json`. Prompts and progress go to stderr.

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

## Try your first search

```sh
mash status
mash search 'migration'
```

Replace the query with a distinctive word from an older chat. If you need a
starting point, run `mash list-chats --limit 10` and pick a conversation you
recognize. Take the `lookup.view.id` from a search hit or chat and run
`mash view MESSAGE_ID --context 3` to read the original message. In the
dashboard, open the result directly.

Word search matches whole words in any order. Pass several words as one
argument, such as `mash search 'rotate logs'`. To require an exact phrase,
include double quotes inside the argument, such as
`mash search '"log rotation"'`.

To include older history, run `mash index --all-history`. Next:
[searching](search.md), [the dashboard](dashboard.md) (source installations
build the web interface first), [agent-assisted recall](agents.md), or
[semantic search](indexing.md#enable-semantic-search).

## Uninstall

```sh
mash uninstall          # lists what would be removed and changes nothing
mash uninstall --yes    # removes it
```

The first command prints the plan: each service, agent connection, file and
directory, with its size. With `--yes`, Mash:

1. stops and removes the services it set up: the search service and its data,
   the [remote dashboard](dashboard.md), and an installed index watcher;
2. disconnects Claude Code, Codex and GitHub Copilot through their own commands, the ones
   under [disconnect an agent](agents.md#disconnect-an-agent);
3. deletes its index (Mash's copy of your chats), models, configuration and
   saved tokens.

Then remove the program itself with whatever installed it. The last line of
the output names the command: `brew uninstall mash`, `scoop uninstall mash`,
`sudo apt remove mash-cli`, `cargo uninstall mash`, or the files to delete after
the install script.

Mash never touches your agents' own chat histories, and history already
published to a [shared index](shared-index.md) stays on that server. Files in
`~/.mash` that these commands did not create are listed and left in place.
If you run your own Qdrant, the service and any Mash search data it holds
stay in place. Manage that data separately.

Use `--keep-history` to keep Mash's searchable copy of your chats, its
configuration and saved tokens for a future reinstall. Mash still removes
its services, agent connections and downloaded models. If you want a separate
backup before uninstalling, run [`mash export DIRECTORY`](upgrades.md).

Preview with `mash uninstall --keep-history`, then confirm with
`mash uninstall --yes --keep-history`.

A step that fails is marked `failed` with the reason and the command exits 1.
Rerun the command with the same flags to retry what is left. Restart your
agents afterwards so they stop loading Mash.

To remove only the search service, run `mash qdrant uninstall`. It keeps
Qdrant's data unless you add `--delete-storage`; that data holds only Mash's
search data, which `mash index --rebuild-local-views --full` rebuilds from
your index.
