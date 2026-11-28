# Windows VM for desktop checks

`tools/win-vm.sh` runs the Mash desktop app on real Windows 11 in a
disposable virtual machine on a Mac reached over SSH (`dev-macbook` by
default), and copies screenshots back. It is the Windows counterpart of
`tools/mac-vm.sh` (see [mac-vm.md](mac-vm.md)) and has the same shape: a
script here syncs `tools/win-vm/` to the Mac and calls
`tools/win-vm/mac.sh` there.

The VM is Windows 11 ARM64 under QEMU with Hypervisor.framework
(`qemu-system-aarch64 -accel hvf`), booted by QEMU's edk2 UEFI firmware with
an NVMe system disk, a virtio network card and a basic firmware framebuffer.
Use `ramfb`: with Homebrew QEMU 11.1.3 and Windows 11 ARM 26H2,
`virtio-gpu-pci` alone hangs before Windows Setup starts. The bundled
firmware defaults to 800×600; this tool does not yet provide a larger desktop.
Screenshots, clicks and keys go through QEMU's VNC server on the Mac's
loopback, driven by `vncdotool` from the macOS VM tool's virtualenv
(`~/scratch/mac-vm/venv`, or set `WIN_VM_VENV`). Commands reach the guest
over OpenSSH on a forwarded loopback port.

## Commands

```sh
tools/win-vm.sh build-image                  # install the golden disk (about an hour)
tools/win-vm.sh desktop --app Mash_0.9.0_x64-setup.exe
tools/win-vm.sh desktop --app ~/Downloads/mash-desktop-x86_64-pc-windows-msvc   # the CI artifact folder
tools/win-vm.sh desktop --app ... --keep     # leave the clone running to look around
tools/win-vm.sh list                         # golden disks and clones
tools/win-vm.sh clean                        # delete clones left by --keep
```

The CI installer is the `mash-desktop-x86_64-pc-windows-msvc` artifact of
the Desktop workflow:

```sh
gh run list --branch main --workflow Desktop
gh run download <id> -n mash-desktop-x86_64-pc-windows-msvc -D target/win-artifact
tools/win-vm.sh desktop --app target/win-artifact
```

Screenshots land in `target/win-vm/`: `1-window-screen.png` (the whole
screen with Mash open) and `1-window.png` (the Mash window alone),
then `2-context-menu-screen.png` and `2-context-menu.png` after a right-click
on the first chat row. The row is clicked at an offset from the window's top
left corner, `WIN_VM_ROW_X`/`WIN_VM_ROW_Y` (170, 170 by default).

With `--keep` the run prints the VNC address (on the Mac's loopback, port
5921 to 5929, password `mashvm`) and the SSH port; reach them through an SSH
tunnel to the Mac, for example `ssh -L 5921:127.0.0.1:5921 dev-macbook`.

## Where things live

On the Mac:

- `~/scratch/win-vm/` holds the synced scripts (`src/`), QEMU logs, pid
  files, build files (`build/`: the unattend ISO, `progress.png` during an
  install, `desktop.png` after it) and the last run's screenshots (`shots/`).
- `WIN_VM_STORE`, by default `/Volumes/Media SSD/mash-vms/win-vm/` on the
  external SSD, holds everything large: `iso/` (Microsoft's Windows ISO and
  `virtio-win.iso`), the golden disks `win11-golden-<yyyymmdd>.qcow2` with
  their UEFI variable stores (`.vars.fd`), and clones under `runs/`.

## Golden image

`build-image` needs two ISOs in `$WIN_VM_STORE/iso/`:

- Microsoft's Windows 11 ARM64 ISO, obtained through the browser form at
  <https://www.microsoft.com/software-download/windows11arm64> (the page
  hands out time-limited links through a form). Choose English (United States),
  matching the answer file and boot-prompt reference. Files named
  `Win11*Arm64*.iso` or `Windows11*arm64*.iso` are recognized, or set
  `WIN_VM_ISO` on the Mac. Use only
  Microsoft's ISO, never a rebuilt image. Verify it with `shasum -a 256`
  against the English ARM64 hash in the page's **Verify your download**
  section. A browser can obtain the link even when a direct scripted request
  to the download API is rejected; use the form's returned Microsoft URL.
- `virtio-win.iso` from the Fedora project's stable virtio-win builds, for
  the ARM64 network and display drivers.

It builds a small ISO holding `tools/win-vm/autounattend.xml`,
`tools/win-vm/setup.ps1` and the Mac's `~/.ssh/id_ed25519.pub`, boots the
installer with all three ISOs attached to an eight-port USB controller, and
waits up to 60 seconds for the ISO's "Press any key to boot from CD" prompt
before pressing Space once. `boot-prompt.png` is a small VNC capture of that
English prompt with its animated dots excluded. Repeated keypresses can
activate Setup's Cancel button and confirm quitting. If the prompt does not
match, the build stops and saves `build/progress.png` for inspection.
Setup then runs unattended:

- the answer file lets Setup continue without a TPM or Secure Boot
  (Microsoft's `LabConfig` switches), installs Windows 11 Pro with the
  generic install key, and creates the local administrator `mash` with the
  test password `mash-vm-test`, logged on automatically. The account exists
  only inside the image; it is not a real credential;
- `setup.ps1` runs at the first logon: it installs the virtio network and
  display drivers, adds OpenSSH Server (a Windows capability that comes from
  Windows Update, so the guest needs the network for it), authorizes the
  Mac's key, makes the automatic logon permanent, turns Windows Update off,
  sets dark mode and a plain dark wallpaper, turns off toasts, tips and the
  "finish setting up" screens, and stops the screen from sleeping or locking.
  Its log is `C:\mash-vm\setup.log` in the guest.

Before installing SSH, setup checks HTTPS with bounded `curl.exe` requests
and stops if they fail. This avoids an observed stall in `Test-NetConnection`
despite working HTTPS. Inspect `setup.log` if the desktop appears but SSH
does not become available. Setup also enables the SSH firewall rule for all
guest network profiles: Windows can install a Private-only rule while QEMU's
network is classified as Public. QEMU exposes the forwarded port only on the
Mac's loopback address.

The build restarts the guest once, saves a screenshot of the desktop in
`build/desktop.png`, shuts it down and keeps the disk as the golden image.
The disk has no Mash on it: each run installs the build under test.

## Run

`desktop` creates a qcow2 clone backed by the newest golden disk (it records
only the run's writes), boots it, and over SSH:

1. writes the made-up chats of `tools/mac-vm/demo-chats.json` into the
   guest's profile, generated on the Mac by `tools/mac-vm/demo_history.py`;
2. installs the NSIS installer with `/S` (per user, no prompt);
3. runs the app's `mash init --yes --no-agents --no-index` and `mash index`
   behind a dead proxy, so the app opens on its dashboard with the demo
   chats; word search needs neither Qdrant nor the models, which would be
   downloads (see [linux-vm.md](linux-vm.md), where this is proven);
4. starts Mash through a scheduled task with an interactive logon. A process
   started over SSH has no desktop, so anything that must appear on screen,
   or look at a window, runs as a task in the logged-on session
   (`tools/win-vm/guest.ps1`);
5. asks the desktop session for Mash's window rectangle, takes the
   screenshots, right-clicks the first chat row and takes them again.

Unless `--keep` is given, the clone is stopped and its disk deleted.

## Limits

- The guest is Windows on ARM. The Desktop workflow builds only the x64
  installer, which runs here under Windows' emulation: that shows what a
  clean Windows 11 does with it (a missing Visual C++ runtime was found this
  way) but not the native x64 build's speed or graphics. A release's
  `aarch64-pc-windows-msvc` installer runs natively.
- Windows is never activated: the desktop shows the "Activate Windows"
  watermark in the lower right corner, and personalization settings are
  locked in the Settings app (the image sets them through the registry).
- The display is QEMU's basic framebuffer without 3D acceleration, so WebView2
  renders in software; animations are slower than on real hardware.
- Windows Update is off in the image, so it stays at the ISO's build.
  Rebuild the image from a newer ISO to test a newer Windows.

## Without a VM: GitHub's Windows runner

`.github/workflows/windows-shots.yml` photographs the app on a GitHub-hosted
Windows runner (Windows Server 2025, which shares Windows 11's shell), using
the same steps as the VM (`tools/win-vm/runner-shots.ps1`). The runner's job
runs in a logged-on desktop session, so no scheduled task is needed. It takes
the x64 installer from a Desktop workflow run:

```bash
gh workflow run windows-shots.yml -f run=<Desktop run id>
gh run download <that run> -n windows-shots
```

It shows Windows' title bar, menus and fonts, not ARM or a desktop machine's
display; the VM remains the closer match once its ISO is in place.
