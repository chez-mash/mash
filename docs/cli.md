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
| `mash search 'QUERY'` | Searches indexed chats or messages. | `--intent exact-mention` or `--intent related-discussion`, `--unit messages`, `--days N`, `--provider PROVIDER`, `--limit N`. Subagent chats are left out unless you add `--include-subagents`. See [search filters](search.md). |
| `mash list-chats` | Lists chats, most recently active first. | `--title-contains 'part of the title'`, `--provider PROVIDER`, `--limit N`. `--exclude-subagents` leaves out chats agents started for their own subtasks. |
| `mash records` | Lists the decisions, preferences and facts Mash remembers, newest first. | `--query 'text'`, `--kind decision`, `--include-inactive`, `--limit N`. |
| `mash view MESSAGE_ID` | Returns one message. | `--context N` adds up to N messages before and after. |
| `mash dash` | Serves the [dashboard](dashboard.md) at `http://127.0.0.1:8790`. | `--listen ADDRESS`. `--services-config PATH` enables [service management](services.md). |
| `mash index` | Reads new or changed history into the index. See [keep history up to date](synchronization.md). | `--watch` keeps refreshing until interrupted. `--all-history` or `--since-days DAYS` includes older history. `--provider PROVIDER` limits the run to one agent. `--reread` reads every chat again after an upgrade that reads an agent's history differently. `--rebuild-local-views --full` recreates local search data from the chats Mash already stored. |
| `mash status` | Reports whether search is ready, including whether the local search service answers. | `--installation` checks agent connections and the search service even if the index is unavailable. `--shared` shows each machine's freshness in the shared index. |
| `mash doctor` | Examines this machine and names what is wrong, why, and the command that repairs it. Changes nothing; exits 1 when it finds a problem. See [troubleshooting](troubleshooting.md). | |
| `mash models install` | Downloads the optional models for [semantic search](indexing.md#enable-semantic-search), about 177 MB. | |
| `mash forget CHAT_ID` | [Removes one chat](privacy.md#remove-a-chat) from the index and does not import it again. | |
| `mash query --request JSON` | Runs one read-only query over your indexed chats and messages, for counts and listings that search does not answer. | `--request '{"target":"messages","filter":{"provider":"codex"},"limit":20}'` is the typed form. `--sql 'SELECT ...'` runs read-only SQL on this machine, with `--limit N`. `--shared` sends a typed request to the shared index. |
| `mash export DIRECTORY` | Writes a [portable backup](backup.md#make-a-portable-backup) of your indexed history and retained attachments to a new directory. Keep it private. | |
| `mash import DIRECTORY --into NEW_DATABASE` | Restores a backup into a new database. | `--keep-identity` is for [upgrades](upgrades.md) only. |
| `mash mcp` | Serves Mash's tools to an agent over stdio. Agents start it themselves; see [connect another MCP client](mcp.md). | `--lean` offers only the eight tools that find and read, for a small model. |
| `mash qdrant uninstall` | Stops and removes the search service that `mash qdrant install` set up. | `--delete-storage` also deletes its data, which `mash index` rebuilds. |
| `mash qdrant prune` | Lists Mash's search data in Qdrant that no index uses. See [remove search data no index uses](local-search.md#remove-search-data-no-index-uses). | Use `--yes` to remove it. `--index PATH` marks another index's data as in use; `--include-unknown GROUP` also removes data from older Mash. |
| `mash uninstall` | Previews removal of Mash services, connections, and local data. See [uninstall](uninstall.md). | Use `--yes` to remove what is listed. Use `--keep-history` to leave Mash's copy of your chats for a reinstall. |
| `mash usage off` | Stops recording calls in the [usage log](#usage-log). What is recorded stays until cleared. | `mash usage on` records again. `mash usage clear` deletes every recorded call; it asks first, or pass `--yes`. |

Every command also accepts `--config PATH`, `--db PATH` and `--models-dir PATH`
for files outside their default locations.

Interactive `mash init` gives a short setup summary in the terminal. For a
script, run `mash init --yes` to accept the detected defaults and get a JSON
report.

`mash uninstall` shows a readable preview in the terminal. Capture its
output in a file or pipe to get the structured JSON report.

For sharing between machines, see [multi-machine setup](shared-index.md).

## Usage log

Mash records every command and tool call in `usage.sqlite3`, beside the index
(by default `~/.mash/usage.sqlite3`): searches, reads, memory lookups and
actions, from the terminal, connected agents and the dashboard, with the
arguments each was given and what it returned. It never leaves this machine
and is kept until you clear it. [What it holds and how to turn it off](privacy.md#the-usage-log)

The log is a plain SQLite file. Query it with any SQLite tool, such as
`sqlite3` or Python's `sqlite3` module. Mash writes in short transactions in
WAL mode, so a reader can query while Mash writes. Query the two views below;
their columns keep their names and meaning across upgrades. The tables behind
them may change.

**`calls`**, one row per call:

| Column | Holds |
| --- | --- |
| `id` | The call's number in this log. |
| `at`, `at_ms` | When it started: ISO 8601 UTC text, and milliseconds since 1970. |
| `surface` | `cli`, `mcp` or `dashboard`. |
| `client`, `client_version` | The MCP client and its version, the agent a command ran under (`claude-code`, or `MASH_CLIENT`), or the browser family. |
| `tool` | The tool or command (`search`, `list-chats`), or the dashboard route (`/api/search`). |
| `arguments` | JSON text of the arguments as given, so `json_extract(arguments, '$.query')` works. Agents and the CLI name the query `query`; the dashboard names it `q`. Recognizable credentials are masked. |
| `outcome` | `ok`, `error`, or `incomplete` (answered, but asked to retry). |
| `error_kind`, `error_message` | Why a call failed. |
| `duration_ms` | How long it took. |
| `result_count` | How many results a listing returned. |
| `response_bytes` | The size of the answer. |
| `results_total`, `results_truncated` | How many items `call_results` found, and whether more than the 100 it keeps. |
| `session` | One MCP connection's calls. |
| `pid`, `build` | The Mash process and the build that served it. |
| `project` | The working directory a command ran in. |

**`call_results`**, what each call returned or opened, in order, as
references rather than content:

| Column | Holds |
| --- | --- |
| `call_id` | The call, `calls.id`. |
| `position` | Its place in the answer, from 0. |
| `kind` | `record`, `chat`, `message` or `attachment`. |
| `record_kind` | For a memory: `decision`, `preference` or `fact`. |
| `id` | The record, chat, message or attachment ID. |
| `node` | The machine it came from, for shared results. |

How often memory surfaced decisions, preferences and facts, by month and tool:

```python title="memory_kinds.py"
"""How often memory surfaced decisions, preferences and facts, by month and tool."""

import sqlite3
import sys
from pathlib import Path

log = (
    Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / ".mash" / "usage.sqlite3"
)
db = sqlite3.connect(f"file:{log}?mode=ro", uri=True)
rows = db.execute(
    """
    SELECT substr(calls.at, 1, 7) AS month, calls.tool, call_results.record_kind,
           COUNT(*) AS times
    FROM call_results JOIN calls ON calls.id = call_results.call_id
    WHERE call_results.kind = 'record' AND call_results.record_kind IS NOT NULL
    GROUP BY month, calls.tool, call_results.record_kind
    ORDER BY month, calls.tool, call_results.record_kind
    """
).fetchall()
for month, tool, kind, times in rows:
    print(f"{month}  {tool:<20} {kind:<11} {times}")
```

How often searches run, what they look for, and which filters they use:

```python title="search_values.py"
"""How often searches run, what they look for, and which filters they use."""

import sqlite3
import sys
from pathlib import Path

log = (
    Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / ".mash" / "usage.sqlite3"
)
db = sqlite3.connect(f"file:{log}?mode=ro", uri=True)
# MCP and the CLI name the search `search`; the dashboard records its routes.
SEARCHES = "tool IN ('search', '/api/search', '/api/search/live', '/api/search/stream')"
# The query is `query` from agents and the CLI, `q` from the dashboard.
QUERY = "COALESCE(json_extract(arguments, '$.query'), json_extract(arguments, '$.q'))"

print("Searches per day")
for day, surface, times in db.execute(
    f"SELECT substr(at, 1, 10), surface, COUNT(*) FROM calls WHERE {SEARCHES} "
    "GROUP BY 1, 2 ORDER BY 1, 2"
):
    print(f"  {day}  {surface:<10} {times}")

print("Most common queries")
for query, times in db.execute(
    f"SELECT {QUERY} AS query, COUNT(*) FROM calls WHERE {SEARCHES} AND query IS NOT NULL "
    "GROUP BY query ORDER BY 2 DESC, query LIMIT 20"
):
    print(f"  {times:>4}  {query}")

print("Filters used")
for name, times in db.execute(
    f"SELECT key, COUNT(*) FROM calls, json_each(calls.arguments) WHERE {SEARCHES} "
    "AND key NOT IN ('query', 'q', 'db', 'config', 'models_dir') "
    "GROUP BY key ORDER BY 2 DESC, key"
):
    print(f"  {times:>4}  {name}")
```

Run either with the log's path, or none for the default:
`python3 memory_kinds.py ~/.mash/usage.sqlite3`.

## Read the built-in help

Help and `--version` print plain text. Run `mash --version-info` for release
and database schema versions in JSON.

```sh
mash --help
mash search --help
```
