# Back up, restore, and upgrade

Updating Mash keeps your settings and indexed chats. Save a backup first,
then install the new version, run `mash index`, and reopen the dashboard or
agents you use. If you just need a backup or want to recover a chat, go to
[Make a portable backup](#make-a-portable-backup) or
[Restore a separate copy](#restore-without-replacing-your-current-history).

## Upgrade an existing installation

If you publish history across machines, read the
[shared index steps](#upgrade-a-shared-index) before updating.

1. **Pause indexing and close connected agents.** Stop any
   `mash index --watch` jobs and let active indexing finish. Stop a
   dashboard running in a terminal with Ctrl-C.
2. **Back up with the version you have.** Keep its executable or release
   archive until the new version is working. Export to a new directory
   you can keep private.

   ```sh
   mash --version
   mash export /absolute/path/to/new-backup
   ```

3. **[Install the new version](#install-the-new-version)** using your usual
   install method. Run `mash --version` again to check that your shell
   finds the updated executable.
4. **Update Qdrant if the release needs a newer one.** Rerun
   `mash qdrant install`; it upgrades the service Mash installed and leaves
   its data alone. If you run Qdrant yourself, upgrade it on the same storage.
5. **Refresh the index.** If a supported database format needs upgrading,
   this command does it before indexing. Read-only commands report that the
   upgrade is needed.

   ```sh
   mash index
   ```

6. **Check search and reopen Mash.** Check that `mash status` reports
   `"ready": true`, then search for a word from an indexed chat.

   ```sh
   mash status
   mash search 'a word you remember' --local
   ```

   Restart connected agents and any watchers you stopped. Run `mash dash`
   again for the local dashboard. For a [remote dashboard](dashboard.md#open-it-from-your-phone-or-another-computer),
   rerun `mash dash remote` with the same options to start its service from
   the new executable.

Mash keeps chat and message IDs and reuses compatible search data. The
indexing pass also picks up history saved since the last pass.

### Install the new version

Choose the method you already use. For the shell or PowerShell script, rerun
the [binary-only installer command](installation.md#recommended-release-installers)
using the same install directory. If you set `MASH_VERSION`, change it to
the release you want.

| Installed with | Update command |
| --- | --- |
| Homebrew | `brew update && brew upgrade mashium/tap/mash` |
| apt | `sudo apt update && sudo apt install --only-upgrade mash-cli` |
| Scoop | Run `scoop update`, then `scoop update mash`. |
| mise | `mise upgrade github:mashium/mash` |
| cargo-binstall | Rerun the [prebuilt install](install-methods.md#cargo-binstall). |
| Source | [Build the revision you want](install-methods.md#build-from-source), including the web files if you use the dashboard. |

If you installed the first release's Debian package under the name `mash`,
follow the [apt install steps](install-methods.md#apt-debian-or-ubuntu) to
switch to `mash-cli`. If you pinned an exact version in mise, change the pin
to the version you want.

If `mash --version` still shows the old version, check which executable your
shell finds with `command -v mash` on macOS or Linux, or
`(Get-Command mash).Source` in PowerShell. Use the updated executable's path
for the checks above.

### If the index format changes

`mash --version-info` prints the index format the new executable writes
(`silver_schema`) and the oldest one it can upgrade (`silver_schema_min`).
If `mash index` reports a format older than that range, see
[Replace an index that cannot be upgraded](#replace-an-index-that-cannot-be-upgraded).
If it reports a newer format, use the executable that wrote it or a compatible
newer version. Restart any agents, dashboards, or watchers still running the
old executable.

**Saved open questions are deleted.** Upgrading an index written before open
questions were retired as a kind of saved record removes them, with their
revisions and the links that resolved them. Decisions, preferences and facts
stay available. Your export still contains those open questions. Keep the
version that wrote it if you need to read them later.

### Upgrade a shared index

If you [share history across machines](shared-index.md), upgrade the server
and every computer to the same release together. Mismatched releases refuse to
synchronize; that never requires deleting or rebuilding search data.

Back up the server's storage and configuration before upgrading it.
`mash export` on a computer backs up that computer's index. On the server,
upgrade Qdrant if needed, then start the new `mash-server`. It upgrades each
machine's database as it opens it. Resume shared indexing after the server
and its publishing computers are on the same release.

## Make a portable backup

Export to a new directory:

```sh
mash export /absolute/path/to/new-backup
```

This saves your indexed conversations, saved memory, and retained attachments.
For a rollback, also keep the version that wrote the export. Keep your
configuration, service credentials, and original agent files separately.
Mash builds search data again after you import the archive.

**Keep backups private.** They contain the full text of your chats, including
any credentials that appeared in them, even though search results mask those.

## Restore without replacing your current history

Import into a new database file, rebuild its search data, and try a search.
Keep the local search service running:

```sh
mash import /absolute/path/to/new-backup --into /absolute/path/to/restored.sqlite3
mash --db /absolute/path/to/restored.sqlite3 index --rebuild-local-views --full
mash --db /absolute/path/to/restored.sqlite3 search 'a word you remember' --local
```

The restored copy is separate from your usual index: pass `--db` every time
you want to use it. Also pass `--config PATH` or `--models-dir PATH` if yours
are not in the default locations.

The backup must come from a release whose archive format this one reads. If
import fails, keep the backup and use the release that wrote it. In particular,
an archive with saved open questions may need that older version.

## Replace an index that cannot be upgraded

Use this when Mash reports an unsupported older index. Ordinary indexing
cannot upgrade it; keep the old executable and the old index until the
replacement works.

1. Export with the version that wrote the index, then import with the new
   version at a separate path. Add `--db PATH` to the export if the old index
   is outside its default location:

   ```sh
   /path/to/old/mash export /absolute/path/to/new-backup
   /path/to/new/mash import /absolute/path/to/new-backup --into /absolute/path/to/staged.sqlite3 --keep-identity
   ```

   Some older backups cannot be imported either. If import fails, keep the
   original data and executable.

2. `--keep-identity` makes the replacement continue as the same machine on a
   shared index. Stop the old dashboard, watcher, and agent connections
   before switching. Only the replacement should publish after the switch.

3. Move the original database and any `-wal` or `-shm` files to a safe
   location before placing the replacement. Build the replacement's search
   data at its final path and check it.

   ```sh
   mash --db /absolute/path/to/replacement.sqlite3 index --rebuild-local-views --full
   mash --db /absolute/path/to/replacement.sqlite3 status
   ```

4. If the replacement is not at the default location, keep passing `--db`,
   including in your agents' MCP setup. Restart your agents and watchers.

## Other changes you may meet

### Coming from Qdrant 1.19

Mash needs Qdrant 1.20 or later. Before running `mash index` with the new
Mash, stop the 1.19 service and start 1.20 on the same storage.
The first index pass adds a word-search index to your existing search
data without rebuilding it or recomputing embeddings; until then, search
reports that the views need upgrading. On a shared server, starting the new
`mash-server` adds the index.

**Storage opened by Qdrant 1.20 should not be opened by 1.19 again.** Back up
first if you may need to go back.

### Settings that are no longer accepted

- `views.query.experimental_gpu_reranking`: remove it. Reranking runs on the
  CPU. Nothing needs reindexing and no new models are needed.
- Old `[embeddings]` remote-provider settings: replace them with the
  configuration in
  [embedding inference](configuration.md#embedding-inference).
- `redaction.remote_embeddings`: remove it, from node and server
  configuration.

### Changing the embedding model

Vectors from the local model you already use stay valid across upgrades. If
you change to a different embedding model, run `mash models install`, then
rebuild local search data:

```sh
mash index --rebuild-local-views --full
```

On a shared index, change the server and all computers together. Install the
models on the server, then start it once with:

```sh
mash-server --server-config PATH --models-dir PATH --activate-views --replay-views
```

This rebuilds the server's search data from the history it already holds; the
history itself is not replaced. Back up first and expect shared search to be
unavailable during the rebuild.
