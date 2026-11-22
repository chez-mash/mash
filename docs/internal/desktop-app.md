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

The dashboard knows it is inside the app only by `window.__MASH_DESKTOP__`,
set by the app's initialization script (`src-tauri/src/window.js`). It uses
that for one thing: copied links are `mash://` links rather than the private
loopback address (`dashboard/src/lib/shareLink.ts`). The same script makes the
title bar follow the dashboard's light or dark theme by watching the root
element's `data-theme`; no dashboard code changes for it.

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

## The window's boundary

The window loads only the app's page and the dashboard's own origin. Any
other address a page tries to open, by link or by a new window, goes to the
reader's browser through `tauri-plugin-opener`. The one exception is the empty
window the dashboard writes a chat into for printing to PDF. The dashboard is
given one Tauri permission, setting the window's theme
(`capabilities/dashboard.json`, scoped to `http://127.0.0.1:*`);
`withGlobalTauri` is off, and the app's own page has `core:default`. Downloads go to the reader's
Downloads folder, numbered so nothing is replaced.

## Building

See `desktop/README.md`. The app is its own Cargo workspace with its own lock
file, so the CLI's build and `cargo deny` never involve a webview toolkit. Its
version is restated in `desktop/src-tauri/Cargo.toml`; a unit test fails when
it differs from the workspace version. `.github/workflows/desktop.yml` builds
unsigned installers on Linux (deb, AppImage), macOS (app, dmg) and Windows
(NSIS). RPM and MSI refuse a version such as `0.1.1-alpha`, so they are left
out while releases are pre-releases.

## Not done yet

- Signing and notarization, and a download channel for the installers.
  Distribution follows the Docker CLI and Docker Desktop split: the CLI stays
  on install scripts, Homebrew, apt, Scoop and winget, and the app becomes a
  separate download (and later a cask and a winget package) that carries its
  own engine.
- Whether the app should keep running in the tray when its window closes.
