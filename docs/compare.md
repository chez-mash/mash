# Is Mash for you?

Mash helps you find the conversations your agents already saved and open
the original message behind a decision. You can search in a browser, from
the terminal, or through a connected agent.

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

Mash uses a local search service called Qdrant. The setup installer can
start it for you. Word search works without a Mash account, model downloads,
or agent connections. You can add local models for search by meaning and
connect an agent whenever you want. [Installation](installation.md) ·
[Privacy](privacy.md) · [Search by meaning](indexing.md#enable-semantic-search)

Ready to look up your own chats? [Install Mash and find your first conversation](../README.md#quick-start).
