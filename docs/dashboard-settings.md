# Change dashboard settings

Open **settings** in the dashboard to change which chats Mash reads and
where it keeps new history. These settings belong to the computer running
the dashboard. If you haven’t set up Mash yet, [choose your first
histories](installation.md#connect-your-histories).

## Choose which histories Mash reads

Under **agent histories**, use the switch beside each agent to choose which
histories Mash reads. The list shows locations Mash found on this computer
and any locations you already configured. Keep at least one history switched
on. To add another folder, [edit the history locations](configuration.md#choose-histories-and-an-age-limit).

Turning a source off stops future indexing from that source. Chats already
indexed stay searchable, including for agents that can access Mash. To limit
agent access, [remove an existing connection](agents.md#disconnect-an-agent).
To delete a specific chat from Mash, [remove the indexed chat](privacy.md#remove-a-chat).

## Change the history range

Under **indexing**, find **history range**. Choose **all history** to include
older chats still on disk, or choose **recent updates** (**recent only**
in older versions) and enter a number of days.

The range is based on when a history file or session was updated. A recently
updated chat can contain older messages. A shorter range applies to future
indexing. Chats that Mash already indexed stay searchable.

## Stop sharing new history

Under **indexing**, find **index into** and choose **this machine only**.
If you already see **this machine only · no shared server is configured**,
indexing is local.

After you save, new index runs keep this machine’s history local. A run
already in progress can finish with the previous setting. Published copies
stay on the server, and this machine can still read shared history if its
shared connection works. [Remove this machine’s published
history or revoke access](shared-index.md#remove-a-machine-or-let-a-rebuilt-one-publish-again).

## Save and apply changes

Saving updates the configuration file. Indexing uses the new choices on its
next run.

1. In the bar at the bottom, choose **Review changes** if you want to compare
   your choices with the saved settings. **Discard** clears unsaved changes.
2. Choose **Save to file**. Mash keeps the previous file beside the current
   one with `.previous` added to its name, usually `config.toml.previous`.
   If Mash says the file changed since you opened Settings, reload the page
   and make your changes again.
3. To index with the updated history choices now, run `mash index` on that
   computer. If a watcher is already running, it reads the saved settings on
   its next pass. If you started the watcher with a range override such as
   `--all-history`, restart it without that flag to use the saved range.
   [Keep history up to date](synchronization.md).

Connected agents use the saved settings on new Mash requests. Start a new
agent session after adding or removing its Mash connection.
[Connect or remove an agent](agents.md).

## Memory, privacy, and Mashy

- Under **indexing**, **memory capture** controls whether the agent plugin
  can capture new decisions and preferences. Turning it off keeps existing
  memories available. [Memory capture settings](configuration.md#memory-capture).
- Under **privacy**, **hide credentials in results** masks recognizable
  credentials in replies. **hide credentials when publishing** masks them in
  history sent to your shared server. With sharing on, the next publication
  after changing this switch replaces this machine’s shared copy using the
  new choice. [Sensitive information](privacy.md#sensitive-information).
- Under **assistant**, you can switch **Mashy** on or off and choose a model if
  you’ve already set it up. The next question uses the saved choices.
  [Set up Mashy](mashy.md#set-up-mashy) or [see what it sends](mashy.md#what-mashy-sends).

## If settings are read-only

A dashboard set up for remote use accepts edits only when service access is
enabled. To make changes on the host, run `mash dash` there and open its local
address, or edit the configuration file shown at the top of **settings**.
[Enable service access](services.md#enable-service-access) or
[edit your configuration](configuration.md).
