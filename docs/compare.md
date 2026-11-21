# Compare coding-agent history tools

Start with Mash if you want to find a past conversation, read the exact
message behind a decision, and keep your searchable history on computers
you run.

In the browser demo, [search the sample chats for invoice exports](https://getmash.io/demo/#/search?q=invoice+exports+queue)
and open the result from the workstation to read the discussion. Then open the
[saved sign-in decision](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
and choose **open the message** to see where the decision came from.

## When Mash fits

- **You already have chats worth keeping.** Mash reads the conversations
  supported agents have saved on disk. Search by word or phrase and jump
  to the match in its conversation, with surrounding dialogue and tool output
  available to read. You can use the dashboard, the terminal, or an agent
  you choose to connect. [Supported agents](supported-agents.md)
- **You want to check what a saved decision is based on.** With the plugin,
  your agent can save decisions and preferences with a quote checked against
  the chat. The dashboard opens the quoted message in context.
  [How Mash saves records](records.md#how-saving-works)
- **You work on more than one machine.** Each machine can search its own
  index offline. You can also publish to a Mash server you run so a laptop
  can find chats from a workstation. [How sharing works](shared-index.md)

Local word search needs the Qdrant search service and works without a Mash
account or model download. Search by meaning is optional and can use local
models. [Installation](installation.md) · [Privacy](privacy.md) · [Search by meaning](indexing.md#enable-semantic-search)

## Other workflows

| If you want to… | Look at | What it offers |
| --- | --- | --- |
| Transfer work between agents or recover an earlier file edit | Déjà Vu | Handoffs carry work to another agent. Code recovery can retrieve earlier text from recorded edits. [Déjà Vu README](https://github.com/vshulcz/deja-vu#overview) |
| Search a wide range of saved agent histories with a keyboard-driven interface | CASS | An interactive terminal search and a CLI with structured results. [CASS repository](https://github.com/Dicklesworthstone/coding_agent_session_search) |
| Bring automatically summarized work into new agent sessions | Claude-Mem | Hooks capture activity and an observer summarizes it. The next session can load the relevant notes. [How Claude-Mem works](https://github.com/thedotmack/claude-mem#how-it-works) |
| Save conversations as Markdown in your project or resume them in an agent | SpecStory | Its extensions and CLI write Markdown files as you work, and the CLI can reopen saved sessions. [SpecStory docs](https://docs.specstory.com/) |

Ready to look up your own chats? [Install Mash and find your first conversation](../README.md#quick-start).
