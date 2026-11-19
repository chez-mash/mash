#!/usr/bin/env bash
# Offline release archive fixtures; never downloads or installs outside the fixture.
set -euo pipefail
repository=$(cd "$(dirname "$0")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:?}/install-shell.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/bin" "$scratch/archive" "$scratch/releases" "$scratch/install"
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
export PATH="$scratch/bin:$PATH" MASH_INSTALL_DIR="$scratch/install"
archive=mash-x86_64-unknown-linux-gnu.tar.gz
printf 'cli fixture\n' > "$scratch/archive/mash"
printf 'server fixture\n' > "$scratch/archive/mash-server"
package() {
    tar -C "$scratch/archive" -czf "$scratch/releases/$archive" "$@"
    (cd "$scratch/releases" && sha256sum "$archive" > SHA256SUMS)
}
package mash mash-server
sh "$repository/install.sh"
cmp "$scratch/archive/mash" "$scratch/install/mash"
cmp "$scratch/archive/mash-server" "$scratch/install/mash-server"
[[ -x $scratch/install/mash && -x $scratch/install/mash-server ]]
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
grep -Fxq 'cli fixture' "$scratch/install/mash"
grep -Fxq 'server fixture' "$scratch/install/mash-server"
echo 'Shell installer fixtures passed.'
