#!/bin/bash
# Runs on the Mac host for tools/mac-vm.sh. Owns the Lume VMs, the guest SSH
# connection and the VNC driver; the guest-side work is in guest-image.sh and
# guest-run.sh. Nothing here changes the host beyond its own working directory
# and Lume's VM store.
#
# Usage: mac.sh build-image | push-image | run | store-shots | list | clean
# Reads MAC_VM_KEEP=1 (run, store-shots: leave the clone running) from the
# environment. push-image, run and store-shots read GHCR credentials from
# standard input, a user name line and a token line, so the token never
# appears on a command line; runs need them only to pull an image this Mac
# does not have.
set -euo pipefail
export PATH="$HOME/.local/bin:/opt/homebrew/bin:$PATH"
WORK=$(cd "$(dirname "$0")/../.." && pwd) # the synced copy: WORK/tools/mac-vm, WORK/raycast
STATE=$(dirname "$WORK")                  # known_hosts, venv, caches, screenshots
CUA_IMAGE=macos-tahoe-cua:26.5.2
PRISTINE=cua-tahoe-26.5.2
VNC_PORT=5905
cd "$STATE"
command -v lume >/dev/null || { echo "lume is not installed on this Mac: see docs/internal/mac-vm.md" >&2; exit 1; }

info() { # info VM FIELD
  lume get "$1" --format json 2>/dev/null | python3 -c '
import json, sys
d = json.load(sys.stdin)
d = d[0] if isinstance(d, list) else d
print(d.get(sys.argv[1]) or "")' "$2"
}
exists() { lume ls 2>/dev/null | awk '{print $1}' | grep -qx "$1"; }
SSH_OPTS=(-o UserKnownHostsFile="$STATE/known_hosts" -o StrictHostKeyChecking=accept-new -o BatchMode=yes -o ConnectTimeout=10)
guest() { ssh "${SSH_OPTS[@]}" "lume@$(info "$VM" ipAddress)" "$@"; }
to_guest() { rsync -a --delete -e "ssh ${SSH_OPTS[*]}" "$1" "lume@$(info "$VM" ipAddress):$2"; }

# vncdotool drives the VM's own VNC server, so no permission is needed on
# either Mac. Over Apple's VNC, Alt is Command and Meta is Option (⌘K is
# `key alt-k`). The guest desktop is 1440x900 at 1x.
v() { venv/bin/python3 -W ignore venv/bin/vncdo -s "127.0.0.1::$VNC_PORT" -p mashvm "$@"; }
[ -x venv/bin/vncdo ] || { python3 -m venv venv && venv/bin/pip -q install vncdotool pillow; }

clone() { # clone SOURCE DEST: a clone that keeps the source's machine identity
  lume clone "$1" "$2" >/dev/null
  # A clone gets a new machine identifier, so the guest would see new hardware
  # and its per-machine keychain items would not follow. Only one of the two
  # ever runs, so the clone takes over the source's identity.
  lume set "$2" --machine-identifier "$(info "$1" machineIdentifier)" >/dev/null
}

wait_ssh() {
  for _ in $(seq 1 80); do
    [ "$(info "$VM" sshAvailable)" = True ] && guest true 2>/dev/null && return
    # A new guest only knows its password (lume/lume); authorize this Mac's key.
    if [ "$(info "$VM" sshAvailable)" = True ]; then
      lume ssh "$VM" "umask 077; mkdir -p ~/.ssh; echo '$(cat ~/.ssh/id_ed25519.pub)' >> ~/.ssh/authorized_keys" >/dev/null 2>&1 || true
    fi
    sleep 3
  done
  echo "$VM never answered SSH; see $STATE/$VM.log" >&2
  exit 1
}
boot() {
  lume run "$VM" --display none --detach --vnc-port "$VNC_PORT" --vnc-password mashvm --log-file "$STATE/$VM.log" >/dev/null
  wait_ssh
  # Give the login session time to settle before driving it.
  sleep 15
}
reboot() {
  lume restart "$VM" >/dev/null
  sleep 20
  wait_ssh
  sleep 15
}
shot() { v capture "$1"; }
dismiss_notices() {
  # macOS posts "Login Item Added" and "App Background Activity" notices when
  # Raycast and Qdrant register themselves. They stay until closed, and their
  # close button shows only while the pointer is over them.
  for _ in 1 2 3; do
    v move 1250 90 pause 1.5 move 1086 52 pause 0.5 click 1 pause 1
  done
  v move 1400 700
}

build_image() {
  local golden
  golden="mash-golden-$(date +%Y%m%d)"
  VM=mash-golden-build
  exists "$PRISTINE" || lume pull "$CUA_IMAGE" "$PRISTINE"
  exists "$VM" && { lume stop "$VM" >/dev/null 2>&1 || true; lume delete "$VM" --force >/dev/null; }
  clone "$PRISTINE" "$VM"
  lume set "$VM" --memory 6GB --display 1440x900 >/dev/null
  boot
  # Passwordless sudo for this guest's own user, so the scripts below can run
  # unattended. It exists only inside the image.
  lume ssh "$VM" "echo lume | sudo -S -p '' sh -c 'echo \"lume ALL=(ALL) NOPASSWD: ALL\" > /etc/sudoers.d/lume && chmod 440 /etc/sudoers.d/lume'" >/dev/null
  guest "mkdir -p ~/mac-vm"
  to_guest "$WORK/tools/mac-vm/" mac-vm/tools/
  guest "~/mac-vm/tools/guest-image.sh system"
  # The corrected login password takes effect at the next login.
  reboot
  guest "~/mac-vm/tools/guest-image.sh apps"
  guest "open /Applications/Raycast.app"
  sleep 10
  mkdir -p build && shot build/raycast-first-launch.png
  v key esc pause 0.5
  dismiss_notices
  # The scenario switches the guest between light and dark through System
  # Events over SSH; macOS asks once whether SSH may control System Events.
  guest "osascript -e 'tell application \"System Events\" to tell appearance preferences to set dark mode to false'" &
  sleep 4
  shot build/automation-prompt.png
  v move 779 388 click 1
  wait
  # Prove the image survives a restart: the login keychain keeps its name, and
  # Raycast opens its database instead of asking to reset it.
  local before after
  before=$(guest "ls ~/Library/Keychains | grep -c renamed || true")
  reboot
  after=$(guest "ls ~/Library/Keychains | grep -c renamed || true")
  guest "pgrep -x Raycast >/dev/null || open /Applications/Raycast.app; sleep 5; open raycast://extensions/raycast/raycast/search-quicklinks" || true
  sleep 4
  shot build/raycast-after-restart.png
  v key esc pause 0.5 key esc
  if [ "$before" != "$after" ]; then
    echo "the login keychain was replaced at restart ($before -> $after renamed keychains)" >&2
    exit 1
  fi
  guest "rm -rf ~/mac-vm; sudo rm -f /etc/sudoers.d/lume; sync"
  lume shutdown "$VM" >/dev/null
  for _ in $(seq 1 40); do [ "$(info "$VM" status)" = stopped ] && break; sleep 3; done
  exists "$golden" && lume delete "$golden" --force >/dev/null
  clone "$VM" "$golden"
  lume delete "$VM" --force >/dev/null
  echo "saved $golden; first-launch and restart screenshots in $STATE/build/"
}

REGISTRY=ghcr.io
ORGANIZATION=mashhq
latest_golden() { lume ls | awk '{print $1}' | grep -E '^mash-golden-[0-9]{8}$' | sort | tail -1; }

credentials() { # GHCR credentials from standard input, exported for Lume only
  local user token
  IFS= read -r user || true
  IFS= read -r token || true
  [ -n "$user" ] && [ -n "$token" ] || return 1
  export GHCR_USERNAME=$user GHCR_TOKEN=$token
}

push_image() {
  local golden
  golden=$(latest_golden)
  [ -n "$golden" ] || { echo "no mash-golden-<yyyymmdd> image to push; run build-image first" >&2; exit 1; }
  credentials || { echo "push-image needs GHCR credentials on standard input" >&2; exit 1; }
  # The image holds macOS, whose licence allows it only on Apple hardware:
  # the package must stay private (tools/mac-vm.sh checks after the push).
  lume push "$golden" "mash-golden:${golden#mash-golden-}" --additional-tags latest \
    --registry "$REGISTRY" --organization "$ORGANIZATION"
  echo "pushed $golden as $REGISTRY/$ORGANIZATION/mash-golden:${golden#mash-golden-} and :latest"
}

pull_golden() { # pull_golden NAME: the registry's latest golden image, as NAME
  credentials || { echo "no local golden image, and no GHCR credentials to pull one" >&2; exit 1; }
  lume pull mash-golden:latest "$1" --registry "$REGISTRY" --organization "$ORGANIZATION" >&2
}

mash_build() { # the Mash build under test, in $WORK/mash; the latest release when none was synced
  ls "$WORK"/mash/mash >/dev/null 2>&1 && return
  local tag
  tag=$(curl -fsSL https://api.github.com/repos/mashhq/mash/releases/latest | python3 -c 'import json,sys; print(json.load(sys.stdin)["tag_name"])')
  mkdir -p "cache/$tag" "$WORK/mash"
  [ -f "cache/$tag/mash.tar.gz" ] ||
    curl -fsSL -o "cache/$tag/mash.tar.gz" "https://github.com/mashhq/mash/releases/download/$tag/mash-aarch64-apple-darwin.tar.gz"
  tar -xzf "cache/$tag/mash.tar.gz" -C "$WORK/mash"
  echo "testing the latest release, $tag"
}

prime_deeplinks() {
  # A fresh Raycast asks once per command before a deep link may run it;
  # answer "Always Run Command" for both commands.
  for command in search memory; do
    guest "open raycast://extensions/james_ko/mash/$command"
    sleep 3
    v move 719 415 click 1 pause 2 key esc pause 0.4 key esc pause 0.4 key esc
  done
}

scenario() {
  rm -rf shots && mkdir shots
  shoot() { # the full screen and the Raycast window
    v capture "shots/$1-screen.png"
    venv/bin/python3 -c "import sys; from PIL import Image; Image.open(sys.argv[1]).crop((330, 100, 1110, 605)).save(sys.argv[2])" \
      "shots/$1-screen.png" "shots/$1.png"
  }
  close() { for _ in 1 2 3 4; do v key esc pause 0.3; done; }
  appearance() { guest "osascript -e 'tell application \"System Events\" to tell appearance preferences to set dark mode to $1'"; }
  v move 1400 700
  for mode in light dark; do
    if [ "$mode" = dark ]; then appearance true; else appearance false; fi
    sleep 2
    close
    guest "open raycast://extensions/james_ko/mash/search"
    sleep 2
    v type checkout pause 4
    shoot "$mode-1-search"
    v key enter pause 3
    shoot "$mode-2-read-messages"
    v key alt-k pause 1.5
    shoot "$mode-3-actions"
    close
    guest "open raycast://extensions/james_ko/mash/memory"
    sleep 3
    shoot "$mode-4-memory"
    close
  done
  appearance false
}

start_clone() { # start_clone [DISPLAY]: a running clone with the build under test and Raycast ready
  local golden
  golden=${MAC_VM_GOLDEN:-$(latest_golden)}
  if [ -z "$golden" ]; then
    golden=mash-golden-registry
    exists "$golden" || pull_golden "$golden"
  fi
  VM="mash-run-$(date +%H%M%S)"
  clone "$golden" "$VM"
  [ -z "${1:-}" ] || lume set "$VM" --display "$1" >/dev/null
  trap '[ "${MAC_VM_KEEP:-}" = 1 ] || { lume stop "$VM" >/dev/null 2>&1; lume delete "$VM" --force >/dev/null 2>&1; }' EXIT
  echo "booting $VM from $golden"
  boot
  mash_build
  guest "mkdir -p ~/mac-vm"
  to_guest "$WORK/tools/mac-vm/" mac-vm/tools/
  to_guest "$WORK/mash/" mac-vm/mash/
  rsync -a --delete --exclude node_modules -e "ssh ${SSH_OPTS[*]}" "$WORK/raycast/" "lume@$(info "$VM" ipAddress):mac-vm/raycast/"
  [ -d cache/models ] && to_guest cache/models/ mac-vm/models/
  guest "~/mac-vm/tools/guest-run.sh"
  # Keep the downloaded models for the next run.
  [ -d cache/models ] || rsync -a -e "ssh ${SSH_OPTS[*]}" "lume@$(info "$VM" ipAddress):.mash/models/" cache/models/
  prime_deeplinks
  dismiss_notices
}
kept() { [ "${MAC_VM_KEEP:-}" != 1 ] || echo "kept $VM running at $(info "$VM" ipAddress); VNC on 127.0.0.1:$VNC_PORT (password mashvm)"; }

run() {
  start_clone
  scenario
  kept
  echo "screenshots in $STATE/shots"
}

# Raycast Store screenshots: 2000x1250 PNGs of the light Raycast window centred
# on a plain neutral background, as Raycast's own Window Capture frames them.
# The guest draws at 2x (1440x900 points on 2880x1800 pixels) and macOS's
# screencapture takes each frame inside it, so nothing is upscaled and no VNC
# encoding touches the pixels; VNC only types the keys.
STORE_BACKGROUND=(222 224 229)
store_shots() {
  start_clone 2880x1800
  rm -rf store && mkdir store
  venv/bin/python3 -c 'import sys; from PIL import Image; Image.new("RGB", (2880, 1800), tuple(map(int, sys.argv[1:4]))).save(sys.argv[4])' \
    "${STORE_BACKGROUND[@]}" store/background.png
  to_guest store/background.png mac-vm/background.png
  guest "~/mac-vm/tools/guest-store.sh setup /Users/lume/mac-vm/background.png"
  close() { for _ in 1 2 3 4; do v key esc pause 0.3; done; }
  capture() { guest "~/mac-vm/tools/guest-store.sh capture $1"; }
  # The VNC framebuffer is 2880x1800 from here on. After the first capture
  # macOS asks once whether SSH may bypass its window picker; answer Allow
  # (the dialog sits in the middle of the empty desktop). A dialog left
  # standing makes store_frame.py fail rather than frame it.
  capture warm-up
  sleep 3
  v move 1440 890 pause 0.5 click 1 pause 1
  # Park the pointer on the desktop, where it highlights nothing;
  # screencapture leaves it out anyway.
  v move 2800 1500
  close
  # "customers" finds a chat from today and two from earlier this week, so
  # the list shows its date groups.
  guest "open raycast://extensions/james_ko/mash/search"
  sleep 2
  v type customers pause 4
  capture 1-search
  v key enter pause 3
  capture 2-read-messages
  v key alt-k pause 1.5
  capture 3-actions
  close
  guest "open raycast://extensions/james_ko/mash/memory"
  sleep 3
  capture 4-memory
  close
  guest "open raycast://extensions/james_ko/mash/search"
  sleep 3
  capture 5-recent
  close
  rsync -a -e "ssh ${SSH_OPTS[*]}" "lume@$(info "$VM" ipAddress):mac-vm/store/" store/screen/
  venv/bin/python3 "$WORK/tools/mac-vm/store_frame.py" store/screen store "${STORE_BACKGROUND[@]}"
  kept
  echo "store screenshots in $STATE/store"
}

case "${1:-}" in
  build-image) build_image ;;
  push-image) push_image ;;
  run) run ;;
  store-shots) store_shots ;;
  list) lume ls ;;
  clean) # stop and delete leftover run clones
    for vm in $(lume ls | awk '{print $1}' | grep '^mash-run-' || true); do
      lume stop "$vm" >/dev/null 2>&1 || true
      lume delete "$vm" --force
    done ;;
  *) sed -n '2,12p' "$0" >&2; exit 2 ;;
esac
