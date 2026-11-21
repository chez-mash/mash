#!/bin/sh
# Install the latest Mash release binary:
#   curl -fsSL https://raw.githubusercontent.com/mashium/mash/main/install.sh | sh
# Add `-s -- --setup` after `sh` to also start local Qdrant and open the setup wizard.
set -eu

usage() {
  cat <<'EOF'
Usage: sh install.sh [--setup]
Installs the verified Mash release into ~/.local/bin by default.
Set MASH_VERSION to pin a release or MASH_INSTALL_DIR to change the directory.
Use --setup in a terminal to also install and start local Qdrant and review
which histories Mash will read. When piping this script to sh, use sh -s -- --setup.
EOF
}

setup=false
if [ "$#" -gt 0 ]; then
  case "$#:$1" in
    1:--setup) setup=true ;;
    1:-h | 1:--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
fi

setup_terminal_required() {
  echo 'mash: --setup needs an interactive terminal. Run without --setup to install the binaries only.' >&2
  exit 2
}

if "$setup"; then
  [ -t 1 ] && [ -t 2 ] || setup_terminal_required
  if [ -t 0 ]; then
    exec 3<&0
  else
    # The installer itself may be piped to sh. Give the wizard the keyboard.
    ( : </dev/tty ) 2>/dev/null || setup_terminal_required
    exec 3</dev/tty
  fi
fi

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
mash="$directory/mash"
echo "Installed $directory/mash and $directory/mash-server"
case ":$PATH:" in
  *":$directory:"*) ;;
  *) echo "Add $directory to your PATH." ;;
esac
quoted_mash=$(printf '%s' "$mash" | sed "s/'/'\\\\''/g")
echo 'The setup wizard lets you choose which coding agents can search your chats.'
if "$setup"; then
  echo 'Preparing the local search service (Qdrant). The setup wizard opens next.'
  "$mash" qdrant install </dev/null >/dev/null
  "$mash" init <&3 3<&-
else
  echo 'For a new local setup, install the search service and then run the setup wizard.'
  printf "  '%s' qdrant install\n  '%s' init\n" "$quoted_mash" "$quoted_mash"
fi
echo 'After setup, start the browser dashboard and open the address it prints.'
printf "  '%s' dash\n" "$quoted_mash"
