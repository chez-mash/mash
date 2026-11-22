# Mash for Raycast

Search the history of your coding agents from Raycast, as you type.

**Search Mash** lists the chats on this machine that mention what you type,
each with the passage that matched. From a result you can:

- **Open in Mash** (↵) — the chat, at that message, in the Mash desktop app.
- **Read Messages** — the match with a few messages on either side, without
  leaving Raycast. This is the main action when the desktop app is not
  installed.
- **Copy Link** (⇧⌘C) — a `mash://` link to the chat.
- **Copy Passage** (⇧⌘.).
- **Search All of Mash** (⌘↵) — the same words in Mash's full search, which
  also reads your other machines, matches exact identifiers and can search by
  meaning.

## Requirements

[Mash](https://getmash.io/installation/), set up with `mash init` and indexed.
The extension runs `mash search --live`, which reads this machine's index only,
and `mash view`. It finds `mash` in the usual install locations (the install
script, Homebrew, cargo-binstall, mise, or the Mash desktop app); set **Mash
Executable** in the command's preferences if yours is elsewhere.

The extension reads only. It never changes, forgets or indexes anything, and
it shows chat text as plain text, so nothing in your history can load a remote
image or link.

## Development

```sh
pnpm install
pnpm dev        # opens the extension in Raycast
pnpm test       # the parsing and link tests
pnpm lint
```

`author` in `package.json` must be the Raycast account that publishes the
extension before it is submitted to the Store.
