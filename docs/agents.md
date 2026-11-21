# Use Mash with your agent

Connect Claude Code, Codex, or GitHub Copilot CLI to let it look up earlier work
when you ask. A connected agent can search any history you've indexed.
[See which histories Mash can read and which agents you can connect](supported-agents.md).

## Connect Claude Code, Codex or Copilot

After [installing Mash and starting Qdrant](installation.md), run:

```bash
mash init
```

Then restart the agent so it loads the connection, and [try it](#try-it).

You can search saved chats in the dashboard or terminal even when the agent
is disconnected. To connect it, its command must work in your terminal. For
example, check Codex with `codex --version`. If the terminal cannot find it,
install the agent's CLI or add it to `PATH`, then run `mash init` again.

Already installed the plugin from your agent's marketplace? Use its
[`setup` skill](plugin.md#start-with-the-plugin) to finish setting up Mash.

For each of the three agent CLIs (`claude`, `codex`, `copilot`) found on your
`PATH`, `mash init` installs two things, using the agent's own commands:

- an **MCP server**: the Mash search tools the agent can call (MCP, the Model
  Context Protocol, is how agents run external tools);
- the **[Mash plugin](plugin.md)**: a `mash-history` skill that tells the
  agent when and how to search, a `remember` skill that saves something to
  [memory](records.md) when you ask (`/mash:remember` in Claude Code), and
  hooks that [keep new chats searchable](#keep-new-chats-searchable).

Those commands edit the agent's user settings: `~/.claude.json`,
`~/.claude/settings.json` and `~/.claude/plugins` for Claude Code;
`~/.codex/config.toml` and `~/.codex/plugins` for Codex; and
`~/.copilot/mcp-config.json`, `~/.copilot/config.json` and
`~/.copilot/installed-plugins` for GitHub Copilot CLI. Mash also records
the connected agents in its own `config.toml`, so that
[`mash status --installation`](#confirm-the-connection) can report one that
later loses its connection.

Requirements for the plugin:

- Installing it needs network access.
- Its hooks need Python 3.9+ (`python3`, `python`, or the Windows `py`
  launcher).

If the plugin fails to install, the agent can still search through the MCP
server when you ask it to; it lacks only the skill and the hooks.

Running `mash init` again repairs a registration that points at a missing or
different Mash executable.

## Try it

Ask your agent:

- “Use Mash to find what we decided about authentication.”
- “Find the chat where we discussed this repository's deployment.”
- “What did we work on last week? Show me the messages behind the answer.”
- “Find the earlier conversation about this error.”

The agent searches, reads the messages behind the results, and answers with
the chat's title, date, or a link as its source. It gives raw session and
message IDs only if you ask for them.

To invoke the skill explicitly, use `/mash:mash-history` in Claude Code. In
Codex or Copilot, select the plugin's `mash-history` skill or ask the agent to
use Mash.

## Confirm the connection

```bash
mash status --installation
```

In the JSON report, `summary.clients` lists each agent with a `connection`:

| `connection` | Meaning |
| --- | --- |
| `full` | Nothing needs attention. |
| `mcp-only` | The Mash tools are connected. The plugin, skill, or hooks need attention. |
| `none` | Mash has no usable connection to that agent. It may have been skipped during setup. |

When something needs repair, `next_action` gives the next step. Indexing
does not fix a connection. If the marketplace plugin is newer than your
CLI, the report checks whether they declare the same tool contract.
Compatible tools and intact plugin files can stay in use. A different
contract needs a matching CLI release.

An `unconfigured` count with `next_action: null` means Mash found agent data
without a connection and has no repair to suggest. You can still search
indexed chats in the dashboard or terminal.

The command exits with status 1, still printing the report, when an agent
needs attention and is one that `mash init` connected, one configured for
Mash, or one named with `--agent`. Agents you never connected are fine to
leave unused.

To check one agent and also launch its MCP server in a fresh process:

```bash
mash status --installation --agent codex --probe
mash status --installation --agent claude-code --probe
mash status --installation --agent copilot --probe
```

A successful probe verifies a new process, not the connection in an
already-open chat. Restart the agent after setup, repairs, or upgrades.

## Keep new chats searchable

The plugin's hook indexes new history in the background when an agent session
starts, at most once every ten minutes. It does not follow the current
conversation as it happens. Set `MASH_AUTO_INDEX=off` in the agent's
environment to turn it off.

To index now, run `mash index`. To index continuously, keep
`mash index --watch` running in another terminal.
See [keeping history current](synchronization.md).

## Disconnect an agent

Run the agent's own removal commands (`mash init` lists them as `undo`):

```bash
claude mcp remove --scope user mash
claude plugin uninstall mash@mash
codex mcp remove mash
codex plugin remove mash@mash
copilot mcp remove mash
copilot plugin uninstall mash@mash
```

To remove Mash from the machine as well, see
[uninstall](installation.md#uninstall).

## Other MCP clients and manual setup

Any other client that supports stdio MCP servers can run `mash mcp`. Use the
[MCP configuration example](mcp.md#connect-another-mcp-client), including any
environment variables your configuration needs.

For common connection problems, see [troubleshooting](troubleshooting.md).
