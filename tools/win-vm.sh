#!/usr/bin/env bash
# Run the Mash desktop app inside a throwaway Windows 11 ARM VM (QEMU on a Mac
# reached over SSH) and fetch the screenshots back here. See
# docs/internal/win-vm.md.
#
# Usage: tools/win-vm.sh build-image
#        tools/win-vm.sh desktop --app PATH [--keep]
#        tools/win-vm.sh list | clean
#
#   build-image  install Windows 11 ARM unattended from Microsoft's ISO into
#                the golden disk win11-golden-<yyyymmdd>.qcow2 on the Mac:
#                local user mash, automatic logon, OpenSSH Server with the
#                Mac's key, dark mode, quiet, Windows Update off
#   desktop      clone the newest golden disk copy-on-write, install the Mash
#                NSIS installer (--app: the .exe, or a folder holding it such
#                as the downloaded mash-desktop-aarch64-pc-windows-msvc CI
#                artifact) silently, start Mash on the desktop, take
#                screenshots, then delete the clone (--keep leaves it running)
#   list         list the golden disks and clones
#   clean        delete clones left behind by --keep or an interrupted run
#
# WIN_VM_HOST (default dev-macbook) names the Mac, WIN_VM_DIR (default
# scratch/win-vm, under its home) the working directory there, WIN_VM_STORE
# (default "/Volumes/Media SSD/mash-vms/win-vm") the Mac directory that holds
# the ISOs and disks, and WIN_VM_OUT (default target/win-vm) where screenshots
# land here.
set -euo pipefail

MAC=${WIN_VM_HOST:-dev-macbook}
DIR=${WIN_VM_DIR:-scratch/win-vm}
STORE=${WIN_VM_STORE:-/Volumes/Media SSD/mash-vms/win-vm}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${WIN_VM_OUT:-$ROOT/target/win-vm}
command=${1:-}
shift || true

app=
keep=
while [ $# -gt 0 ]; do
  case "$1" in
    --app) app=$2; shift 2 ;;
    --keep) keep=1; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

sync_sources() {
  ssh "$MAC" "mkdir -p $DIR/src/tools/win-vm && rm -rf $DIR/src/app"
  rsync -a --delete "$ROOT/tools/win-vm/" "$MAC:$DIR/src/tools/win-vm/"
  # The made-up chats come from the macOS VM tool; demo_history.py reads the
  # recorded Antigravity tables from beside itself.
  rsync -a "$ROOT/tools/mac-vm/demo_history.py" "$ROOT/tools/mac-vm/demo-chats.json" \
    "$ROOT/fixtures/antigravity-native-runtime-2.5.5.json" "$MAC:$DIR/src/tools/win-vm/"
}

installer() { # the NSIS installer named by --app, a file or a folder holding one
  local found
  if [ -d "$app" ]; then
    found=$(find "$app" -iname '*setup.exe' | sort | head -1)
    [ -n "$found" ] || found=$(find "$app" -iname '*.exe' | sort | head -1)
    [ -n "$found" ] || { echo "no .exe installer in $app" >&2; exit 1; }
    printf '%s\n' "$found"
  elif [ -f "$app" ]; then
    printf '%s\n' "$app"
  else
    echo "desktop needs --app PATH: the installer .exe or the folder holding it" >&2
    exit 1
  fi
}

# The store path holds a space, so it goes to the Mac through printf %q.
on_mac() { ssh "$MAC" "WIN_VM_KEEP=$keep WIN_VM_STORE=$(printf %q "$STORE") bash $DIR/src/tools/win-vm/mac.sh $1" < /dev/null; }

case "$command" in
  build-image) sync_sources && on_mac build-image ;;
  desktop)
    exe=$(installer)
    sync_sources
    ssh "$MAC" "mkdir -p $DIR/src/app"
    rsync -a "$exe" "$MAC:$DIR/src/app/mash-setup.exe"
    echo "testing $(basename "$exe")"
    on_mac desktop
    mkdir -p "$OUT"
    rsync -a --delete "$MAC:$DIR/shots/" "$OUT/"
    echo "screenshots in $OUT"
    ;;
  list | clean) sync_sources && on_mac "$command" ;;
  *) sed -n '2,26p' "$0" >&2; exit 2 ;;
esac
