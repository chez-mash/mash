#!/bin/bash
# Runs on the Mac host for tools/linux-vm.sh. Owns the QEMU Ubuntu Desktop
# ARM64 VMs: the autoinstall that makes the golden disk, copy-on-write clones
# of it for runs, the guest SSH connection and the VNC driver. The installed
# system's own setup is finish.sh. Nothing here changes the host beyond QEMU,
# its working directory and the VM store.
#
# Usage: mac.sh build-image | desktop | list | clean
# Reads LINUX_VM_KEEP=1 (desktop: leave the clone running) and LINUX_VM_STORE
# (where the ISO, the golden disk and clones live) from the environment.
set -euo pipefail
export PATH="/opt/homebrew/bin:$PATH"
WORK=$(cd "$(dirname "$0")/../.." && pwd) # the synced copy: WORK/tools/linux-vm, WORK/app
STATE=$(dirname "$WORK")                  # logs, pid files, build files, screenshots
STORE=${LINUX_VM_STORE:-/Volumes/Media SSD/mash-vms/linux-vm}
TOOLS=$WORK/tools/linux-vm
FIRMWARE=/opt/homebrew/share/qemu/edk2-aarch64-code.fd
VARS_TEMPLATE=/opt/homebrew/share/qemu/edk2-arm-vars.fd
# The macOS VM tool's virtualenv already has vncdotool and Pillow; reuse it.
VENV=${LINUX_VM_VENV:-$HOME/scratch/mac-vm/venv}
VNC_PASSWORD=mashvm
cd "$STATE"
mkdir -p run build
command -v qemu-system-aarch64 >/dev/null || { echo "QEMU is not installed on this Mac: brew install qemu" >&2; exit 1; }
[ -x "$VENV/bin/vncdo" ] || { echo "no vncdotool in $VENV; set LINUX_VM_VENV to a virtualenv that has vncdotool and pillow" >&2; exit 1; }
[ -d "$STORE" ] || mkdir -p "$STORE"

# One VM is NAME with SSH on 127.0.0.1:22NN and VNC on 127.0.0.1:59NN, both on
# the Mac's loopback only. The build uses slot 40; runs take 41 to 49, clear
# of the Windows tool's 20 to 29.
SLOT=40
NAME=
v() { "$VENV/bin/python3" -W ignore "$VENV/bin/vncdo" -s "127.0.0.1::59$SLOT" -p "$VNC_PASSWORD" "$@"; }
SSH_OPTS=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o BatchMode=yes -o ConnectTimeout=10)
guest() { ssh "${SSH_OPTS[@]}" -p "22$SLOT" mash@127.0.0.1 "$@"; }
to_guest() { scp -q "${SSH_OPTS[@]}" -P "22$SLOT" "$1" "mash@127.0.0.1:$2"; }

running() { [ -f "run/$1.pid" ] && kill -0 "$(cat "run/$1.pid")" 2>/dev/null; }

start_vm() { # start_vm DISK VARS [QEMU ARGS...]: boot NAME headless
  local disk=$1 vars=$2
  shift 2
  # GICv3 and the host CPU under Hypervisor.framework; virtio disk, network
  # and display, which Ubuntu drives out of the box. The display offers
  # 1440x900 as its preferred mode, and the tablet gives VNC absolute pointing.
  qemu-system-aarch64 \
    -name "$NAME" -M virt,gic-version=3 -accel hvf -cpu host -smp 6 -m 8G \
    -drive if=pflash,format=raw,readonly=on,file="$FIRMWARE" \
    -drive if=pflash,format=raw,file="$vars" \
    -drive if=none,id=disk,format=qcow2,file="$disk" -device virtio-blk-pci,drive=disk,bootindex=0 \
    -netdev "user,id=net,hostfwd=tcp:127.0.0.1:22$SLOT-:22" -device virtio-net-pci,netdev=net \
    -device virtio-gpu-pci,xres=1440,yres=900 \
    -device qemu-xhci -device usb-kbd -device usb-tablet \
    -object secret,id=vncpw,data="$VNC_PASSWORD" -vnc "127.0.0.1:$SLOT,password-secret=vncpw" \
    -display none -daemonize -pidfile "run/$NAME.pid" "$@" \
    >"$NAME.log" 2>&1
}

wait_ssh() { # wait_ssh SECONDS
  local deadline=$((SECONDS + $1))
  while [ "$SECONDS" -lt "$deadline" ]; do
    guest true 2>/dev/null && return
    running "$NAME" || { echo "$NAME stopped; see $STATE/$NAME.log" >&2; exit 1; }
    sleep 5
  done
  echo "$NAME never answered SSH on 127.0.0.1:22$SLOT" >&2
  exit 1
}

wait_session() { # wait for mash's GNOME session to be up
  for _ in $(seq 1 60); do
    guest 'pgrep -u mash -x gnome-shell >/dev/null' && { sleep 15; return; }
    sleep 3
  done
  echo "no GNOME session for mash on $NAME" >&2
  exit 1
}

stop_vm() { # stop_vm: power the guest off cleanly, then make sure QEMU is gone
  guest "sudo systemctl poweroff" >/dev/null 2>&1 || true
  for _ in $(seq 1 60); do running "$NAME" || break; sleep 3; done
  running "$NAME" && kill "$(cat "run/$NAME.pid")"
  rm -f "run/$NAME.pid"
}

latest_golden() { find "$STORE" -maxdepth 1 -name 'ubuntu-golden-*.qcow2' | sort | tail -1; }

ubuntu_iso() {
  local iso=${LINUX_VM_ISO:-}
  [ -n "$iso" ] || iso=$(find "$STORE/iso" -maxdepth 1 -name 'ubuntu-24.04*-desktop-arm64.iso' 2>/dev/null | sort -V | tail -1)
  if [ -z "$iso" ] || [ ! -f "$iso" ]; then
    echo "no Ubuntu Desktop ARM64 ISO in $STORE/iso/: download ubuntu-24.04.x-desktop-arm64.iso" >&2
    echo "and SHA256SUMS from https://cdimage.ubuntu.com/releases/24.04/release/ there" >&2
    exit 1
  fi
  printf '%s\n' "$iso"
}

build_image() {
  local iso golden disk vars
  iso=$(ubuntu_iso)
  golden="$STORE/ubuntu-golden-$(date +%Y%m%d)"
  NAME=ubuntu-golden-build SLOT=40
  disk="$STORE/$NAME.qcow2" vars="$STORE/$NAME.vars.fd"
  running "$NAME" && kill "$(cat "run/$NAME.pid")"
  rm -f "$disk" "$vars"

  # The autoinstall answers and finish.sh on a CIDATA ISO, which cloud-init's
  # NoCloud source reads in the live session.
  rm -rf build/seed build/seed.iso && mkdir -p build/seed
  sed "s|AUTHORIZED_KEY|$(cat ~/.ssh/id_ed25519.pub)|" "$TOOLS/user-data" >build/seed/user-data
  printf 'instance-id: mash-linux-build\n' >build/seed/meta-data
  cp "$TOOLS/finish.sh" build/seed/
  hdiutil makehybrid -quiet -iso -joliet -default-volume-name CIDATA -o build/seed.iso build/seed
  # Boot the installer's own kernel directly, so the command line can say
  # `autoinstall` without editing the ISO's GRUB menu.
  rm -rf build/casper && mkdir -p build/casper
  tar -xf "$iso" -C build/casper --strip-components 1 casper/vmlinuz casper/initrd

  qemu-img create -q -f qcow2 "$disk" 40G
  cp "$VARS_TEMPLATE" "$vars"
  echo "installing Ubuntu from $(basename "$iso") into $disk"
  start_vm "$disk" "$vars" \
    -kernel "$STATE/build/casper/vmlinuz" -initrd "$STATE/build/casper/initrd" -append "autoinstall ds=nocloud ---" \
    -device virtio-scsi-pci,id=scsi \
    -drive if=none,id=cd,media=cdrom,readonly=on,file="$iso" -device scsi-cd,drive=cd \
    -drive if=none,id=seed,media=cdrom,readonly=on,file="$STATE/build/seed.iso" -device scsi-cd,drive=seed
  # The installer powers the VM off when it is done (`shutdown: poweroff`).
  # A progress screenshot every minute shows where a stuck install stopped.
  local deadline=$((SECONDS + 120 * 60))
  while running "$NAME"; do
    [ "$SECONDS" -lt "$deadline" ] || { echo "the install did not finish in 120 minutes; see $STATE/build/progress.png" >&2; exit 1; }
    v capture build/progress.png >/dev/null 2>&1 || true
    sleep 60
  done
  rm -f "run/$NAME.pid"
  echo "Ubuntu is installed; booting it from its disk"
  start_vm "$disk" "$vars"
  wait_ssh 600
  wait_session
  v capture build/desktop.png
  stop_vm
  mv "$disk" "$golden.qcow2"
  mv "$vars" "$golden.vars.fd"
  echo "saved $golden.qcow2; the desktop after the first boot is in $STATE/build/desktop.png"
}

free_slot() {
  local slot
  for slot in 41 42 43 44 45 46 47 48 49; do
    running "run-$slot" || { echo "$slot"; return; }
  done
  echo "all nine run slots are busy; run clean" >&2
  exit 1
}

desktop() {
  local golden package
  golden=$(latest_golden)
  [ -n "$golden" ] || { echo "no golden image in $STORE; run build-image first" >&2; exit 1; }
  package=$WORK/app/mash.deb
  [ -f "$package" ] || { echo "no package synced to $package" >&2; exit 1; }
  SLOT=$(free_slot)
  NAME=run-$SLOT
  mkdir -p "$STORE/runs"
  disk="$STORE/runs/$NAME.qcow2" vars="$STORE/runs/$NAME.vars.fd"
  # Copy-on-write: the clone's disk records only what the run writes.
  qemu-img create -q -f qcow2 -F qcow2 -b "$golden" "$disk"
  cp "${golden%.qcow2}.vars.fd" "$vars"
  trap '[ "${LINUX_VM_KEEP:-}" = 1 ] || { running "$NAME" && kill "$(cat "run/$NAME.pid")"; rm -f "run/$NAME.pid" "$disk" "$vars"; }' EXIT
  echo "booting $NAME from $(basename "$golden")"
  start_vm "$disk" "$vars"
  wait_ssh 600
  wait_session

  # Made-up agent histories, so Mash has chats to list, in each agent's own
  # layout under the home directory (demo_history.py from the macOS tool).
  rm -rf build/demo-home && mkdir -p build/demo-home
  python3 "$TOOLS/demo_history.py" build/demo-home >/dev/null
  tar -C build/demo-home -cf - . | guest 'tar -xf - -C ~'
  to_guest "$package" /tmp/mash.deb
  guest 'sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -q /tmp/mash.deb >/tmp/mash-install.log 2>&1 || { tail -20 /tmp/mash-install.log; exit 1; }'
  echo "installed $(guest 'dpkg-query -W -f "\${Package} \${Version}" mash')"

  # Set Mash up and save the demo chats, so the app opens on its dashboard
  # rather than the setup wizard. Ranked search would need Qdrant and the
  # models, both downloads; word search reads what `mash index` saves before
  # it asks for Qdrant, so the index "fails" only at that last step. A dead
  # proxy keeps both commands from downloading anything.
  guest 'export HTTPS_PROXY=http://127.0.0.1:9 HTTP_PROXY=http://127.0.0.1:9 https_proxy=http://127.0.0.1:9 http_proxy=http://127.0.0.1:9
    mash init --yes --no-agents --no-index >/tmp/mash-init.log 2>&1 && { mash index >/tmp/mash-index.log 2>&1 || true; }'

  rm -rf shots && mkdir shots
  v move 1430 890 pause 0.5 capture shots/0-desktop.png
  # An SSH session is outside the graphical session. The user's systemd
  # manager has the session's WAYLAND_DISPLAY, DISPLAY and D-Bus address
  # (GNOME imports them at login), so a transient user unit started there
  # opens its window on the logged-on desktop.
  guest 'XDG_RUNTIME_DIR=/run/user/$(id -u) systemd-run --user --collect --unit mash-desktop /usr/bin/mash-desktop'
  for _ in $(seq 1 60); do guest 'pgrep -x mash-desktop >/dev/null' && break; sleep 1; done
  # Give the window time to draw and load the dashboard.
  sleep 25
  v capture shots/1-window-screen.png
  # The window is what changed against the empty desktop, leaving out the
  # top bar and the dock, where the app's icon appears too. Under Wayland
  # there is no window list to ask, and GNOME places the window itself
  # (maximized at 1440x900, filling the area right of the dock).
  local window x y w h
  window=$("$VENV/bin/python3" -c 'import sys; from PIL import Image, ImageChops
a = Image.open(sys.argv[1]).convert("RGB"); b = Image.open(sys.argv[2]).convert("RGB")
box = ImageChops.difference(a, b).convert("L").point(lambda p: 255 if p > 6 else 0).crop((66, 32, a.width, a.height)).getbbox()
print("none" if not box or (box[2] - box[0]) < 200 else f"{box[0] + 66} {box[1] + 32} {box[2] - box[0]} {box[3] - box[1]}")' \
    shots/0-desktop.png shots/1-window-screen.png)
  echo "window: $window"
  crop() { # crop NAME: the Mash window out of NAME-screen.png
    "$VENV/bin/python3" -c 'import sys; from PIL import Image
x, y, w, h = map(int, sys.argv[3:7]); Image.open(sys.argv[1]).crop((x, y, x + w, y + h)).save(sys.argv[2])' \
      "shots/$1-screen.png" "shots/$1.png" $x $y $w $h
  }
  if [ "$window" != none ]; then
    read -r x y w h <<<"$window"
    crop 1-window
    # The first row of the dashboard's Recent chats list.
    v move $((x + ${LINUX_VM_ROW_X:-574})) $((y + ${LINUX_VM_ROW_Y:-517})) pause 1 click 3 pause 2
    v capture shots/2-context-menu-screen.png
    crop 2-context-menu
    v key esc
  else
    echo "no Mash window found; kept the full screen only" >&2
  fi
  [ "${LINUX_VM_KEEP:-}" != 1 ] ||
    echo "kept $NAME running: VNC on the Mac at 127.0.0.1:59$SLOT (password $VNC_PASSWORD), SSH mash@127.0.0.1 -p 22$SLOT"
  echo "screenshots in $STATE/shots"
}

case "${1:-}" in
  build-image) build_image ;;
  desktop) desktop ;;
  list)
    echo "golden images in $STORE:"
    find "$STORE" -maxdepth 1 -name 'ubuntu-golden-*.qcow2' -exec du -h {} \; | sort -k2
    echo "clones:"
    for disk in "$STORE"/runs/run-*.qcow2; do
      [ -f "$disk" ] || continue
      NAME=$(basename "$disk" .qcow2)
      if running "$NAME"; then echo "  $NAME running: VNC 127.0.0.1:59${NAME#run-}, SSH -p 22${NAME#run-}"; else echo "  $NAME stopped"; fi
    done ;;
  clean) # stop and delete clones left by --keep or an interrupted run
    for disk in "$STORE"/runs/run-*.qcow2; do
      [ -f "$disk" ] || continue
      NAME=$(basename "$disk" .qcow2)
      running "$NAME" && kill "$(cat "run/$NAME.pid")"
      rm -f "run/$NAME.pid" "$disk" "${disk%.qcow2}.vars.fd"
      echo "deleted $NAME"
    done ;;
  *) sed -n '2,10p' "$0" >&2; exit 2 ;;
esac
