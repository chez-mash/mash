#!/usr/bin/env bash
# A stand-in for `gh` in the tests of the merge queue flow (test-worktree.sh,
# test-release.sh). It answers the calls tools/merge-queue.sh and
# tools/worktree.sh make, and plays GitHub's merge queue against a bare
# repository: a queued pull request is rebased onto main there, under new
# commit names as GitHub's rebase gives them, or removed from the queue.
#
#   FAKE_QUEUE          a directory for its state; every call is logged in calls
#   FAKE_QUEUE_REMOTE   the bare repository that stands for GitHub
#
# Files in FAKE_QUEUE steer it:
#   verdict       what the queue does with a queued pull request: `merge` (the
#                 default), `reject` (remove it, with a failed check), or
#                 `own-check` (a required check fails on the pull request
#                 itself, so it is never queued)
#   no-queue      when present, `pr merge` fails as it does without a queue
#   keep-branch   when present, a merged branch is left on the remote
set -euo pipefail
state=${FAKE_QUEUE:?} remote=${FAKE_QUEUE_REMOTE:?}
mkdir -p "$state/prs"
printf '%s\n' "$*" >> "$state/calls"
area=${1:-} action=${2:-}
shift 2 || true
address=https://github.example/fixture/mash

head='' wanted=open title='' body='' number='' subject=''
while [[ $# -gt 0 ]]; do
    case "$1" in
        --head) head=$2; shift 2 ;;
        --state) wanted=$2; shift 2 ;;
        --title) title=$2; shift 2 ;;
        --body) body=$2; shift 2 ;;
        -F) [[ $2 != number=* ]] || number=${2#number=}; shift 2 ;;
        --repo | --base | --json | --jq | --limit | -f) shift 2 ;;
        -*) shift ;;
        *) subject=$1; shift ;;
    esac
done
field() { cat "$state/prs/$1/$2" 2> /dev/null || true; }
row() { local IFS=$'\t'; printf '%s\n' "$*"; }

# GitHub's rebase merge: the branch's commits again on top of main, with
# GitHub as their committer, so none keeps its name.
merge() {
    local pr=$1 branch work
    branch=$(field "$pr" branch)
    work=$(mktemp -d "$state/merge.XXXXXX")
    git clone --quiet "$remote" "$work/clone" 2> /dev/null
    (
        cd "$work/clone"
        export GIT_COMMITTER_NAME=GitHub GIT_COMMITTER_EMAIL=noreply@github.example GIT_COMMITTER_DATE='2030-01-01T00:00:00Z'
        git checkout --quiet main
        # shellcheck disable=SC2046 # one argument per commit
        git cherry-pick --allow-empty $(git rev-list --reverse "main..origin/$branch") > /dev/null
        git push --quiet origin main
        [[ -e $state/keep-branch ]] || git push --quiet origin --delete "$branch"
        git rev-parse HEAD > "$state/prs/$pr/merged"
    ) > /dev/null 2>&1
    rm -rf "$work"
    echo merged > "$state/prs/$pr/status"
}

case "$area $action" in
    'repo view') echo fixture/mash ;;
    'pr list')
        for pr in "$state"/prs/*/; do
            [[ -d $pr ]] || continue
            pr=$(basename "$pr")
            [[ $(field "$pr" branch) == "$head" && $(field "$pr" status) == "$wanted" ]] || continue
            if [[ $wanted == merged ]]; then field "$pr" head; else echo "$pr $address/pull/$pr"; fi
        done ;;
    'pr create')
        pr=$(($(find "$state/prs" -mindepth 1 -maxdepth 1 -type d | wc -l) + 1))
        mkdir "$state/prs/$pr"
        echo "$head" > "$state/prs/$pr/branch"
        echo open > "$state/prs/$pr/status"
        printf '%s\n' "$title" > "$state/prs/$pr/title"
        printf '%s\n' "$body" > "$state/prs/$pr/body"
        echo "$address/pull/$pr" ;;
    'pr merge')
        [[ ! -e $state/no-queue ]] || { echo 'Pull request is in clean status' >&2; exit 1; }
        git -C "$remote" rev-parse "refs/heads/$(field "$subject" branch)" > "$state/prs/$subject/head"
        echo 0 > "$state/prs/$subject/looks"
        touch "$state/prs/$subject/queued" ;;
    'api graphql')
        removed=$(field "$number" removed)
        if [[ $(field "$number" status) == merged ]]; then
            row MERGED "$(field "$number" merged)" - - - - - - - - -
        elif [[ -e $state/prs/$number/queued ]]; then
            looks=$(($(field "$number" looks) + 1))
            echo "$looks" > "$state/prs/$number/looks"
            verdict=$(cat "$state/verdict" 2> /dev/null || echo merge)
            if [[ $verdict == own-check ]]; then
                rm "$state/prs/$number/queued"
                row OPEN - - - - yes "ci / required  $address/runs/7" - "${removed:--}" - -
            elif [[ $looks -lt 2 ]]; then
                row OPEN - AWAITING_CHECKS 1 "$(field "$number" head)" - - 2030-01-01T00:00:00Z "${removed:--}" - -
            elif [[ $verdict == reject ]]; then
                rm "$state/prs/$number/queued"
                echo 2030-01-01T00:10:00Z > "$state/prs/$number/removed"
                row OPEN - - - - - - 2030-01-01T00:00:00Z 2030-01-01T00:10:00Z "$(field "$number" head)" FAILED_CHECKS
            else
                merge "$number"
                row MERGED "$(field "$number" merged)" - - - - - - - - -
            fi
        else
            row OPEN - - - - - - - "${removed:--}" - -
        fi ;;
    'api repos/'*) echo "  ci / required  $address/runs/9" ;;
    *) echo "unexpected gh $area $action" >&2; exit 2 ;;
esac
