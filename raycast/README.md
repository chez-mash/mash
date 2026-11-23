# Mash for Raycast

Search the history of your coding agents from Raycast, as you type.

**Search Mash** shows your latest chats on this machine before you type, and
then the chats that mention what you type, each with the passage that matched.
From a chat you can:

- **Open Chat in Mash** (↵) — the chat, at that message, in the Mash desktop
  app. Offered only when the app is installed.
- **Read Messages** — the match with a few messages on either side, without
  leaving Raycast. This is the main action when the desktop app is not
  installed.
- **Copy Link** (⇧⌘C) — a `mash://` link to the chat, which opens in the
  desktop app. Offered only when the app is installed.
- **Copy Message ID** — the ID that `mash view` and your agents' Mash tools
  read the message by (⇧⌘C without the desktop app).
- **Copy Passage** (⇧⌘.).
- **Search Everywhere** (⌘↵) — the same words in Mash's full search, which
  also reads your other machines, matches exact identifiers and can search by
  meaning. Without the desktop app, this copies the `mash search` command
  that does the same in a terminal.
- **Open Mash** (⌘O) — the desktop app's main window.

When Mash fails, the extension shows Mash's own message with the next steps:
try again, the [troubleshooting guide](https://getmash.io/troubleshooting/),
the installation guide, or **Set Mash Location**.

## Requirements

[Mash](https://getmash.io/installation/), set up with `mash init` and indexed.
The extension runs `mash list-chats --local` and `mash search --live`, which
read this machine's index only, and `mash view`. It finds `mash` in the usual
install locations (the install script, Homebrew, cargo-binstall, mise, or the
Mash desktop app in `/Applications` or `~/Applications`); set **Mash
Executable** in the Mash extension's preferences if yours is elsewhere.

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

## Before submitting to the Store

- **`author`**: `package.json` says `mashium` for now. Replace it with the
  Raycast username of the account that publishes the extension (the handle on
  its raycast.com profile); the Store rejects an author it does not know.
- **Screenshots**: none exist yet. TODO: add them to `metadata/` as
  `mash-1.png`, `mash-2.png` and so on, 2000 × 1250 PNG, taken with Raycast's
  Window Capture while `pnpm dev` runs. They are needed for:
  1. Recent chats, before anything is typed.
  2. A search with matches, with the passage and details beside the list.
  3. **Read Messages** on a match.
  4. The action panel on a match with the desktop app installed (**Open Chat
     in Mash**, **Copy Link**, **Search Everywhere**).
  5. No matches, with its description and next step.

  Take them only from made-up chats, never real history. Raycast runs on macOS,
  so these cannot be rendered on Linux or in CI; one way is a Mac with Mash set
  up on a demo history, using the **Mash Executable** preference to point at
  that install.
