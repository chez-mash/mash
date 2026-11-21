# Your history and privacy

## What stays on your machine

The default setup indexes and searches locally without changing your agents'
original history files. You run Mash on your own machine and need no Mash
account. Mash keeps its own copy of the indexed chats, which can include
sensitive text. The [public demo](https://getmash.io/demo/) uses sample chats
and has no access to your local history.

The default dashboard and the search service Mash installs accept requests
from programs on the same computer without a login. Use the default setup on
a computer whose local users you trust. To open the dashboard from another
device, use the [dashboard with sign-in](dashboard-access.md).

An agent that keeps history in a SQLite database, such as OpenCode, may gain
`-wal` and `-shm` files beside that database when Mash reads it. SQLite
creates them for any reader; the database's contents are unchanged.

## Choose agent access

You can search your chats in the local dashboard or terminal without
connecting an agent. Choose which histories Mash reads separately from which
agents you want to connect.

- **During setup**, leave all agents unchecked to skip new connections.
  Existing Mash tools and plugins stay in place.
  [Set up without connecting agents](installation.md#skip-agent-connections).
- **To remove an existing integration**, remove the Mash tools and disable or
  remove the Mash plugin, then restart the agent. This includes plugins you
  installed separately. [Remove an agent connection](agents.md#disconnect-an-agent).
- **To stop indexing a source**, [change which histories Mash reads](dashboard-settings.md#choose-which-histories-mash-reads).
  Existing connections stay in place, and the chats Mash already indexed stay
  searchable. [Remove a chat from Mash](#remove-a-chat).

When you connect an agent during Mash setup, Mash registers its tools and
tries to install its plugin. [What the connection installs](agents.md#what-the-connection-installs).

## When history can leave your machine

| How you use Mash | What can be read or sent |
| --- | --- |
| Local dashboard or terminal search | Mash reads its index on your computer and shows the results to you. |
| Connected agent tools (MCP) | An agent can search indexed chats and read matching messages with their surrounding conversation. The agent can send what it reads to its model provider. |
| Mash plugin with hooks enabled | The plugin can give an agent recent chat details and saved memories at session start, and add chat excerpts when the agent reads a related file. These hints can reach its model provider even without an MCP connection. [What happens automatically and how to turn it off](plugin.md#what-happens-without-asking). |
| Sharing across machines | Mash uploads indexed chats and retained attachments to your server. Any computer with a token for the same group can read them. You can [stop sharing new history](dashboard-settings.md#stop-sharing-new-history). Copies already on the server stay there. [Review what you will share](shared-index.md#before-sharing). |
| Mashy in the dashboard | If you enable Mashy, it sends your questions, selected passages, and the chat text and memories it reads to the [model service you configure](configuration.md#the-dashboard-assistant). [Turn Mashy off](mashy.md#turn-mashy-off). |

- **A remote embedding worker** receives prepared searchable text, which may
  contain sensitive history. Embedding inference runs locally by default, and
  query embeddings always stay local. [Embedding inference](configuration.md#embedding-inference).
- **A remote reranking worker** receives each query and the excerpts being
  ranked. Reranking stays local unless you opt in. [Remote reranking](configuration.md#remote-reranking).
- **Memory backfill** sends selected existing chats to the provider of the
  signed-in agent you choose. Mash asks before doing this during setup.
  [How memory backfill works](records.md#backfill-earlier-memory).

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

## Try a small local setup

[Try a sample chat search](https://getmash.io/demo/#/search?q=invoice+exports+queue)
in the demo first. For a fresh setup with one of your own histories:

1. Run the [quick start installer](../README.md#install-it-yourself). Open
   the address it prints in your browser and keep the terminal running.
2. If you see the review first, choose **Customize setup**. Keep only the
   history for one agent you used in the past week, then choose **Next**.
3. Choose **another number of days** and enter **7** to select history
   updated in the past seven days. Mash checks the conversation's recorded
   activity when available, or when the history file last changed.
   The conversations it indexes can contain older messages.
   Choose **Next**, turn off any selected agent connections, and continue to
   the review.
4. Check that the review says **this machine only** and **no agents**.
   Choose **Set up and index**. When indexing finishes, open
   **chats** and search for a word from that conversation.

You can widen the range or connect an agent later.
[Dashboard setup](dashboard.md#set-up-mash-from-the-dashboard)

If you decide to stop, run `mash uninstall` to preview removal. When you
confirm, Mash removes its own index and services and keeps your agents'
original chats. [See what uninstall removes](uninstall.md)

## Shorten the history range

The range applies to future indexing. If you change it from 90 days to 7, a
chat last updated 30 days ago that Mash already indexed stays searchable,
even after another index run. A recently updated chat can still contain older
messages. [Change the history range](dashboard-settings.md#change-the-history-range)
or [remove a chat from Mash](#remove-a-chat).

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

Mash removes the local chat and any saved memories that have cited it. Mash
remembers not to import the chat again. With sharing enabled, it also attempts
to publish the removal. Check the command's result for any pending work.

This does not delete the agent's original files, earlier backups, or copies
independently indexed on another machine. There is no undo command. Make
[a backup](backup.md#make-a-portable-backup) first if you may need the chat later.
