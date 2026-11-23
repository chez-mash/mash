# Mash desktop app

The Mash dashboard in its own window, with `mash://` links, and Mash in the
menu bar: click the icon (or press ⌥⇧Space) to search as you type. The app bundles
the `mash` binary and runs `mash dash` on loopback; it has no front end of its
own beyond a loading page. Design notes: `docs/internal/desktop-app.md`.

## Build

The app needs the dashboard built into a `mash` binary, placed beside it as
`src-tauri/binaries/mash-<target triple>`:

```sh
(cd dashboard && pnpm install && pnpm build)
MASH_REQUIRE_DASHBOARD=1 cargo build --release --bin mash
mkdir -p desktop/src-tauri/binaries
cp target/release/mash "desktop/src-tauri/binaries/mash-$(rustc -vV | sed -n 's/^host: //p')"
cd desktop && pnpm install && pnpm tauri build
```

On Linux the app also needs WebKitGTK 4.1 and its companions
(`libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev`
on Debian and Ubuntu).

While developing, `pnpm tauri dev` runs the app, and `MASH_DESKTOP_ENGINE`
names a `mash` binary to use instead of the bundled one. Run the unit tests
with `cargo test` in `src-tauri/`.

The app icons in `src-tauri/icons/` are generated from `icons/mash-app.svg`:
render it to `icons/mash-app.png` at 1024 pixels, then run `pnpm icons`. The
macOS menu bar icon is `icons/tray.svg`, rendered with `pnpm tray-icon`, which
uses Playwright's Chromium from the dashboard's packages (run `pnpm install`
in `dashboard/` first). Windows and Linux show the app icon in the tray.

On macOS 26 and later the app's icon comes from `src-tauri/icons/Assets.car`,
compiled from the layered `icons/Mash.icon` (editable in Icon Composer). Its
dark appearance keeps the blue tile, so the icon stays the same when the
system's icon style is Dark. Rebuild it on a Mac after changing the icon:

```sh
xcrun actool icons/Mash.icon --compile /tmp/mash-icon --app-icon Mash \
  --platform macosx --minimum-deployment-target 11.0 --target-device mac \
  --output-partial-info-plist /tmp/mash-icon/partial.plist
cp /tmp/mash-icon/Assets.car src-tauri/icons/Assets.car
```

## Try a link

With the app installed, open `mash://search?q=deploy` from a browser or a
terminal (`xdg-open`, `open` or `start`). A link is the dashboard's route with
`mash://` in place of `#/`.
