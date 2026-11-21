# Keep history up to date

A new chat becomes searchable after Mash indexes the history your agent saved.
Use [Refresh now](#refresh-now) when you're looking for a chat you just had.
For updates while you work, [keep a watcher running](#keep-refreshing-while-you-work).

## Refresh now

Run these commands on the computer where the agent saved the chat. If you
haven't chosen which histories Mash should read, [finish setup](installation.md#connect-your-histories)
first.

```bash
mash index
mash status
```

`mash index` reads new or changed history and shows its progress. Check the
final result for any incomplete scan or pending publication. Follow the reason
it gives, then run `mash index` again to retry. `mash status` reports whether
local search is ready.

The dashboard chat list updates after indexing. If you have a search open,
submit it again to include the new chats.

## Keep refreshing while you work

Run this in a separate terminal and leave it running:

```bash
mash index --watch
```

It indexes once, then refreshes when history changes settle. It waits at least
30 seconds between passes, and can wait longer after a slow pass. Ctrl-C lets
the active pass finish and stops watching. Closing the terminal also stops it.

Your configured providers, age limit, and local/shared settings apply. To include
older history as you work:

```bash
mash index --watch --all-history
```

For a permanent age setting, edit [your configuration](configuration.md).
`--watch` cannot be combined with `--full` or `--reread`.

When you upgrade Mash, the watcher finishes its active pass and then starts the
new `mash` with the same options, so a terminal or service keeps watching
without a restart. If it cannot start the new executable, it exits with an
error; start it again yourself.

## What the agent plugin does automatically

The [Mash plugin](plugin.md) starts a low-priority catch-up index at session
start, at most once every ten minutes.
The agent does not wait for it. If another indexing run is active, that attempt
exits without changes.

The hook catches up when you start a session. For updates during a
long-running session, use the watcher. To disable the catch-up,
set `MASH_AUTO_INDEX=off` in the agent's environment.

## Understand freshness warnings

Manual updates work after `mash init` or a one-time `mash index`. Search uses
the history from the last indexing run. If an older release says “Live
indexing is inactive,” it means updates happen when indexing runs again.

A search can still return earlier chats when the last scan was incomplete or
Mash could not read its status. Follow the warning to check or refresh your
index. If a recent chat is missing, run `mash index` and search again.

- Keep a watcher running if you need ongoing updates. A running watcher is not
  proof that every source was read successfully.
- If the warning follows an upgrade or mentions an incompatible index, follow
  the [upgrade guide](upgrades.md) before retrying.

## In the dashboard

Indexing makes new history available to the dashboard. An open transcript
follows new messages when you are at the bottom of the chat. Elsewhere in the
transcript, choose **N new messages** to see them. An earlier message stays as
displayed until you choose **Show updated messages**. [Dashboard refresh behavior](dashboard.md#keep-your-place)

## Across machines

If a chat from another computer is missing, check that computer first. It
indexes the chat and publishes the shared copy. Run `mash status --shared`
there to inspect publication. If publication is pending, restore the
connection and run `mash index` there again.

See [multi-machine setup](shared-index.md).
