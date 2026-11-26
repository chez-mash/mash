#!/usr/bin/env bash
# tools/release.sh against a throwaway clone of this checkout with a bare
# repository as its origin and a fake `gh`: the dry run, the refusals, a whole
# release, and resuming one that stopped. Nothing reaches the real remote and
# no real tag is made.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/release-test.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
export GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.invalid
export GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.invalid
fail() {
    printf 'FAIL %s\n' "$1" >&2
    [[ ! -f $scratch/output ]] || sed 's/^/  /' "$scratch/output" >&2
    exit 1
}
refused() { # refused NAME PATTERN COMMAND...: the command fails and says why
    local name=$1 pattern=$2
    shift 2
    if "$@" > "$scratch/output" 2>&1; then fail "$name: was not refused"; fi
    grep -Eq "$pattern" "$scratch/output" || fail "$name: no message matching $pattern"
}

# The fake reports each workflow's newest run for a commit from a file,
# runs/WORKFLOW/COMMIT holding "status conclusion id", and serves one artifact.
mkdir -p "$scratch/bin" "$scratch/runs"
cat > "$scratch/bin/gh" <<'GH'
#!/usr/bin/env bash
set -euo pipefail
echo "$*" >> "$FAKE_GH/calls"
area=$1 action=$2
shift 2
subject='' workflow='' commit='' directory=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo | --json | --jq | --limit | --name) shift 2 ;;
        --workflow) workflow=$2; shift 2 ;;
        --commit) commit=$2; shift 2 ;;
        --dir) directory=$2; shift 2 ;;
        *) subject=$1; shift ;;
    esac
done
case "$area $action" in
    'run list') cat "$FAKE_GH/runs/$workflow/$commit" 2>/dev/null || echo 'none  ' ;;
    'run download') mkdir -p "$directory"; echo "PackageVersion: fixture" > "$directory/MashHQ.Mash.yaml" ;;
    *) echo "unexpected gh $area $action" >&2; exit 2 ;;
esac
GH
chmod 755 "$scratch/bin/gh"
export FAKE_GH="$scratch" MASH_GH="$scratch/bin/gh"
export MASH_RELEASE_POLL=0 MASH_RELEASE_WAIT=2 MASH_RELEASE_OUTPUT="$scratch/out"
mark() { mkdir -p "$scratch/runs/$1"; echo "$3" > "$scratch/runs/$1/$2"; }

# A clone holding this checkout's files as they are now, committed or not, so
# the test judges the release configuration being changed.
tree="$scratch/mash"
git clone --quiet "$repository" "$tree"
(cd "$repository" && git ls-files --cached --others --exclude-standard -z -- \
    Cargo.toml Cargo.lock release.toml 'crates/*/Cargo.toml' app/Cargo.toml desktop/src-tauri/Cargo.toml \
    desktop/src-tauri/Cargo.lock tools/release.sh tools/release-notes-draft.sh |
    xargs -0 -I{} cp --parents {} "$tree/")
git -C "$tree" add -A
git -C "$tree" commit --quiet --allow-empty -m 'Fixture: the checkout under test'
current=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$tree/Cargo.toml")
git -C "$tree" tag --force "v$current" HEAD~0 >/dev/null
printf 'A change since the last release\n' >> "$tree/README.md"
git -C "$tree" commit --quiet -am 'Explain the fixture in the README'
git init --quiet --bare -b main "$scratch/remote.git"
git -C "$tree" remote set-url origin "$scratch/remote.git"
git -C "$tree" push --quiet origin HEAD:refs/heads/main "v$current"
git -C "$tree" fetch --quiet origin
git -C "$tree" checkout --quiet --detach origin/main
base=$(git -C "$tree" rev-parse HEAD)
next="${current%.*}.$(( ${current##*.} + 1 ))"
release() { (cd "$tree" && bash tools/release.sh "$@") > "$scratch/output" 2>&1; }
remote() { git -C "$scratch/remote.git" rev-parse --verify --quiet "$1"; }

# --- Dry run: the commit, the tag and the notes exist only in a temporary clone. ---
release --dry-run "$next" || fail 'dry run failed'
grep -q "NOTE: ci.yml has not passed" "$scratch/output" || fail 'dry run did not say CI has not passed'
clone=$(sed -n 's/^The clone, .*: //p' "$scratch/output")
[[ $(git -C "$clone" log -1 --format=%s) == "Release $next" ]] || fail 'dry run made no release commit'
[[ $(git -C "$clone" show --name-only --format= HEAD | sort | tr '\n' ' ') == 'Cargo.lock Cargo.toml desktop/src-tauri/Cargo.lock desktop/src-tauri/Cargo.toml ' ]] ||
    fail 'dry run changed other files'
# Every version-inheriting package moved in the lockfile; crates with their own versions did not.
[[ $(git -C "$clone" show --format= HEAD -- Cargo.lock | grep -c "^+version = \"$next\"$") == 7 ]] || fail 'the lockfile bump is not the seven workspace packages'
grep -qx "version = \"$next\"" "$clone/desktop/src-tauri/Cargo.toml" || fail 'the desktop manifest was not bumped'
grep -A1 '^name = "mash-desktop"$' "$clone/desktop/src-tauri/Cargo.lock" | grep -qx "version = \"$next\"" || fail 'the desktop lockfile was not bumped'
grep -q '^version = "0.1.0"$' "$clone/crates/mash-model/Cargo.toml" || fail 'an independently versioned crate was bumped'
[[ $(git -C "$clone" rev-parse "v$next^{commit}") == "$(git -C "$clone" rev-parse HEAD)" ]] || fail 'dry run did not tag the release commit'
notes=$(sed -n 's/^Notes draft: //p' "$scratch/output")
grep -q "^## What's new in $next$" "$notes" && grep -q '^Mash searches the chats' "$notes" || fail 'the notes draft lacks the standard opening'
grep -q -- '- Explain the fixture in the README' "$notes" || fail 'the notes draft lacks the commit since the last release'
grep -q "compare/v$current...v$next" "$notes" || fail 'the notes draft lacks the changelog link'
[[ $(git -C "$tree" rev-parse HEAD) == "$base" && -z $(git -C "$tree" status --porcelain) ]] || fail 'dry run changed the checkout'
[[ $(remote main) == "$base" ]] && ! remote "refs/tags/v$next" >/dev/null || fail 'dry run pushed something'
! grep -q 'run download' "$scratch/calls" || fail 'dry run downloaded an artifact'
rm -rf "$(dirname "$clone")"

# --- Refusals: nothing is committed or pushed. ---
refused 'unvalidated commit' 'ci.yml has not passed' release "$next"
mark ci.yml "$base" 'completed success 1'
refused 'Windows not validated' 'windows.yml has not passed' release "$next"
mark windows.yml "$base" 'completed failure 2'
refused 'Windows failed' 'windows.yml has not passed' release "$next"
mark windows.yml "$base" 'completed success 2'
touch "$tree/stray"
refused 'dirty checkout' 'uncommitted or untracked' release "$next"
rm "$tree/stray"
refused 'older version' 'older than the current' release 0.0.1
refused 'same version' 'already' release "$current"
refused 'malformed version' 'usage|tools/release.sh' release "v$next"
git -C "$tree" checkout --quiet --detach HEAD~1
refused 'not origin/main' 'not at origin/main' release "$next"
git -C "$tree" checkout --quiet --detach origin/main
[[ $(remote main) == "$base" && $(git -C "$tree" rev-parse HEAD) == "$base" ]] || fail 'a refused release changed something'

# --- A release that stops while CI runs on the bump, then resumes. ---
refused 'CI still running on the bump' "ci.yml has not finished.*--resume $next" release "$next"
bump=$(git -C "$tree" rev-parse HEAD)
[[ $(git -C "$tree" log -1 --format=%s) == "Release $next" && $(remote main) == "$bump" ]] || fail 'the release commit was not pushed to main'
! remote "refs/tags/v$next" >/dev/null || fail 'the tag was pushed before CI passed on the bump'
! git -C "$tree" rev-parse --verify --quiet "refs/tags/v$next" >/dev/null || fail 'the tag was made before CI passed on the bump'
refused 'second run without --resume' 'continue with --resume' release "$next"
mark ci.yml "$bump" 'completed failure 3'
refused 'CI failed on the bump' "ci.yml did not pass.*--resume $next" release --resume "$next"
! remote "refs/tags/v$next" >/dev/null || fail 'the tag was pushed although CI failed'
mark ci.yml "$bump" 'completed success 3'
mark windows.yml "$bump" 'completed success 4'
refused 'release workflow still running' "release.yml has not finished.*--resume $next" release --resume "$next"
[[ $(remote "refs/tags/v$next^{commit}") == "$bump" ]] || fail 'the tag was not pushed after CI passed'
mark release.yml "$bump" 'completed success 5'
release --resume "$next" || fail 'the resumed release failed'
[[ $(remote main) == "$bump" ]] || fail 'resuming pushed another commit'
grep -q "gh release edit v$next --repo mashhq/mash --notes-file $scratch/out/v$next-notes.md" "$scratch/output" || fail 'no command to publish the notes'
grep -q -- '- Explain the fixture in the README' "$scratch/out/v$next-notes.md" || fail 'no notes draft'
[[ -f "$scratch/out/v$next-winget-manifests/MashHQ.Mash.yaml" ]] && grep -q 'wingetcreate submit' "$scratch/output" || fail 'no winget manifests or steps'
grep -q 'run download 5 ' "$scratch/calls" || fail 'the winget manifests did not come from the release run'
! grep -q 'release edit\|release create' "$scratch/calls" || fail 'the script published notes itself'
refused 'released twice' 'already exists' release "$next"
echo 'Release script: dry run, refusals, CI-gated tag, resume, notes draft and winget steps passed (throwaway remote, fake gh).'
