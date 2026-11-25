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
is back. The page's two requests, `retry_engine` and `engine_log`, are the
app's only commands and only its own page may call them
(`capabilities/default.json`).

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

"Install Command-Line Tool…" (macOS app menu and tray menu; macOS and Linux)
links the bundled `mash` into `~/.local/bin`. It never replaces anything
already called `mash` there, such as the install script's copy, and says so;
it says whether `~/.local/bin` is on the app's PATH (a terminal's may differ);
choosing it again offers to remove the link. A .deb already puts `mash` in
`/usr/bin`, and an AppImage's files move on every start, so in those cases it
only explains.

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
(Search Mash…, Open Mash, Settings…, Install Command-Line Tool…, Quit Mash);
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
window the dashboard writes a chat into for printing to PDF. The dashboard is
given no Tauri permissions; `withGlobalTauri` is off, and the app's own page
has `core:default` and its two commands. Downloads go to the reader's
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
unsigned installers on Linux (deb, AppImage), macOS (app, dmg) and Windows
(NSIS). RPM and MSI refuse a version such as `0.1.1-alpha`, so they are left
out while releases are pre-releases.

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

## Not done yet

- A Homebrew cask and a winget package for the app. Distribution follows the
  Docker CLI and Docker Desktop split: the CLI stays on install scripts,
  Homebrew, apt, Scoop and winget, and the app is a separate download that
  carries its own engine.
