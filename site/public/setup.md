# Set up Mash on this machine

You are a coding agent, and the person you are working with asked you to
install Mash. Mash is a local command-line tool and MCP server that indexes
their past chats with coding agents (Claude Code, Codex, GitHub Copilot,
Cursor, OpenCode, Antigravity) and lets them, and you, search those chats.
Mash is open source and self-hosted. Installation needs no Mash account.
The index and searches stay on this machine. If a connected agent looks up
history, the messages it receives enter that agent's context. The source is
at https://github.com/mashium/mash and the privacy guide is at
https://getmash.io/privacy/.

Work through the steps in order. Every `mash` command prints one JSON result
on stdout; read it to decide what to do next. Long commands also write
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

Tell the person what setup will do, and get a yes before you continue:

- Install the `mash` and `mash-server` programs for their user (no
  administrator rights).
- Download and run Qdrant, the local search service: about 30 MB, checksum
  verified, reachable only from this machine, started again at each login.
- Read the chat history files of every supported agent it finds on this
  machine, including the chat you are in now. Mash never changes those files.
- Register Mash with the Claude Code, Codex and GitHub Copilot command-line
  tools it finds, by
  running those tools' own commands. This edits the agents' settings files;
  the result lists each file and the command that undoes it.
- Open one matching message from an older chat to confirm search.
  That excerpt enters this agent's context. The person can choose to run the
  first search themselves in the local dashboard or terminal instead.

If they want to try Mash without connecting agents, add `--no-agents` to
`mash init --yes` in step 4. If they prefer a smaller history range or only
one agent's chats, direct them to the interactive review in `mash init` or
https://getmash.io/dashboard/#set-up-mash-from-the-dashboard.

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
this page's order and use `mash init --yes` in step 4.

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

## 4. Connect their histories

```sh
mash init --yes
```

This finds supported agents' histories in their usual places, indexes the last
90 days, and registers Mash with the Claude Code, Codex and GitHub Copilot
command-line tools it finds. It checks Qdrant first and changes nothing if Qdrant does not
answer.

Read the result:

- `agents` lists each agent that was connected, the settings `files` that were
  edited and the `undo` commands. Report these to the person. An empty list
  means no agent was connected; `next` says why.
- `index_error` means setup was saved but the first index failed. Fix the
  cause it names, then run `mash index`.
- `written: false` on a rerun means the existing configuration was kept. That
  is not a failure.

To index everything instead of the last 90 days, which can take much longer on
a large history:

```sh
mash index --all-history
```

## 5. Find and open an earlier message

```sh
mash status
```

If the person chose to run the first search themselves, stop after status.
Tell them how to use `mash dash` or `mash search`, and leave the results for
them to open locally.

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

- Pass several words as one shell argument, for example
  `mash search 'log rotation'`. Word search checks for every word in any
  order. To require that exact phrase, use `mash search '"log rotation"'`.
- Word search matches whole words, so `rotating` does not find `rotate`. If
  search by meaning was enabled and indexing finished, use it for that query.
  `mash search "how do we rotate logs" --intent related-discussion`.

If step 4 connected an agent, you can also probe its connection in a fresh
process. For example, run `mash status --installation --agent codex --probe`
for Codex. An open chat still needs a restart to load the connection.

## 6. Tell the person

Report, in a few lines:

- what was installed and where;
- which agents were connected, which files were edited, and the undo commands;
- how many chats and messages were indexed, and which earlier chat the search
  opened;
- that they must restart each connected agent, including you, before its Mash
  tools appear;
- one thing to try next, such as asking Codex to find a Claude Code decision
  and show the original message, or browsing the history with `mash dash`.

## If something fails

Each error is JSON with a `kind` and a message that names the fix.

| `kind` or symptom                         | What to do                                                                           |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| `mash: command not found`                 | Use the full path from step 1, or add the install directory to `PATH`.               |
| `usage` on a search                       | Put a query of several words in quotes.                                              |
| `qdrant-unavailable`                      | Run `mash qdrant install`, then repeat the command that failed.                      |
| `database-missing`                        | Nothing is indexed yet. Run `mash init --yes`, or `mash index` if setup already ran. |
| `model` or `search-not-ready` on a search | Search by words instead: add `--intent exact-mention`.                               |
| `index-busy`                              | Another Mash process is indexing. Wait, then retry.                                  |
| Anything else                             | Show the person the message and https://getmash.io/troubleshooting/. Do not guess.   |

Do not edit Mash's configuration file, delete its data directory or change an
agent's settings by hand to work around a failure. Ask the person first.
