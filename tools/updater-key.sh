#!/usr/bin/env bash
# Switch on the desktop app's self-update: make the key pair that signs
# updates, put its public half in the app, and give GitHub the private half.
# Run it once, from a checkout of main, on the machine that keeps the key.
#
#   tools/updater-key.sh [OWNER/REPO]
#
# It writes three files, readable only by you, into ~/.config/mash/updater
# (MASH_UPDATER_KEY_DIR to keep them elsewhere):
#
#   mash-updater.key       the private key, encrypted with the password
#   mash-updater.key.pub   the public key
#   password               the password, 48 random characters
#
# Then it writes the public key into desktop/src-tauri/tauri.conf.json, and
# uploads TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD to
# the repository's Actions secrets with `gh secret set`, each read from its
# file. Neither secret is printed, and neither reaches GitHub's CLI as an
# argument. The Tauri CLI takes a new key's password only as an argument, so
# for the moment the key is made the password is visible to this machine's
# own process list, and to nothing else.
#
# Afterwards: commit and land the tauri.conf.json change. From that commit on,
# builds of main publish the main channel and the next release publishes the
# stable one. Back the folder up somewhere safe: an app only installs updates
# signed with this key, so losing it means every installed app has to be
# updated by hand once. Running this again with the folder in place uploads
# the same key again and never makes a new one.
set -euo pipefail
umask 077

repo=${1:-mashhq/mash}
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
folder=${MASH_UPDATER_KEY_DIR:-$HOME/.config/mash/updater}
key=$folder/mash-updater.key
password=$folder/password
# The app's configuration; a test names a copy.
config=${MASH_UPDATER_CONFIG:-$root/desktop/src-tauri/tauri.conf.json}
# The Tauri CLI and the GitHub CLI; tests name stand-ins.
tauri=${MASH_TAURI:-}
gh=${MASH_GH:-gh}

fail() { echo "updater-key: $1" >&2; exit 1; }
command -v python3 >/dev/null || fail 'python3 is needed to edit tauri.conf.json.'
if [[ -z ${MASH_SKIP_SECRETS:-} ]]; then
    command -v "$gh" >/dev/null || fail 'install the GitHub CLI and run gh auth login first.'
    "$gh" auth status >/dev/null 2>&1 || fail 'run gh auth login first.'
fi

mkdir -p "$folder"
chmod 700 "$folder"
if [[ -f $key && -f $key.pub && -f $password ]]; then
    echo "Using the key already in $folder."
elif [[ -e $key || -e $key.pub || -e $password ]]; then
    fail "$folder holds part of a key. Restore the missing files from your backup, or move the folder away to make a new key (installed apps would then stop updating)."
else
    # 48 characters from the system's random source; no newline in the file.
    LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 48 > "$password" || true
    [[ $(wc -c < "$password") -eq 48 ]] || fail 'could not make a password.'
    # `signer generate` reads the password from nowhere but this option.
    if [[ -n $tauri ]]; then
        "$tauri" signer generate --ci -p "$(cat "$password")" -w "$key" >/dev/null
    else
        (cd "$root/desktop" && pnpm install --frozen-lockfile >/dev/null &&
            pnpm exec tauri signer generate --ci -p "$(cat "$password")" -w "$key" >/dev/null)
    fi
    [[ -s $key && -s $key.pub ]] || fail 'the Tauri CLI did not write a key.'
    echo "Made a new key in $folder."
fi
chmod 600 "$key" "$key.pub" "$password"

# The public key goes into the app. It is not a secret.
python3 - "$config" "$key.pub" <<'PYTHON'
import json
import sys
from pathlib import Path

config = Path(sys.argv[1])
public = Path(sys.argv[2]).read_text(encoding="utf-8").strip()
settings = json.loads(config.read_text(encoding="utf-8"))
settings["plugins"]["updater"]["pubkey"] = public
config.write_text(json.dumps(settings, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
PYTHON
echo "Wrote the public key into ${config#"$root"/}."

if [[ -n ${MASH_SKIP_SECRETS:-} ]]; then
    echo 'MASH_SKIP_SECRETS is set: nothing was uploaded.'
else
    "$gh" secret set TAURI_SIGNING_PRIVATE_KEY --repo "$repo" < "$key"
    "$gh" secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo "$repo" < "$password"
    echo "Set TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD on $repo."
fi
cat <<NEXT

Next:
  1. Back up $folder (a password manager or an encrypted disk).
  2. Commit desktop/src-tauri/tauri.conf.json and land it on main.
See "Updates" in docs/internal/desktop-app.md.
NEXT
