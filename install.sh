#!/bin/sh
# Install the latest Mash release binary:
#   curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
# Set MASH_VERSION (for example v0.8.0) to pin a release and MASH_INSTALL_DIR to
# choose the directory (default ~/.local/bin). Verifies the release checksum.
set -eu

case "${1:-}" in
  "") ;;
  -h | --help)
    sed -n '2,5s/^# \{0,1\}//p' "$0" 2> /dev/null || echo 'Installs the latest Mash release; see https://getmash.io/installation/'
    exit 0 ;;
  *) echo "mash: install.sh takes no arguments; set MASH_VERSION or MASH_INSTALL_DIR instead" >&2; exit 2 ;;
esac

repository=mashium/mash
directory=${MASH_INSTALL_DIR:-$HOME/.local/bin}
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-gnu ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  *) echo "mash: no prebuilt binary for $(uname -s) $(uname -m); build from source with cargo" >&2; exit 1 ;;
esac
# The Linux binaries are built against glibc 2.34; say so before downloading
# something that would not start.
if [ "$(uname -s)" = Linux ]; then
  libc=$(getconf GNU_LIBC_VERSION 2> /dev/null || true)
  case "$libc" in
    "glibc "*.*)
      found=${libc#glibc }
      major=${found%%.*}
      minor=${found#*.}
      minor=${minor%%.*}
      if [ "$major" -lt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -lt 34 ]; }; then
        echo "mash: the prebuilt binary needs glibc 2.34 or newer and this system has $found (Ubuntu 22.04, Debian 12 and RHEL 9 are the oldest that work)" >&2
        exit 1
      fi ;;
    *)
      echo "mash: the prebuilt binary needs glibc 2.34 or newer, which this system does not appear to use (Alpine and other musl systems are not supported)" >&2
      exit 1 ;;
  esac
fi
if [ -n "${MASH_VERSION:-}" ]; then
  base="https://github.com/$repository/releases/download/$MASH_VERSION"
else
  base="https://github.com/$repository/releases/latest/download"
fi
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT HUP INT TERM
archive="mash-$target.tar.gz"
curl -fsSL "$base/$archive" -o "$scratch/$archive"
curl -fsSL "$base/SHA256SUMS" -o "$scratch/SHA256SUMS"
(
  cd "$scratch"
  grep " $archive\$" SHA256SUMS > expected
  if command -v sha256sum > /dev/null; then sha256sum -c expected; else shasum -a 256 -c expected; fi
) > /dev/null
tar -xzf "$scratch/$archive" -C "$scratch"
[ -f "$scratch/mash" ] && [ -f "$scratch/mash-server" ] || {
  echo 'Release archive must contain mash and mash-server.' >&2; exit 1;
}
mkdir -p "$directory"
install -m 0755 "$scratch/mash" "$directory/mash"
install -m 0755 "$scratch/mash-server" "$directory/mash-server"
echo "Installed $directory/mash and $directory/mash-server"
case ":$PATH:" in
  *":$directory:"*) ;;
  *) echo "Add $directory to your PATH." ;;
esac
quoted_mash=$(printf '%s' "$directory/mash" | sed "s/'/'\\\\''/g")
echo "For a new local setup, install the search service and then run the setup wizard."
printf "  '%s' qdrant install\n  '%s' init\n" "$quoted_mash" "$quoted_mash"
