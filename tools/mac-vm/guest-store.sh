#!/bin/bash
# Runs inside a run clone for `tools/mac-vm.sh store-shots`, after
# guest-run.sh. Prepares the guest for sharp Raycast Store screenshots and
# takes them with macOS's own screencapture, so they are drawn at 2x and carry
# no VNC artefacts. Everything it changes lives in the disposable clone.
#
# Usage: guest-store.sh setup BACKGROUND.png | capture NAME
#   setup    switch the display to 1440x900 points at 2x, let SSH sessions
#            capture the screen, set a plain wallpaper, and turn on light mode
#   capture  save the whole screen to ~/mac-vm/store/NAME.png (2880x1800)
set -euo pipefail
eval "$(/opt/homebrew/bin/brew shellenv)"

setup() {
  # The clone's display is 2880x1800 pixels, but macOS starts it at 1x.
  # Pick the scaled 1440x900 mode, which draws every point as 2x2 pixels.
  command -v displayplacer >/dev/null ||
    HOMEBREW_NO_AUTO_UPDATE=1 brew install -q jakehilborn/jakehilborn/displayplacer >/dev/null
  local screen mode
  screen=$(displayplacer list | sed -n 's/^Persistent screen id: //p' | head -1)
  mode=$(displayplacer list | sed -n 's/^  mode \([0-9]*\): res:1440x900 .*scaling:on.*/\1/p' | head -1)
  [ -n "$screen" ] && [ -n "$mode" ] || { echo "no scaled 1440x900 display mode" >&2; displayplacer list >&2; exit 1; }
  displayplacer "id:$screen mode:$mode"
  sleep 2

  # screencapture over SSH needs Screen Recording for sshd's wrapper. SIP is
  # off in the image, so grant it in the system TCC database directly; the
  # password is the image's own (lume) and exists only in the guest.
  echo lume | sudo -S -p '' sqlite3 "/Library/Application Support/com.apple.TCC/TCC.db" \
    "INSERT OR REPLACE INTO access (service, client, client_type, auth_value, auth_reason, auth_version, flags)
     VALUES ('kTCCServiceScreenCapture', '/usr/libexec/sshd-keygen-wrapper', 1, 2, 4, 1, 0);"

  # A plain neutral wallpaper behind the window, as the Store asks for one
  # background with good contrast. The run already has SSH's permission to
  # control System Events (granted in the golden image).
  osascript -e "tell application \"System Events\" to tell every desktop to set picture to POSIX file \"$1\""
  osascript -e 'tell application "System Events" to tell appearance preferences to set dark mode to false'
  sleep 2
}

capture() {
  mkdir -p ~/mac-vm/store
  screencapture -x -t png ~/mac-vm/store/"$1".png
}

case "${1:-}" in
  setup) setup "$2" ;;
  capture) capture "$2" ;;
  *) sed -n '2,11p' "$0" >&2; exit 2 ;;
esac
