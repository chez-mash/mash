# Troubleshooting

| What you see | Where to start |
| --- | --- |
| The terminal cannot find `mash`, or PowerShell says the term is unrecognized | [Find the installed command](#mash-command-not-found) |
| Setup says it could not reach the search service | [Start Qdrant](#qdrant-connection-failed) |
| The dashboard address does not open | [Check dashboard startup](#the-dashboard-doesnt-open) |
| Mash found no history, or nothing is indexed yet | [Check history setup](#nothing-is-indexed-yet) |
| A chat is missing, or a search finds nothing | [Find a missing chat](find-missing-chat.md) |
| A coding agent cannot see Mash | [Check the agent connection](#my-agent-cannot-find-mash) |
| An upgrade says the index schema is incompatible | [Recover or upgrade the index](#the-index-schema-is-older-or-newer-than-mash) |

Still stuck or concerned about a security problem? [Get help](#get-help).

## Check what Mash sees

If the symptom is unclear, run the read-only diagnosis.

```sh
mash doctor
```

Look in `findings` for entries with `"state": "problem"`. The `summary` says
what is wrong, and `repair`, when present, gives a command or instruction.
The command exits 1 if it finds a problem. Entries marked `warning` point out
things worth checking. `mash doctor` leaves your setup
and history as they were.

A `copies` warning means something on this computer runs a different copy of
Mash from the one you ran: the `mash` your terminal finds, another `mash`
behind it on your PATH (such as the Windows app's command behind a winget
copy), a background service, or an agent's connection. Two copies at different versions is a
common reason search stops after an update. Update the other copy with the
command shown, or, in the Mac app, choose **Use the App’s Mash Everywhere**
when it offers.

To inspect a particular part of your setup, use the matching status command.

```sh
mash status                  # index and search readiness
mash status --installation   # history locations and agent connections
mash status --shared         # the index shared with your other machines
```

`mash status --installation` works even when the index is not ready. For an
error, look up its `error.kind` or a phrase from its `error.message` below.

## `mash: command not found`

Use the full path the installer printed, or add its directory to `PATH` in
the terminal you already have open. These commands use the default release
installer location.

```sh
export PATH="$HOME/.local/bin:$PATH"
mash --version
```

In Windows PowerShell, run the equivalent commands in your current window.

```powershell
$env:Path = "$HOME\.local\bin;$env:Path"
mash --version
```

If you chose a different install directory, use it in place of `.local/bin`.
For `cargo install`, the usual directory is `.cargo/bin` under your home.
When `mash --version` prints a version, continue in that same terminal. To
make the change permanent, add the directory in your shell profile or to
your Windows user **Path** in Environment Variables. New terminals will use
that saved setting.

## The dashboard doesn’t open

Keep `mash dash` running and open the address it prints in a browser on the
same machine. If you already started Mash in another terminal, use the URL
from that terminal. If another program is using the address, ask Mash to
choose a free local port.

```sh
mash dash --listen 127.0.0.1:0
```

The `0` asks for a free port. Open the actual address Mash prints. If you need
a fixed port, use an available one such as `127.0.0.1:8791` instead.

If the page says the dashboard has not been built, install a release archive or
[include the dashboard in your source build](install-methods.md#include-the-web-dashboard).

## Qdrant connection failed

Run `mash doctor` and look for `search-service` to see what's blocking search.

If `mash status` shows `local_qdrant` as `rejected`, check the reported reason
before retrying setup or indexing. Mash may be connecting to the wrong port,
using the wrong API key, or reaching an older Qdrant. The HTTP and gRPC
addresses must reach the same Qdrant 1.20+ instance in the 1.x series. See
[custom endpoints and API keys](configuration.md#local-qdrant).

If you see `qdrant-unavailable`, `qdrant-install`, “Could not reach Qdrant
at …”, or `local_qdrant` is `unreachable`, check whether the service is running.

If setup failed on Linux or WSL with a `systemctl` error,
[check your user session and choose a way to start Qdrant](local-search.md#linux-without-a-systemd-user-session).

Start Qdrant the way you installed it.

- **`mash qdrant install`:** run it again. It restarts a stopped service and
  repairs missing files. `local_qdrant.installed` in `mash status` shows the
  service's state and where its output is, such as
  `journalctl --user --unit mash-qdrant.service` on Linux.
- **Docker:** start the existing container and read its logs.

  ```sh
  docker start mash-local-qdrant
  docker logs mash-local-qdrant
  ```

- **Your own Qdrant:** restart it from its original folder and read its
  terminal output.

If Qdrant was never set up, follow
[installation](installation.md#start-the-local-search-service). It is ready
when [the readiness check](http://127.0.0.1:6333/readyz) reports
“all shards are ready.”

`qdrant-failure` means Qdrant answered with an error. Read its output (above)
for storage, credential or schema errors.

You can still list chats, read messages and export a backup while Qdrant is
down. If you are setting up Mash, choose **Save configuration only** in the
wizard or run `mash init --no-index`. Then run `mash index` to save chats
locally. It stops with `qdrant-unavailable` and reports how many chats it
saved. Read them with `mash list-chats` and `mash view`. Once Qdrant is
running, run `mash index` again to make search current.

On native Windows, a Qdrant log mentioning `gridstore`, “path specified,” and
`os error 3` can mean its data path is too long. Use the short absolute paths in
the [native Windows setup](local-search.md#native-windows-without-docker).
For an existing installation, stop Qdrant before moving its data and keep a
backup. Do not delete storage as a first step. Changing only the configured
path starts a separate empty store; it does not move your data.

## Nothing is indexed yet

**You see:** `database-missing`.

| Message | Fix |
| --- | --- |
| “Mash is not set up on this machine” | `mash init` |
| “Mash is configured, but nothing is indexed yet” | `mash index` |

If you see “database does not exist,” use `mash init` for a first setup or
`mash index` if setup already saved your settings.
Both need Qdrant running to build search.

If setup says it found no coding-agent chats, save a conversation in a
supported agent on this machine and run `mash init` again. If you already have
chats, use `mash status --installation` to see the history folders Mash found.
Review the selected agents and folders in [setup](installation.md#connect-your-histories).
If the missing chats are older, check the [history range](indexing.md#add-older-history).

## My agent cannot find Mash

Check the agent you want to use. For Claude Code, run:

```sh
mash status --installation --agent claude-code
```

Use `--agent codex` for Codex. For Copilot CLI, see [checks for your Mash version](agents.md#confirm-the-connection).
Follow the report's `next_action` if it gives a repair. You can keep searching in the dashboard
while you [set up the agent connection](agents.md#connect-claude-code-codex-or-copilot).

Restart the agent after setup, a repair, or a binary or plugin upgrade; an open
chat keeps its old connection. To test a fresh connection:

```sh
mash status --installation --agent claude-code --probe
```

A successful probe checks a new MCP process, not a chat that is already open.
See [agent setup](agents.md).

## Search returns no results

Start by [checking whether the chat is listed](find-missing-chat.md). If it is,
follow the guide’s [query and filter checks](find-missing-chat.md#the-chat-is-listed-but-search-finds-nothing).

## Models are missing

**You see:** “semantic models are not installed” (`model`), or
“related-discussion needs semantic search, which this configuration turns off”
(`usage`).

Search by meaning is optional and needs downloaded models. Word and phrase
search works without them:

```sh
mash search 'auth_token' --intent exact-mention
```

To search by meaning, [install the models](indexing.md#install-the-models)
and [build search data](indexing.md#build-search-data). Model installation
leaves your current settings in place. If you already use Mash, the guide
shows how to enable indexing by meaning in those settings.

## Embedding failed during indexing

**You see:** `embedding_error` in the result of `mash index`, which exits with
the retry-required status.

The pass finished but could not compute embeddings, for example because the
models are not installed or a remote embedding worker did not answer. The new messages are searchable by word
(`exact-mention`) straight away. Searching by meaning (`related-discussion`)
waits until a later `mash index` succeeds; fix the reported cause, then run it
again.

## New chats are missing or search is behind

**You see:** recent conversations missing, or `search-not-ready`.

On the computer where the agent saved the chat, run:

```sh
mash index
```

When indexing finishes, choose **Check for updates** at the foot of the
dashboard's rail. If the button is missing, reload the search page.
For a chat from another computer, [check that it was published](synchronization.md#across-machines).

- “search is catching up with new history”: an index pass is running. Retry in
  a moment.
- “search is behind the stored history and nothing is refreshing it”: run
  `mash index`.

The agent plugin starts a catch-up index when a session begins, at most once
every ten minutes. For continuous updates, see
[keep history up to date](synchronization.md).

If the chat still doesn’t appear after refreshing, [check its original
machine, history range, and agent](find-missing-chat.md).

## A chat still shows what an older release read

Mash skips a chat whose files have not changed since it last read them. When
an upgrade changes how an agent's history is read, for example which folder a
chat is filed under, chats indexed before the upgrade keep the old result
until they change. To apply it to everything now:

```sh
mash index --reread
```

Chats that come out the same keep their search data, so this is quicker than a
first index. Add `--provider codex` to limit it to one agent.

## Search views are missing or stale

If the database is compatible and its stored history is ready, rebuild the
local search data from it:

```sh
mash index --rebuild-local-views --full
```

This does not read agent history files, fix incomplete source coverage, or
migrate schemas. For new or previously failed sources, run `mash index` and
check its final result.

## The index schema is older or newer than Mash

**You see:** `schema-incompatible`.

Stop and follow [upgrades and backups](upgrades.md). Keep the old executable
and data. Export with the release that wrote the index, and validate an import
with the new release at a separate path.

Do not delete the database, change schema numbers, or expect `index --full` to
migrate it. If an upgrade marker is present, recover or finish that upgrade
before creating a new index.

## Indexing is busy, incomplete, or shared-pending

- **Busy** (`index-busy`, “another Mash operation is updating this local
  index”): another writer is active. Let it finish, or stop your
  `mash index --watch` process, then retry. Do not delete lock files.
- **Incomplete:** read the reported source error, correct it, and run
  `mash index` again. History already stored is kept.
- **Shared-pending:** the local index may have succeeded. Check connectivity,
  the token configuration and `mash status --shared`, then run `mash index`
  again.

A search across machines can return local results with a `shared_error`. Use
`--local` for a local-only search, or repair the shared connection before
relying on results from all machines.

## Get help

[Open a bug report](https://github.com/mashhq/mash/issues/new?template=bug_report.yml)
for problems with installation, setup, or everyday use. Include your operating
system, how you tried to install Mash, the step that failed, and what you
expected to happen. If `mash --version` works, include its output. An installer
error is enough to start a report if Mash never installed.

GitHub issues are public. Share just the relevant error or a short excerpt from
[`mash doctor`](#check-what-mash-sees). Remove any private paths, addresses,
chat text, or credentials. Use made-up chat text if an example is needed.

If you think Mash exposed private history, bypassed sign-in, or shipped a
package with the wrong checksum, use [GitHub's private reporting form](https://github.com/mashhq/mash/security/advisories/new).
See [what to include in a security report](../SECURITY.md#report-a-vulnerability).
