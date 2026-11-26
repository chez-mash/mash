#!/usr/bin/env bash
# The cheap checks CI would fail a push on, for the files that push changes:
# the line limit, the global CSS budget, the dashboard theme test, Biome,
# rustfmt and the Raycast lint's Prettier and ESLint. No test suites and no builds, so a typical change
# takes a few seconds. A missing tool or node_modules skips its check with a
# note; CI still runs everything.
#
#   tools/pre-push.sh [BASE [HEAD]]   the commit HEAD, for changes in BASE...HEAD
#                                     (BASE defaults to origin/main)
#   tools/pre-push.sh --worktree [BASE]
#                                     the files on disk, committed or not
#   tools/pre-push.sh --hook          as Git's pre-push hook, reading the pushed
#                                     refs from standard input (.githooks/pre-push)
#
# The checks judge the commit being pushed, which is what CI will see, not the
# working tree: a checkout with uncommitted or untracked files is not trusted,
# and the commit is checked out to a temporary directory instead. Use
# --worktree to check edits before committing them. `git push --no-verify`
# skips the hook; keep that for emergencies, because CI runs the same checks.
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

if [[ ${1:-} == --hook ]]; then
    # One run of this script per pushed ref, each for its own commit.
    result=0
    while read -r _ local_sha _ remote_sha; do
        [[ -n ${local_sha:-} && $local_sha != "$zero" ]] || continue # a deleted ref
        # `tools/worktree.sh land` has just run these checks for this commit.
        [[ $local_sha != "${MASH_PRE_PUSH_CHECKED:-}" ]] || continue
        if [[ $remote_sha == "$zero" ]] || ! git cat-file -e "$remote_sha^{commit}" 2>/dev/null; then
            remote_sha=$(default_base)
        fi
        bash "${BASH_SOURCE[0]}" "$remote_sha" "$local_sha" < /dev/null || result=1
    done
    exit "$result"
fi

on_disk=no
[[ ${1:-} != --worktree ]] || { on_disk=yes; shift; }
base=${1:-$(default_base)}
git rev-parse --verify --quiet "$base^{tree}" >/dev/null || { echo "pre-push: unknown revision $base" >&2; exit 2; }
head=$(git rev-parse --verify --quiet "${2:-HEAD}^{commit}") || { echo "pre-push: unknown revision ${2:-}" >&2; exit 2; }
fork=$(git merge-base "$base" "$head" 2>/dev/null) || fork=$base
growth_base=''
[[ $fork == "$empty_tree" ]] || growth_base=$fork
# Changed, still existing files since BASE and HEAD diverged.
if [[ $on_disk == yes ]]; then
    files=$(git diff --name-only --diff-filter=ACMR "$fork")
else
    files=$(git diff --name-only --diff-filter=ACMR "$fork" "$head")
fi
[[ -n $files ]] || { echo 'pre-push: no changed files to check'; exit 0; }

# Where the checks read files. This checkout serves when it holds exactly the
# commit: the same HEAD and nothing uncommitted or untracked (an untracked
# source file would reach the theme test). Otherwise the commit is checked out
# to a temporary worktree, removed on exit.
tree=$root
scratch=''
cleanup() {
    [[ -z $scratch ]] || { git worktree remove --force "$scratch/tree" > /dev/null 2>&1; rm -rf "$scratch"; }
}
trap cleanup EXIT
main_checkout=$(git worktree list --porcelain | sed -n '1s/^worktree //p')
# A fresh worktree has no node_modules; the main checkout's are borrowed by
# linking them into a temporary checkout, never into this one.
borrows_modules() {
    local package
    for package in dashboard site raycast; do
        printf '%s\n' "$files" | grep -q "^$package/" || continue
        [[ -d $root/$package/node_modules || ! -d $main_checkout/$package/node_modules ]] || return 0
    done
    return 1
}
differs_from_commit() {
    [[ $head != "$(git rev-parse HEAD)" || -n $(git status --porcelain | head -n 1) ]]
}
if [[ $on_disk == no ]] && { differs_from_commit || borrows_modules; }; then
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/mash-pre-push.XXXXXXXX") || exit 2
    tree=$scratch/tree
    git worktree add --quiet --detach "$tree" "$head" > /dev/null 2>&1 || { echo "pre-push: could not check out ${head:0:8} to $tree" >&2; exit 2; }
    echo "note  checking commit ${head:0:8} in a temporary checkout"
fi

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

# Biome, the theme test and the Raycast lint need a package's node_modules:
# this checkout's, or in a temporary checkout a link to this one's or the main
# checkout's.
modules_for() {
    local candidate
    [[ ! -d $tree/$1/node_modules ]] || return 0
    [[ $tree != "$root" ]] || return 1
    for candidate in "$root/$1/node_modules" "$main_checkout/$1/node_modules"; do
        [[ -d $candidate ]] || continue
        ln -s "$candidate" "$tree/$1/node_modules"
        return 0
    done
    return 1
}
in_dir() {
    local dir=$1
    shift
    (cd "$tree/$dir" && "$@")
}

if any '\.(rs|ts|svelte|py)$|^tools/(file-lines-baseline\.txt|file-reviews/)'; then
    if command -v python3 >/dev/null; then
        # -B: a bytecode cache in tools/ would be an untracked file, and
        # `worktree.sh done` and `deploy-tree` refuse a tree that has one.
        against=()
        [[ -z $growth_base ]] || against=(--against "$growth_base")
        check 'line limit' 'split the file along a real responsibility, or write the growth review the message names; rerun python3 tools/check_file_lines.py' \
            in_dir . python3 -B tools/check_file_lines.py "${against[@]}"
    else
        skip 'line limit: python3 is not installed'
    fi
fi

if any '\.css$|^tools/(css-budget\.json|check_css_budget\.py)$'; then
    if command -v python3 >/dev/null; then
        check 'css budget' "use Tailwind utilities or the component's scoped styles (token roles only) instead of global CSS; after removing CSS run python3 tools/check_css_budget.py --update" \
            in_dir . python3 -B tools/check_css_budget.py
    else
        skip 'css budget: python3 is not installed'
    fi
fi

biome_files='\.(ts|js|mjs|cjs|svelte|css|json|jsonc|html|astro)$'
for package in dashboard site; do
    mapfile -t changed < <(under "$package/" "$biome_files")
    [[ ${#changed[@]} -gt 0 ]] || continue
    if modules_for "$package" && [[ -x $tree/$package/node_modules/.bin/biome ]]; then
        check "biome ($package)" "cd $package && pnpm exec biome check --write ${changed[*]}" \
            in_dir "$package" node_modules/.bin/biome check --no-errors-on-unmatched "${changed[@]}"
    else
        skip "biome ($package): $package/node_modules is missing; run pnpm install --frozen-lockfile in $package/"
    fi
done

if any '^dashboard/src/'; then
    if modules_for dashboard && [[ -x $tree/dashboard/node_modules/.bin/vitest ]]; then
        check 'dashboard theme test' 'name a role token from dashboard/src/theme/tokens.css, or remove the unused one; rerun cd dashboard && pnpm exec vitest run src/theme/theme.test.ts' \
            in_dir dashboard node_modules/.bin/vitest run src/theme/theme.test.ts
    else
        skip 'dashboard theme test: dashboard/node_modules is missing; run pnpm install --frozen-lockfile in dashboard/'
    fi
fi

rust_root=$(printf '%s\n' "$files" | grep -E '\.rs$' | grep -Ev '^desktop/src-tauri/' | head -n 1)
rust_desktop=$(under 'desktop/src-tauri/' '\.rs$' | head -n 1)
if [[ -z $rust_root && -z $rust_desktop ]]; then
    :
elif ! in_dir . cargo fmt --version >/dev/null 2>&1; then
    skip 'rustfmt: cargo fmt is not installed for this toolchain'
else
    [[ -z $rust_root ]] || check 'cargo fmt' 'cargo fmt' in_dir . cargo fmt --check
    [[ -z $rust_desktop ]] || check 'cargo fmt (desktop)' 'cargo fmt --manifest-path desktop/src-tauri/Cargo.toml' \
        in_dir . cargo fmt --check --manifest-path desktop/src-tauri/Cargo.toml
fi

mapfile -t changed < <(under 'raycast/' '^src/.*\.(ts|tsx|js|jsx|json)$')
if [[ ${#changed[@]} -gt 0 ]]; then
    if modules_for raycast && [[ -x $tree/raycast/node_modules/.bin/prettier && -x $tree/raycast/node_modules/.bin/eslint ]]; then
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
