#!/usr/bin/env bash
# Drive the Raycast extension inside a throwaway macOS VM on a Mac reached over
# SSH, and fetch the screenshots back here. See docs/internal/mac-vm.md.
#
# Usage: tools/mac-vm.sh build-image | push-image
#        tools/mac-vm.sh run | store-shots [--mash PATH] [--keep]
#        tools/mac-vm.sh desktop --app PATH
#        tools/mac-vm.sh list | clean
#
#   build-image  build mash-golden-<yyyymmdd> on the Mac from the Cua macOS
#                image: Homebrew, Node 22, Git, Raycast past onboarding, the
#                agents Mash detects, and made-up demo histories
#   push-image   push the newest golden image to the private package
#                ghcr.io/mashhq/mash-golden:<yyyymmdd> and :latest, and stop
#                unless GitHub reports the package private
#   run          clone the newest golden image (pulled from GHCR when the Mac
#                has none), install the Mash build under
#                test (--mash: a mash-aarch64-apple-darwin.tar.gz or a folder
#                holding its mash binaries; default: the latest release) and
#                raycast/ from this working tree, take the Search Mash, Read
#                Messages, Actions and Search Memory screenshots in light and
#                dark, then delete the clone (--keep leaves it running)
#   store-shots  like run, but draw the guest at 2x and write the Raycast Store
#                screenshots raycast/metadata/mash-1.png ... mash-5.png:
#                2000x1250, light, the window centred on a plain background
#   desktop      install the desktop app (--app: a .dmg or Mash.app, such as
#                CI's mash-desktop-aarch64-apple-darwin artifact) in a clone
#                with the demo histories indexed, open it, take
#                desktop-window.png, and leave the clone running
#   list         list the Mac's VMs
#   clean        delete run clones left behind by --keep or an interrupted run
#
# MAC_VM_HOST (default dev-macbook) names the Mac, MAC_VM_DIR (default
# scratch/mac-vm, under its home) the working directory there, and
# MAC_VM_OUT (default target/mac-vm) where screenshots land here.
set -euo pipefail

MAC=${MAC_VM_HOST:-dev-macbook}
DIR=${MAC_VM_DIR:-scratch/mac-vm}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${MAC_VM_OUT:-$ROOT/target/mac-vm}
command=${1:-run}
shift || true

mash=
app=
keep=
while [ $# -gt 0 ]; do
  case "$1" in
    --mash) mash=$2; shift 2 ;;
    --app) app=$2; shift 2 ;;
    --keep) keep=1; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

sync_sources() {
  ssh "$MAC" "mkdir -p $DIR/src/tools/mac-vm $DIR/src/raycast && rm -rf $DIR/src/mash"
  rsync -a --delete "$ROOT/tools/mac-vm/" "$MAC:$DIR/src/tools/mac-vm/"
  # demo_history.py reads the recorded Antigravity tables from beside itself in a guest.
  rsync -a "$ROOT/fixtures/antigravity-native-runtime-2.5.5.json" "$MAC:$DIR/src/tools/mac-vm/"
  rsync -a --delete --exclude node_modules --exclude .git "$ROOT/raycast/" "$MAC:$DIR/src/raycast/"
  ssh "$MAC" "rm -rf $DIR/src/desktop && mkdir -p $DIR/src/desktop"
  [ -z "$app" ] || rsync -a "$app" "$MAC:$DIR/src/desktop/"
  if [ -n "$mash" ]; then
    ssh "$MAC" "mkdir -p $DIR/src/mash"
    if [ -d "$mash" ]; then
      rsync -a "$mash/" "$MAC:$DIR/src/mash/"
    else
      ssh "$MAC" "tar -xzf - -C $DIR/src/mash" < "$mash"
    fi
  fi
}

on_mac() { ssh "$MAC" "MAC_VM_KEEP=$keep bash $DIR/src/tools/mac-vm/mac.sh $1" < /dev/null; }

# GHCR credentials from this host's gh login, sent to the Mac on standard
# input: never on a command line, in a file or in output. Pushing needs the
# write:packages scope; pulling the private image needs read:packages.
ghcr_credentials() { # ghcr_credentials SCOPE
  local scopes
  scopes=$(gh api -i /user 2>/dev/null | tr -d '\r' | sed -n 's/^[Xx]-[Oo][Aa]uth-[Ss]copes: //p')
  case ", $scopes," in
    *", $1,"* | *", write:packages,"*) ;;
    *)
      echo "the gh token lacks $1; run: gh auth refresh -h github.com -s $1" >&2
      return 1
      ;;
  esac
  printf '%s\n%s\n' "$(gh api /user --jq .login)" "$(gh auth token)"
}
with_credentials() { # with_credentials SCOPE COMMAND: run COMMAND on the Mac with credentials on stdin
  local credentials
  credentials=$(ghcr_credentials "$1") || exit 1
  printf '%s\n' "$credentials" | ssh "$MAC" "MAC_VM_KEEP=$keep bash $DIR/src/tools/mac-vm/mac.sh $2"
}

case "$command" in
  build-image) sync_sources && on_mac build-image ;;
  push-image)
    sync_sources
    with_credentials write:packages push-image
    visibility=$(gh api /orgs/mashhq/packages/container/mash-golden --jq .visibility)
    if [ "$visibility" != private ]; then
      echo "ghcr.io/mashhq/mash-golden is $visibility; it must be private (macOS may only run on Apple hardware). Make it private now: https://github.com/orgs/mashhq/packages/container/mash-golden/settings" >&2
      exit 1
    fi
    echo "ghcr.io/mashhq/mash-golden is private"
    ;;
  run | store-shots | desktop)
    sync_sources
    if ssh "$MAC" "~/.local/bin/lume ls" | grep -q '^mash-golden-'; then
      on_mac "$command"
    else
      with_credentials read:packages "$command"
    fi
    if [ "$command" = run ] || [ "$command" = desktop ]; then
      mkdir -p "$OUT"
      rsync -a --delete "$MAC:$DIR/shots/" "$OUT/"
      echo "screenshots in $OUT"
    else
      # The full-screen captures stay beside the run's other output for review.
      mkdir -p "$OUT/store"
      rsync -a --delete "$MAC:$DIR/store/" "$OUT/store/"
      cp "$OUT"/store/mash-[0-9].png "$ROOT/raycast/metadata/"
      echo "Store screenshots in $ROOT/raycast/metadata; full screens in $OUT/store/screen"
    fi
    ;;
  list | clean) on_mac "$command" ;;
  *) sed -n '2,35p' "$0" >&2; exit 2 ;;
esac
