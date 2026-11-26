#!/usr/bin/env bash
# Cut a release: bump the version, wait for CI on the bump, tag, wait for the
# release workflow, and draft the notes. cargo-release (release.toml) does the
# bump, the commit and the tag; this script adds the order a Mash release
# needs, because a tag publishes only a commit CI has validated and the bump
# is a new commit.
#
#   tools/release.sh VERSION              run the whole release
#   tools/release.sh --resume VERSION     continue one that stopped part way
#   tools/release.sh --dry-run VERSION    do it all in a temporary clone; push nothing
#   tools/release.sh --install-tools      install the pinned cargo-release
#
# Run it in a clean checkout of origin/main (tools/worktree.sh deploy-tree).
# The steps, each skipped by --resume when already done:
#   1. Refuse unless the checkout is clean, is origin/main, and ci.yml and
#      windows.yml passed for that commit: the rule release.yml enforces.
#   2. cargo release version/replace/commit: "Release X.Y.Z" changing the
#      workspace version, the desktop app's manifest and both lockfiles.
#   3. Push that commit to main and wait for ci.yml and windows.yml on it.
#   4. cargo release tag: vX.Y.Z. Push the tag and wait for release.yml.
#   5. Write a notes draft and download the winget manifests. Nothing is
#      published from here: the commands to do so are printed.
set -euo pipefail
cargo_release_version=1.1.7
usage() { sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2; exit 2; }
say() { printf '%s\n' "$*" >&2; }
die() { say "release: $*"; exit 1; }
mode=run
version=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) mode=dry-run ;;
        --resume) mode=resume ;;
        --install-tools)
            exec cargo binstall --no-confirm "cargo-release@$cargo_release_version" ;;
        -*) usage ;;
        *) [[ -z $version ]] || usage; version=$1 ;;
    esac
    shift
done
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]] || usage
tag="v$version"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
repo=${MASH_ARTIFACT_REPO:-mashhq/mash}
gh=${MASH_GH:-gh}
poll=${MASH_RELEASE_POLL:-30}
patience=${MASH_RELEASE_WAIT:-7200}
output=${MASH_RELEASE_OUTPUT:-/home/james/scratch/mash-artifacts/releases}
installed=$(cargo release --version 2>/dev/null || true)
[[ $installed == "cargo-release $cargo_release_version" ]] ||
    die "needs cargo-release $cargo_release_version (found: ${installed:-none}); run: make release-tools"

# The newest run of a workflow for a commit: "status conclusion id".
run_state() {
    "$gh" run list --repo "$repo" --workflow "$1" --commit "$2" --limit 1 \
        --json status,conclusion,databaseId --jq '.[0] | "\(.status // "none") \(.conclusion // "") \(.databaseId // "")"' 2>/dev/null || echo 'none  '
}
passed() { [[ $(run_state "$1" "$2") == 'completed success '* ]]; }
# Wait for a workflow's run on a commit to finish, and require success.
await() {
    local workflow=$1 commit=$2 deadline=$((SECONDS + patience)) state last=''
    while :; do
        state=$(run_state "$workflow" "$commit")
        case "$state" in
            'completed success '*) say "$workflow passed for ${commit:0:12}."; return 0 ;;
            'completed '*) die "$workflow did not pass for ${commit:0:12} (${state% *}). Fix it on main, or rerun the workflow, then: tools/release.sh --resume $version" ;;
        esac
        [[ $state == "$last" ]] || say "Waiting for $workflow on ${commit:0:12} (${state%% *})..."
        last=$state
        (( SECONDS < deadline )) || die "$workflow has not finished for ${commit:0:12}. When it has: tools/release.sh --resume $version"
        sleep "$poll"
    done
}
# The bump must be exactly what a release commit has always been.
check_bump() {
    local changed lines
    changed=$(git -C "$tree" show --name-only --format= HEAD | sort | tr '\n' ' ')
    [[ $changed == 'Cargo.lock Cargo.toml desktop/src-tauri/Cargo.lock desktop/src-tauri/Cargo.toml ' ]] ||
        die "the release commit changed unexpected files: $changed"
    lines=$(git -C "$tree" show --format= --unified=0 HEAD | grep -E '^[+-][^+-]' | grep -vcE '^[+-]version = "[^"]+"$' || true)
    [[ $lines == 0 ]] || die 'the release commit changed something other than version lines.'
    for manifest in Cargo.toml desktop/src-tauri/Cargo.toml; do
        grep -qx "version = \"$version\"" "$tree/$manifest" || die "$manifest is not at $version after the bump."
    done
}
bump() {
    (cd "$tree" &&
        cargo release version "$version" --execute --no-confirm --quiet &&
        cargo release replace --execute --no-confirm --quiet &&
        cargo release commit --execute --no-confirm --quiet) || die 'cargo release could not make the release commit.'
    check_bump
    say "Committed $(git -C "$tree" log -1 --format='%h %s')."
}
make_tag() {
    (cd "$tree" && cargo release tag --execute --no-confirm --quiet) || die "cargo release could not create $tag."
    [[ $(git -C "$tree" rev-parse "refs/tags/$tag^{commit}") == "$(git -C "$tree" rev-parse HEAD)" ]] || die "$tag is not the release commit."
}
write_notes() {
    mkdir -p "$output"
    notes="$output/$tag-notes.md"
    (cd "$tree" && bash "$root/tools/release-notes-draft.sh" "$version" "$previous" HEAD) > "$notes"
}

if [[ $mode == dry-run ]]; then
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/mash-release-dry-run.XXXXXXXX")
    tree="$scratch/mash"
    output="$scratch"
    git -C "$root" rev-parse --verify --quiet 'origin/main^{commit}' >/dev/null || die 'this checkout has no origin/main.'
    git clone --quiet "$root" "$tree"
    git -C "$tree" fetch --quiet "$root" 'refs/remotes/origin/main:refs/remotes/origin/main' 'refs/tags/*:refs/tags/*'
    git -C "$tree" checkout --quiet --detach origin/main
    base=$(git -C "$tree" rev-parse HEAD)
    say "Dry run in $tree at origin/main ${base:0:12}; nothing is pushed."
    for workflow in ci.yml windows.yml; do
        passed "$workflow" "$base" || say "NOTE: $workflow has not passed for ${base:0:12}; a real run would refuse here."
    done
else
    tree=$root
    git -C "$tree" fetch --quiet origin main --tags
    base=$(git -C "$tree" rev-parse HEAD)
fi
git -C "$tree" rev-parse --verify --quiet "refs/tags/$tag" >/dev/null && [[ $mode != resume ]] && die "$tag already exists."
previous=$(git -C "$tree" describe --tags --abbrev=0 --match 'v[0-9]*' --exclude "$tag" HEAD 2>/dev/null) || die 'no previous release tag to draft notes from.'

if [[ $(git -C "$tree" log -1 --format=%s) == "Release $version" ]]; then
    [[ $mode == resume ]] || die "HEAD is already \"Release $version\"; continue with --resume."
    check_bump
    say "Resuming at $(git -C "$tree" log -1 --format='%h %s')."
else
    [[ -z $(git -C "$tree" status --porcelain) ]] || die 'the checkout has uncommitted or untracked files; release from a clean tree (tools/worktree.sh deploy-tree).'
    [[ $base == "$(git -C "$tree" rev-parse origin/main)" ]] || die 'the checkout is not at origin/main; update it first (tools/worktree.sh deploy-tree).'
    if [[ $mode != dry-run ]]; then
        for workflow in ci.yml windows.yml; do
            passed "$workflow" "$base" || die "$workflow has not passed for ${base:0:12}; releases ship validated commits only."
        done
    fi
    current=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$tree/Cargo.toml")
    [[ $current != "$version" ]] || die "the version is already $version."
    [[ $(printf '%s\n%s\n' "$current" "$version" | sort -V | tail -n 1) == "$version" ]] || die "$version is older than the current $current."
    bump
fi
commit=$(git -C "$tree" rev-parse HEAD)

if [[ $mode == dry-run ]]; then
    make_tag
    write_notes
    git -C "$tree" show --stat --format='%h %s' HEAD >&2
    cat >&2 <<PLAN
A real run would now:
  git push origin $commit:refs/heads/main      then wait for ci.yml and windows.yml on it
  git push origin refs/tags/$tag               then wait for release.yml
  download the winget-manifests artifact of that release run
Notes draft: $notes
The clone, with the commit and the tag, is kept for inspection: $tree
PLAN
    exit 0
fi

if ! git -C "$tree" merge-base --is-ancestor "$commit" origin/main; then
    say "Pushing $(git -C "$tree" log -1 --format='%h %s') to main."
    git -C "$tree" push origin "$commit:refs/heads/main" ||
        die "main moved on, so the release commit was not pushed. Refresh the tree (tools/worktree.sh deploy-tree) and start again."
fi
await ci.yml "$commit"
await windows.yml "$commit"
git -C "$tree" rev-parse --verify --quiet "refs/tags/$tag" >/dev/null || make_tag
if ! git -C "$tree" ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
    say "Pushing $tag."
    git -C "$tree" push origin "refs/tags/$tag"
fi
await release.yml "$commit"
release_run=$(run_state release.yml "$commit")
release_run=${release_run##* }
write_notes
manifests="$output/$tag-winget-manifests"
rm -rf "$manifests"
if "$gh" run download "$release_run" --repo "$repo" --name winget-manifests --dir "$manifests" >/dev/null 2>&1; then
    winget="Manifests: $manifests
  On Windows, with winget and wingetcreate (packaging/winget/README.md):
    winget validate --manifest $tag-winget-manifests
    wingetcreate submit $tag-winget-manifests
  Or by hand: copy them to manifests/m/MashHQ/Mash/$version/ in a fork of
  microsoft/winget-pkgs and open a pull request titled \"New version: MashHQ.Mash version $version\"."
else
    winget="The winget-manifests artifact of run $release_run could not be downloaded; get it with:
    gh run download $release_run --repo $repo --name winget-manifests --dir $manifests"
fi
cat >&2 <<DONE

$tag is released: https://github.com/$repo/releases/tag/$tag
Left for you:
  1. Notes. Edit the draft, then publish it:
       \$EDITOR $notes
       gh release edit $tag --repo $repo --notes-file $notes
  2. winget. $winget
  3. Deploy when you want it on the fleet (the release's own binaries):
       bash tools/fleet.sh deploy --release $tag shared-api xenia
DONE
