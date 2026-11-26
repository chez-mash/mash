#!/usr/bin/env bash
# The cheap checks CI would fail a push on, for the files that push changes:
# the line limit, the dashboard theme test, Biome, rustfmt and the Raycast
# lint's Prettier and ESLint. No test suites and no builds, so a typical change
# takes a few seconds. A missing tool or node_modules skips its check with a
# note; CI still runs everything.
#
#   tools/pre-push.sh              changes in origin/main...HEAD
#   tools/pre-push.sh BASE [HEAD]  changes in BASE...HEAD
#   tools/pre-push.sh --hook       as Git's pre-push hook, reading the pushed
#                                  refs from standard input (.githooks/pre-push)
#
# The checks read the files in the working tree, so run it where the pushed
# commit is checked out. `git push --no-verify` skips the hook; keep that for
# emergencies, because CI runs the same checks on the commit.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)" || exit 2
root=$PWD
zero=0000000000000000000000000000000000000000
empty_tree=$(git hash-object -t tree /dev/null)
started=$(date +%s.%N)

# The commit a push is measured from when its remote ref is new or unknown.
default_base() {
    git rev-parse --verify --quiet 'origin/main^{commit}' || echo "$empty_tree"
}
# Changed, still existing files between where BASE and HEAD diverged and HEAD.
changed_between() {
    local base=$1 head=$2 fork
    fork=$(git merge-base "$base" "$head" 2>/dev/null) || fork=$base
    git diff --name-only --diff-filter=ACMR "$fork" "$head"
}

growth_base=''
pushed_heads=()
files=''
add_range() {
    local base=$1 head=$2
    files+=$(changed_between "$base" "$head")$'\n'
    pushed_heads+=("$(git rev-parse "$head")")
    [[ $base == "$empty_tree" ]] || growth_base=$(git merge-base "$base" "$head" 2>/dev/null || echo "$base")
}
if [[ ${1:-} == --hook ]]; then
    while read -r _ local_sha _ remote_sha; do
        [[ -n ${local_sha:-} && $local_sha != "$zero" ]] || continue # a deleted ref
        # `tools/worktree.sh land` has just run these checks for this commit.
        [[ $local_sha != "${MASH_PRE_PUSH_CHECKED:-}" ]] || continue
        if [[ $remote_sha == "$zero" ]] || ! git cat-file -e "$remote_sha^{commit}" 2>/dev/null; then
            remote_sha=$(default_base)
        fi
        add_range "$remote_sha" "$local_sha"
    done
    [[ ${#pushed_heads[@]} -gt 0 ]] || exit 0
else
    base=${1:-$(default_base)}
    git rev-parse --verify --quiet "$base^{tree}" >/dev/null || { echo "pre-push: unknown revision $base" >&2; exit 2; }
    add_range "$base" "${2:-HEAD}"
fi
files=$(printf '%s' "$files" | sort -u | sed '/^$/d')
[[ -n $files ]] || { echo 'pre-push: no changed files to check'; exit 0; }

# Changed files under PREFIX whose names match the extended regex, without the prefix.
under() {
    printf '%s\n' "$files" | grep -E "^$1" | sed "s|^$1||" | grep -E "$2"
}
any() {
    printf '%s\n' "$files" | grep -Eq "$1"
}

failures=()
notes=()
ran=0
# check NAME FIX COMMAND...: run one check and keep its output only on failure.
check() {
    local name=$1 fix=$2 output
    shift 2
    ran=$((ran + 1))
    if output=$("$@" 2>&1); then
        echo "ok    $name"
    else
        echo "FAIL  $name"
        printf '%s\n' "$output" | tail -n 25 | sed 's/^/      /'
        failures+=("$name: $fix")
    fi
}
skip() {
    echo "skip  $1"
    notes+=("$1")
}

head_now=$(git rev-parse HEAD)
for pushed in "${pushed_heads[@]}"; do
    [[ $pushed == "$head_now" ]] || skip "the pushed commit ${pushed:0:8} is not the one checked out here; the checks read this working tree"
done
[[ -z $(git status --porcelain --untracked-files=no | head -n 1) ]] ||
    echo 'note  uncommitted changes are present; the checks read the working tree, not the commit'

# Biome and the theme test come from the package's own node_modules. A fresh
# worktree has none, so Biome alone falls back to the main checkout's copy.
main_checkout=$(git worktree list --porcelain | sed -n '1s/^worktree //p')
biome_for() {
    local candidate
    for candidate in "$root/$1/node_modules/.bin/biome" "$main_checkout/$1/node_modules/.bin/biome"; do
        [[ -x $candidate ]] && { echo "$candidate"; return; }
    done
    return 1
}
in_dir() {
    local dir=$1
    shift
    (cd "$root/$dir" && "$@")
}

if any '\.(rs|ts|svelte|py)$|^tools/(file-lines-baseline\.txt|file-reviews/)'; then
    if command -v python3 >/dev/null; then
        # -B: a bytecode cache in tools/ would be an untracked file, and
        # `worktree.sh done` and `deploy-tree` refuse a tree that has one.
        against=()
        [[ -z $growth_base ]] || against=(--against "$growth_base")
        check 'line limit' 'split the file along a real responsibility, or write the growth review the message names; rerun python3 tools/check_file_lines.py' \
            python3 -B tools/check_file_lines.py "${against[@]}"
    else
        skip 'line limit: python3 is not installed'
    fi
fi

biome_files='\.(ts|js|mjs|cjs|svelte|css|json|jsonc|html|astro)$'
for package in dashboard site; do
    mapfile -t changed < <(under "$package/" "$biome_files")
    [[ ${#changed[@]} -gt 0 ]] || continue
    if biome=$(biome_for "$package"); then
        check "biome ($package)" "cd $package && pnpm exec biome check --write ${changed[*]}" \
            in_dir "$package" "$biome" check --no-errors-on-unmatched "${changed[@]}"
    else
        skip "biome ($package): $package/node_modules is missing; run pnpm install --frozen-lockfile in $package/"
    fi
done

if any '^dashboard/src/'; then
    if [[ -x dashboard/node_modules/.bin/vitest ]]; then
        check 'dashboard theme test' 'name a role token from dashboard/src/theme/tokens.css, or remove the unused one; rerun cd dashboard && pnpm exec vitest run src/theme/theme.test.ts' \
            in_dir dashboard node_modules/.bin/vitest run src/theme/theme.test.ts
    else
        skip 'dashboard theme test: dashboard/node_modules is missing; run pnpm install --frozen-lockfile in dashboard/'
    fi
fi

rust_root=$(printf '%s\n' "$files" | grep -E '\.rs$' | grep -Ev '^desktop/src-tauri/' | head -n 1)
rust_desktop=$(under 'desktop/src-tauri/' '\.rs$' | head -n 1)
if [[ -n $rust_root || -n $rust_desktop ]] && ! command -v cargo >/dev/null; then
    skip 'rustfmt: cargo is not installed'
else
    [[ -z $rust_root ]] || check 'cargo fmt' 'cargo fmt' cargo fmt --check
    [[ -z $rust_desktop ]] || check 'cargo fmt (desktop)' 'cargo fmt --manifest-path desktop/src-tauri/Cargo.toml' \
        cargo fmt --check --manifest-path desktop/src-tauri/Cargo.toml
fi

mapfile -t changed < <(under 'raycast/' '^src/.*\.(ts|tsx|js|jsx|json)$')
if [[ ${#changed[@]} -gt 0 ]]; then
    if [[ -x raycast/node_modules/.bin/prettier && -x raycast/node_modules/.bin/eslint ]]; then
        check 'prettier (raycast)' 'cd raycast && npm run fix-lint' in_dir raycast node_modules/.bin/prettier --check "${changed[@]}"
        mapfile -t scripts < <(printf '%s\n' "${changed[@]}" | grep -Ev '\.json$')
        [[ ${#scripts[@]} -eq 0 ]] || check 'eslint (raycast)' 'cd raycast && npm run fix-lint' in_dir raycast node_modules/.bin/eslint "${scripts[@]}"
    else
        skip 'raycast lint: raycast/node_modules is missing; run npm ci in raycast/'
    fi
fi

elapsed=$(awk -v start="$started" -v end="$(date +%s.%N)" 'BEGIN { printf "%.1f", end - start }')
count=$(printf '%s\n' "$files" | wc -l)
if [[ $ran -eq 0 && ${#notes[@]} -eq 0 ]]; then
    echo "pre-push: none of the quick checks covers the $count changed files"
    exit 0
fi
if [[ ${#failures[@]} -eq 0 ]]; then
    echo "pre-push: $ran checks passed for $count changed files in ${elapsed}s${notes:+ (${#notes[@]} skipped)}"
    exit 0
fi
echo
for failure in "${failures[@]}"; do
    echo "pre-push FAILED  ${failure%%: *}  ->  fix: ${failure#*: }" >&2
done
echo "pre-push: ${#failures[@]} of $ran checks failed in ${elapsed}s; CI would fail this push. (Emergencies only: git push --no-verify)" >&2
exit 1
