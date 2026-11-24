# Set up Mash on this machine

You are a coding agent, and the person you are working with asked you to
install Mash. Mash is a local command-line tool and MCP server that indexes
their past chats with coding agents (Claude Code, Codex, GitHub Copilot,
Cursor, OpenCode, Antigravity) and lets them, and you, search those chats.
Mash is open source and self-hosted. The prebuilt releases include its browser
dashboard. Installation needs no Mash account.
A fresh default setup keeps the index and searches on this machine. If a
connected agent looks up history, the messages it receives enter that agent's
context. See the [source](https://github.com/grep-stack/mash) and
[privacy guide](https://getmash.io/privacy/).

Work through the steps in order. Read the JSON results from the installation,
indexing, and status commands to decide what to do next. Long commands also write
progress events to stderr, one JSON object per line: those are not errors and
not the result. Stop and tell the person when a step says to
ask them, or when a command fails in a way this page does not cover. Full
guides: https://getmash.io/installation/ and
https://getmash.io/troubleshooting/.

## Review the setup with the person

Identify the computer where your terminal runs. If it is remote, explain that
the loopback dashboard opens on that computer. For access from their own
browser, show the [private dashboard setup](https://getmash.io/dashboard/#open-it-from-your-phone-or-another-computer).

If they want to see Mash first, send them to https://getmash.io/demo/. It uses
sample chats and works before they install anything. Point them to
https://getmash.io/installation/#check-the-release-download if they want to
inspect the installers or check a release archive themselves.

Check whether Mash is already installed with `mash --version`. If the command
is missing from `PATH`, check `MASH_INSTALL_DIR` if set and the user's
`.local/bin` directory (`mash.exe` on Windows). Use the full path if needed.

For a new setup, explain these changes together before asking the person to proceed.

- Install the `mash` and `mash-server` programs for their user (no
  administrator rights).
- Download and run Qdrant, the local search service: about 30 MB, checksum
  verified, reachable only from this machine, started again at each login.
  Programs on this machine can reach it without a login.
  On Windows it stores search data outside the user profile in a short folder
  on the system drive, usually `C:\mash-qdrant\storage`. The main history
  database stays in their profile. See https://getmash.io/privacy/#where-mash-stores-data.
- Index history files or sessions updated in the last 90 days. These can
  include older messages as well as the current chat. Mash keeps a local copy
  and reads the original files without changing their contents.
- Check that search is ready and show them how to find their first chat in
  the dashboard or terminal.

Use one review for the history range, search destination and any agent
connections. Offer **Dashboard and terminal** as the default, and let them
choose which available agents to connect. Connecting uses those agents' CLIs
to edit their user settings and lets them retrieve excerpts into their
context. Use the choices the person has already given you.
If they already installed the Mash plugin, explain that enabled hooks can add
excerpts even without an agent connection. Point them to the
[plugin controls](https://getmash.io/plugin/#what-happens-without-asking).

If they only want one agent's history or a shorter range, send them to
https://getmash.io/privacy/#try-a-small-local-setup for the interactive choices.
For an existing install, run the read-only `mash status` first. If
`configuration.loaded` is `true`, summarize the current histories in
`configuration.providers`, the range and destination in `configuration.index`,
and the agent connections in `installation.clients`.
Describe any day limit in terms of when the files or sessions were last updated.

- With `configuration.index.destination` set to `both`, tell them that
  indexing also sends chats to `configuration.shared.api_url`. Proceed with
  indexing only if they choose to use that destination. If they want local
  search only, use the [settings review](https://getmash.io/installation/#connect-your-histories)
  to select **Just this computer (recommended)** before indexing.
- `--no-agents` skips new connections. If they want to use the dashboard
  without an agent that is already connected, show them how to
  [disconnect it](https://getmash.io/agents/#disconnect-an-agent).

Start with word and phrase search. If the person also asks for search by
meaning, explain the optional model download in step 3 and get their approval
for that download.

## 1. Install Mash

If `mash --version` worked in the review, continue to step 2.

For macOS on Apple silicon, Linux x86-64 and ARM64 (including WSL), or
Windows x64, check whether a release is published:

```sh
curl -fsSL -o /dev/null -w '%{http_code}\n' https://github.com/grep-stack/mash/releases/latest/download/SHA256SUMS
```

In PowerShell, call `curl.exe` and write `-o NUL` instead of `-o /dev/null`.

If it prints `200`, install the prebuilt release.

macOS (Apple silicon) or Linux (x86-64 or ARM64), including WSL:

```sh
curl -fsSL https://raw.githubusercontent.com/grep-stack/mash/main/install.sh | sh
```

Windows x64, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/grep-stack/mash/main/install.ps1 | iex
```

On Linux the prebuilt binary needs glibc 2.34 or newer (Ubuntu 22.04,
Debian 12, RHEL 9 or later). On an older or a musl system the installer stops
and says so: tell the person, and do not expect a source build to work there
without a newer toolchain.

Both install to the user's `.local/bin` directory and verify the release
checksum. If that directory is not on `PATH`, call Mash by its full path for
the rest of this page, and tell the person to add the directory to `PATH`:
`export PATH="$HOME/.local/bin:$PATH"` in their shell profile, or in
PowerShell
`[Environment]::SetEnvironmentVariable('Path', "$env:USERPROFILE\.local\bin;" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')`.
The installer prints the next commands with the path to the binary. Follow
this page's order and use the command for the person's choice in step 4.

If this check fails on a supported system, report the download error and link
to https://getmash.io/installation/. If there is no prebuilt release for the
system, build from source. To give the person a working browser dashboard,
follow https://getmash.io/install-methods/#include-the-web-dashboard. This
builds the web files first and then compiles Mash from the same checkout.
It needs Git, Node.js 24+ and a stable Rust toolchain. Ask before installing
any missing tools.

If the person only wants terminal search and agent tools, use the shorter
source command. It produces a build without the dashboard.

```sh
cargo +stable install --locked --git https://github.com/grep-stack/mash mash
```

Confirm:

```sh
mash --version
```

## 2. Start the local search service

Run `mash status` and check `local_qdrant.state`. If it is `reachable`, reuse
the running service. If it is `rejected`, use `local_qdrant.message` and the
[connection guide](https://getmash.io/troubleshooting/#qdrant-connection-failed)
to resolve the reported problem. If it is `unreachable` and the person already
runs their own Qdrant, start that service using their existing setup. The HTTP
and gRPC addresses must reach the same instance.

If the local setup needs Mash to install or start Qdrant, use the command
below. On Linux and WSL, first run
`systemctl --user show-environment >/dev/null` in the same terminal. If it
fails, explain the available options at
https://getmash.io/local-search/#linux-without-a-systemd-user-session and
agree on one with the person. Continue when Qdrant is running.

```sh
mash qdrant install
```

Success is a JSON result with `"outcome"` of `installed`, `current`,
`repaired`, `upgraded` or `already_running`. Rerunning it is safe.
`already_running` means Mash found a compatible service and didn't install one.
If Mash installed Qdrant, include the `service.storage` path when reporting
what was installed and where.

## 3. Optional search by meaning

Skip this for word and phrase search. If the person requested search by
meaning, this downloads local models (about 177 MB, more on Apple silicon and
Windows) from huggingface.co. Run it after they approve:

```sh
mash models install
```

Run it before step 4 when creating a new Mash configuration. A new
configuration includes the data for search by meaning when the models are
already installed. If Mash already has a configuration, follow
https://getmash.io/indexing/#build-search-data to enable it without replacing
the person's settings. Word search stays the default unless they change it.

## 4. Index their histories

To search on your own from the dashboard or terminal:

```sh
mash init --yes --no-agents
```

If they chose to connect their installed Claude Code, Codex or GitHub Copilot
CLIs as well, run `mash init --yes`. For specific agents, send them to the
[interactive setup guide](https://getmash.io/installation/#connect-your-histories).
If they chose Copilot CLI and installed v0.1.1-alpha or earlier, follow the
[manual Copilot connection](https://getmash.io/plugin/#github-copilot-cli).
On a fresh setup, both commands find saved chats in their usual locations and
use the default range described in the setup review. Both check Qdrant before
applying settings.

Read the result:

- `agents` lists connection attempts, the settings `files` involved and the
  `undo` commands. A `done`, `already-present`, or `replaced` MCP step means
  the connection is in place. Report these to the person. An empty `agents`
  list is fine for dashboard or terminal search.
- `index_error` means setup was saved but the first index failed. Fix the
  cause it names, then run `mash index`.
- `written: false` on a rerun means the existing configuration was kept. That
  is not a failure.

## 5. Check readiness and show how to search

```sh
mash status
```

Read `ready` to see whether local search is usable. If the person chose search
by meaning, look for `realized_mode` set to `hybrid`. If `installation` lists
unconfigured agents and has `next_action: null`, their connections are optional.
If `ready` is `true`, show them the first step for the build they have.

- With the dashboard, give them the command `mash dash` and tell them to
  open the address it prints, usually http://127.0.0.1:8790. In **chats**,
  they can search for a word they remember and open a result.
- With only the terminal and agent tools, show them
  `mash search migration --local --intent exact-mention`, using a word they
  remember in place of `migration`, and
  https://getmash.io/search/#your-first-search. If they want the dashboard too,
  show them https://getmash.io/install-methods/#include-the-web-dashboard.

For an existing install whose dashboard support is unclear, you can start
`mash dash --listen 127.0.0.1:0` in a temporary process, read the page at the
address it prints, and stop that process. A page that says the dashboard has
not been built means they have the terminal and agent tools build. Use the
full executable path in commands if needed.

If step 4 connected an agent, you can probe its connection in a fresh process.
For example, run `mash status --installation --agent codex --probe` for Codex.
An open chat still needs a restart to load the connection.

Continue to step 6 after these checks unless the person asked you to run their
first search for them. An agent connection gives them a way to ask for lookups
later.

### If the person asks you to run the first search

Search for a distinctive term from an older conversation, such as a function
name, error, or project term the person remembers. If you need a starting
point, list recent chats from an imported provider and open a result whose
title and date point to a conversation before this setup. Use that message
to choose a term. For example, with Claude Code history:

```sh
mash list-chats --local --provider claude-code --limit 10
mash search migration --local --provider claude-code --intent exact-mention
```

Use the actual provider and term from the earlier chat. Copy the matching
result's `lookup.view.id` into `mash view ID --context 3` and confirm that the
message comes from the earlier chat. The setup conversation can
appear in results too. An empty `results` list with no error means the search
found no match. Try a different word from the earlier chat.

If search says “Live indexing is inactive,” this is normal for manual
updates. Run `mash index` to pick up newer chats.

- Pass several words as one shell argument, for example
  `mash search 'log rotation'`. Word search checks for every word in any
  order. To require that exact phrase, use `mash search '"log rotation"'`.
- Word search matches whole words, so `rotating` does not find `rotate`. If
  search by meaning was enabled and indexing finished, use it for that query.
  `mash search "how do we rotate logs" --intent related-discussion`.

## 6. Tell the person

Report, in a few lines:

- what was installed and where;
- which agents were connected, which files were edited, and the undo commands;
- whether search is ready and how many chats and messages were indexed;
- which earlier message matched, if they asked you to run a search;
- how to start the dashboard or run the first terminal search for this build,
  and whether to restart any connected agent to load its Mash tools;
- how to review removal with `mash uninstall`, which previews what it
  would remove and keeps the original agent chats. See
  https://getmash.io/uninstall/.

## If something fails

Each error is JSON with a `kind` and a message that names the fix.

| `kind` or symptom                         | What to do                                                                         |
| ----------------------------------------- | ---------------------------------------------------------------------------------- |
| `mash: command not found`                 | Use the full path from step 1, or add the install directory to `PATH`.             |
| `usage` on a search                       | Put a query of several words in quotes.                                            |
| `qdrant-unavailable`                      | Follow the connection check in step 2 before retrying.                             |
| `database-missing`                        | Use the setup command from step 4, or run `mash index` if setup succeeded.         |
| `model` or `search-not-ready` on a search | Search by words instead: add `--intent exact-mention`.                             |
| `index-busy`                              | Another Mash process is indexing. Wait, then retry.                                |
| Anything else                             | Show the person the message and https://getmash.io/troubleshooting/. Do not guess. |

Do not edit Mash's configuration file, delete its data directory or change an
agent's settings by hand to work around a failure. Ask the person first.
