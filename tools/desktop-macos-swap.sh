#!/usr/bin/env bash
# Runs on the Mac: replace /Applications/Mash.app with the app in a release's
# disk image, which install-desktop-macos.sh has already checked against the
# release's checksums. bash 3.2.
#
#   desktop-macos-swap.sh DISK_IMAGE VERSION
#
# The new app must be one Gatekeeper accepts and signed by Mash's Apple
# Developer team. The app in place is kept as Mash.app.previous until the new
# one has started, and put back if it does not. An app that was running is
# open again afterwards; one that was not is closed again.
set -uo pipefail
[[ $# == 2 ]] || { echo 'usage: desktop-macos-swap.sh DISK_IMAGE VERSION' >&2; exit 2; }
image=$1 version=$2
team=JZ665W692Z
bundle=io.getmash.desktop
applications=${MASH_DESKTOP_APPLICATIONS:-/Applications}
app="$applications/Mash.app"
previous="$applications/Mash.app.previous"
fail() { echo "desktop: $1" >&2; exit 1; }
app_version() { defaults read "$1/Contents/Info" CFBundleShortVersionString 2>/dev/null; }
running() { pgrep -f "^$app/Contents/MacOS/" >/dev/null 2>&1; }
wait_until() { # wait_until SECONDS COMMAND...: true once COMMAND succeeds
    local remaining=$1
    shift
    while ! "$@"; do
        remaining=$((remaining - 1))
        [[ $remaining -gt 0 ]] || return 1
        sleep 1
    done
}
stopped() { ! running; }
quit_app() {
    # Ask the app to quit as the Dock would. Over SSH the request can wait on
    # a permission prompt nobody sees, so it is given a few seconds and then
    # the app is asked through a signal instead.
    osascript -e "tell application id \"$bundle\" to quit" >/dev/null 2>&1 &
    local asker=$!
    wait_until 10 stopped || pkill -TERM -f "^$app/Contents/MacOS/" 2>/dev/null
    kill "$asker" 2>/dev/null
    wait_until 15 stopped
}

if [[ -d $app && $(app_version "$app") == "$version" ]]; then
    echo "desktop: Mash.app $version is already installed."
    exit 0
fi
mount=$(mktemp -d "${TMPDIR:-/tmp}/mash-desktop.XXXXXXXX") || fail 'cannot create a mount point.'
detach() { hdiutil detach "$mount" -quiet >/dev/null 2>&1; rmdir "$mount" 2>/dev/null; }
trap detach EXIT
hdiutil attach -nobrowse -readonly -mountpoint "$mount" "$image" >/dev/null || fail 'the disk image does not open.'
new="$mount/Mash.app"
[[ -d $new ]] || fail 'the disk image has no Mash.app.'
spctl -a -t exec -vv "$new" >/dev/null 2>&1 || fail 'Gatekeeper does not accept the app in the disk image.'
codesign --verify --deep --strict "$new" >/dev/null 2>&1 || fail "the app's signature does not verify."
codesign -dv --verbose=4 "$new" 2>&1 | grep -qx "TeamIdentifier=$team" ||
    fail "the app is not signed by team $team."
[[ $(app_version "$new") == "$version" ]] || fail "the disk image is not version $version."

was_running=no
if running; then
    was_running=yes
    quit_app || fail 'the running app did not quit; nothing was changed.'
fi
rm -rf "$previous"
if [[ -d $app ]]; then mv "$app" "$previous" || fail 'cannot move the installed app aside; nothing was changed.'; fi
restore() {
    rm -rf "$app"
    [[ ! -d $previous ]] || mv "$previous" "$app"
    [[ $was_running == no ]] || open "$app" >/dev/null 2>&1
}
if ! ditto "$new" "$app"; then restore; fail 'copying the new app failed; the previous app is back in place.'; fi
open "$app" >/dev/null 2>&1
if ! wait_until 30 running; then
    restore
    fail "Mash.app $version did not start; the previous app is back in place."
fi
rm -rf "$previous"
if [[ $was_running == no ]]; then
    quit_app || true
    echo "desktop: installed Mash.app $version; it started, and was closed again because it was not open before."
else
    echo "desktop: installed Mash.app $version and reopened it."
fi
