# Your history and privacy

[Español](es/privacy.md)

## What stays on your machine

The default setup indexes and searches locally without changing your agents'
original history files. You run Mash on your own machine and need no Mash
account. Mash keeps its own copy of the indexed chats, which can include
sensitive text. The [public demo](https://getmash.io/demo/) uses sample chats
and has no access to your local history.

The default dashboard and the search service Mash installs accept requests
from programs on the same computer without a login. Use the default setup on
a computer whose local users you trust. To open the dashboard from another
device, use the [dashboard with sign-in](dashboard.md#open-it-from-your-phone-or-another-computer).

An agent that keeps history in a SQLite database, such as OpenCode, may gain
`-wal` and `-shm` files beside that database when Mash reads it. SQLite
creates them for any reader; the database's contents are unchanged.

## Where Mash stores data

By default, your indexed chats are in `~/.mash/mash.sqlite3`, alongside your
settings in `~/.mash/config.toml`. On Windows, `~` is your user profile.
Qdrant keeps data for search separately, including searchable chat text. On
Windows, Mash uses a short path on the system drive for that data, usually
`C:\mash-qdrant\storage`. [Search service files by platform](local-search.md#where-qdrant-stores-data)

Run `mash status` to see `database_path` and, when Mash installed Qdrant,
`local_qdrant.installed.storage`. Run `mash uninstall` to preview what would
be removed. It lists the managed directories, including the separate Qdrant
folder on Windows. Add `--yes` when you are ready to uninstall.
[See what uninstall removes](uninstall.md)

## When history can leave your machine

- **Agent connections** let the connected agent request search results and
  excerpts. Those results enter the agent's context and can be sent to its
  model provider. The [plugin’s hooks](plugin.md#what-happens-without-asking)
  can also add excerpts to the agent’s context without an MCP connection. To
  keep searches in the local dashboard or terminal, leave agents disconnected
  and disable or remove any Mash plugin you installed.
- **Sharing across machines** uploads retained indexed chats and attachments
  to the Mash server you configure. Each computer with a token for that group
  can read them. [Review what you will share before publishing](shared-index.md#before-sharing).
- **A remote embedding worker** receives prepared searchable text, which may
  contain sensitive history. Embedding inference runs locally by default, and
  query embeddings always stay local.
- **A remote reranking worker** receives each query and the excerpts being
  ranked. Reranking stays local unless you opt in.
- **The dashboard assistant** sends your questions, and the chat text and
  memories it reads to answer them, to the
  [model service you configure](configuration.md#the-dashboard-assistant).
  There is no assistant unless you configure one.
- **Memory backfill** sends selected existing chats to the provider of the
  signed-in agent you choose. Mash asks before doing this during setup.
  [How memory backfill works](records.md#backfill-earlier-memory)

## Try a small local setup

[Try a sample chat search](https://getmash.io/demo/#/search?q=invoice+exports+queue)
in the demo first. For a fresh setup with one of your own histories:

1. Run the [quick start installer](../README.md#install-it-yourself). Open
   the address it prints in your browser and keep the terminal running.
2. If you see the review first, choose **Customize setup**. Keep only one
   history with a chat from the past week, then choose **Next**.
3. Choose **another number of days**, enter **7**, and choose **Next**. Turn
   off any selected agent connections, then continue to the review.
4. Check that the review says **this machine only** and **no agents**.
   Choose **Set up and index**. When indexing finishes, open
   **chats** and search for a word from that conversation.

You can widen the range or connect an agent later.
[Dashboard setup](dashboard.md#set-up-mash-from-the-dashboard)

If you decide to stop, run `mash uninstall` to preview removal. When you
confirm, Mash removes its own index and services and keeps your agents'
original chats. [See what uninstall removes](uninstall.md)

## Sensitive information

Mash masks recognizable credentials in results and outgoing shared history by
default. Detection is best effort: review anything you intend to share.

Local history and portable backups can still contain credentials. Store
backups privately, just as you would the original conversations.

Text enclosed in `<private>…</private>` within a single message is omitted
from Mash's stored message text. Both tags must be present. This does not
remove text from the agent's own files or hide content inside attachments.

## Remove a chat

Find its `chat_id` with `mash list-chats` or `mash search`, then run:

```sh
mash forget CHAT_ID
```

Mash removes the local chat and remembers not to import it again. With sharing
enabled, it also attempts to publish the removal; check the command's result
for any pending work.

This does not delete the agent's original files, earlier backups, or copies
independently indexed on another machine. There is no undo command. Make
[a backup](backup.md#make-a-portable-backup) first if you may need the chat later.
