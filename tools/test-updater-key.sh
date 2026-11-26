#!/usr/bin/env bash
# tools/updater-key.sh against stand-ins for the Tauri CLI and the GitHub
# CLI: it makes a key once, keeps it private, writes only the public half
# into the app's configuration, hands each secret to gh on standard input and
# never prints one.
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
fail() { echo "FAIL: $1" >&2; exit 1; }

cat > "$scratch/tauri" <<'TAURI'
#!/usr/bin/env bash
# signer generate --ci -p PASSWORD -w FILE
[[ "$1 $2 $3 $4" == 'signer generate --ci -p' ]] || exit 2
printf 'PRIVATE-KEY-MATERIAL locked-with:%s\n' "$5" > "$7"
printf 'PUBLIC-KEY-TEXT\n' > "$7.pub"
TAURI
cat > "$scratch/gh" <<'GH'
#!/usr/bin/env bash
case "$1 $2" in
    'auth status') exit 0 ;;
    'secret set') cat > "$GH_LOG/$3"; printf '%s\n' "$*" >> "$GH_LOG/arguments" ;;
    *) exit 2 ;;
esac
GH
chmod +x "$scratch/tauri" "$scratch/gh"
mkdir "$scratch/log"
cp "$root/desktop/src-tauri/tauri.conf.json" "$scratch/tauri.conf.json"
run() {
    MASH_UPDATER_KEY_DIR=$scratch/key MASH_UPDATER_CONFIG=$scratch/tauri.conf.json \
        MASH_TAURI=$scratch/tauri MASH_GH=$scratch/gh GH_LOG=$scratch/log \
        bash "$root/tools/updater-key.sh" example/repo
}

run > "$scratch/output" 2>&1 || { cat "$scratch/output" >&2; fail 'the first run failed'; }
password=$(cat "$scratch/key/password")
[[ ${#password} == 48 ]] || fail 'the password is not 48 characters'
for file in mash-updater.key mash-updater.key.pub password; do
    [[ $(stat -c %a "$scratch/key/$file" 2>/dev/null || stat -f %Lp "$scratch/key/$file") == 600 ]] ||
        fail "$file is not private"
done
python3 - "$scratch/tauri.conf.json" <<'PYTHON' || fail 'the public key was not written into the configuration'
import json, sys
settings = json.load(open(sys.argv[1]))
assert settings["plugins"]["updater"]["pubkey"] == "PUBLIC-KEY-TEXT"
assert settings["plugins"]["updater"]["requireSignedVersion"] is True
assert settings["plugins"]["deep-link"]["desktop"]["schemes"] == ["mash"]
PYTHON
cmp -s "$scratch/log/TAURI_SIGNING_PRIVATE_KEY" "$scratch/key/mash-updater.key" || fail 'the private key was not uploaded from its file'
cmp -s "$scratch/log/TAURI_SIGNING_PRIVATE_KEY_PASSWORD" "$scratch/key/password" || fail 'the password was not uploaded from its file'
grep -q -- '--repo example/repo' "$scratch/log/arguments" || fail 'the secrets went to another repository'
if grep -qF -e "$password" -e PRIVATE-KEY-MATERIAL "$scratch/output" "$scratch/log/arguments"; then
    fail 'a secret was printed or passed to gh as an argument'
fi

# A second run uploads the same key and makes no new one.
cp "$scratch/key/mash-updater.key" "$scratch/first"
run > "$scratch/output" 2>&1 || fail 'the second run failed'
cmp -s "$scratch/first" "$scratch/key/mash-updater.key" || fail 'a second run replaced the key'
grep -q 'Using the key already in' "$scratch/output" || fail 'a second run did not say it reused the key'

# Part of a key is refused and nothing is made over it.
rm "$scratch/key/password"
if run > "$scratch/output" 2>&1; then fail 'half a key was accepted'; fi
cmp -s "$scratch/first" "$scratch/key/mash-updater.key" || fail 'a refused run changed the key'
echo 'Updater key: makes one private key, writes the public half, uploads from files, prints no secret.'
