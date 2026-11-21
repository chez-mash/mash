# Navigating the dashboard

Try a [search for invoice exports](https://getmash.io/demo/#/search?q=invoice+exports+queue)
in the browser demo. Open a result to see the original conversation. You can
also [open a saved decision](https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop)
and select **open the message** to see where it came from. The demo uses
sample chats.

Prebuilt Mash releases include the dashboard. If you compile Mash yourself,
[build with the dashboard](install-methods.md#include-the-web-dashboard).
After [installing Mash](installation.md), start it in a terminal.

```sh
mash dash
```

Open the address it prints, usually **[http://127.0.0.1:8790](http://127.0.0.1:8790)**,
in your browser. Once Mash is set up, the overview shows your latest chats and
memories. Leave the terminal running. Ctrl-C stops the dashboard. If the page fails to load,
[check dashboard startup](troubleshooting.md#the-dashboard-doesnt-open).

With the [plugin](plugin.md) installed, `/mash:dashboard` in Claude Code, or
asking your agent to open the Mash dashboard, does both steps for you.

The default local dashboard opens without a sign-in. Read [Keep access private](#keep-access-private)
before changing its listening address.

## Set up Mash from the dashboard

You can do first-run setup in the browser instead of running `mash init` in a
terminal. Start `mash dash` and open the dashboard: on a machine with no
configuration it opens on setup by itself, as a screen of its own without the
rail or search. **Skip to the dashboard** leaves it; **settings** has a link
back, which is also how you go through setup again later.

Review the histories Mash found, the history range, and the installed agents it
can connect so they can search. **Show history folders** lists the paths. For a
new setup the default is to index the last 90 days on this machine. Choose
**Set up and index** to accept the review, or **Customize setup** to change
the histories, range, or agent connections before confirming.
On the first review, **Skip agent connections** lets you index the same
histories and search them yourself from the dashboard or terminal.
If your version opens on the history choices, use **Next** to reach the review.

Nothing changes until you choose **Set up and index** or **Set up without
indexing** on the review. Mash then writes its configuration file, has each
chosen agent's own command-line tool connect it, and starts indexing in the
background if you asked for it. The page shows what was written, which files
each agent's tool edited and the commands that undo it. It follows the first
index until your chats appear.

- Indexing needs the search service. If it is down, run `mash qdrant install`
  and choose **Check search service again**, or choose **Set up without
  indexing** and index later. The customization review also offers saving
  without indexing when the service is ready.
- To search by meaning from the dashboard, follow the
  [model setup guide](indexing.md#enable-semantic-search) and choose it as your default.
- Going through setup again on a machine that is already set up rewrites the
  configuration with your answers and keeps the previous file beside it.
- Sharing with other machines, memory backfill and history kept in other
  locations are not offered here: use `mash init` or edit the configuration
  file.
- A dashboard that other machines can reach runs setup only when it was
  started with a services configuration, like the settings page.

## The rail

The left rail is the way around. On a phone it opens from the menu button.

- **overview:** your latest chats and memories, and each machine and agent
  with its chat count.
- **chats:** every chat, newest first, with a search box. **viewing history**
  lists chats you opened in this browser session; each reopens where you left
  it.
- **agents**, **machines** and **projects:** one page each, with its recent
  chats and how they break down by the other two.
- **memory:** saved decisions, preferences and facts.
- **services** and **fleet manager:** process and database checks, each
  machine's agent setup, and fleet actions. The fleet manager appears only
  where fleet control is set up. See [services](services.md).
- **tool explorer:** every tool your agents can call on Mash, with its
  parameters. Run a read-only tool to see the reply as an agent receives it.
  Tools that change data are listed but cannot be run here.
- **settings:** this machine's configuration and index readiness. You can
  change which agents are indexed, how far back, whether history also goes to
  the shared server, memory capture and credential masking. Where
  [Mashy](#ask-mashy) is set up you can also switch it off and on and choose
  its model. Saving rewrites
  the file and keeps the previous one beside it as `config.toml.previous`;
  restart the watcher and reconnect agents to apply. Mashy follows its own
  settings at once. Search and storage
  settings are shown but cannot be changed here. A dashboard that other
  machines can reach changes settings only when it was started with
  `--services-config`.

The button in the top right corner switches between light and dark; the
choice is remembered in that browser.

## Find and read a chat

Open **chats**, type a word you remember into **Search your history**, and
press Enter. Open a result to jump to the matching message in its chat.
Without a query, the page lists your chats. Narrow either list with the chips
for agent, machine, project and date. A changed chip has a dashed border until
you choose **Apply filters**. Applied filters appear as tags in the search box.
Choose a tag to remove it.

If a filtered search comes back empty, choose **Clear filters** to try the
same query across your chats. If your dashboard opens the chat list, submit
the query again. For an empty search without filters, see
[how to find the missing chat](troubleshooting.md#search-returns-no-results).

A transcript shows your messages on the right and the agent's replies on the
left. Runs of tool calls and reasoning are folded into one line; open it to see
each step and its output. The side panel lists your prompts on the current
page, other chats from the same project and memory saved from this chat.
**Copy session ID** copies the ID to resume the chat in its agent.

Chat and memory views are read-only.

### Choose a search mode

After submitting a search, open the **search: default** menu beside the
filters. Choose **words** or **meaning**, then select **Apply filters**.
The **words** mode matches terms from the chat. The **meaning** mode uses the
[optional models](indexing.md#enable-semantic-search) and takes English
queries. The choice follows the result pages for this query. A different
query starts with [the configured default](configuration.md#search-defaults).
If your dashboard does not show the menu, choose the mode with
[`mash search --intent`](search.md).

## Inspect conversation records

Open **memory** in the rail, or one of the kinds beneath it. Switch kinds with
the chips at the top, search statements with **find a statement**, and filter
with the status, review and source chips. Source chooses this machine's
records or those published to your shared group. Choose **Apply filters** to
run the search and **Next page** to continue.

Open a record to read its revision history, quoted supporting messages and
related records. **open the message** under a quote opens the cited chat
message. In a transcript, **memory from this chat** lists records that cite
the chat in any retained revision, and a quoted message shows **Saved as**
beneath it.

Lists include candidates and retired records. Check the status, expiry dates,
evidence warnings and declared conflicts before relying on a record. Reading
never confirms or changes one; ask your agent to make changes through Mash's
[record tools](records.md). If a shared record's source is unavailable, its
link produces an error rather than opening a local copy.

## Keep your place

The dashboard follows history as Mash updates it, on this machine and on the
machines that publish to your shared index; what you are reading never moves
by itself.

- **Lists of chats and memories** (the overview, the rail, recent chats, the
  session list beside a transcript, the agent, machine and project pages, and
  the memory pages) show what is new within a few seconds.
- **An open transcript** gains new messages at the end while you are at the
  bottom of the chat, and the view follows them. Anywhere else in the chat, an
  **N new messages** button appears instead; choose it to go to them. Text
  already shown never changes by itself: if Mash re-reads an earlier message,
  **Show updated messages** applies it.
- **A submitted search**, of chats or of memories, keeps its results and
  filters until you submit again, including when you open a result and return
  with your browser's Back button.
- **An open memory record** changes only when you choose **check for
  updates**.

Chats and memories from other machines arrive as each machine publishes them.
That needs a shared server at least as new as this dashboard. With an older
one, or with more than 16 dashboard tabs open at once, load them yourself:
**reload chat list** at the foot of the rail, and the refresh button in a
transcript.

If the connection drops, in the browser or between the dashboard and the
shared server, it reconnects by itself and then catches up.

New conversations appear only after indexing: run `mash index`, or leave
`mash index --watch` running in another terminal. The dashboard notices either
one; it does not index by itself.

If a request fails, an error panel names what could not load and offers
**Retry**. A connection error means the browser could not reach Mash, not that
chats or memories were deleted. Check the connection and that `mash dash` is
still running, then choose **Retry**.

## Ask Mashy

If you have [configured a model](configuration.md#the-dashboard-assistant),
a circle with the Mash mascot sits in the bottom-right corner of every page.
Choose it to open a chat with Mashy, and ask about your history in plain
words:

- "What did I work on yesterday?"
- "What did we decide about the restart test?"
- On a chat's page: "Summarise this chat."
- "Why can't I find this morning's chat?"

On a wide screen, drag the window by its top bar to move it off what you are
reading; double-click the bar to put it back. On a phone it fills the screen.

Mashy searches your chats and memories and answers briefly. What it mentions
you can open in the dashboard behind:

- A chat or memory is a chip. When the answer rests on one message, the chip
  opens the chat at that message.
- Quoted words appear in a card with who said them and when; your own words
  carry a coloured edge.
- A project, an agent or a machine opens its page.
- When an answer covers only part of what matched, **See all** opens the full
  search for the same words.
- A command such as `mash index` has a **Copy** button.

Under an answer it suggests questions that follow from it. If it finds
nothing, it says so. Use the stop button to cut an answer short.

To ask about something you are reading, select the text in a transcript and
choose **Ask Mashy**. The passage stays pinned above the conversation and goes
with each question until you remove it.

Mashy can also tell you how Mash itself is doing: whether search is ready, how
recent the newest indexed chat is, which agents are connected and when your
other machines last synced. It cannot fix anything, so it names the command to
run.

It answers questions about Mash itself from these guides, which are built into
Mash: "How do I share my index with my laptop?" or "What does `mash init`
change?" A guide it names opens on this site.

Dates and times are on your browser's clock, so "yesterday" is your yesterday.
Questions about a period ("what did I do last week", "how many chats in
October") are answered from the chats that had messages then. A long chat is
read in parts: Mashy says which part an answer covers.

Mashy uses read-only tools that can look up your history and the built-in
guides. It cannot index, forget, save memories, run commands or change settings. The
conversation lasts as long as the browser tab and is not added to your
history.

Each question sends the text Mashy reads to the model you configured,
including a passage you selected. Without
that configuration there is no circle and nothing is sent.

If Mashy fails or answers badly, start the dashboard with `MASH_TRACE=1` and
ask again. Its log then records each step of a question: which lookups ran and
how much they returned, how long the model took, how many tokens it used and
why it stopped. Questions, answers and history text are never logged.

## Save a transcript

Use the transcript's **Export** menu to download the whole chat as Markdown or
HTML, including tool output. **Export to PDF** opens a printable copy; choose
**Save as PDF** in your browser's print dialog.

For a stable copy, wait for indexing to finish first. Downloaded transcripts
may contain sensitive information.

Each chat has its own address, `http://127.0.0.1:8790/#/chat/<chat id>`.
With the [plugin](plugin.md), `/mash:export` finds the chat you mean and
opens it there.

## Export many chats at once

Open **chats**, then **export chats**. Narrow the list with the agent, machine,
project and time filters; every matching chat is exported, on every page,
unless you untick it. Choose **Markdown** to read or **JSON** for scripts, then
**Export**. Your browser saves one `.zip` file with a folder per project and a
file per chat.

Keep the page open until the download starts. If any chat cannot be read, the
export stops and saves nothing. Attachments are not included. The files hold
the chats in full, including anything sensitive said in them.

These exports and saved transcripts cannot be brought back into Mash. For a
copy Mash can restore, use
[a portable backup](upgrades.md#make-a-portable-backup). Chats from your other
machines appear in the dashboard through [sharing](shared-index.md).

## Open it from your phone or another computer

By default, only your machine can reach the dashboard. On Linux or macOS with
[Tailscale](https://tailscale.com) connected, one command makes it reachable
from your other devices, and only from them:

```sh
mash dash remote
```

It keeps the dashboard running as a user service behind a sign-in (a systemd
user service on Linux, a LaunchAgent on macOS), gives it an HTTPS address
inside your tailnet, and prints the address, a one-time code and a QR code:

```json
{
  "dashboard": "https://laptop.your-tailnet.ts.net:8790/",
  "service": "mash-dash.service",
  "code": "K7M-2QX",
  "code_minutes": 10
}
```

On the phone, with Tailscale connected, scan the QR code. Or open the address
and type the code. Either way the browser stays signed in for 30 days.

| To | Run |
| --- | --- |
| Sign another device in | `mash dash remote --code` |
| Show the QR code again | `mash dash remote --qr` |
| Use another port | `mash dash remote --port 8791` (with every command) |
| Host it on a machine with no history of its own | `mash dash remote --shared-only` |
| Reach it without Tailscale on the device | `mash dash remote --public --port 8443` |
| Stop serving | `mash dash remote --remove` |

A code works once, within ten minutes. The QR code holds the long-lived token
itself, after `#` in the address, where browsers never send it to a server;
the dashboard signs in and removes it from the address at once. Treat a picture
of it like a password.

The dashboard searches whatever that machine can search: its own history and,
with a [shared index](shared-index.md), your other machines' history too. For
the shared index the service needs the node token: run `mash dash remote` in a
shell where that variable is set, and it is saved for the service in
`~/.mash/dashboard.env`, readable only by you. On macOS the LaunchAgent's
property list (`~/Library/LaunchAgents/dev.mash.dash.plist`) holds the same
tokens and is just as private; its output goes to `~/.mash/dashboard.log`.

A machine with no agent history of its own, such as the server that holds the
shared index, can still host the dashboard: with `--shared-only`, chats, search
and memories come from the shared index alone.

`--public` publishes the address on the internet with
[Tailscale Funnel](https://tailscale.com/kb/1223/funnel), so a phone or
borrowed computer that is not on your tailnet can open it. Tailscale must allow
Funnel for the machine, and only ports 443, 8443 and 10000 can be published.
Anyone can then reach the sign-in page: the token and one-time codes are all
that protect your full chat history, and ten wrong attempts in a minute pause
sign-in for everyone, you included. Prefer the default unless you need this.

### Set it up by hand

On Windows, or behind another HTTPS reverse proxy such as Caddy:

1. Choose a token of at least 32 characters and put it in an environment
   variable, for example `MASH_DASH_TOKEN`.
2. Give the dashboard an HTTPS address. With Tailscale:
   `tailscale serve --bg 8790`.
3. Start the dashboard with that name and the token variable:

   ```sh
   mash dash --allow-host laptop.your-tailnet.ts.net --access-token-env MASH_DASH_TOKEN
   ```

`--allow-host` is the name browsers use for the dashboard, with its port when
the address shows one; repeat it for more than one. Changing the token signs
every browser out.

## Keep access private

With `--access-token-env`, every page of history needs the sign-in. Ten wrong
tokens in a minute pause sign-in for everyone until the minute passes.

Without it the dashboard has no login. Keep it on its default address, which
only your machine can reach: `mash dash --listen` with another address serves
your history to anyone who can connect, and prints a warning when started that
way. `--allow-host` refuses to start without a token.

Prefer a private network over a public address. On the public internet the
token is the only thing protecting your full chat history, and a tunnel service
that ends HTTPS for you, such as Cloudflare Tunnel, can read what passes
through it.
