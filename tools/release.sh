#!/usr/bin/env bash
# Cut a release: bump the version, land the bump through the merge queue,
# tag, wait for the release workflow, and draft the notes. cargo-release
# (release.toml) does the bump, the commit and the tag; this script adds the
# order a Mash release needs, because a tag publishes only a commit CI has
# validated and the bump is a new commit.
#
#   tools/release.sh VERSION              run the whole release
#   tools/release.sh --resume VERSION     continue one that stopped part way
#   tools/release.sh --dry-run VERSION    do it all in a temporary clone; push nothing
#   tools/release.sh --direct VERSION     emergencies only, administrators only: push
#                                         the bump straight to main, past the queue
#   tools/release.sh --install-tools      install the pinned cargo-release
#
# Run it in a clean checkout of origin/main (tools/worktree.sh deploy-tree).
# The steps, each skipped by --resume when already done:
#   1. Refuse unless the checkout is clean, is origin/main, and ci.yml and
#      windows.yml passed for that commit: the rule release.yml enforces.
#   2. cargo release version/replace/commit: "Release X.Y.Z" changing the
#      workspace version, the desktop app's manifest and both lockfiles.
#   3. Push that commit as the branch release/vX.Y.Z and land it through the
#      merge queue (tools/merge-queue.sh), which rebases it onto main under a
#      new name. Check out the commit main got, and wait for ci.yml and
#      windows.yml on it: the queue's own runs count.
#   4. cargo release tag: vX.Y.Z on that commit. Push the tag and wait for
#      release.yml.
#   5. Write a notes draft and download the winget manifests. Nothing is
#      published from here: the commands to do so are printed.
#   6. Render the Scoop manifest (bucket/mash.json) from the release's
#      SHA256SUMS and land "Scoop: mash vX.Y.Z" through the merge queue.
# The release workflow also publishes latest.json, which installed desktop
# apps on the stable channel read to update themselves (see "Updates" in
# docs/internal/desktop-app.md).
set -euo pipefail
cargo_release_version=1.1.7
usage() { sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2; exit 2; }
say() { printf '%s\n' "$*" >&2; }
die() { say "release: $*"; exit 1; }
mode=run
direct=no
version=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) mode=dry-run ;;
        --resume) mode=resume ;;
        --direct) direct=yes ;;
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
    die "needs cargo-release $cargo_release_version (found: ${installed:-none}); run: just release-tools"

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
# The release commit on main, if main has one. The merge queue rebases what
# it merges, so that commit is a copy of the local one under another name and
# is found by its subject.
release_on_main() {
    # awk reads to the end: leaving early would end `git log` with a broken
    # pipe, which pipefail turns into a failure now and then.
    git -C "$tree" log -200 --format='%H %s' origin/main | awk -v subject="Release $version" '
        !found && substr($0, index($0, " ") + 1) == subject { print $1; found = 1 }'
}
# land BRANCH: put HEAD on main, through the merge queue as BRANCH, or with
# --direct straight to main, which GitHub accepts only from an administrator.
land() {
    local branch=$1 head
    head=$(git -C "$tree" rev-parse HEAD)
    if [[ $direct == yes ]]; then
        say "Pushing $(git -C "$tree" log -1 --format='%h %s') straight to main, past the merge queue."
        MASH_PUSH_MAIN=1 git -C "$tree" push origin "$head:refs/heads/main"
        return
    fi
    say "Pushing $(git -C "$tree" log -1 --format='%h %s') as $branch for the merge queue."
    # The branch is this release's alone; a second attempt replaces the first.
    git -C "$tree" push --quiet --force origin "$head:refs/heads/$branch" || return 1
    git -C "$tree" fetch --quiet origin "+refs/heads/$branch:refs/remotes/origin/$branch"
    (cd "$tree" && MASH_GH=$gh MASH_ARTIFACT_REPO=$repo MASH_QUEUE_POLL=$poll MASH_QUEUE_WAIT=$patience \
        bash "$root/tools/merge-queue.sh" submit "$branch" --wait > /dev/null) || return 1
    # GitHub deletes a merged branch when the repository is set to; if not, do.
    git -C "$tree" push --quiet --no-verify origin --delete "$branch" 2> /dev/null || true
}
# The Scoop bucket is bucket/mash.json on main. The release workflow cannot
# commit there, so the manifest is rendered here from the checksums it
# published and landed like any other change. Says what is left when it fails.
land_scoop() {
    local sums
    git -C "$tree" fetch --quiet origin main
    if git -C "$tree" show origin/main:bucket/mash.json 2> /dev/null | grep -q "\"version\": \"$version\""; then
        scoop="The bucket on main is at $version."
        return
    fi
    scoop="The bucket on main is NOT at $version yet; look at the pull request, then: tools/release.sh --resume $version"
    sums=$(mktemp -d "${TMPDIR:-/tmp}/mash-release-sums.XXXXXXXX")
    "$gh" release download "$tag" --repo "$repo" --pattern SHA256SUMS --dir "$sums" > /dev/null 2>&1 || { rm -rf "$sums"; return; }
    git -C "$tree" checkout --quiet --detach origin/main
    python3 "$tree/packaging/render.py" "$tag" "$sums/SHA256SUMS" "$tree/packaging/scoop/mash.json.in" "$tree/bucket/mash.json" || { rm -rf "$sums"; return; }
    rm -rf "$sums"
    git -C "$tree" add bucket/mash.json
    git -C "$tree" commit --quiet -m "Scoop: mash $tag" || return 0
    ! land "release/$tag-scoop" || scoop="The bucket on main is at $version (Scoop: mash $tag)."
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

# A release resumed after the queue merged its commit continues from the
# commit main got, wherever the tree was left.
if [[ $mode == resume ]] && landed=$(release_on_main) && [[ -n $landed && $landed != "$base" ]]; then
    [[ -z $(git -C "$tree" status --porcelain) ]] || die 'the checkout has uncommitted or untracked files; release from a clean tree (tools/worktree.sh deploy-tree).'
    git -C "$tree" checkout --quiet --detach "$landed"
fi
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
  git push origin $commit:refs/heads/release/$tag   and land it through the merge queue
  tag the commit main got; git push origin refs/tags/$tag   then wait for release.yml
  render bucket/mash.json from the release's SHA256SUMS and land "Scoop: mash $tag" the same way
  download the winget-manifests artifact of that release run
Notes draft: $notes
The clone, with the commit and the tag, is kept for inspection: $tree
PLAN
    exit 0
fi

if [[ -z $(release_on_main) ]]; then
    land "release/$tag" ||
        die "the release commit is not on main. When the merge queue has merged it, or once the cause is fixed: tools/release.sh --resume $version"
    git -C "$tree" fetch --quiet origin main
fi
landed=$(release_on_main)
[[ -n $landed ]] || die "main has no \"Release $version\" commit after landing it; look at the pull request, then: tools/release.sh --resume $version"
if [[ $landed != "$commit" ]]; then
    git -C "$tree" checkout --quiet --detach "$landed"
    check_bump
    commit=$landed
    say "main has the release commit as $(git -C "$tree" log -1 --format='%h %s')."
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
land_scoop
cat >&2 <<DONE

$tag is released: https://github.com/$repo/releases/tag/$tag
Left for you:
  1. Notes. Edit the draft, then publish it:
       \$EDITOR $notes
       gh release edit $tag --repo $repo --notes-file $notes
  2. winget. $winget
  3. Deploy when you want it on the fleet (the release's own binaries):
       bash tools/fleet.sh deploy --release $tag shared-api xenia
Scoop: $scoop
Desktop apps on the stable channel update themselves from this release's
latest.json; check it is among the assets (it is missing when the updater
key is not set up: tools/updater-key.sh):
       gh release view $tag --repo $repo --json assets -q '.assets[].name' | grep -x latest.json
DONE
