# macOS VM for Raycast checks

`tools/mac-vm.sh` runs the Raycast extension in a disposable macOS virtual
machine on a Mac reached over SSH (`dev-macbook` by default) and copies the
screenshots back. Agents can drive Raycast there without Accessibility or
Screen Recording grants on anyone's real Mac, and the guest only ever sees
made-up history.

The VMs run under [Lume](https://github.com/trycua/cua) (Apple's
Virtualization framework). Screenshots, clicks and keys go through the VM's
own VNC server on the Mac's loopback, driven by `vncdotool` in a virtualenv
under the Mac's working directory, so neither the Mac nor the guest grants
anything to the driver. Over that VNC connection Alt is Command and Meta is
Option.

## Commands

```sh
tools/mac-vm.sh build-image            # build mash-golden-<yyyymmdd> on the Mac
tools/mac-vm.sh push-image             # store it as a private GHCR package
tools/mac-vm.sh run                    # screenshots of the latest Mash release
tools/mac-vm.sh run --mash mash-aarch64-apple-darwin.tar.gz   # of a build
tools/mac-vm.sh run --keep             # leave the clone running to look around
tools/mac-vm.sh store-shots            # Raycast Store screenshots into raycast/metadata/
tools/mac-vm.sh clean                  # delete clones left by --keep
tools/mac-vm.sh desktop --app mash-desktop.dmg   # the desktop app in a clone, kept running
```

Screenshots land in `target/mac-vm/`: `light-` and `dark-` versions of
`1-search` (Search Mash, "checkout"), `2-glance` (⌘Y), `3-actions` (⌘K) and
`4-memory` (Search Memory), each as the Raycast window and as the full
1440x900 screen (`-screen.png`). A run takes about two and a half minutes.

## Store screenshots

`store-shots` boots the same kind of clone with a 2880x1800 display, switches
macOS to its scaled 1440x900 mode (installing `displayplacer` in the guest),
lets SSH capture the screen, sets a plain grey wallpaper and light mode, and
captures five scenes with the guest's own `screencapture`: Search Mash for
"customers" (date groups), the glance beside the list (⌘Y), ⌘K, Search Memory
with a decision selected, and the latest chats. `tools/mac-vm/store_frame.py`
then places each window, at its native 2x pixels, in the middle of a
2000x1250 canvas of the wallpaper colour, as Raycast's Window Capture does,
and writes `raycast/metadata/mash-1.png` … `mash-5.png`. It fails instead of
framing a capture where it does not find exactly one Raycast-sized window.
The full screens stay in `target/mac-vm/store/screen/`; look at every image
at full size before committing it.

The first capture makes macOS ask whether SSH may bypass its window picker;
the run clicks Allow over VNC, so the guest must stay at its default layout.

## Golden image

`build-image` starts from Cua's `macos-tahoe-cua:26.5.2` image (user and
password `lume`, SIP off, SSH on) and installs, through Homebrew: Node 22,
Git, Raycast, and the agents Mash detects (Claude Code, Codex, OpenCode,
GitHub Copilot CLI, Cursor and Antigravity). It then:

- writes the made-up chats in `tools/mac-vm/demo-chats.json` into each
  agent's own history location and format (`tools/mac-vm/demo_history.py`);
- marks Raycast's onboarding done and turns off its automatic updates
  (`DisableAutomaticUpdates`), and turns off macOS software updates;
- answers macOS's one-time question whether SSH may control System Events,
  which the scenario uses to switch between light and dark;
- rewrites `/etc/kcpassword`, which the Cua image encodes without the cipher's
  padding. With the wrong automatic-login password, macOS replaced the login
  keychain at every boot and Raycast lost its database key ("Encryption Key
  Not Found"). The build reboots and fails if the keychain is replaced again.

The image has no Mash, no Raycast extension and no desktop app: each run
installs the build under test into a fresh clone, so an image never goes
stale with a Mash version. Nothing in the image is signed in and no agent has
a key, so none can reach a model. Raycast needs no account to run a
development extension.

Rebuild the image when Raycast, an agent or macOS needs updating, when the
demo histories change, or monthly; a build takes about 20 minutes. Delete old
`mash-golden-*` VMs on the Mac with `lume delete` once a new one works: clones
share unchanged blocks, but each image keeps its own writes.

Clones take over their source's machine identifier, so the guest sees the
same hardware and its keychain items stay readable; only one of them runs at
a time.

## Private only

The image contains macOS, which Apple licenses to run only on Apple hardware.
Keep it on the Mac or in the private package `ghcr.io/mashhq/mash-golden`.
Never make that package public, push the image anywhere else, or attach it to
a release.

`push-image` pushes the newest local image as `:<yyyymmdd>` and `:latest`
with Lume's registry push, then asks GitHub for the package's visibility and
fails unless it is `private`. `run` pulls `:latest` when the Mac has no
golden image. Both take credentials from `gh` on the machine running
`tools/mac-vm.sh` and pass them to the Mac on standard input, so the token
never appears on a command line or in a file; pushing needs the
`write:packages` scope (`gh auth refresh -h github.com -s write:packages`),
pulling `read:packages`.

## Run

A run clones the newest `mash-golden-*` VM, boots it headless, and in the
guest (`tools/mac-vm/guest-run.sh`):

1. installs the given Mash binaries, or the latest release, into
   `~/.local/bin`;
2. rewrites the demo histories with current times, installs Qdrant and the
   models (cached on the Mac after the first run), runs `mash init --yes
   --no-agents` and `mash index`, and records the demo decisions through
   `mash mcp` (`tools/mac-vm/demo_records.py`);
3. copies `raycast/` from the working tree, runs `npm ci` and `npm run dev`.

The Mac side then answers Raycast's one-time "Always Run Command" question for
both commands, closes the "Login Item Added" and "App Background Activity"
notices that Raycast and Qdrant leave on screen, takes the screenshots, and
deletes the clone. Without `--mash`, a run tests the latest published release.

## Lume or Tart

Lume was chosen for agent-driven checks: it has a scriptable CLI, clones in a
second through APFS copy-on-write, and Cua publishes macOS images prepared for
automation. Tart would work the same way (it is also an Apple Virtualization
front end) and stays the fallback if Lume stops fitting.
