# Set up Mash on this machine

You are a coding agent, and the person you are working with asked you to
install Mash. Mash is a local command-line tool and MCP server that indexes
their past chats with coding agents (Claude Code, Codex, GitHub Copilot,
Cursor, OpenCode, Antigravity) and lets them, and you, search those chats.
Mash is open source and self-hosted. Installation needs no Mash account.
A fresh default setup keeps the index and searches on this machine. If a
connected agent looks up history, the messages it receives enter that agent's
context. See the [source](https://github.com/mashium/mash) and
[privacy guide](https://getmash.io/privacy/).

Work through the steps in order. Read the JSON results from the installation,
indexing, and status commands to decide what to do next. Long commands also write
progress events to stderr, one JSON object per line: those are not errors and
not the result. Stop and tell the person when a step says to
ask them, or when a command fails in a way this page does not cover. Full
guides: https://getmash.io/installation/ and
https://getmash.io/troubleshooting/.

## Review the setup with the person

If they want to see Mash first, send them to https://getmash.io/demo/. It uses
sample chats and works before they install anything. Point them to
https://getmash.io/installation/#check-the-release-download if they want to
inspect the installers or check a release archive themselves.

Explain what a new setup will do before asking the person to proceed:

- Install the `mash` and `mash-server` programs for their user (no
  administrator rights).
- Download and run Qdrant, the local search service: about 30 MB, checksum
  verified, reachable only from this machine, started again at each login.
  Programs on this machine can reach it without a login.
  On Windows it stores search data outside the user profile in a short folder
  on the system drive, usually `C:\mash-qdrant\storage`. The main history
  database stays in their profile. See https://getmash.io/privacy/#where-mash-stores-data.
- Index chats from the last 90 days across the supported histories it finds.
  That can include this chat. Mash keeps a local copy and reads the original
  files without changing their contents.
- Check that search is ready and show them how to open the dashboard.
  They can read their first result in their own browser.

Offer an agent connection as part of the same review. Ask whether they want
Mash connected to the supported agent CLIs it finds, or want to search from the
dashboard and terminal. Connecting uses those CLIs to edit their user settings
and lets connected agents retrieve chat excerpts into their context. If they
say only to proceed with setup, use the dashboard and terminal option.

If they only want one agent's history or a shorter range, send them to
https://getmash.io/privacy/#try-a-small-local-setup for the interactive choices.
For an existing configuration, explain its search destination and agent
connections during the review. If the person wants to change what is indexed or
where it goes, use the settings review in https://getmash.io/installation/#connect-your-histories.

Start with word and phrase search. If the person also asks for search by
meaning, explain the optional model download in step 3 and get their approval
for that download.

## 1. Install Mash

Skip this step if `mash --version` already prints a version.
If `mash` is missing from `PATH`, also check for an existing binary in the
user's `.local/bin` directory or in `MASH_INSTALL_DIR` if set
(`mash.exe` on Windows). If it runs, use its full path through the rest
of setup.

Check whether a release is published:

```sh
curl -fsSL -o /dev/null -w '%{http_code}\n' https://github.com/mashium/mash/releases/latest/download/SHA256SUMS
```

In PowerShell, call `curl.exe` and write `-o NUL` instead of `-o /dev/null`.

If it prints `200`, install the prebuilt release.

macOS (Apple silicon) or Linux (x86-64 or ARM64), including WSL:

```sh
curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
```

Windows x64, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/mashium/mash/main/install.ps1 | iex
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

If the check prints anything else, or the machine is an Intel Mac, build from
source. This needs a stable Rust toolchain and takes several minutes; if
`cargo` is missing, ask the person before installing Rust.

```sh
cargo +stable install --locked --git https://github.com/mashium/mash mash
```

Confirm:

```sh
mash --version
```

## 2. Start the local search service

```sh
mash qdrant install
```

Success is a JSON result with `"outcome"` of `installed`, `current`,
`repaired` or `upgraded`. Rerunning it is safe. If the person already runs
their own Qdrant 1.20 or later on `127.0.0.1:6333`, skip this step.
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

For dashboard and terminal use:

```sh
mash init --yes --no-agents
```

If they chose to connect their installed Claude Code, Codex or GitHub Copilot
CLIs as well, run `mash init --yes`. For specific agents, send them to the
[interactive setup guide](https://getmash.io/installation/#connect-your-histories).
On a fresh setup, both commands find saved chats in their usual locations and
index the last 90 days. Both check Qdrant before applying settings.

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

Read `ready` to see whether local search is usable. If `installation` lists
unconfigured agents and has `next_action: null`, their connections are optional.
If `ready` is `true`, give them the command `mash dash` and tell them to open
the address it prints, usually http://127.0.0.1:8790. In **chats**, they can
search for a word they remember and open the result in its conversation.
Use the full executable path if needed.

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
mash list-chats --provider claude-code --limit 10
mash search migration --provider claude-code --intent exact-mention
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
- how to start the dashboard with `mash dash`, or restart any connected
  agent to load its Mash tools;
- how to review removal with `mash uninstall`, which previews what it
  would remove and keeps the original agent chats. See
  https://getmash.io/installation/#uninstall.

## If something fails

Each error is JSON with a `kind` and a message that names the fix.

| `kind` or symptom                         | What to do                                                                         |
| ----------------------------------------- | ---------------------------------------------------------------------------------- |
| `mash: command not found`                 | Use the full path from step 1, or add the install directory to `PATH`.             |
| `usage` on a search                       | Put a query of several words in quotes.                                            |
| `qdrant-unavailable`                      | Run `mash qdrant install`, then repeat the command that failed.                    |
| `database-missing`                        | Use the setup command from step 4, or run `mash index` if setup succeeded.         |
| `model` or `search-not-ready` on a search | Search by words instead: add `--intent exact-mention`.                             |
| `index-busy`                              | Another Mash process is indexing. Wait, then retry.                                |
| Anything else                             | Show the person the message and https://getmash.io/troubleshooting/. Do not guess. |

Do not edit Mash's configuration file, delete its data directory or change an
agent's settings by hand to work around a failure. Ask the person first.
