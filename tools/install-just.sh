#!/usr/bin/env bash
# Installs the pinned `just` (the task runner; see the justfile) from its own
# release, checked against a pinned checksum. No sudo and no package manager.
#
#   tools/install-just.sh [DIRECTORY]   install into DIRECTORY (default ~/.local/bin)
#   tools/install-just.sh --version     print the pinned version
#
# For Linux, here and in CI. On macOS use `brew install just`, on Windows
# `scoop install just` or `winget install --id Casey.Just --exact`; see
# docs/internal/development.md. To move the pin, change the version and both
# checksums together, from the release's SHA256SUMS file.
set -euo pipefail
version=1.59.0
[[ ${1:-} != --version ]] || { echo "$version"; exit 0; }
[[ $# -le 1 && ${1:-} != -* ]] || { sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2; exit 2; }
directory=${1:-$HOME/.local/bin}
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)
        target=x86_64-unknown-linux-musl
        checksum=409044617b6f06517c1035e55ca38c8c856ff8b650e41bc2aa1ef82d5903a23e ;;
    Linux-aarch64 | Linux-arm64)
        target=aarch64-unknown-linux-musl
        checksum=23871b1702bfd550fb5f901778fd4290ecf2ee034a45a889fbe96d89b188086d ;;
    *)
        echo "install-just: no pinned build for $(uname -s) $(uname -m); use brew, scoop or winget (docs/internal/development.md)" >&2
        exit 2 ;;
esac
if [[ -x $directory/just && $("$directory/just" --version 2> /dev/null) == "just $version" ]]; then
    echo "just $version is already in $directory" >&2
    exit 0
fi
scratch=$(mktemp -d "${TMPDIR:-/tmp}/install-just.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
curl --fail --location --silent --show-error --retry 3 \
    "https://github.com/casey/just/releases/download/$version/just-$version-$target.tar.gz" \
    --output "$scratch/just.tar.gz"
printf '%s  %s\n' "$checksum" "$scratch/just.tar.gz" | sha256sum --check --quiet
tar --no-same-owner -xzf "$scratch/just.tar.gz" -C "$scratch" just
mkdir -p "$directory"
install -m 755 "$scratch/just" "$directory/just"
"$directory/just" --version >&2
case ":$PATH:" in
    *":$directory:"*) ;;
    *) echo "install-just: $directory is not on PATH; add it" >&2 ;;
esac
