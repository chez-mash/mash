# Find a missing chat

Open **chats** in the dashboard sidebar, or choose **recent chats** under it.
This opens the unfiltered list of chats Mash has already read, most recently
active first. A conversation your agent saved may still need to be indexed
by Mash. If you see **Browse chats** after an empty search, it keeps your
applied filters. Use the sidebar to start from the unfiltered list.

If you remember part of the title, type it in the search box and look under
**chats** in the suggestions. You can also look up the title in the terminal.

```sh
mash list-chats --title-contains 'part of the title'
```

## The chat is listed but search finds nothing

- If you see **Clear filters**, choose it to remove the agent, machine, project,
  and date filters. If it opens the chat list, type the search again.
- In [word search](dashboard.md#choose-a-search-mode), every word you enter
  must match in the same message. Try one distinctive word from the chat.
  For search by meaning, try a short question in English.
  [Search chats in other languages](search-languages.md).
- Search covers what you and your agent wrote. To see a tool result or the
  agent’s reasoning, find a nearby message and open that part of the
  transcript.

## A recent chat or message is missing

Run this on the computer where the agent saved the chat.

```sh
mash index
```

Check the final result for errors. Return to the chat list or choose
**Check for updates** to refresh an open search. If the button is missing, reload the search
page. [Keep new chats searchable](synchronization.md#keep-refreshing-while-you-work).

If Mash reports a search service problem,
[check Qdrant](troubleshooting.md#qdrant-connection-failed).
If search is still behind, [check what’s holding it up](troubleshooting.md#new-chats-are-missing-or-search-is-behind).

## Older chats or one agent’s history are missing

If the missing chat was last updated before the start of your history range, use
[Add older history](indexing.md#add-older-history) to include saved conversations
outside that range. The default on a new setup is 90 days.

If one agent’s chats never appear, [check which histories Mash reads](dashboard-settings.md#choose-which-histories-mash-reads).
Run `mash status --installation` to
check which history locations Mash found. See [supported histories and how
to select them](supported-agents.md#if-a-chat-is-missing) for custom locations
or histories saved outside the usual folders.

## The chat is on another machine

Check the machine where the agent saved the chat. Once it’s indexed there,
the other machine can read it after it is published to your shared group.
Run `mash status --shared` on the source machine to check publication.
[Check for pending publication](synchronization.md#across-machines).

If the machines still need to join the same shared group, see
[Set up a shared index](shared-index.md).
