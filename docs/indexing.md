# Include older chats and search by meaning

## Add older history

On a new setup, Mash reads history that your agents updated in the last 90
days. A recently updated conversation can contain older messages. To include
earlier conversations still on disk:

```sh
mash index --all-history
```

To go back a fixed number of days instead:

```sh
mash index --since-days 365
```

To refresh just one agent's history:

```sh
mash index --provider codex
```

The provider names are `claude-code`, `codex`, `opencode`, `copilot`, `cursor`
and `antigravity`.

Indexing needs the local search service (Qdrant) running and never changes your
original history files. Check the command's final result for errors.

These flags apply to one run. To change the age limit for future indexing,
[change the history range in Settings](dashboard-settings.md#change-the-history-range)
or [edit the configuration file](configuration.md#choose-histories-and-an-age-limit).
For continuous updates, see [keep history up to date](synchronization.md).

### Copilot in VS Code remote windows

Index each kind of session on the machine that stores it:

- **Copilot Chat** sessions from a Remote-SSH, container or Codespaces window
  are stored on the computer running VS Code. Their project is the folder on
  the remote machine.
- **Copilot CLI** sessions started from the chat view of a remote window run on
  the remote machine and are stored in its `~/.copilot/session-state`.

If Copilot is still writing a session while Mash reads it, Mash keeps that
chat's previously indexed messages and reads the session again on the next
refresh.

## Enable semantic search

Search by meaning, also called semantic search, lets you ask a question even
when you do not remember the original wording. Use English for these queries.
Word and phrase search works without models and stays the default unless you
change it. For a chat in another language, see
[word search examples](search-languages.md).

### Install the models

```sh
mash models install
```

This downloads about 177 MB from `https://huggingface.co` into
`~/.mash/models`. Apple silicon and Windows also download the unquantized
embedding model for their GPU engines, about 90 MB more. Set `HF_ENDPOINT`
to use a Hugging Face mirror, or pass `--models-dir PATH` to use another
directory.

### Build search data

If this is your first setup, run `mash init` after installing the models.
When it creates your settings, Mash enables indexing by meaning and builds
the search data during the first index. [First-time setup](installation.md#connect-your-histories)

If you already use Mash, edit the `[views.build]` table in your existing
[configuration](configuration.md). Open **settings** in the dashboard or look
for `configuration.path` in `mash status` to find the file. Keep your other
settings and add the table only if it is absent.

```toml
[views.build]
representations = "lexical-and-dense"
```

Then run `mash index`. The first pass takes longer because Mash prepares
your existing history for search by meaning. Later passes reuse unchanged work.

Check `mash status` afterward. `realized_mode` is `hybrid` when both word
search and search by meaning are ready on this machine.

### Try a question

```sh
mash search 'why did we choose a separate database' --intent related-discussion
```

### Use search by meaning by default

To use it in the dashboard and for searches that omit an intent, set `intent`
in your existing `[views.query]` table. Add the table if needed. Once the
search data is built, changing this default takes effect without reindexing.

```toml
[views.query]
intent = "related-discussion"
```

To run a word search even with that default, pass `--intent exact-mention`.

```sh
mash search 'auth_token' --intent exact-mention
```

If something fails, see [troubleshooting](troubleshooting.md).
