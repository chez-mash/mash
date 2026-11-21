# Back up and restore your history

- **Keep a separate copy you can restore.** Use [`mash export`](#make-a-portable-backup)
  before upgrading or uninstalling, or when moving your history to another computer.
- **Save a transcript to read or share.** Use the
  [dashboard export](dashboard.md#export-many-chats-at-once). It gives you files
  to open outside Mash.
- **Reinstall on the same computer later.** Use
  [`mash uninstall --keep-history`](uninstall.md#keep-history-for-a-reinstall) to leave Mash's
  searchable copy and settings in place. Make a separate backup if you need one.

## Make a portable backup

Choose a new directory in a private location, separate from [Mash's data folder](privacy.md#where-mash-stores-data).
Replace the example paths below with absolute paths on your computer.
Keep the quotes around paths with spaces.

```sh
mash export "/absolute/path/to/new-backup"
```

The result reports how many chats, messages, and retained attachments were
copied. Keep the whole backup directory together. It contains the history and
saved memory in this computer's index. For a shared index, run the export on
each computer whose indexed history you want to back up.

**Keep backups private.** They contain the saved chat text, including
credentials that may be masked in search results. Keep your settings, service
credentials, and the original agent history files separately. Keep the Mash
version that wrote the backup if you may need it for an
[upgrade or rollback](upgrades.md#upgrade-an-existing-installation).

## Restore without replacing your current history

Choose a new database path. Mash refuses to import into an existing database.
Import the backup and list a few restored chats to check the copy.

```sh
mash import "/absolute/path/to/new-backup" --into "/absolute/path/to/restored.sqlite3"
mash --db "/absolute/path/to/restored.sqlite3" list-chats --local --limit 5
```

Export, import, and local chat listing work without the search service. If
import reports an incompatible archive, keep the backup and use the Mash
version that wrote it. A replacement that needs to continue publishing under
the same machine identity uses the [upgrade procedure](upgrades.md#replace-an-index-that-cannot-be-upgraded).

## Make the restored chats searchable

Start the [local search service](installation.md#start-the-local-search-service)
and build search data from the restored copy. Search with a word from one of
the chats you listed.

```sh
mash --db "/absolute/path/to/restored.sqlite3" index --rebuild-local-views --full
mash --db "/absolute/path/to/restored.sqlite3" search 'a word you remember' --local --intent exact-mention
```

Keep passing `--db` when you want to use the restored copy. If your settings
or models are in a different location, pass `--config PATH` or `--models-dir PATH`
as well.
