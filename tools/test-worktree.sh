#!/usr/bin/env bash
# Checks tools/worktree.sh against a throwaway bare remote and clone: new,
# land (rebased, checked, fast-forward only), done and its refusals, and the
# deploy tree. Nothing here touches the network or the real repository.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/worktree-test.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
export GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.invalid
export GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.invalid
unset MASH_WORKTREES
fail() {
    printf 'FAIL %s\n' "$1" >&2
    [[ ! -f $scratch/output ]] || sed 's/^/  /' "$scratch/output" >&2
    exit 1
}
# refused NAME PATTERN COMMAND...: the command fails and says why.
refused() {
    local name=$1 pattern=$2
    shift 2
    if "$@" > "$scratch/output" 2>&1; then fail "$name: was not refused"; fi
    grep -Eq "$pattern" "$scratch/output" || fail "$name: no message matching $pattern"
}
remote_main() { git -C "$scratch/remote.git" rev-parse main; }

git init -q --bare -b main "$scratch/remote.git"
main=$scratch/mash
git init -q -b main "$main"
mkdir -p "$main/tools" "$main/.githooks" "$main/src"
cp "$repository/tools/worktree.sh" "$repository/tools/pre-push.sh" "$repository/tools/install-hooks.sh" \
    "$repository/tools/check_file_lines.py" "$repository/tools/file_growth.py" "$main/tools/"
cp "$repository/.githooks/pre-push" "$main/.githooks/"
: > "$main/tools/file-lines-baseline.txt"
printf 'one\n' > "$main/notes.md"
# Ignored in the real repository: the agent skills and the Claude settings.
mkdir -p "$main/.agents/skills/example" "$main/.agents/plugins" "$main/.claude"
printf 'skill\n' > "$main/.agents/skills/example/SKILL.md"
printf '{}\n' > "$main/.agents/plugins/marketplace.json"
printf '{}\n' > "$main/.claude/settings.json"
printf '/.agents/*\n!/.agents/plugins/\n/.claude/\n' > "$main/.gitignore"
git -C "$main" add -A
git -C "$main" commit -qm fixture
git -C "$main" remote add origin "$scratch/remote.git"
git -C "$main" push -q origin main > /dev/null 2>&1
bash "$main/tools/install-hooks.sh" > /dev/null
worktree() { bash "$main/tools/worktree.sh" "$@"; }

refused 'a name with a slash' 'a name starts with' worktree new bad/name
tree=$(worktree new alpha 2> "$scratch/output") || fail 'new alpha'
[[ $tree == "$scratch/mash-worktrees/alpha" && -d $tree ]] || fail "new prints the worktree path, got $tree"
[[ $(git -C "$tree" rev-parse --abbrev-ref HEAD) == work/alpha ]] || fail 'new creates branch work/alpha'
[[ $(git -C "$tree" rev-parse HEAD) == $(remote_main) ]] || fail 'new starts from origin/main'
grep -q DEV_TARGET_DIR "$scratch/output" || fail 'new explains the shared Cargo target'
[[ -L $tree/.agents/skills && -f $tree/.agents/skills/example/SKILL.md ]] || fail 'new links the skills from the main checkout'
[[ -L $tree/.claude && -f $tree/.claude/settings.json ]] || fail 'new links .claude from the main checkout'
[[ -z $(git -C "$tree" status --porcelain) ]] || fail 'the links leave the new worktree clean'
grep -q 'bd ready' "$scratch/output" || fail 'new says where to run bd'
refused 'a second worktree of the same name' 'already exists' worktree new alpha
worktree list | grep -q 'work/alpha' || fail 'list shows the worktree'

# Main moves while alpha is in progress, as another session's landing would.
printf 'two\n' > "$main/other.md"
git -C "$main" add other.md
git -C "$main" commit -qm other
git -C "$main" push -q origin main > /dev/null 2>&1

printf 'alpha\n' > "$tree/alpha.md"
refused 'done with an untracked file' 'uncommitted or untracked' worktree done alpha
git -C "$tree" add alpha.md
refused 'land with staged changes' 'uncommitted changes' worktree land alpha
git -C "$tree" commit -qm alpha
refused 'done with unlanded commits' 'not on origin/main' worktree done alpha

# A file over the line limit stops the landing before anything is pushed.
mkdir -p "$tree/src"
seq 1 501 | sed 's/^/const LINE_&: u16 = &;/' > "$tree/src/long.rs"
git -C "$tree" add src/long.rs
git -C "$tree" commit -qm long
before=$(remote_main)
refused 'land with a failing check' 'pre-push check failed' worktree land alpha
[[ $(remote_main) == "$before" ]] || fail 'a failed landing reached the remote'
git -C "$tree" reset -q --hard HEAD~1

worktree land alpha > "$scratch/output" 2>&1 || fail 'land alpha'
[[ $(remote_main) == $(git -C "$tree" rev-parse HEAD) ]] || fail 'land fast-forwards origin main to the branch'
git -C "$scratch/remote.git" cat-file -e main:other.md || fail 'land rebased onto the moved main'
[[ $(git -C "$scratch/remote.git" rev-list --count --merges main) == 0 ]] || fail 'land keeps history linear'
[[ -d $tree ]] || fail 'land leaves the worktree for done'
worktree done alpha > "$scratch/output" 2>&1 || fail 'done alpha'
[[ ! -e $tree ]] || fail 'done removes the worktree'
! git -C "$main" show-ref --verify --quiet refs/heads/work/alpha || fail 'done deletes the merged branch'

# --force discards a dirty, unlanded worktree.
tree=$(worktree new beta 2> /dev/null)
printf 'beta\n' > "$tree/beta.md"
git -C "$tree" add beta.md
git -C "$tree" commit -qm beta
printf 'draft\n' > "$tree/draft.md"
refused 'done on unfinished work' 'uncommitted or untracked' worktree done beta
worktree done beta --force > "$scratch/output" 2>&1 || fail 'done --force'
[[ ! -e $tree ]] || fail 'done --force removes the worktree'
! git -C "$main" show-ref --verify --quiet refs/heads/work/beta || fail 'done --force deletes the branch'

# The deploy tree follows origin/main, detached, and refuses local changes.
deploy=$(worktree deploy-tree 2> /dev/null)
[[ $deploy == "$scratch/mash-worktrees/_deploy" ]] || fail "deploy-tree prints its path, got $deploy"
[[ $(git -C "$deploy" rev-parse HEAD) == $(remote_main) ]] || fail 'the deploy tree is at origin/main'
[[ $(git -C "$deploy" rev-parse --abbrev-ref HEAD) == HEAD ]] || fail 'the deploy tree is detached'
git -C "$main" pull -q --ff-only origin main
printf 'three\n' > "$main/third.md"
git -C "$main" add third.md
git -C "$main" commit -qm third
git -C "$main" push -q origin main > /dev/null 2>&1
worktree deploy-tree > /dev/null 2>&1 || fail 'deploy-tree refreshes'
[[ $(git -C "$deploy" rev-parse HEAD) == $(remote_main) ]] || fail 'the deploy tree follows origin/main'
printf 'stray\n' > "$deploy/stray.md"
refused 'a deploy tree with local changes' 'must stay clean' worktree deploy-tree
rm -f "$scratch/output"
echo 'worktree: ok'
