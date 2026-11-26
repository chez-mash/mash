#!/usr/bin/env bash
# Fetch the binaries CI already built for this checkout's commit and put them
# where a source build would have, so the configured install steps run
# unchanged. Nothing is installed here.
#
#   fetch-artifact.sh SOURCE PLATFORM_JSON [REPORT]
#
# SOURCE is `commit` (the Dev artifacts workflow's build of HEAD) or
# `release:vX.Y.Z` (that release's assets; the tag must be HEAD, because the
# readiness check expects the installed build to be this checkout's commit).
# PLATFORM_JSON is the platform's `mash_artifacts` entry from the inventory:
#   {"asset": "x86_64-unknown-linux-gnu", "mash": "/abs/mash", "mash_server": "/abs/mash-server"}
#   {"asset": "x86_64-pc-windows-msvc", "windows_host": "winbox", "windows_checkout": "C:\\mash-src"}
# given as JSON or as base64 of it (which survives Ansible's argument
# templating).
#
# Exit 0: verified and in place. Exit 3: nothing to fetch, so build from
# source. Any other status refuses: the download did not match its checksum,
# or is not the program it claims to be, and nothing was put in place.
# REPORT receives the one-line outcome either way.
set -euo pipefail
[[ $# == 2 || $# == 3 ]] || { echo 'usage: fetch-artifact.sh commit|release:vX.Y.Z PLATFORM_JSON [REPORT]' >&2; exit 2; }
source_kind=$1 platform=$2 report=${3:-}
[[ $platform == '{'* ]] || platform=$(base64 -d <<<"$platform")
cd "$(dirname "${BASH_SOURCE[0]}")/.."
repo=${MASH_ARTIFACT_REPO:-mashhq/mash}
# The GitHub CLI; tests name a double here, since a launcher may put its own
# directories ahead of PATH.
gh=${MASH_GH:-gh}
say() {
    echo "$1"
    [[ -z $report ]] || printf '%s\n' "$1" > "$report"
}
unavailable() { say "$1; building from source."; exit 3; }
refuse() { say "REFUSED: $1 Nothing was installed."; exit 1; }
field() { jq -r --arg name "$1" '.[$name] // ""' <<<"$platform"; }
asset=$(field asset)
[[ "$asset" =~ ^[a-z0-9_]+(-[a-z0-9_]+)+$ ]] || { echo 'mash_artifacts needs an "asset" target triple.' >&2; exit 2; }
revision=$(git rev-parse HEAD)
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
extension=tar.gz suffix=''
if [[ $asset == *windows* ]]; then extension=zip suffix=.exe; fi
archive="mash-$asset.$extension"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/mash-artifact.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
mkdir "$scratch/download" "$scratch/unpacked"

case "$source_kind" in
    release:v[0-9]*)
        tag=${source_kind#release:}
        [[ "$tag" =~ ^v[0-9][0-9A-Za-z.+-]*$ ]] || { echo 'Invalid release tag.' >&2; exit 2; }
        tagged=$(git rev-parse --verify --quiet "refs/tags/$tag^{commit}") ||
            refuse "This checkout does not know the tag $tag; run git fetch --tags."
        [[ $tagged == "$revision" ]] ||
            refuse "$tag is commit ${tagged:0:12}, but this checkout is at ${revision:0:12}. Check out $tag in a clean tree and deploy from there."
        assets=$("$gh" release view "$tag" --repo "$repo" --json assets --jq '.assets[].name' 2>/dev/null) ||
            unavailable "$asset: there is no published release $tag"
        grep -qxF "$archive" <<<"$assets" || unavailable "$asset: release $tag has no $archive"
        "$gh" release download "$tag" --repo "$repo" --dir "$scratch/download" \
            --pattern SHA256SUMS --pattern "$archive" >/dev/null ||
            refuse "$asset: downloading $archive from release $tag failed."
        sums="$scratch/download/SHA256SUMS"
        origin="release $tag"
        ;;
    commit)
        # The fleet installs what CI validated: the artifact build finished and
        # the test workflow passed for this exact commit.
        run=$("$gh" run list --repo "$repo" --workflow dev-artifacts.yml --commit "$revision" \
            --status success --limit 1 --json databaseId --jq '.[0].databaseId // ""' 2>/dev/null) || run=''
        [[ "$run" =~ ^[0-9]+$ ]] || unavailable "$asset: CI has no finished artifact build for ${revision:0:12}"
        ci=$("$gh" run list --repo "$repo" --workflow ci.yml --commit "$revision" \
            --status success --limit 1 --json conclusion --jq '.[0].conclusion // ""' 2>/dev/null) || ci=''
        [[ $ci == success ]] || unavailable "$asset: CI has not passed for ${revision:0:12}, so its artifact is not trusted"
        "$gh" run download "$run" --repo "$repo" --name "mash-$asset-$revision" --dir "$scratch/download" >/dev/null 2>&1 ||
            unavailable "$asset: artifact build $run kept no mash-$asset-$revision (expired after 14 days, or not built for this platform)"
        sums="$scratch/download/$archive.sha256"
        origin="CI build $run of ${revision:0:12}"
        ;;
    *) echo 'SOURCE must be commit or release:vX.Y.Z.' >&2; exit 2 ;;
esac

[[ -f "$scratch/download/$archive" ]] || refuse "$asset: the download from $origin did not contain $archive."
[[ -f $sums ]] || refuse "$asset: $origin has no checksum for $archive."
expected=$(awk -v name="$archive" '{ file = $2; sub(/^\*/, "", file) } file == name { print $1 }' "$sums")
[[ "$expected" =~ ^[0-9a-f]{64}$ ]] || refuse "$asset: $origin lists no checksum for $archive."
actual=$(sha256sum "$scratch/download/$archive")
actual=${actual%% *}
[[ $actual == "$expected" ]] || refuse "$asset: $archive from $origin does not match its checksum."

if [[ $extension == zip ]]; then
    python3 -m zipfile -e "$scratch/download/$archive" "$scratch/unpacked" >/dev/null
else
    tar -xzf "$scratch/download/$archive" -C "$scratch/unpacked"
fi
# Each platform's executable format, read from the first bytes so the
# controller needs no `file` utility: ELF, 64-bit ARM Mach-O, or a PE program.
case "$asset" in
    *linux*) magic=7f454c46 ;;
    aarch64-apple-darwin) magic=cffaedfe0c000001 ;;
    *windows*) magic=4d5a ;;
    *) echo "No known executable format for $asset." >&2; exit 2 ;;
esac
for name in mash mash-server; do
    binary="$scratch/unpacked/$name$suffix"
    [[ -f $binary ]] || refuse "$asset: $archive from $origin has no $name$suffix."
    header=$(od -An -tx1 -N$(( ${#magic} / 2 )) "$binary" | tr -d ' \n')
    [[ $header == "$magic" ]] || refuse "$asset: $name$suffix from $origin is not a $asset program."
    chmod 755 "$binary"
done

host=$(field windows_host) checkout=$(field windows_checkout)
if [[ -n $host ]]; then
    # A Windows machine installs from its checkout's staging folder; see
    # install-dev-windows.sh, which requires the revision file written here.
    [[ "$host" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ && "$checkout" =~ ^[A-Za-z]:\\[A-Za-z0-9\\._-]+$ ]] ||
        { echo 'Invalid windows_host or windows_checkout.' >&2; exit 2; }
    transport=(-o ControlPath=none -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=4)
    staged="$checkout\\target\\release"
    ssh "${transport[@]}" "$host" "powershell -NoProfile -NonInteractive -Command \"New-Item -ItemType Directory -Force -Path '$staged' | Out-Null; Remove-Item -Force -ErrorAction SilentlyContinue '$staged\\mash.revision'\""
    for name in mash.exe mash-server.exe; do
        digest=$(sha256sum "$scratch/unpacked/$name")
        digest=${digest%% *}
        scp "${transport[@]}" "$scratch/unpacked/$name" "$host:${staged//\\//}/$name"
        remote=$(ssh "${transport[@]}" "$host" "powershell -NoProfile -NonInteractive -Command \"(Get-FileHash -Algorithm SHA256 '$staged\\$name').Hash.ToLower()\"" | tr -d '\r\n ')
        [[ $remote == "$digest" ]] || refuse "$asset: $name did not reach $host intact."
    done
    ssh "${transport[@]}" "$host" "powershell -NoProfile -NonInteractive -Command \"Set-Content -Path '$staged\\mash.revision' -Value '$revision'\""
else
    cli=$(field mash) server=$(field mash_server)
    [[ $cli == /* && $server == /* ]] || { echo 'mash_artifacts needs absolute "mash" and "mash_server" output paths.' >&2; exit 2; }
    place() {
        local from=$1 to=$2
        mkdir -p "$(dirname "$to")"
        cp -- "$from" "$to.incoming"
        chmod 755 "$to.incoming"
        mv -f -- "$to.incoming" "$to"
        printf '%s\n' "$revision" > "$to.revision"
        sha256sum "$to" > "$to.sha256"
    }
    place "$scratch/unpacked/mash" "$cli"
    place "$scratch/unpacked/mash-server" "$server"
fi
say "$asset: using $origin ($archive, sha256 ${expected:0:12})."
