#!/usr/bin/env bash
# One worktree and one short-lived branch per task or agent session, so
# parallel work never shares an index, a half-edited file or a push.
#
#   tools/worktree.sh new NAME [BASE] [--deps dashboard,site,raycast]
#   tools/worktree.sh list
#   tools/worktree.sh land NAME [--wait]
#                                      rebase on origin/main, check, push the branch, open its
#                                      pull request and put it in the merge queue for main;
#                                      --wait follows it until it is merged or rejected
#   tools/worktree.sh land NAME --direct
#                                      emergencies only, administrators only: push straight
#                                      to main, past the queue and its checks
#   tools/worktree.sh pr NAME          push the branch and open a pull request without queueing it
#   tools/worktree.sh done NAME [--force]
#   tools/worktree.sh deploy-tree      a clean tree at origin/main for fleet.sh and releases
#   tools/worktree.sh preview NAME [--site|--storybook] [--real] [--local] [--stop]
#                                      a live dev server on the tailnet (worktree-preview.sh)
#   tools/worktree.sh previews         the previews that are recorded, and whether each answers
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
    # No upstream: the branch lands on main through `land`, and a bare
    # `git push` must not guess where it goes.
    repo worktree add --quiet --no-track -b "$branch" "$tree" "$base" >&2
    link_ignored
    [[ -z $deps ]] || install_dependencies "$deps"
    say "Created $tree on $branch from $base ($(git -C "$tree" rev-parse --short HEAD))."
    [[ -n $deps ]] || say 'Front-end checks need node_modules here: rerun with --deps dashboard,site,raycast or install in the package you touch.'
    # The justfile pins one absolute target directory (DEV_TARGET_DIR) and
    # build-dev-linux.sh serializes on a lock inside it, so worktrees share
    # compiled dependencies without any per-worktree Cargo configuration.
    say 'Rust: just iterate, dev-build and validate-dev share the justfile'"'"'s DEV_TARGET_DIR across worktrees (one build at a time);'
    say '      its debug/mash is the last worktree'"'"'s build, so pass DEV_TARGET_DIR=... to keep a binary of your own.'
    say "Skills (.agents/skills, .claude) are links to the main checkout. Beads lives there too: (cd $main_checkout && bd ready)"
    say "When the work is committed: tools/worktree.sh land $name --wait   (the merge queue), then: done $name"
    printf '%s\n' "$tree"
}

command_list() {
    [[ $# -eq 0 ]] || usage
    repo worktree list
}

# main takes changes only through GitHub's merge queue, which rebases the
# branch's commits onto main, runs the required checks on the result and moves
# main to that commit when they pass (tools/merge-queue.sh). The rebase and
# the quick checks here fail in seconds what the queue would fail in minutes.
command_land() {
    local wait='' direct=no arguments=()
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --wait) wait=--wait ;;
            --direct) direct=yes ;;
            -*) usage ;;
            *) arguments+=("$1") ;;
        esac
        shift
    done
    [[ ${#arguments[@]} -eq 1 ]] || usage
    [[ $direct == no || -z $wait ]] || die '--direct pushes at once; there is nothing for --wait to follow'
    existing_tree "${arguments[0]}"
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
    # The hook would repeat the check that just passed for this exact commit.
    if [[ $direct == yes ]]; then
        # Past the queue: only an administrator's push is accepted, and
        # nothing has run the test suites on this commit. A plain push, so the
        # remote still refuses anything but a fast-forward.
        say 'Pushing straight to main, past the merge queue and its checks (emergencies only).'
        MASH_PRE_PUSH_CHECKED=$head git -C "$tree" push --quiet origin "$branch:main" >&2 ||
            die 'origin refused the push (main moved, the push failed, or you may not bypass the rules for main); run land again'
        repo fetch --quiet origin
        say "Landed $(git -C "$tree" rev-parse --short HEAD) on origin/main. Remove the worktree with: tools/worktree.sh done $name"
        return
    fi
    # The branch on origin is this worktree's alone and was just rebased, so
    # it is replaced, but only if it is still what this clone last saw there.
    MASH_PRE_PUSH_CHECKED=$head git -C "$tree" push --quiet --force-with-lease origin "$branch:$branch" >&2 ||
        die "origin refused $branch (someone else pushed to it, or the push failed); fetch and look, then run land again"
    repo fetch --quiet origin
    (cd "$tree" && queue_tool submit "$branch" $wait) || die "$branch is not on main; when the cause is fixed and committed, run land again"
    if [[ -n $wait ]]; then
        repo fetch --quiet --prune origin
        say "Landed $branch on origin/main. Remove the worktree with: tools/worktree.sh done $name"
    else
        say "$branch is in the merge queue; nothing is on main until the queue's checks pass."
        say "Follow it with the pull request's number: tools/merge-queue.sh wait NUMBER   then: tools/worktree.sh done $name"
    fi
}
queue_tool() { bash "$(dirname "${BASH_SOURCE[0]}")/merge-queue.sh" "$@"; }

command_pr() {
    [[ $# -eq 1 ]] || usage
    existing_tree "$1"
    ! dirty "$tree" || die "$tree has uncommitted changes; commit them first"
    git -C "$tree" push --quiet --set-upstream origin "$branch" >&2 || die 'the push failed'
    (cd "$tree" && gh pr create --fill --base main --head "$branch")
}

# Whether origin/main holds the branch's work. The merge queue rebases, so the
# commits on main are copies with other names: every commit of the branch must
# have a commit with the same change on main. Failing that, GitHub is asked
# whether a pull request merged this exact branch head, which also covers a
# squash merge.
landed() {
    repo merge-base --is-ancestor "$branch" origin/main && return 0
    ! repo cherry origin/main "$branch" | grep -q '^+' && return 0
    local head
    head=$(repo rev-parse "$branch")
    (cd "$main_checkout" && "${MASH_GH:-gh}" pr list --head "$branch" --base main --state merged \
        --json headRefOid --jq '.[].headRefOid' 2> /dev/null) | grep -qx "$head"
}

command_done() {
    local force=''
    [[ ${2:-} != --force ]] || { force=--force; set -- "$1"; }
    [[ $# -eq 1 ]] || usage
    select_tree "$1"
    repo worktree prune
    repo show-ref --verify --quiet "refs/heads/$branch" || [[ -d $tree ]] || die "no worktree or branch for $name"
    fetch
    local merged=no
    ! repo show-ref --verify --quiet "refs/heads/$branch" || ! landed || merged=yes
    if [[ -z $force ]]; then
        [[ ! -d $tree ]] || ! dirty "$tree" all || die "$tree has uncommitted or untracked files; commit or remove them, or pass --force to discard them"
        ! repo show-ref --verify --quiet "refs/heads/$branch" || [[ $merged == yes ]] ||
            die "$branch has commits that are not on origin/main; land it and wait for the merge queue, or pass --force to discard them"
    fi
    # A dev server left running would keep serving a directory that is gone.
    preview_tool stop-all "$name"
    [[ ! -d $tree ]] || repo worktree remove ${force:+--force} "$tree"
    ! repo show-ref --verify --quiet "refs/heads/$branch" || repo branch --quiet -D "$branch"
    # GitHub deletes a merged pull request's branch itself; this removes a
    # pushed copy it left. An unmerged one stays on the remote even with
    # --force, where a pull request may still need it.
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

# Live previews are their own file; this one only finds the worktree.
preview_tool() { bash "$(dirname "${BASH_SOURCE[0]}")/worktree-preview.sh" "$@"; }
command_preview() {
    [[ $# -ge 1 ]] || usage
    select_tree "$1"
    shift
    preview_tool preview "$name" "$tree" "$@"
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
    preview) command_preview "$@" ;;
    previews) [[ $# -eq 0 ]] || usage; preview_tool list ;;
    *) usage ;;
esac
