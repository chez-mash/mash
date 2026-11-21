# Useful commands

Find a conversation, then read the message behind a result:

```sh
mash search 'a word you remember'
mash view MESSAGE_ID --context 3
```

Each result carries a `lookup.view.id`; pass it to `view` as `MESSAGE_ID`.
`list-chats` rows carry one too. `view` does not accept chat IDs.

Search, indexing, and status print JSON. To browse and read conversations,
open the [dashboard](dashboard.md) with `mash dash` or ask a
[connected agent](agents.md).

## Commands

| Command | What it does | Most useful options |
| --- | --- | --- |
| `mash qdrant install` | Downloads, verifies and starts the local search service. Rerun it to repair or upgrade; its data is kept. | |
| `mash init` | Sets up Mash for the histories it finds, optionally [connects agents](agents.md), and indexes. | `--yes` accepts the defaults without prompts. `--no-index` configures without the search service running. `--no-agents` leaves agent settings unchanged. `--force` replaces an existing configuration, kept as `config.toml.previous`. |
| `mash search 'QUERY'` | Searches indexed chats or messages. | `--intent exact-mention` or `--intent related-discussion`, `--unit messages`, `--days N`, `--provider PROVIDER`, `--limit N`. See [search filters](search.md). |
| `mash list-chats` | Lists chats, most recently active first. | `--title-contains 'part of the title'`, `--provider PROVIDER`, `--limit N`. |
| `mash view MESSAGE_ID` | Returns one message. | `--context N` adds up to N messages before and after. |
| `mash dash` | Serves the [dashboard](dashboard.md) at `http://127.0.0.1:8790`. | `--listen ADDRESS`. `--services-config PATH` enables [service management](services.md). |
| `mash index` | Reads new or changed history into the index. See [keep history up to date](synchronization.md). | `--watch` keeps refreshing until interrupted. `--all-history` or `--since-days DAYS` includes older history. `--provider PROVIDER` limits the run to one agent. `--reread` reads every chat again after an upgrade that reads an agent's history differently. `--full` recreates all derived search data. |
| `mash status` | Reports whether search is ready, including whether the local search service answers. | `--installation` checks agent connections and the search service even if the index is unavailable. `--shared` shows each machine's freshness in the shared index. |
| `mash doctor` | Examines this machine and names what is wrong, why, and the command that repairs it. Changes nothing; exits 1 when it finds a problem. See [troubleshooting](troubleshooting.md). | |
| `mash models install` | Downloads the optional models for [semantic search](indexing.md#enable-semantic-search), about 177 MB. | |
| `mash forget CHAT_ID` | [Removes one chat](privacy.md#remove-a-chat) from the index and does not import it again. | |
| `mash query --request JSON` | Runs one read-only query over your indexed chats and messages, for counts and listings that search does not answer. | `--request '{"target":"messages","filter":{"provider":"codex"},"limit":20}'` is the typed form. `--sql 'SELECT ...'` runs read-only SQL on this machine, with `--limit N`. `--shared` sends a typed request to the shared index. |
| `mash export DIRECTORY` | Writes a [portable backup](backup.md#make-a-portable-backup) of your indexed history and retained attachments to a new directory. Keep it private. | |
| `mash import DIRECTORY --into NEW_DATABASE` | Restores a backup into a new database. | `--keep-identity` is for [upgrades](upgrades.md) only. |
| `mash mcp` | Serves Mash's tools to an agent over stdio. Agents start it themselves; see [connect another MCP client](mcp.md). | `--lean` offers only the eight tools that find and read, for a small model. |
| `mash qdrant uninstall` | Stops and removes the search service that `mash qdrant install` set up. | `--delete-storage` also deletes its data, which `mash index` rebuilds. |
| `mash uninstall` | Lists everything Mash set up on this machine: services, agent connections, index, models and configuration. See [uninstall](installation.md#uninstall). | `--yes` removes what is listed; without it nothing changes. `--keep-history` keeps the index, its search data and the configuration. |

Every command also accepts `--config PATH`, `--db PATH` and `--models-dir PATH`
for files outside their default locations.

Interactive `mash init` gives a short setup summary in the terminal. For a
script, run `mash init --yes` to accept the detected defaults and get a JSON
report.

`mash uninstall` shows a readable preview in the terminal. Capture its
output in a file or pipe to get the structured JSON report.

For sharing between machines, see [multi-machine setup](shared-index.md).

## Read the built-in help

Help and `--version` print plain text. Run `mash --version-info` for release
and database schema versions in JSON.

```sh
mash --help
mash search --help
```
