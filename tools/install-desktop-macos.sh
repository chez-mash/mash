#!/usr/bin/env bash
# Install a published release's desktop app on a Mac: download its disk image,
# check it against the release's SHA256SUMS, and hand it to
# desktop-macos-swap.sh on that Mac, which checks the signature and replaces
# /Applications/Mash.app. Releases only; there are no desktop builds of
# unreleased commits.
set -euo pipefail
[[ $# == 2 ]] || { echo 'usage: install-desktop-macos.sh SSH_HOST vX.Y.Z' >&2; exit 2; }
host=$1 tag=$2
[[ "$host" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ && "$tag" =~ ^v[0-9][0-9A-Za-z.+-]*$ ]] || { echo 'Invalid host or release tag.' >&2; exit 2; }
repo=${MASH_ARTIFACT_REPO:-mashhq/mash}
# The GitHub CLI; tests name a double here, since a launcher may put its own
# directories ahead of PATH.
gh=${MASH_GH:-gh}
image=mash-desktop-macos-arm64.dmg
# Use a fresh connection: a reused control master can retain disabled keepalives.
transport=(-o ControlPath=none -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=2)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/mash-desktop.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
"$gh" release download "$tag" --repo "$repo" --dir "$scratch" --pattern SHA256SUMS --pattern "$image" >/dev/null ||
    { echo "desktop: release $tag has no $image to download." >&2; exit 1; }
expected=$(awk -v name="$image" '{ file = $2; sub(/^\*/, "", file) } file == name { print $1 }' "$scratch/SHA256SUMS")
[[ "$expected" =~ ^[0-9a-f]{64}$ ]] || { echo "desktop: release $tag lists no checksum for $image." >&2; exit 1; }
actual=$(sha256sum "$scratch/$image")
[[ ${actual%% *} == "$expected" ]] || { echo "desktop: REFUSED: $image from release $tag does not match its checksum." >&2; exit 1; }
staging="/tmp/mash-desktop-$expected"
ssh "${transport[@]}" "$host" "mkdir -p $staging && chmod 700 $staging"
scp "${transport[@]}" "$scratch/$image" "$(dirname "${BASH_SOURCE[0]}")/desktop-macos-swap.sh" "$host:$staging/"
result=0
ssh "${transport[@]}" "$host" \
    "test \$(shasum -a 256 $staging/$image | cut -d ' ' -f 1) = $expected && bash $staging/desktop-macos-swap.sh $staging/$image ${tag#v}" || result=$?
ssh "${transport[@]}" "$host" "rm -rf $staging" || true
exit "$result"
