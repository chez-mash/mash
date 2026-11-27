#!/usr/bin/env bash
# Run the Mash desktop app on GNOME inside a throwaway Ubuntu 24.04 Desktop
# ARM64 VM (QEMU on a Mac reached over SSH) and fetch the screenshots back
# here. See docs/internal/linux-vm.md.
#
# Usage: tools/linux-vm.sh build-image
#        tools/linux-vm.sh desktop --app PATH [--keep]
#        tools/linux-vm.sh list | clean
#
#   build-image  autoinstall Ubuntu Desktop from Canonical's ISO into the
#                golden disk ubuntu-golden-<yyyymmdd>.qcow2 on the Mac: user
#                mash logged on automatically, OpenSSH Server with the Mac's
#                key, dark style, 1440x900, no lock, updates or tours
#   desktop      clone the newest golden disk copy-on-write, install Mash's
#                .deb (--app: the .deb, or a folder holding it such as the
#                downloaded mash-desktop-aarch64-unknown-linux-gnu CI
#                artifact), start Mash on the desktop, take screenshots, then
#                delete the clone (--keep leaves it running)
#   list         list the golden disks and clones
#   clean        delete clones left behind by --keep or an interrupted run
#
# LINUX_VM_HOST (default dev-macbook) names the Mac, LINUX_VM_DIR (default
# scratch/linux-vm, under its home) the working directory there,
# LINUX_VM_STORE (default "/Volumes/Media SSD/mash-vms/linux-vm") the Mac
# directory that holds the ISO and disks, and LINUX_VM_OUT (default
# target/linux-vm) where screenshots land here.
set -euo pipefail

MAC=${LINUX_VM_HOST:-dev-macbook}
DIR=${LINUX_VM_DIR:-scratch/linux-vm}
STORE=${LINUX_VM_STORE:-/Volumes/Media SSD/mash-vms/linux-vm}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${LINUX_VM_OUT:-$ROOT/target/linux-vm}
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
  ssh "$MAC" "mkdir -p $DIR/src/tools/linux-vm && rm -rf $DIR/src/app"
  rsync -a --delete "$ROOT/tools/linux-vm/" "$MAC:$DIR/src/tools/linux-vm/"
  # The made-up chats come from the macOS VM tool; demo_history.py reads the
  # recorded Antigravity tables from beside itself.
  rsync -a "$ROOT/tools/mac-vm/demo_history.py" "$ROOT/tools/mac-vm/demo-chats.json" \
    "$ROOT/fixtures/antigravity-native-runtime-2.5.5.json" "$MAC:$DIR/src/tools/linux-vm/"
}

package() { # the .deb named by --app, a file or a folder holding one
  local found
  if [ -d "$app" ]; then
    found=$(find "$app" -name '*.deb' | sort | head -1)
    [ -n "$found" ] || { echo "no .deb in $app" >&2; exit 1; }
    printf '%s\n' "$found"
  elif [ -f "$app" ]; then
    printf '%s\n' "$app"
  else
    echo "desktop needs --app PATH: the .deb or the folder holding it" >&2
    exit 1
  fi
}

# The store path holds a space, so it goes to the Mac through printf %q.
on_mac() { ssh "$MAC" "LINUX_VM_KEEP=$keep LINUX_VM_STORE=$(printf %q "$STORE") bash $DIR/src/tools/linux-vm/mac.sh $1" < /dev/null; }

case "$command" in
  build-image) sync_sources && on_mac build-image ;;
  desktop)
    deb=$(package)
    sync_sources
    ssh "$MAC" "mkdir -p $DIR/src/app"
    rsync -a "$deb" "$MAC:$DIR/src/app/mash.deb"
    echo "testing $(basename "$deb")"
    on_mac desktop
    mkdir -p "$OUT"
    rsync -a --delete "$MAC:$DIR/shots/" "$OUT/"
    echo "screenshots in $OUT"
    ;;
  list | clean) sync_sources && on_mac "$command" ;;
  *) sed -n '2,26p' "$0" >&2; exit 2 ;;
esac
