#!/usr/bin/env bash
# One worktree and one short-lived branch per task or agent session, so
# parallel work never shares an index, a half-edited file or a push.
#
#   tools/worktree.sh new NAME [BASE] [--deps dashboard,site,raycast]
#   tools/worktree.sh list
#   tools/worktree.sh land NAME        rebase on origin/main, check, fast-forward main
#   tools/worktree.sh pr NAME          push the branch and open a pull request instead
#   tools/worktree.sh done NAME [--force]
#   tools/worktree.sh deploy-tree      a clean tree at origin/main for fleet.sh and releases
#
# Worktrees live in ../mash-worktrees beside the main checkout (set
# MASH_WORKTREES to move them) on branches named work/NAME. Messages go to
# standard error; `new` and `deploy-tree` print only the path on standard
# output. It can be run from the main checkout or from any worktree.
set -euo pipefail

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
say() { printf '%s\n' "$*" >&2; }
die() { say "worktree: $*"; exit 1; }

main_checkout=$(git -C "$(dirname "${BASH_SOURCE[0]}")" worktree list --porcelain | sed -n '1s/^worktree //p')
[[ -n $main_checkout ]] || die 'not inside a Git repository'
trees=${MASH_WORKTREES:-$(dirname "$main_checkout")/mash-worktrees}
repo() { git -C "$main_checkout" "$@"; }

# Sets `name`, `branch` and `tree` from a task name.
select_tree() {
    name=${1:-}
    [[ $name =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || die 'a name starts with a letter or digit and holds only letters, digits, dot, dash and underscore'
    branch=work/$name
    tree=$trees/$name
}
existing_tree() {
    select_tree "$@"
    [[ -d $tree ]] || die "no worktree at $tree; see tools/worktree.sh list"
}
fetch() {
    repo fetch --quiet --prune origin || die 'git fetch origin failed'
}
# Whether a tree has uncommitted work; `all` also counts untracked files.
dirty() {
    local untracked=no
    [[ ${2:-} != all ]] || untracked=normal
    [[ -n $(git -C "$1" status --porcelain --untracked-files=$untracked | head -n 1) ]]
}

install_dependencies() {
    local package
    for package in ${1//,/ }; do
        case "$package" in
            dashboard|site)
                say "Installing $package dependencies"
                (cd "$tree/$package" && pnpm install --frozen-lockfile --prefer-offline >&2) ;;
            raycast)
                say 'Installing raycast dependencies'
                (cd "$tree/raycast" && npm ci >&2) ;;
            *) die "unknown --deps entry $package (dashboard, site, raycast)" ;;
        esac
    done
}

# Git ignores the agent skills and the Claude settings, so a new worktree has
# neither. Link them to the main checkout's copies, which stay the one place
# they are edited. `.claude` is ignored only as a directory, so the link is
# also named in the clone's own exclude file to keep the worktree clean.
link_ignored() {
    local exclude
    if [[ -d $main_checkout/.agents/skills && ! -e $tree/.agents/skills ]]; then
        mkdir -p "$tree/.agents"
        ln -s "$main_checkout/.agents/skills" "$tree/.agents/skills"
    fi
    if [[ -d $main_checkout/.claude && ! -e $tree/.claude ]]; then
        exclude=$(repo rev-parse --path-format=absolute --git-common-dir)/info/exclude
        mkdir -p "$(dirname "$exclude")"
        grep -qxF '/.claude' "$exclude" 2> /dev/null || printf '/.claude\n' >> "$exclude"
        ln -s "$main_checkout/.claude" "$tree/.claude"
    fi
}

command_new() {
    local deps='' arguments=()
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --deps) [[ $# -ge 2 ]] || usage; deps=$2; shift 2 ;;
            --deps=*) deps=${1#--deps=}; shift ;;
            -*) usage ;;
            *) arguments+=("$1"); shift ;;
        esac
    done
    [[ ${#arguments[@]} -ge 1 && ${#arguments[@]} -le 2 ]] || usage
    select_tree "${arguments[0]}"
    local base=${arguments[1]:-origin/main}
    [[ ! -e $tree ]] || die "$tree already exists"
    ! repo show-ref --verify --quiet "refs/heads/$branch" || die "branch $branch already exists; choose another name or run done $name"
    fetch
    repo rev-parse --verify --quiet "$base^{commit}" > /dev/null || die "unknown base $base"
    mkdir -p "$trees"
    # No upstream: the branch lands on main through `land` or a pull request,
    # and a bare `git push` must not guess either.
    repo worktree add --quiet --no-track -b "$branch" "$tree" "$base" >&2
    link_ignored
    [[ -z $deps ]] || install_dependencies "$deps"
    say "Created $tree on $branch from $base ($(git -C "$tree" rev-parse --short HEAD))."
    [[ -n $deps ]] || say 'Front-end checks need node_modules here: rerun with --deps dashboard,site,raycast or install in the package you touch.'
    # The Makefile pins one absolute target directory (DEV_TARGET_DIR) and
    # build-dev-linux.sh serializes on a lock inside it, so worktrees share
    # compiled dependencies without any per-worktree Cargo configuration.
    say 'Rust: make iterate, dev-build and validate-dev share the Makefile'"'"'s DEV_TARGET_DIR across worktrees (one build at a time);'
    say '      its debug/mash is the last worktree'"'"'s build, so pass DEV_TARGET_DIR=... to keep a binary of your own.'
    say "Skills (.agents/skills, .claude) are links to the main checkout. Beads lives there too: (cd $main_checkout && bd ready)"
    say "When the work is committed: tools/worktree.sh land $name   (or: pr $name), then: done $name"
    printf '%s\n' "$tree"
}

command_list() {
    [[ $# -eq 0 ]] || usage
    repo worktree list
}

command_land() {
    [[ $# -eq 1 ]] || usage
    existing_tree "$1"
    [[ $(git -C "$tree" rev-parse --abbrev-ref HEAD) == "$branch" ]] || die "$tree is not on $branch"
    ! dirty "$tree" || die "$tree has uncommitted changes; commit them first"
    fetch
    git -C "$tree" rebase --quiet origin/main >&2 ||
        die "the rebase onto origin/main stopped; resolve it in $tree (git rebase --continue), then run land again"
    local head
    head=$(git -C "$tree" rev-parse HEAD)
    if [[ $head == $(repo rev-parse origin/main) ]]; then
        say "Nothing to land: $branch has no commits beyond origin/main."
        return
    fi
    (cd "$tree" && bash tools/pre-push.sh origin/main >&2) || die 'the pre-push check failed; fix it, commit and run land again'
    # A plain push: the remote refuses anything but a fast-forward, which
    # happens only when main moved after the fetch above. The hook would
    # repeat the check that just passed for this exact commit.
    MASH_PRE_PUSH_CHECKED=$head git -C "$tree" push --quiet origin "$branch:main" >&2 ||
        die 'origin refused the push (main moved, or the push failed); run land again'
    repo fetch --quiet origin
    say "Landed $(git -C "$tree" rev-parse --short HEAD) on origin/main. Remove the worktree with: tools/worktree.sh done $name"
}

command_pr() {
    [[ $# -eq 1 ]] || usage
    existing_tree "$1"
    ! dirty "$tree" || die "$tree has uncommitted changes; commit them first"
    git -C "$tree" push --quiet --set-upstream origin "$branch" >&2 || die 'the push failed'
    (cd "$tree" && gh pr create --fill --base main --head "$branch")
}

command_done() {
    local force=''
    [[ ${2:-} != --force ]] || { force=--force; set -- "$1"; }
    [[ $# -eq 1 ]] || usage
    select_tree "$1"
    repo worktree prune
    repo show-ref --verify --quiet "refs/heads/$branch" || [[ -d $tree ]] || die "no worktree or branch for $name"
    fetch
    if [[ -z $force ]]; then
        [[ ! -d $tree ]] || ! dirty "$tree" all || die "$tree has uncommitted or untracked files; commit or remove them, or pass --force to discard them"
        ! repo show-ref --verify --quiet "refs/heads/$branch" || repo merge-base --is-ancestor "$branch" origin/main ||
            die "$branch has commits that are not on origin/main; land it or open a pull request, or pass --force to discard them"
    fi
    [[ ! -d $tree ]] || repo worktree remove ${force:+--force} "$tree"
    local merged=no
    if repo show-ref --verify --quiet "refs/heads/$branch"; then
        ! repo merge-base --is-ancestor "$branch" origin/main || merged=yes
        repo branch --quiet -D "$branch"
    fi
    # A pushed copy of a merged branch goes too; an unmerged one is left on
    # the remote even with --force, where a pull request may still need it.
    if [[ $merged == yes ]] && repo show-ref --verify --quiet "refs/remotes/origin/$branch"; then
        repo push --quiet --no-verify origin --delete "$branch" >&2 || say "Could not delete origin/$branch; remove it by hand."
    fi
    say "Removed $tree and $branch."
}

command_deploy_tree() {
    [[ $# -eq 0 ]] || usage
    tree=$trees/_deploy
    fetch
    if [[ -d $tree ]]; then
        ! dirty "$tree" all || die "$tree has local changes; a deploy tree must stay clean (inspect it, then git -C $tree clean/checkout by hand)"
        git -C "$tree" checkout --quiet --detach origin/main
    else
        mkdir -p "$trees"
        repo worktree add --quiet --detach "$tree" origin/main >&2
    fi
    say "Deploy tree at origin/main $(git -C "$tree" rev-parse --short HEAD). Run fleet commands from it: (cd $tree && bash tools/fleet.sh ...)"
    printf '%s\n' "$tree"
}

operation=${1:-}
[[ $# -eq 0 ]] || shift
case "$operation" in
    new) command_new "$@" ;;
    list) command_list "$@" ;;
    land) command_land "$@" ;;
    pr) command_pr "$@" ;;
    done) command_done "$@" ;;
    deploy-tree) command_deploy_tree "$@" ;;
    *) usage ;;
esac
