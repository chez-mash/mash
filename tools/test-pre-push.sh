#!/usr/bin/env bash
# Checks tools/pre-push.sh and the tracked hook in a throwaway repository with
# a bare remote: a clean change passes, a too-long file and a mis-formatted
# file stop the push with the fix named, only changed files reach the linters,
# the pushed commit is judged and not a differing working tree, and
# --no-verify still pushes.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pre-push-test.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
fail() {
    printf 'FAIL %s\n' "$1" >&2
    [[ ! -f $scratch/output ]] || sed 's/^/  /' "$scratch/output" >&2
    exit 1
}
git() {
    command git -c user.name=Fixture -c user.email=fixture@example.invalid -c init.defaultBranch=main "$@"
}
commit() {
    git -C "$work" add -A
    git -C "$work" commit -qm "$1"
}
# expect STATUS NAME [PATTERN]: run the check against origin/main and compare.
expect() {
    local status=$1 name=$2 pattern=${3:-} actual=0
    (cd "$work" && bash tools/pre-push.sh) > "$scratch/output" 2>&1 || actual=$?
    [[ $actual == "$status" ]] || fail "$name: exit $actual, expected $status"
    [[ -z $pattern ]] || grep -Eq "$pattern" "$scratch/output" || fail "$name: no line matching $pattern"
}
reset_work() {
    git -C "$work" reset -q --hard origin/main
}

git init -q --bare "$scratch/remote.git"
work=$scratch/work
git init -q "$work"
mkdir -p "$work/tools" "$work/.githooks" "$work/src" "$work/dashboard/src/lib" "$work/dashboard/node_modules/.bin"
cp "$repository/tools/pre-push.sh" "$repository/tools/install-hooks.sh" \
    "$repository/tools/check_file_lines.py" "$repository/tools/file_growth.py" "$work/tools/"
cp "$repository/.githooks/pre-push" "$work/.githooks/"
: > "$work/tools/file-lines-baseline.txt"
printf '[package]\nname = "fixture"\nversion = "0.0.0"\nedition = "2021"\n' > "$work/Cargo.toml"
printf 'pub fn one() -> u8 {\n    1\n}\n' > "$work/src/lib.rs"
printf 'export const a = 1;\n' > "$work/dashboard/src/lib/a.ts"
printf 'export const b = 1;\n' > "$work/dashboard/src/lib/b.ts"
printf 'node_modules/\n' > "$work/.gitignore"
# A stand-in Biome: records the files it was given and rejects a marked one.
cat > "$work/dashboard/node_modules/.bin/biome" <<'BIOME'
#!/usr/bin/env bash
shift 2
printf '%s\n' "$@" >> "$BIOME_CALLS"
! grep -q MISFORMATTED "$@"
BIOME
chmod +x "$work/dashboard/node_modules/.bin/biome"
export BIOME_CALLS=$scratch/biome-calls
commit fixture
git -C "$work" remote add origin "$scratch/remote.git"
git -C "$work" push -q origin main
bash "$work/tools/install-hooks.sh" > /dev/null
[[ $(git -C "$work" config core.hooksPath) == .githooks ]] || fail 'install-hooks sets core.hooksPath'

expect 0 'nothing changed' 'no changed files'

printf 'A note.\n' > "$work/notes.md"
commit docs
expect 0 'a change outside every checked area runs no check' 'none of the quick checks covers'
reset_work

seq 1 501 | sed 's/^/const LINE_&: u16 = &;/' > "$work/src/long.rs"
commit long
expect 1 'a file over the limit' 'pre-push FAILED +line limit .*check_file_lines'
grep -q 'src/long.rs' "$scratch/output" || fail 'the long file is named'
git -C "$work" push -q origin main > "$scratch/output" 2>&1 && fail 'the hook lets a too-long file through'
grep -q 'pre-push FAILED' "$scratch/output" || fail 'the hook reports the failure'
[[ $(git -C "$scratch/remote.git" rev-parse main) != $(git -C "$work" rev-parse HEAD) ]] || fail 'a refused push reached the remote'
reset_work

# The case that reached CI: the committed file is too long while the copy on
# disk, edited since, is short. The commit is what gets pushed, so it is what
# gets checked; --worktree asks about the files on disk instead.
seq 1 501 | sed 's/^/const LINE_&: u16 = &;/' > "$work/src/long.rs"
commit long
printf 'const SHORT: u16 = 1;\n' > "$work/src/long.rs"
printf 'export const stray = 1; // MISFORMATTED\n' > "$work/dashboard/src/lib/untracked.ts"
expect 1 'a too-long commit under a shorter working copy' 'pre-push FAILED +line limit'
grep -q 'temporary checkout' "$scratch/output" || fail 'a differing working tree is not trusted'
(cd "$work" && bash tools/pre-push.sh --worktree) > "$scratch/output" 2>&1 || fail '--worktree checks the files on disk'
git -C "$work" push -q origin main > "$scratch/output" 2>&1 && fail 'the hook lets a too-long commit through under a shorter working copy'
grep -Eq 'pre-push FAILED +line limit' "$scratch/output" || fail 'the hook names the line limit'
[[ $(git -C "$work" worktree list | wc -l) == 1 ]] || fail 'the temporary checkout is removed'
[[ $(cat "$work/src/long.rs") == 'const SHORT: u16 = 1;' ]] || fail 'the working tree is left alone'
rm "$work/dashboard/src/lib/untracked.ts"
reset_work

if (cd "$work" && cargo fmt --version) > /dev/null 2>&1; then
    printf 'pub fn one()->u8{1}\n' > "$work/src/lib.rs"
    commit misformatted
    expect 1 'a mis-formatted Rust file' 'pre-push FAILED +cargo fmt +-> +fix: cargo fmt'
    reset_work
else
    echo 'pre-push: skipped the rustfmt case; cargo fmt is not installed'
fi

: > "$BIOME_CALLS"
printf 'export const a = 2; // MISFORMATTED\n' > "$work/dashboard/src/lib/a.ts"
commit biome
expect 1 'a dashboard file Biome rejects' 'pre-push FAILED +biome \(dashboard\).*biome check --write src/lib/a.ts'
[[ $(cat "$BIOME_CALLS") == src/lib/a.ts ]] || fail 'Biome is given only the changed file'
grep -Eq 'skip +dashboard theme test' "$scratch/output" || fail 'a missing test runner is a note, not a failure'

git -C "$work" push -q --no-verify origin main > "$scratch/output" 2>&1 || fail '--no-verify pushes'
[[ $(git -C "$scratch/remote.git" rev-parse main) == $(git -C "$work" rev-parse HEAD) ]] || fail '--no-verify reached the remote'

printf 'export const a = 3;\n' > "$work/dashboard/src/lib/a.ts"
commit repaired
git -C "$work" push -q origin main > "$scratch/output" 2>&1 || fail 'the hook passes a repaired change'
rm -f "$scratch/output"
echo 'pre-push: ok'
