# Desktop app

`desktop/` holds the Mash desktop app: a Tauri 2 window that shows the Mash
dashboard. It exists so people who never run `mash dash` still find the
dashboard, and so a page of it can be linked from anywhere with `mash://`.

## One dashboard

The app has no front end of its own. It bundles the `mash` binary (a Tauri
sidecar, `bundle.externalBin`), starts `mash dash` on loopback, and loads the
dashboard into its window from that address, exactly as a browser would. Every
page, theme, route and API is the dashboard's; a change to the dashboard
reaches the app with no change here. The only page the app serves itself is
`ui/index.html`, shown while the engine starts and when it fails, and it draws
with the dashboard's own theme tokens, mark and typefaces, which
`src-tauri/build.rs` copies in at build time.

The engine runs as the signed-in user with the user's normal `~/.mash`, so the
app and the CLI share one configuration and one index. The app starts the
engine with `--listen 127.0.0.1:47290` and falls back to a free port when that
is taken. The fixed port matters because the dashboard keeps its theme, rail
width and Mashy's position in browser storage, which belongs to one origin.
The app records the engine's process number in `engine.pid` in its data
folder; if an earlier run ended without quitting (a crash or a forced quit)
and left its engine holding the port, the next start stops that engine and
takes the port back. It does so only when the recorded process is a `mash`
and a Mash dashboard answers on the port; anything else there is left alone,
the app moves to a free port, and the move is noted in the engine's output.

On quit the app asks the engine to stop (SIGTERM on macOS and Linux, which
`mash dash` treats like Ctrl+C; Ctrl+C through the engine's console on
Windows) and ends it only if it is still running three seconds later. The
app keeps the engine's last 40 lines of output. When the engine fails to
start or stops, the main window comes forward with the app's error page,
which offers Try again (start the engine again), shows those lines with a
Copy button, and names `mash doctor` by the app's own binary; the search
panel, whose page belonged to the stopped engine, is closed until the engine
is back. The page's two requests, `retry_engine` and `engine_log`, are
commands only the app's own page may call (`capabilities/default.json`).

The dashboard knows it is inside the app only by `window.__MASH_DESKTOP__`,
set by the app's initialization script (`src-tauri/src/window.js`). It uses
that for a few things: copied links are `mash://` links rather than the
private loopback address (`dashboard/src/lib/shareLink.ts`); a PDF export asks
the app to print (below); the export page says Mash, not the browser, saves
the file; and commands it asks the reader to run name the app's own `mash`.
The window's title bar is light, like the dashboard, which has no dark theme.

## The mash command

The app does not put `mash` on PATH, so a dashboard page that says "run mash
index" would send an app user to a command that does not exist. Inside the
app the initialization script also sets `window.__MASH_DESKTOP_COMMAND__`, the
bundled binary's full path quoted for the shell (`src-tauri/src/cli.rs`), and
the dashboard writes commands with it (`dashboard/src/lib/command.ts`). Where
the dashboard can do the job itself it offers that instead: setup's first
index has an Index now button, and Settings points to the setup page rather
than `mash init`.

"Install Command-Line Tool…" (macOS app menu and tray menu) puts a `mash`
command in place that runs the bundled `mash`, so one update covers the app
and the command. On macOS and Linux it goes in `~/.local/bin`. It never replaces anything already
called `mash` there, such as the install script's copy, and says that copy
updates separately; it says whether `~/.local/bin` is on the app's PATH (a
terminal's may differ); choosing it again offers to remove the command.

On macOS the command is a small shell script, not a link. It looks for the
app where it was when the command was written, then in `/Applications` and
`~/Applications`, and replaces itself with that app's `mash` (`exec`), so the
arguments, standard input and output, the exit status and signals are those
of `mash` itself, which `mash mcp` and `mash dash` depend on. When the app
has been moved somewhere else or removed, it prints one line saying so and
exits 127, where a link would fail with "No such file". The script's second
line (`# mash-desktop-command`) is how the app knows the file is its own.

On Linux the command stays a separate install. An AppImage's files move on
every start, so nothing may point into it; the `mash-desktop` apt package
depends on `mash-cli` at its own version and runs that package's
`/usr/bin/mash`; and for any other copy the item makes a link as before.

### On Windows

The installer brings the command with it (`src-tauri/windows/hooks.nsh`,
`src-tauri/src/cli/`). After installing the files it starts the app with
`--command-line-tool installed`, and the app copies its `mash.exe` into a
`bin` folder of the install folder (`%LOCALAPPDATA%\Mash\bin`) and appends
that folder to the user's own PATH (`HKCU\Environment`, keeping the value's
type and every other entry as written), then tells running programs that
the environment changed (`WM_SETTINGCHANGE`), so terminals opened after it
find `mash`. The install is per user, so none of this needs an administrator.
Before the uninstaller removes the files it starts the app with
`--command-line-tool uninstalling`, which takes the entry off PATH and
deletes the copy and its folder (the uninstaller also deletes those files
itself, in case the app could not run, such as when an antivirus holds an
unsigned build back; the PATH entry is then left naming a folder that is
gone); an install over an earlier version
(`/UPDATE`) skips that and refreshes the copy instead. Neither request starts
the app itself, and a failure never fails the installer: the menu item puts
the command in place later. The same hook registers `QuietUninstallString`
(`"<install folder>\uninstall.exe" /S`) beside Tauri's `UninstallString`,
which `winget uninstall --silent` runs.

Why a real `.exe`, and why a copy:

- A program started without a shell, as agents start `mash mcp` (a
  long-lived stdio server), is found by `CreateProcess`, Node's `spawn` or
  Rust's `Command`, which add `.exe` to a bare name but never run a `.cmd`
  or `.ps1`. A `mash.cmd` shim would work in cmd and PowerShell and fail for
  exactly the agents the command is for, and would put `cmd.exe` between an
  agent and the server's input and output. A small launcher `.exe` would be
  a second program to build, sign and keep transparent to Ctrl+C.
- A folder of its own, because the install folder also holds the app and
  `uninstall.exe`, which do not belong on PATH.
- A copy rather than a hard link: a `mash mcp` running from the command
  holds only the copy open, so the installer can always replace the app's
  `mash.exe`. A copy that is running cannot be overwritten but can be
  renamed, so it is moved aside (`mash.old-N`) and deleted the next time the
  command is placed. The copy costs the disk space of a second `mash.exe`.
- The copy carries the engine's size and modification time; at launch the
  app replaces a copy that no longer matches, in case something replaced the
  app without its installer.

The folder is appended, never put first, so a `mash` already on PATH
(winget's, Scoop's, the install script's) keeps winning and nothing is
shadowed. The item then says, in the words the Mac app uses for a separate
copy, that the other copy comes first and updates separately, and `mash
doctor` reports both (One Mash on the machine). Chosen when the command is
in place, the item offers to remove it; a removal is remembered
(`command-line-tool-removed` in `%APPDATA%\io.getmash.desktop`), so a later
install leaves the command off until the item is chosen again.

The installer puts the command in place by default because the point of the
Windows app is that one download gives a working `mash` for terminals and
agents, and appending cannot take over another copy; the item is there to
remove it. Only `mash` is exposed: the app does not carry `mash-server`,
which belongs to the shared service and its own install.

## One Mash on the machine

When the app is installed, its bundled `mash` should be the only one in use,
or an update leaves a watcher or an agent's connection on the version before:
the mismatch whose schema check then stops search. The app does not work out
what else is installed. `mash doctor` does, in its `copies` check
(`app/core/doctor/copies.rs`): the `mash` a login shell finds, the program
the dashboard and watcher service definitions start, the build a dashboard
service answers with, and each agent's registered command, each compared
with the running copy by file and, failing that, by the build `mash
--version-info` reports. Another `mash` later on the process's PATH is
reported too, so whichever of two copies runs `mash doctor`, the other is
named, such as the Windows app's command behind a winget copy. It names
Homebrew, Scoop, winget or apt when the path shows one installed the other
copy, and recognises the Windows app's command by its place (`bin\mash.exe`
beside the app's `mash.exe` and `uninstall.exe`). The existing `processes` and `watcher` checks
cover what is already running. The schema and contract refusals in the
commands themselves are unchanged and remain what actually stops a mismatch.

At launch the app runs the bundled `mash doctor` and, when `copies` reports
anything, shows one notice (`src-tauri/src/lockstep.rs`). On macOS it offers
**Use the App’s Mash Everywhere**, **Keep Separate** (remembered) and **Not
Now**. Everywhere does three things and then says what it did and what is
left:

1. Installs the command as above. A link to another `mash` is re-pointed; a
   separate copy in `~/.local/bin` is kept beside it as `mash.separate`.
2. Runs `mash init --yes --no-index` from the bundled binary when the doctor
   names it as a repair, which registers the bundled `mash` through each
   agent's own CLI.
3. Restarts the services whose definitions name the bundled `mash` or the
   command.

Nothing a package manager installed is removed. A Homebrew or Scoop copy
that a shell still finds first stays in the notice with its own update
command; a dashboard service whose definition names another copy stays with
`mash dash remote`, the command that rewrites it. On Linux and Windows the
notice only says the versions differ, with **Don’t Show Again**.

## Updates

The app updates itself with `tauri-plugin-updater` (`src-tauri/src/update.rs`
and `update/`). A build has updates on when `plugins.updater.pubkey` in
`tauri.conf.json` is not empty and it is installed as something the updater
replaces: the `.app` bundle on macOS or an AppImage on Linux. A build
without them still learns that a newer release exists and says so in the
banner (see The banner); its **Check for Updates…** brings that banner
forward and, with nothing newer, opens the downloads page, or for the apt
package names the `apt` command that brings a new version.
Windows is off until the installer's handover has been tested on a real
machine: a running `mash.exe` cannot be replaced, so the order there must be
stop the watcher, the dashboards and the engine, install, start.

### Channels

| Channel | Reads | Published by |
| --- | --- | --- |
| Stable (default) | `releases/latest/download/latest.json` | `release.yml`, for a tag |
| Main (latest build) | `releases/download/main-channel/latest.json` | `desktop.yml`, for each build of main |

`latest.json` names the version and, per platform, the update file and its
signature (`tools/desktop_updates.py manifest`). A build of main is versioned
as a prerelease of the release that will follow it: `0.2.5-main.118` after
0.2.4, where 118 is the workflow's run number (`desktop_updates.py
main-version`). By semantic-version order that is newer than 0.2.4, older
than build 119 and older than 0.2.5, so stable to main, main to main and
main to the next release are all ordinary "newer" updates
(`update/channel.rs`, with tests on both sides). Nothing moves backwards:
someone who returns to Stable keeps their build of main until the release
after it, and Stable never takes a build of main whatever its address
serves. `requireSignedVersion` makes the updater refuse a manifest that pairs
a version with another version's validly signed file.

The channel decides which versions are offered and nothing else. It is the
app's own setting (`updates.json` in its configuration folder), changed from
**Updates: Stable / Main (latest build)** in the menu bar menu and the app
menu. Earlier versions also kept an idle-install choice in that file; it is
no longer read, and a file that still has it loads as before.

### What happens

There is one way an update is installed: the person chooses **Update and
restart** in the banner, or the same thing on the menu line. Nothing is
installed without that choice, on either channel. In the background the app
only checks, at launch and every six hours, and downloads a newer version
and verifies it against the public key, so that the choice is quick.

The menu line follows the banner and calls the same request
(`install::request`), bringing the main window forward so the banner shows
how it goes:

| State | Menu line | Chosen |
| --- | --- | --- |
| Nothing newer | **Mash (version)** | nothing |
| Checking | **Checking for Updates…** | nothing |
| A version downloaded, or still arriving | **Mash (version) Is Available — Update and Restart** | installs, after the download if need be |
| Arriving, after that choice | **Downloading Mash (version)…** | nothing |
| Installing | **Installing Mash (version)…** | nothing |

**Check for Updates…** that finds a version, or already has one, brings the
main window forward with the banner in it (a **Later** chosen before no
longer hides it) and tells the dashboard to ask at once. With nothing newer
it says the running version is the newest on the channel; a check that
fails says why. Only if the window cannot be shown does a notice name the
menu line instead.

Installing (`update/install.rs`): the download is read back and verified
again, so a file changed on disk is refused with the app untouched; the
engine stops; the installed app is copied to `previous/` in the app's data
folder; the updater swaps the bundle; the app restarts. If the swap fails
the engine starts again and the app carries on.

The first start of the new version (`update/after.rs`) runs the bundled
`mash status` before starting the engine and, when it refuses the index for
a schema that indexing upgrades in place, runs the bundled `mash index`,
which is what `tools/upgrade-local-index.sh` does for a deployment. A schema
that needs export and import is left to the engine's own error. Once the
engine is up the kept copy is deleted, the services that run the app's
`mash` are restarted (`launchctl kickstart -k`, `systemctl --user restart`),
and `mash doctor` is asked again. Anything still on the earlier version,
and any `mash dash` someone started by hand (listed, never stopped), goes
into one notice.

### The banner

A newer version is also a banner across the top of the main window, drawn by
the dashboard (`UpdateBanner.svelte`, `lib/appUpdate.svelte.ts`) from what
the app tells it (`update/banner.rs`). The dashboard asks through three
commands that only the main window's dashboard page may call
(`capabilities/updates.json`): `update_banner` (what there is to offer),
`update_install` and `update_later`. It asks every 20 seconds, every
second while the app downloads or installs, and at once when the app sends
the page a `mash-update` event. A dashboard in a browser has no
app to ask and never shows the banner; the `mash` command has no equivalent.

| Build | The banner offers | Button |
| --- | --- | --- |
| Updates itself | the version it has downloaded or is downloading | **Update and restart** |
| No key, Windows, a development build | the newer stable release | **Download update** (the downloads page) |
| The apt package | the newer stable release | **Copy command** (the `apt` command, shown) |

**Update and restart** installs the downloaded update through the same
request as the menu line (`install::request`); chosen while the download is still
arriving, the banner shows the download's progress and the install follows
as soon as the file has been verified. While the app installs, the banner
says so and offers nothing. An install that fails, wherever it was chosen,
leaves the banner saying why with **Download instead**, which opens the
downloads page. **Later** hides the banner for that version until the app
starts again; a newer version brings it back, and so does **Check for
Updates…** or the menu line. The banner never installs anything by itself,
and it leaves the channel as it is.

A build that does not update itself has no `latest.json` to read, since
none is published without the key, so it reads GitHub's record of the
latest release instead (`update/release.rs`,
`api.github.com/repos/mashhq/mash/releases/latest`), at launch and every six
hours like the updater, and compares the tag with its own version by the
stable channel's rule (`channel::offers`). That is its only request, it
carries nothing but the app's version in its `User-Agent`, and a check that
fails says nothing. There is no setting that turns update checks off;
if one is added, this check must follow it.

If the new version's engine does not start, the menus gain **Go Back to
Previous Version**, which puts the kept copy back, restarts, and records the
version so it is not installed again (a later one is). If the app itself
cannot start, the copy is still in
`~/Library/Application Support/io.getmash.desktop/previous/`.

### The key

Updates are signed with a Tauri updater key (minisign), separate from the
Apple and Windows code-signing identities. `tools/updater-key.sh`, run once,
makes it in `~/.config/mash/updater/` (private key, public key, password;
mode 0600), writes the public key into `tauri.conf.json` and uploads
`TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to the
repository's secrets. Until both the public key and the secrets exist, the
workflows build as before with a notice, publish no `latest.json`, and the
app has updates off. Back the folder up: an app installs only what this key
signed, so a lost key means one manual reinstall for everyone.

### Signing on the main channel

An update the app installs itself is not quarantined, so Gatekeeper does not
check notarization when it starts. What must not change is the signing
identity: macOS ties privacy permissions and keychain items to the app's
designated requirement (bundle identifier and Developer ID team), and an
unsigned build replacing a signed one would lose them. `desktop.yml`
therefore signs builds of main with the Developer ID when the Apple secrets
are set, as `release.yml` does, and Tauri notarizes and staples the app in
the same step, so a build of main downloaded by hand also opens normally.

What was checked, in a disposable macOS VM: two builds signed with one
identity and not notarized update one to the other and start, with the same
designated requirement before and after and no quarantine attribute, though
Gatekeeper rejects the app when asked to assess it. The released app's
requirement names only the identifier and the team (`identifier
"io.getmash.desktop" … certificate leaf[subject.OU] = JZ665W692Z`), so it does
not change between versions. The bundled engine is signed as its own program
(`identifier mash`, same team): a launchd job that runs it is a `mash` to
macOS, not Mash.app. Login Items lists such a job under the path its
definition names, so a definition that names the `~/.local/bin/mash` script
shows as an unidentified "mash", while one written by `mash dash remote`
names the engine inside the app. Not checked: whether a Files and Folders or
Full Disk Access grant made to a launchd-run engine survives an update. The
VM image does not ask for those permissions at all, so there was no grant to
follow; the unchanged requirement is the reason to expect it to survive.

### Taking a bad build off the main channel

Run `desktop.yml` from main with the last good commit:

```sh
gh workflow run desktop.yml --ref main -f commit=<good commit>
```

It is built again under a higher build number, which every app on the
channel takes as its next update, including those that installed the bad
build. Deleting the `main-channel` release's `latest.json` stops updates at
once in the meantime. Do not upload an older build's files by hand: their
lower version would be ignored by the apps that most need it.

### Testing

`cargo test` covers the channel order, the Windows command's PATH edits,
its copy and the words it uses, what the banner shows in each state,
reading a release's version, the settings, keeping and restoring
the previous app, the command script (arguments with spaces, input, exit
status, one process, a missing or moved app), the service and process
listings, and the notices. An end-to-end run needs two builds signed with a
throwaway key (`tauri signer generate`), passed with
`--config '{"version":…,"bundle":{"createUpdaterArtifacts":true},"plugins":{"updater":{"pubkey":…,"dangerousInsecureTransportProtocol":true}}}'`.
Only a build with that insecure-transport setting reads
`MASH_DESKTOP_UPDATE_ENDPOINT`, which points it at a local file server in
place of GitHub.

## mash:// links

A link is the dashboard's route with `mash://` in place of `#/`:
`mash://chat/abc?node=laptop` opens `#/chat/abc?node=laptop`. The app checks
only that a link is well formed (`src-tauri/src/links.rs`); the dashboard's
router decides what it means and shows its not-found page for anything else,
so the app never needs to learn about new pages.

Links reach the app through `tauri-plugin-deep-link`. A second launch (the OS
starting the app for a link on Windows and Linux) hands its link to the
running app through `tauri-plugin-single-instance` and quits. Links that
arrive before the engine is ready wait for it. Installers register the scheme;
the app also registers it at start on Windows and Linux so an AppImage or a
development build works.

## The menu bar and the search panel

The app lives in the menu bar (`src-tauri/src/menubar.rs`). Clicking its icon
drops a search panel down under it; right-clicking opens a native menu
(Search Mash…, Open Mash, Settings…, Install Command-Line Tool…, the update
items, Quit Mash);
⌥⇧Space (Alt+Shift+Space) and `mash://quick` drop the same panel from the
same icon, like a menu bar extra; it never appears mid-screen. When the icon
is on no screen (a menu bar manager such as Ice or Bartender hides it off
screen, or the system gives no position), the panel drops from the top right
of the screen with the pointer. Positions are physical pixels in the
desktop's one coordinate space, so the screen showing the icon, and its own
scale, are found by containment; the panel stays inside that screen's work
area, and rises above an icon in a taskbar at the bottom of the screen. ⌥Space is left to launchers such as
Raycast and Alfred, and the shortcut is registered after start so an app that
already holds it costs only the shortcut. The tray menu names only shortcuts
that work: the global one, and on macOS ⌘, and ⌘Q, which the app menu
(the standard one plus Settings… and Install Command-Line Tool…) provides
while Mash is active. On macOS and Windows, closing the main window leaves
Mash in the menu bar; on Linux, where not every desktop shows tray icons, it
quits the app. Should the window be gone anyway, anything that shows it (the
tray, a relaunch, a mash:// link) opens it again.

On macOS the tray icon is `icons/tray.svg` drawn as a template image in the
menu bar's colour, rendered to `src-tauri/icons/tray.png` with `pnpm
tray-icon` (Playwright's Chromium from the dashboard's packages). Windows and
Linux cannot tint a tray icon to suit a light or dark bar, so there it is the
app's full-colour tile (`src-tauri/icons/64x64.png`).

The panel is the dashboard's `#/quick` page in a second, borderless window,
so its live search, recent chats, agent logos and links are the dashboard's
own (`dashboard/src/components/QuickSearch.svelte`, `dashboard/src/lib/quick.ts`).
It is drawn in the dashboard's theme (Geist, its tokens, a neutral selected
row), light or dark as the system is, and the window's title-bar theme sync
skips it. Before anything is typed it lists recent chats and suggests full
search, Memory and Machines; ⌘K (Ctrl+K) opens the selected chat's actions,
the ones the chat page offers. The window is too narrow for the preview
of the selected result, which shows only where the page is 600 pixels wide or
more, as in a browser. Its rows and menu open mash:// links, which the app
follows in the main window before hiding the panel (an agent's own link goes
to that agent's app, as from the main window); the only Tauri permission the panel has is hiding its own window
(`capabilities/quick.json`). It hides whenever it loses focus, and it is on
every desktop (Space). The window is
transparent, for its rounded edge, on macOS and Windows only: Linux desktops
without a compositor paint a transparent window black. On macOS the window is
drawn on the system's popover material (a Tauri window effect) and the page,
told so by `window.__MASH_PANEL_MATERIAL__`, uses the translucent
`--quick-surface-material`, so the panel looks like a menu bar extra in
light and dark. Tauri's window theme sets the appearance of the whole app on
macOS, so the main window's light title bar is set on that NSWindow alone
(`light_title_bar` in `lib.rs`); otherwise the panel and the app's notices
would stay light in dark mode. Showing it over
another app's full-screen window on macOS would need a non-activating panel,
which Tauri does not offer, so that is not attempted.

## The window's boundary

The window loads only the app's page and the dashboard's own origin. Any
other address a page tries to open, by link or by a new window, goes to the
reader's browser through `tauri-plugin-opener`, if it is a web or mail
address, or to its agent's app, if it is a `codex://`, `claude://` or
`ghapp://` link from the chat page's Open in Codex, Claude or Copilot; other
schemes are dropped. The
one exception is the empty
window the dashboard writes a chat into for printing to PDF. The dashboard in
the main window is given no Tauri plugin and three of the app's commands, the
update banner's (`capabilities/updates.json`; see Updates);
`withGlobalTauri` is off, and the app's own page has `core:default` and its
two commands. Downloads go to the reader's
Downloads folder, numbered so nothing is replaced, and when one finishes a
notice names the file and offers Show in Finder (Show in Folder elsewhere)
(`src-tauri/src/downloads.rs`).

Printing: in a browser the PDF export writes the chat into an empty tab and
calls its `print()`. A webview has nothing behind `print()` (WKWebView
ignores it), so the app opens that tab as a window of its own
(`src-tauri/src/print.rs`), and the dashboard, inside the app, sends the tab
to `mash://print` once the chat is written instead of calling `print()`. The
app catches that address in the tab's window and prints the window natively;
the system print dialog offers Save as PDF. The tab stays open with the chat
until the reader closes it.

## Building

See `desktop/README.md`. The app is its own Cargo workspace with its own lock
file, so the CLI's build and `cargo deny` never involve a webview toolkit. Its
version is restated in `desktop/src-tauri/Cargo.toml`; a unit test fails when
it differs from the workspace version. `.github/workflows/desktop.yml` builds
installers on Linux (deb, AppImage), macOS (app, dmg) and Windows (NSIS) for
pull requests and for main, and publishes the main update channel (see
Updates). On Windows it then installs the app silently, runs the `mash`
command from cmd, PowerShell and with no shell, `mash mcp` included, and
uninstalls it again (`.github/scripts/windows-command-check.ps1`, which also
runs by hand on a Windows machine, one step per new session). RPM and MSI refuse a version such as `0.1.1-alpha`, so they are
left out while releases are pre-releases.

## Releases and signing

`.github/workflows/release.yml` builds each installer from the engine the
release built for the same platform, and attaches it under a fixed name
(`mash-desktop-<system>-<arch>…`) so the website can link to the latest one.
Signing turns on by itself when the repository has the secrets, and is skipped
otherwise:

- macOS: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_API_KEY_P8`,
  `APPLE_API_KEY_ID` and `APPLE_API_ISSUER` (uploaded with
  `tools/apple-signing-secrets`). `.github/scripts/macos-signing-setup` imports
  the certificate; Tauri signs with the hardened runtime and notarizes, and the
  disk image is notarized after it.
- Windows: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`,
  `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT` and `AZURE_SIGNING_PROFILE`
  for Azure Trusted Signing. `.github/scripts/windows-sign.ps1` signs `mash.exe`
  and `mash-server.exe`, and Tauri calls it as its sign command for the app and
  the installer. It checks every signature it makes.

With the updater key in place the release also signs the macOS and Linux
builds for the app's updater and publishes `latest.json`, the stable
channel's manifest (see Updates).

On Linux the release also builds the app's `.deb`, which
`packaging/apt/desktop-deb.sh` turns into the `mash-desktop` package of the
apt repository. Tauri's package is named `mash` and carries its own
`/usr/bin/mash`, the file `mash-cli` installs. Debian policy has one package
own a file, so `mash-desktop` drops its copy and declares `Depends: mash-cli
(= its own version)`: the app runs `mash-cli`'s binary, `apt upgrade` moves
both together, and removing `mash-cli` removes the app. The alternative,
shipping the binary in `mash-desktop` with `Conflicts`, `Replaces` and
`Provides` against `mash-cli`, would take `mash-server` away from anyone who
installed the app and split one program across two packages.

A fleet deployment of a release can put the signed Mac app on a machine:
`tools/fleet.sh deploy --release vX.Y.Z` runs `tools/install-desktop-macos.sh`
for Macs whose inventory entry asks for it. It checks the disk image against
the release's checksums, requires Gatekeeper's approval and Mash's team ID,
and keeps the previous app until the new one has started. See
[deployment/README.md](../../deployment/README.md#deploying-built-artifacts).

## Not done yet

- Updates on Windows (see Updates). When they come, the installer's
  `/UPDATE` run already refreshes the `mash` command, but a running command
  (an agent's `mash mcp`) keeps the build before until it restarts, as on
  the other systems.
- Publishing the app's winget package. The manifests exist
  (`packaging/winget/MashHQ.Mash`, checked against each release by
  `release-check.yml`'s `winget` job) and wait on Microsoft accepting the
  first submission; no user guide gives a winget command until then. In
  winget the app is `MashHQ.Mash` and the command line is `MashHQ.Mash.CLI`;
  the app's package needs no dependency on the command line's, since the app
  installs its own `mash` command. Distribution otherwise follows the Docker
  CLI and Docker Desktop split: the CLI stays on install scripts, Homebrew,
  apt, Scoop and its own winget package, and the app is a separate download
  that carries its own engine, except under apt, where the app's package uses
  `mash-cli`'s.
- The Windows installer is not signed, so SmartScreen stops a `winget
  install MashHQ.Mash` at "Windows protected your PC" until the user chooses
  Run anyway (see `packaging/winget/README.md`).
