# Uninstall Mash

Mash leaves your agents' original conversations in place. Uninstalling can
remove the separate copy Mash made for search, including any saved memories.

| What you want to do | Where to start |
| --- | --- |
| Remove Mash and its local copy of your chats | [Preview removal](#remove-mash-and-its-local-copy) |
| Reinstall on the same computer later and keep your indexed chats and memories | [Keep Mash's copy in place](#keep-history-for-a-reinstall) |
| Save a separate copy to restore on this computer or another one | [Make a portable backup first](backup.md#make-a-portable-backup) |

## Remove Mash and its local copy

Start with a preview. This command lists what would be removed and changes
nothing.

```sh
mash uninstall
```

Read what Mash would remove and what it would keep. The preview includes the
size of Mash's copy of your chats. If the report appears as JSON, look at
`paths`, `kept`, and `next`.

When you're ready, close any Mash dashboard or watcher you started in a
terminal and confirm.

```sh
mash uninstall --yes
```

This removes Mash's index, settings, saved tokens, [usage log](privacy.md#the-usage-log)
and downloaded models. Mash
stops and removes the services it set up, including its search service and that
service's data. It removes both the search tools and the plugin from connected
[agents](agents.md#disconnect-an-agent) through the agents' own commands.

Restart any connected agents that are still open. If the report lists a failed
step, fix the reported issue and rerun the command with the same options.
Once removal succeeds, follow the last instruction to remove the program
itself. In JSON, see `binary.remove`. The instruction depends on how you
installed Mash, such as `brew uninstall mash`, `scoop uninstall mash`,
`cargo uninstall mash`, or which files to delete.

If you installed the `mash-cli` package through apt, run `sudo apt remove mash-cli`.
In `v0.1.1-alpha` and earlier, the removal report can show the wrong package name.

## Copies that stay in place

- Your agents' original chats and unrelated files in `~/.mash` stay on your computer.
- If you've published chats to a shared server, its copy stays there. To remove
  it and revoke this computer's access, follow
  [Remove a machine from the shared index](shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again).
- If you run your own Qdrant service, manage it and its Mash search data
  separately.

To remove only the search service installed by Mash, use `mash qdrant uninstall`.
It keeps the search data unless you pass `--delete-storage`. You can rebuild
that data from the local Mash index with `mash index --rebuild-local-views --full`.

## Keep history for a reinstall

Use `--keep-history` if you plan to return to the same computer and want the
same indexed chats and saved memories. Mash keeps its index, configuration,
saved tokens, usage log, and the data for its managed search service. It removes the
services, agent connections and downloaded models.

Preview with the flag, then use the same flag when you confirm.

```sh
mash uninstall --keep-history
```

Confirming with `--yes` alone deletes Mash's indexed chats and saved memories.
In `v0.1.1-alpha` and earlier, the preview can leave `--keep-history` out of the
suggested confirmation command, so type the full command.

```sh
mash uninstall --yes --keep-history
```

Before removing the program, run `mash --version` and save the version number
for the reinstall. Follow the same steps to retry failures, restart connected
agents, and remove the program as in
[Remove Mash and its local copy](#remove-mash-and-its-local-copy).

### Reopen the retained history

1. On the same computer and user account, install the Mash release you noted.
   Set `MASH_VERSION` to its tag, such as `v0.2.0`, before running the
   [basic installer](installation.md#recommended-release-installers).
   To install from an archive, open that tag on
   [GitHub Releases](https://github.com/mashhq/mash/releases) and download
   the archive for your system and `SHA256SUMS` from that release.
   [Verify the archive](downloads.md#verify-the-checksum) before extracting it.
   To install a newer release, follow the
   [upgrade guide](upgrades.md#upgrade-an-existing-installation).
2. If Mash installed the search service, run `mash qdrant install` to reinstall
   the service and reuse its data. If you [ran Qdrant yourself](local-search.md#run-qdrant-yourself),
   start it the same way as before.
3. Run `mash dash` and open the address it prints, or run
   `mash list-chats --local --limit 5` to check the saved chats in your
   terminal. Mash uses the saved settings and index. If you used custom
   paths, pass them again.

If search needs a refresh, see [Keep your history up to date](synchronization.md).
If you used search by meaning, [restore its models](indexing.md#install-the-models).
You can also [reconnect an agent](agents.md#connect-claude-code-codex-or-copilot).
