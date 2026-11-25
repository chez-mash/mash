# The Mash plugin for Claude Code, Codex and Copilot

The Mash plugin lets Claude Code, Codex, or GitHub Copilot CLI look up previous
work as you chat, save a decision when you ask, and open the Mash dashboard.
It adds skills and hooks. Mash’s [MCP server](mcp.md) gives the agent the
search tools those skills use. You can also use MCP on its own and ask the agent
to search with Mash.

- **Already using the Mash dashboard or terminal?** [Connect your agent](agents.md#connect-claude-code-codex-or-copilot)
  without indexing again. Setup installs the plugin for agents it connects.
- **Already installed the plugin from your agent’s marketplace?** [Finish setup in the agent](#finish-setup-in-the-agent).
- **Starting from scratch?** Follow the [quick start](../README.md#quick-start),
  or [install the plugin first](#start-with-the-plugin) and let your agent walk
  you through setup.

<span id="install-update-and-remove"></span>

## Install the plugin

<span id="requirements"></span>

Installation needs internet access. The hooks need Python 3.9 or later
(`python3`, `python`, or the Windows `py` launcher).

### Start with the plugin

Run the two terminal commands for your agent. If you already added the Mash
marketplace, use the second command.

#### Claude Code

```sh
claude plugin marketplace add chez-mash/mash
claude plugin install mash@mash
```

#### Codex

```sh
codex plugin marketplace add chez-mash/mash
codex plugin add mash@mash
```

#### Install in GitHub Copilot CLI

```sh
copilot plugin marketplace add chez-mash/mash
copilot plugin install mash@mash
```

### Finish setup in the agent

Start a new session. In Claude Code, run `/mash:setup`. In Codex or Copilot,
say “Use the Mash setup skill to set up Mash on this computer.” The agent
reviews what will be installed and which chats to index with you, then shows
you how to open the dashboard. Agent connections are optional. If you choose
to connect Copilot CLI with Mash v0.1.1-alpha or earlier, [complete the Copilot connection](#github-copilot-cli)
after setup.

If you connected an agent, restart it and ask “Use Mash to find an earlier chat
about authentication. Show me the message behind the answer.” Use a topic you
have discussed before. To turn on the Codex hooks, open `/hooks` in Codex and review
and trust Mash’s hooks. [Check a connection](agents.md#confirm-the-connection)
if the agent cannot find Mash.

## Skills and commands

| Skill | In Claude Code | In Codex | What it does |
| --- | --- | --- | --- |
| `setup` | `/mash:setup` | pick the `setup` skill, or ask “Set up Mash on this machine” | Installs Mash with your approval, connects your history, and checks that you can find an earlier conversation. |
| `remember` | `/mash:remember what to keep` | pick the `remember` skill, or say "remember this" | Saves a decision, preference or note to [memory](records.md), with your words as the evidence. With nothing after it, saves the latest decision in the conversation. |
| `note` | `/mash:note what to keep` | pick the `note` skill, or say "note that…" | Saves a note to [memory](records.md): something worth keeping that isn't a decision or a preference, such as where a service runs. |
| `resume` | `/mash:resume` | pick the `resume` skill, or ask "where did we leave off?" | Reads this project's most recent chats, from every agent and machine, and tells you what was in progress, what was left open and what was decided. |
| `handoff` | `/mash:handoff` | pick the `handoff` skill, or say "hand this off" | Writes a brief of the session into the conversation: goal, what is done, what is half done, decisions and the next step. `resume` finds it from any agent or machine once the chat is indexed. |
| `recall` | `/mash:recall a topic` | pick the `recall` skill, or ask "what does Mash remember about…?" | Shows the [memory](records.md) records for a topic or this project, with where each came from. It tells you when records need review. |
| `review` | `/mash:review` | pick the `review` skill, or say "review Mash memory" | Goes through the memory records that need a decision from you, a few at a time: ones an agent saved without your confirmation, stale ones and conflicting pairs. You keep, fix, withdraw, remove or skip each. |
| `forget` | `/mash:forget what to remove` | pick the `forget` skill, or ask explicitly to forget or delete something Mash stored | Finds one memory record or indexed chat, shows exactly what would go, and removes it only after you say yes. Offers to correct a wrong record instead of deleting it. |
| `standup` | `/mash:standup`, or `/mash:standup this week` | pick the `standup` skill, or ask "what did I work on yesterday?" | Summarises your work over a period, grouped by project, across agents and machines. With nothing after it, covers the previous working day. |
| `export` | `/mash:export`, or `/mash:export a chat` | pick the `export` skill, or say "export this chat" | Opens the [dashboard](dashboard.md#save-a-transcript) on the chat, where the Export menu saves it as Markdown, HTML or PDF. Can write a Markdown file itself if you ask for that. |
| `status` | `/mash:status` | pick the `status` skill, or ask "is Mash working?" | A plain-language check of the index, search, agent connections and your other machines, with the one next step when something is off. |
| `dashboard` | `/mash:dashboard` | pick the `dashboard` skill, or say "open the Mash dashboard" | Finds the running [dashboard](dashboard.md), or starts one, and opens it in your browser. Ask for "this chat" to land on the current conversation. |
| `mash-history` | `/mash:mash-history`, or just ask | pick the `mash-history` skill, or just ask | Searches your past chats and answers with the messages behind the result. The agent also uses it unprompted when you refer to earlier work. |

GitHub Copilot CLI loads the same skills. There, name the skill or ask
in plain words.

You do not need the commands for everyday use. Ask in plain words:

- "What did we decide about the database migration?"
- "Find the chat where we set up deployment."
- "Remember that we deploy from `main` only."
- "Open the Mash dashboard."
- "Where did we leave off on this project?"
- "What does Mash remember about deployment?"
- "That memory about the staging URL is wrong, fix it."

After saving, the agent confirms the statement it saved. It refuses to save
credentials and tokens. Removing is permanent: `forget` handles one record or
chat at a time and always shows it and asks first.

## What happens without asking

| When | What the plugin does | Turn it off |
| --- | --- | --- |
| A session starts | Shows the agent this project's saved records and recent chats, and starts a background `mash index` at most once every ten minutes | `MASH_AUTO_INDEX=off` stops the indexing. `MASH_RECORD_RECALL=off` stops the saved records in session and file hints. Chat hints still appear. |
| You send a message that looks back at earlier work, or names something the conversation has not mentioned | Reminds the agent to search Mash before guessing | `MASH_PROMPT_RECALL=off` |
| You state a lasting decision or preference | Reminds the agent how to save it. A session that runs long without saving anything gets one reminder at most every fifteen minutes | `MASH_RECORD_CAPTURE=off` |
| The agent reads a file in the project | Shows records and earlier chats about that file, once per file and session | `MASH_FILE_RECALL=off` |

In Copilot CLI the note about a file arrives with what the agent read, just
after the read, not before it.

Set a switch in the environment the agent starts in. Every hook does nothing,
silently, when Mash is missing, not set up, or slow to answer, so it never
blocks the agent. The hooks call Mash on your machine. Excerpts they show at
session start or on file reads enter the agent’s context, which it can send to
its model provider. Enabled hooks can run without an MCP connection. For
dashboard or terminal use only, [remove any installed Mash plugin](#update-or-remove-the-plugin)
and [disconnect the agent](agents.md#disconnect-an-agent).
[When history can leave your machine](privacy.md#when-history-can-leave-your-machine)

## Update or remove the plugin

The plugin and the `mash` program are released together. A marketplace can
offer a newer plugin before the matching binary is published. Run
`mash status --installation` to see which component needs an update. A
newer plugin can work with the current binary when its tool contract and
other installation checks match. Start a new session after an update.
To remove the plugin, run the command for your agent.

```sh
claude plugin uninstall mash@mash
codex plugin remove mash@mash
copilot plugin uninstall mash@mash
```

To remove the search tools too, [disconnect the agent](agents.md#disconnect-an-agent).

## GitHub Copilot CLI

Mash 0.2.0 and later connect GitHub Copilot CLI from `mash init` when
`copilot` is on your `PATH`. In v0.1.1-alpha and earlier, Mash can read
Copilot’s saved chats, and you can manually give Copilot access to Mash search.
After [setting up Mash](../README.md#quick-start), register its tools from a
terminal.

```sh
copilot mcp add mash -- mash mcp
```

If Copilot cannot find `mash`, replace the executable after `--` with the
quoted full path to your installation. Optionally [install the Copilot plugin](#install-in-github-copilot-cli)
for skills and hooks. Open a new Copilot session and ask it to use Mash to
find an earlier chat. `copilot mcp list` shows `mash`, and `copilot plugin list`
shows `mash@mash` if you installed the plugin.

In v0.1.1-alpha and earlier, remove the Copilot connection before uninstalling Mash.
If you installed the plugin, run its removal command too.

```sh
copilot plugin uninstall mash@mash
copilot mcp remove mash
```

- Copilot keeps these in `~/.copilot` (or the directory `COPILOT_HOME`
  names): the tools in `mcp-config.json`, the installed plugin in
  `config.json` and `installed-plugins`.
- `mash init --backfill-memory` does not accept Copilot. Reviewing old chats
  that way would spend your premium requests, so only Claude Code and Codex
  are offered for it.
- Saving to [memory](records.md) from a Copilot session needs a `mash` that
  can read Copilot's running session. `mash --version-info` lists `copilot`
  under `session_citations` when it can; with an older `mash` the plugin
  recalls but does not ask Copilot to save.
- On Windows the Copilot hooks run through PowerShell and call `python`.
  The hooks have no recorded Windows verification yet.

## Check that it is working

```sh
mash status --installation
```

Each agent's `connection` is `full` when the tools and plugin are both in
place, and `mcp-only` when only the tools are connected. `next_action` gives
the next step when an integration needs attention. For Copilot in v0.1.1-alpha and
earlier, use [Copilot’s own checks](#github-copilot-cli).
[Use Mash with your agent](agents.md#confirm-the-connection)
