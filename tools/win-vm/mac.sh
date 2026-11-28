#!/bin/bash
# Runs on the Mac host for tools/win-vm.sh. Owns the QEMU Windows 11 ARM VMs:
# the unattended install that makes the golden disk, copy-on-write clones of
# it for runs, the guest SSH connection and the VNC driver. The guest-side
# work is in setup.ps1 (first logon of the image) and guest.ps1 (runs).
# Nothing here changes the host beyond QEMU, its working directory and the
# VM store.
#
# Usage: mac.sh build-image | desktop | list | clean
# Reads WIN_VM_KEEP=1 (desktop: leave the clone running) and WIN_VM_STORE
# (where ISOs, the golden disk and clones live) from the environment.
set -euo pipefail
export PATH="/opt/homebrew/bin:$PATH"
WORK=$(cd "$(dirname "$0")/../.." && pwd) # the synced copy: WORK/tools/win-vm, WORK/app
STATE=$(dirname "$WORK")                  # logs, pid files, build files, screenshots
STORE=${WIN_VM_STORE:-/Volumes/Media SSD/mash-vms/win-vm}
TOOLS=$WORK/tools/win-vm
FIRMWARE=/opt/homebrew/share/qemu/edk2-aarch64-code.fd
VARS_TEMPLATE=/opt/homebrew/share/qemu/edk2-arm-vars.fd
# The macOS VM tool's virtualenv already has vncdotool and Pillow; reuse it.
VENV=${WIN_VM_VENV:-$HOME/scratch/mac-vm/venv}
VNC_PASSWORD=mashvm
cd "$STATE"
mkdir -p run build
command -v qemu-system-aarch64 >/dev/null || { echo "QEMU is not installed on this Mac: brew install qemu" >&2; exit 1; }
[ -x "$VENV/bin/vncdo" ] || { echo "no vncdotool in $VENV; set WIN_VM_VENV to a virtualenv that has vncdotool and pillow" >&2; exit 1; }
[ -d "$STORE" ] || mkdir -p "$STORE"

# One VM is NAME with SSH on 127.0.0.1:22NN and VNC on 127.0.0.1:59NN, both on
# the Mac's loopback only. The build uses slot 20; runs take 21 to 29.
SLOT=20
NAME=
v() { "$VENV/bin/python3" -W ignore "$VENV/bin/vncdo" -s "127.0.0.1::59$SLOT" -p "$VNC_PASSWORD" "$@"; }
SSH_OPTS=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o BatchMode=yes -o ConnectTimeout=10)
# Commands run in the guest's default shell, cmd.exe, from C:\Users\mash.
guest() { ssh "${SSH_OPTS[@]}" -p "22$SLOT" mash@127.0.0.1 "$@"; }
to_guest() { scp -q "${SSH_OPTS[@]}" -P "22$SLOT" "$1" "mash@127.0.0.1:$2"; }
guest_ps() { guest "powershell -NoProfile -ExecutionPolicy Bypass -File win-vm\\guest.ps1 $*"; }

running() { [ -f "run/$1.pid" ] && kill -0 "$(cat "run/$1.pid")" 2>/dev/null; }

start_vm() { # start_vm DISK VARS [QEMU ARGS...]: boot NAME headless
  local disk=$1 vars=$2
  shift 2
  # Windows' ARM boot loader needs a linear framebuffer: virtio-gpu-pci's
  # firmware display alone hangs before Setup. Keep ramfb through installation
  # and subsequent boots. Eight USB ports keep all three installer drives off
  # QEMU's automatically inserted USB 1.1 hub.
  qemu-system-aarch64 \
    -name "$NAME" -M virt,gic-version=3 -accel hvf -cpu host -smp 6 -m 8G \
    -drive if=pflash,format=raw,readonly=on,file="$FIRMWARE" \
    -drive if=pflash,format=raw,file="$vars" \
    -drive if=none,id=disk,format=qcow2,file="$disk" \
    -device nvme,drive=disk,serial=mashwin,bootindex=0 \
    -netdev "user,id=net,hostfwd=tcp:127.0.0.1:22$SLOT-:22" -device virtio-net-pci,netdev=net \
    -device ramfb \
    -device qemu-xhci,p2=8,p3=8 -device usb-kbd -device usb-tablet \
    -object secret,id=vncpw,data="$VNC_PASSWORD" -vnc "127.0.0.1:$SLOT,password-secret=vncpw" \
    -display none -daemonize -pidfile "run/$NAME.pid" "$@" \
    >"$NAME.log" 2>&1
}

wait_ssh() { # wait_ssh SECONDS
  local deadline=$((SECONDS + $1))
  while [ "$SECONDS" -lt "$deadline" ]; do
    guest "exit 0" 2>/dev/null && return
    running "$NAME" || { echo "$NAME stopped; see $STATE/$NAME.log" >&2; exit 1; }
    sleep 5
  done
  echo "$NAME never answered SSH on 127.0.0.1:22$SLOT" >&2
  exit 1
}

stop_vm() { # stop_vm: shut Windows down cleanly, then make sure QEMU is gone
  guest "shutdown /s /t 0" >/dev/null 2>&1 || true
  for _ in $(seq 1 60); do running "$NAME" || break; sleep 3; done
  running "$NAME" && kill "$(cat "run/$NAME.pid")"
  rm -f "run/$NAME.pid"
}

latest_golden() { find "$STORE" -maxdepth 1 -name 'win11-golden-*.qcow2' | sort | tail -1; }

windows_iso() {
  local iso=${WIN_VM_ISO:-}
  [ -n "$iso" ] || iso=$(find "$STORE/iso" -maxdepth 1 \
    \( -iname 'Win11*arm64*.iso' -o -iname 'Windows11*arm64*.iso' \) \
    2>/dev/null | sort | tail -1)
  if [ -z "$iso" ] || [ ! -f "$iso" ]; then
    echo "no Windows 11 ARM64 ISO: download Microsoft's from https://www.microsoft.com/software-download/windows11arm64" >&2
    echo "on this Mac and save it in $STORE/iso/ (a name like Win11_25H2_English_Arm64.iso)" >&2
    exit 1
  fi
  printf '%s\n' "$iso"
}

build_image() {
  local iso golden disk vars
  iso=$(windows_iso)
  [ -f "$STORE/iso/virtio-win.iso" ] || { echo "no $STORE/iso/virtio-win.iso (Fedora's virtio-win build)" >&2; exit 1; }
  golden="$STORE/win11-golden-$(date +%Y%m%d)"
  NAME=win-golden-build SLOT=20
  disk="$STORE/$NAME.qcow2" vars="$STORE/$NAME.vars.fd"
  running "$NAME" && kill "$(cat "run/$NAME.pid")"
  rm -f "$disk" "$vars"

  # The answer file, the first-logon script and this Mac's public key, on a
  # small ISO that Setup searches for autounattend.xml.
  rm -rf build/unattend build/unattend.iso && mkdir -p build/unattend
  cp "$TOOLS/autounattend.xml" "$TOOLS/setup.ps1" build/unattend/
  cp ~/.ssh/id_ed25519.pub build/unattend/authorized_keys
  hdiutil makehybrid -quiet -iso -joliet -default-volume-name UNATTEND -o build/unattend.iso build/unattend

  qemu-img create -q -f qcow2 "$disk" 64G
  cp "$VARS_TEMPLATE" "$vars"
  echo "installing Windows from $(basename "$iso") into $disk"
  start_vm "$disk" "$vars" \
    -drive if=none,id=wincd,media=cdrom,readonly=on,file="$iso" -device usb-storage,drive=wincd,bootindex=1 \
    -drive if=none,id=unattend,media=cdrom,readonly=on,file="$STATE/build/unattend.iso" -device usb-storage,drive=unattend \
    -drive if=none,id=virtio,media=cdrom,readonly=on,file="$STORE/iso/virtio-win.iso" -device usb-storage,drive=virtio
  # Match the English ISO's boot prompt, then answer it exactly once. Fixed
  # delays can miss it; repeated keys can cancel Setup. The reference covers
  # only the prompt's first line, excluding its animated trailing dots.
  if ! v -t 60 rexpect "$TOOLS/boot-prompt.png" 0 0 0 key space; then
    v capture build/progress.png || true
    echo "Windows boot prompt not found; see $STATE/build/progress.png" >&2
    exit 1
  fi

  # Setup copies files, restarts a few times and finishes with the first
  # logon, where setup.ps1 writes ready.txt. A progress screenshot every
  # minute shows where a stuck install stopped.
  local deadline=$((SECONDS + 150 * 60))
  until guest "if exist C:\\mash-vm\\ready.txt echo ready" 2>/dev/null | grep -q ready; do
    running "$NAME" || { echo "$NAME stopped during the install; see $STATE/$NAME.log and build/progress.png" >&2; exit 1; }
    [ "$SECONDS" -lt "$deadline" ] || { echo "the install did not finish in 150 minutes; see $STATE/build/progress.png" >&2; exit 1; }
    v capture build/progress.png >/dev/null 2>&1 || true
    sleep 60
  done
  echo "Windows is installed; restarting to check the automatic logon"
  # Restart once, so the image is saved as it looks after a normal boot and the
  # settings that need a new logon (theme, wallpaper) have applied.
  guest "shutdown /r /t 0" >/dev/null 2>&1 || true
  sleep 30
  wait_ssh 600
  sleep 45
  v capture build/desktop.png
  stop_vm
  mv "$disk" "$golden.qcow2"
  mv "$vars" "$golden.vars.fd"
  echo "saved $golden.qcow2; the desktop after a restart is in $STATE/build/desktop.png"
}

free_slot() {
  local slot
  for slot in 21 22 23 24 25 26 27 28 29; do
    running "run-$slot" || { echo "$slot"; return; }
  done
  echo "all nine run slots are busy; run clean" >&2
  exit 1
}

desktop() {
  local golden installer
  golden=$(latest_golden)
  [ -n "$golden" ] || { echo "no golden image in $STORE; run build-image first" >&2; exit 1; }
  installer=$WORK/app/mash-setup.exe
  [ -f "$installer" ] || { echo "no installer synced to $installer" >&2; exit 1; }
  SLOT=$(free_slot)
  NAME=run-$SLOT
  mkdir -p "$STORE/runs"
  disk="$STORE/runs/$NAME.qcow2" vars="$STORE/runs/$NAME.vars.fd"
  # Copy-on-write: the clone's disk records only what the run writes.
  qemu-img create -q -f qcow2 -F qcow2 -b "$golden" "$disk"
  cp "${golden%.qcow2}.vars.fd" "$vars"
  trap '[ "${WIN_VM_KEEP:-}" = 1 ] || { running "$NAME" && kill "$(cat "run/$NAME.pid")"; rm -f "run/$NAME.pid" "$disk" "$vars"; }' EXIT
  echo "booting $NAME from $(basename "$golden")"
  start_vm "$disk" "$vars"
  wait_ssh 600
  # The desktop session logs on by itself; let it settle before using it.
  sleep 30

  # Made-up agent histories, so Mash has chats to list. demo_history.py
  # writes them under a home directory in each agent's own layout, which is
  # the same under a Windows profile for the agents it covers.
  rm -rf build/demo-home build/demo.tar && mkdir -p build/demo-home
  python3 "$TOOLS/demo_history.py" build/demo-home >/dev/null
  tar -C build/demo-home -cf build/demo.tar .
  guest "if not exist win-vm mkdir win-vm" >/dev/null
  to_guest "$TOOLS/guest.ps1" win-vm/guest.ps1
  to_guest build/demo.tar win-vm/demo.tar
  to_guest "$installer" win-vm/mash-setup.exe
  guest "tar -xf win-vm\\demo.tar -C %USERPROFILE%"

  guest_ps install "C:\\Users\\mash\\win-vm\\mash-setup.exe"
  guest_ps index
  guest_ps launch
  # Give the window time to draw and load the dashboard.
  sleep 25
  local window x y w h
  window=$(guest_ps window | tr -d '\r')
  echo "window: $window"
  rm -rf shots && mkdir shots
  shoot() { # shoot NAME: the full screen, and the Mash window when one was found
    v capture "shots/$1-screen.png"
    [ "$window" = none ] ||
      "$VENV/bin/python3" -c 'import sys; from PIL import Image
x, y, w, h = map(int, sys.argv[3:7])
im = Image.open(sys.argv[1]); im.crop((max(x, 0), max(y, 0), min(x + w, im.width), min(y + h, im.height))).save(sys.argv[2])' \
        "shots/$1-screen.png" "shots/$1.png" $x $y $w $h
  }
  read -r x y w h _ <<<"$window"
  [ "$window" = none ] && echo "no Mash window found; taking the full screen only" >&2
  v move 1400 880
  shoot 1-window
  if [ "$window" != none ]; then
    # The first chat row of the list on the left of the window.
    v move $((x + ${WIN_VM_ROW_X:-170})) $((y + ${WIN_VM_ROW_Y:-170})) pause 1 click 3 pause 2
    shoot 2-context-menu
    v key esc
  fi
  [ "${WIN_VM_KEEP:-}" != 1 ] ||
    echo "kept $NAME running: VNC on the Mac at 127.0.0.1:59$SLOT (password $VNC_PASSWORD), SSH mash@127.0.0.1 -p 22$SLOT"
  echo "screenshots in $STATE/shots"
}

case "${1:-}" in
  build-image) build_image ;;
  desktop) desktop ;;
  list)
    echo "golden images in $STORE:"
    find "$STORE" -maxdepth 1 -name 'win11-golden-*.qcow2' -exec du -h {} \; | sort -k2
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
  *) sed -n '2,11p' "$0" >&2; exit 2 ;;
esac
