# Linux VM for desktop checks

`tools/linux-vm.sh` runs the Mash desktop app on GNOME in a disposable
Ubuntu 24.04 Desktop ARM64 virtual machine on a Mac reached over SSH
(`dev-macbook` by default), and copies screenshots back. It is a sibling of
the [Windows VM](win-vm.md) tool and uses the same QEMU setup: Hypervisor.framework
(`-accel hvf`), edk2 UEFI firmware, virtio disk, network and display at
1440x900, and VNC on the Mac's loopback driven by `vncdotool` from the macOS
VM tool's virtualenv (`~/scratch/mac-vm/venv`, or set `LINUX_VM_VENV`).

## Commands

```sh
tools/linux-vm.sh build-image                 # autoinstall the golden disk
tools/linux-vm.sh desktop --app Mash_0.3.2_arm64.deb
tools/linux-vm.sh desktop --app target/linux-artifact   # the CI artifact folder
tools/linux-vm.sh desktop --app ... --keep    # leave the clone running
tools/linux-vm.sh list                        # golden disks and clones
tools/linux-vm.sh clean                       # delete clones left by --keep
```

The CI package is in the `mash-desktop-aarch64-unknown-linux-gnu` artifact of
the Desktop workflow, which holds a `.deb` and an AppImage; the tool installs
the `.deb`:

```sh
gh run list --workflow Desktop --branch main
gh run download <id> -n mash-desktop-aarch64-unknown-linux-gnu -D target/linux-artifact
tools/linux-vm.sh desktop --app target/linux-artifact
```

Screenshots land in `target/linux-vm/`: `0-desktop.png` (the empty desktop
before Mash starts), `1-window-screen.png` and `1-window.png` (the screen and
the Mash window), and `2-context-menu-screen.png` and `2-context-menu.png`
after a right-click on the first row of the dashboard's Recent chats, at an
offset from the window's top left corner set by
`LINUX_VM_ROW_X`/`LINUX_VM_ROW_Y` (574, 517 by default, for the window GNOME
maximizes at 1440x900).

With `--keep` the run prints the VNC address (on the Mac's loopback, port
5941 to 5949, password `mashvm`) and the SSH port.

## Where things live

On the Mac, `~/scratch/linux-vm/` holds the synced scripts, QEMU logs, pid
files, build files (`build/`: the seed ISO, the installer kernel,
`progress.png` during an install and `desktop.png` after it) and the last
run's screenshots. `LINUX_VM_STORE`, by default
`/Volumes/Media SSD/mash-vms/linux-vm/` on the external SSD, holds the ISO
and Canonical's `SHA256SUMS` (`iso/`), the golden disks
`ubuntu-golden-<yyyymmdd>.qcow2` with their UEFI variable stores, and clones
under `runs/`.

## Golden image

`build-image` needs `ubuntu-24.04.x-desktop-arm64.iso` in
`$LINUX_VM_STORE/iso/`, from
<https://cdimage.ubuntu.com/releases/24.04/release/>, checked against the
`SHA256SUMS` beside it (or set `LINUX_VM_ISO`).

It boots the ISO's own kernel and initrd with `autoinstall ds=nocloud` on the
command line, with a `CIDATA` seed ISO holding `tools/linux-vm/user-data`,
`finish.sh` and the Mac's public key. The installer then runs unattended:

- `user-data` creates the user `mash` (test password `mash-vm-test`, only in
  the tool's files), installs OpenSSH Server with the Mac's key and the
  desktop app's runtime libraries (WebKitGTK 4.1, GTK 3, Ayatana
  AppIndicator), so a run installs Mash's `.deb` without the network;
- `finish.sh` runs in the installed system: GDM logs `mash` on
  automatically, `mash` may use `sudo` without a password, GNOME defaults to
  the dark style with a plain dark background, no notification banners, no
  screen lock, blanking or sleep, and no welcome tour or initial-setup
  wizard, and the update notifier, unattended upgrades and Apport are off.

The installer powers the VM off; the build boots the installed disk once,
waits for the GNOME session, saves `build/desktop.png` and keeps the disk as
the golden image.

## Run

`desktop` creates a qcow2 clone backed by the newest golden disk, boots it,
waits for `mash`'s GNOME session, and over SSH writes the made-up chats from
`tools/mac-vm/demo_history.py` into the home directory and installs the
`.deb` with `apt-get install`. It then runs `mash init --yes --no-agents
--no-index` and `mash index`, so the app opens on its dashboard with the demo
chats instead of the setup wizard. Ranked search would need Qdrant and the
models, which are downloads; `mash index` saves the chats for word search
before it stops for want of Qdrant, which is all the dashboard needs. Both
commands run behind a dead proxy so they cannot download anything.

The run starts Mash with `systemd-run --user`: an SSH login is not part of
the graphical session, but the user's systemd manager carries the session's `WAYLAND_DISPLAY`, `DISPLAY` and D-Bus address,
which GNOME imports at login, so the app opens on the logged-on desktop.

The session is GNOME on Wayland. Screenshots come from QEMU's display rather
than from inside the session, and the USB tablet moves the pointer, so
neither needs X11. Wayland gives no window list to ask, so the run finds the
Mash window as the region that changed against the empty desktop.

## Limits

- The guest is ARM64, so it tests the `aarch64-unknown-linux-gnu` build
  only.
- The display has no 3D acceleration; GNOME and WebKitGTK render in
  software.
- Updates are off in the image; rebuild from a newer point release to test
  newer packages.
