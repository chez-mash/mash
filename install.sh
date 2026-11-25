#!/bin/sh
# Install the latest Mash release binary:
#   curl -fsSL https://raw.githubusercontent.com/mashhq/mash/main/install.sh | sh
# Add `-s -- --dashboard` after `sh` for browser setup, or `-s -- --setup` for terminal setup.
set -eu

usage() {
  cat <<'EOF'
Usage: sh install.sh [--dashboard | --setup]
Installs the verified Mash release into ~/.local/bin by default.
Set MASH_VERSION to pin a release or MASH_INSTALL_DIR to change the directory.
Both options also start local Qdrant so you can choose which chats to index.
Use --dashboard for setup in your browser or --setup for the terminal wizard.
When piping this script to sh, pass an option with sh -s -- OPTION.
EOF
}

mode=install
if [ "$#" -gt 0 ]; then
  case "$#:$1" in
    1:--dashboard) mode=dash ;;
    1:--setup) mode=init ;;
    1:-h | 1:--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
  esac
fi

setup_terminal_required() {
  echo 'mash: --setup needs an interactive terminal. Run without --setup to install the binaries only.' >&2
  exit 2
}

if [ "$mode" = init ]; then
  [ -t 1 ] && [ -t 2 ] || setup_terminal_required
  if [ -t 0 ]; then
    exec 3<&0
  else
    # The installer itself may be piped to sh. Give the wizard the keyboard.
    ( : </dev/tty ) 2>/dev/null || setup_terminal_required
    exec 3</dev/tty
  fi
fi

repository=mashhq/mash
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
if [ "$mode" != install ]; then
  echo 'Preparing the local search service (Qdrant).'
  "$mash" qdrant install </dev/null >/dev/null || {
    status=$?
    {
      echo 'Mash is installed.'
      echo 'See https://getmash.io/troubleshooting/#qdrant-connection-failed to start the search service.'
      printf "Then resume setup with:\n  '%s' %s\n" "$quoted_mash" "$mode"
    } >&2
    exit "$status"
  }
fi
if [ "$mode" = dash ]; then
  echo 'Open the address below in your browser to choose which chats to index.'
  echo 'Keep this terminal open while you use the dashboard.'
  rm -rf -- "$scratch"
  trap - EXIT HUP INT TERM
  exec "$mash" dash
fi
echo 'The setup wizard lets you choose which coding agents can search your chats.'
if [ "$mode" = init ]; then
  "$mash" init <&3 3<&-
  echo 'After setup, start the browser dashboard and open the address it prints.'
else
  echo 'For a new local setup, start the search service first.'
  printf "  '%s' qdrant install\n" "$quoted_mash"
  echo 'Then choose either the terminal wizard or the browser dashboard.'
  echo 'To set up in the terminal, run the wizard.'
  printf "  '%s' init\n" "$quoted_mash"
  echo 'To set up in your browser, start the dashboard and open the address it prints.'
fi
printf "  '%s' dash\n" "$quoted_mash"
