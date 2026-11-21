#!/usr/bin/env bash
# Offline release archive fixtures; never downloads or installs outside the fixture.
set -euo pipefail
repository=$(cd "$(dirname "$0")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:?}/install-shell.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
destination="$scratch/install with 'quotes'"
mkdir -p "$scratch/bin" "$scratch/archive" "$scratch/releases" "$destination"
export TEST_INSTALL_RELEASES="$scratch/releases"
cat > "$scratch/bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
while [[ $# -gt 0 ]]; do
    case $1 in
        -o) output=$2; shift 2 ;;
        https:*) url=$1; shift ;;
        *) shift ;;
    esac
done
cp "$TEST_INSTALL_RELEASES/${url##*/}" "$output"
SH
cat > "$scratch/bin/uname" <<'SH'
#!/usr/bin/env bash
if [[ $1 == -s ]]; then echo Linux; else echo x86_64; fi
SH
# The C library the fixture machine claims to have; empty stands for musl.
cat > "$scratch/bin/getconf" <<'SH'
#!/usr/bin/env bash
[[ $1 == GNU_LIBC_VERSION && -n ${TEST_INSTALL_LIBC-glibc 2.34} ]] || exit 1
echo "${TEST_INSTALL_LIBC-glibc 2.34}"
SH
chmod +x "$scratch/bin/curl" "$scratch/bin/uname" "$scratch/bin/getconf"
export PATH="$scratch/bin:$PATH" MASH_INSTALL_DIR="$destination"
archive=mash-x86_64-unknown-linux-gnu.tar.gz
printf 'cli fixture\n' > "$scratch/archive/mash"
printf 'server fixture\n' > "$scratch/archive/mash-server"
package() {
    tar -C "$scratch/archive" -czf "$scratch/releases/$archive" "$@"
    (cd "$scratch/releases" && sha256sum "$archive" > SHA256SUMS)
}
package mash mash-server
sh "$repository/install.sh" >"$scratch/output"
cmp "$scratch/archive/mash" "$destination/mash"
cmp "$scratch/archive/mash-server" "$destination/mash-server"
[[ -x $destination/mash && -x $destination/mash-server ]]
expected="'$scratch/install with '\\''quotes'\\''/mash'"
grep -Fxq "  $expected qdrant install" "$scratch/output"
grep -Fxq "  $expected init" "$scratch/output"
grep -Fxq "  $expected dash" "$scratch/output"
grep -Fq 'which coding agents can search your chats' "$scratch/output"
# A C library too old for the binary, or not glibc at all, stops the install
# before anything is downloaded or replaced.
for libc in 'glibc 2.33' 'glibc 1.99' ''; do
    if TEST_INSTALL_LIBC="$libc" sh "$repository/install.sh" >"$scratch/output" 2>&1; then
        echo "Installed on an unsupported C library: '$libc'." >&2; exit 1
    fi
    grep -q 'glibc 2.34 or newer' "$scratch/output"
done
TEST_INSTALL_LIBC='glibc 2.41' sh "$repository/install.sh" >/dev/null
# A missing companion must be detected before replacing either installed file.
printf 'replacement\n' > "$scratch/archive/mash"
package mash
if sh "$repository/install.sh" >"$scratch/output" 2>&1; then
    echo 'Incomplete release archive was installed.' >&2; exit 1
fi
grep -Fxq 'cli fixture' "$destination/mash"
grep -Fxq 'server fixture' "$destination/mash-server"

export TEST_INSTALL_SCRIPT="$repository/install.sh" TEST_INSTALL_STEPS="$scratch/steps"
cat > "$scratch/archive/mash" <<'SH'
#!/bin/sh
printf '%s\n' "$*" >> "$TEST_INSTALL_STEPS"
case "$*" in
    'qdrant install')
        echo 'search service JSON fixture'
        if [ "${TEST_INSTALL_QDRANT_FAIL:-0}" = 1 ]; then
            echo 'search service failure fixture' >&2
            exit 7
        fi ;;
    init)
        [ -t 0 ] && [ -t 1 ] && [ -t 2 ] || exit 8
        printf 'Review the histories\n'
        IFS= read -r answer && [ "$answer" = 'fixture consent' ] || exit 9 ;;
    *) exit 10 ;;
esac
SH
package mash mash-server
if sh "$repository/install.sh" --setup >"$scratch/output" 2>&1; then
    echo 'Setup ran with redirected output.' >&2; exit 1
fi
grep -q -- '--setup needs an interactive terminal' "$scratch/output"
grep -Fxq 'cli fixture' "$destination/mash"
[[ ! -e $TEST_INSTALL_STEPS ]]

printf 'fixture consent\n' | script -eqc 'cat "$TEST_INSTALL_SCRIPT" | sh -s -- --setup' /dev/null >"$scratch/output"
diff -u <(printf 'qdrant install\ninit\n') "$TEST_INSTALL_STEPS"
grep -q 'Review the histories' "$scratch/output"
tr -d '\r' <"$scratch/output" | grep -Fxq "  $expected dash"
if grep -q 'search service JSON fixture' "$scratch/output"; then
    echo 'Setup printed the search service JSON.' >&2; exit 1
fi

: > "$TEST_INSTALL_STEPS"
setup_status=0
TEST_INSTALL_QDRANT_FAIL=1 script -eqc 'sh "$TEST_INSTALL_SCRIPT" --setup' /dev/null </dev/null >"$scratch/output" || setup_status=$?
[[ $setup_status == 7 ]]
diff -u <(printf 'qdrant install\n') "$TEST_INSTALL_STEPS"
grep -q 'search service failure fixture' "$scratch/output"
echo 'Shell installer fixtures passed.'
