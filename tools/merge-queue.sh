#!/usr/bin/env bash
# Puts a pushed branch into GitHub's merge queue for main and follows it
# there. `tools/worktree.sh land` and `tools/release.sh` both land through
# this, so the queue has one way in.
#
#   tools/merge-queue.sh submit BRANCH [--wait]
#       open a pull request for BRANCH (or reuse its open one), put it in the
#       queue once its own checks have passed (about a minute for one opened
#       here), and print its address on standard output
#   tools/merge-queue.sh wait PR
#       follow pull request number PR until it is merged, or say why it was not
#
# The queue rebases the branch's commits onto main, runs the required checks
# on the result, and moves main to that exact commit when they pass; main is
# untouched when they fail. `wait` exits 0 once the pull request is merged and
# 1 when it left the queue, was closed, or its own checks failed, after
# printing the failing checks with their addresses.
#
# Run it inside the repository. MASH_GH names another `gh`, MASH_ARTIFACT_REPO
# another repository (OWNER/NAME), MASH_QUEUE_POLL the seconds between looks
# (20), MASH_QUEUE_ENTER how long to wait for the pull request's own checks
# before giving up on queueing it (1200) and MASH_QUEUE_WAIT how long to
# follow it in the queue (5400).
set -euo pipefail

# A pull request this script opens exists only to be queued. The queue runs
# the required checks on the commit main will get, so the pull request's own
# run would repeat them for nothing: `.github/actions/changes` answers
# "nothing" when it finds this marker in the description.
marker='[merge queue]'

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
say() { printf '%s\n' "$*" >&2; }
die() { say "merge-queue: $*"; exit 1; }

gh=${MASH_GH:-gh}
poll=${MASH_QUEUE_POLL:-20}
patience=${MASH_QUEUE_WAIT:-5400}
entering=${MASH_QUEUE_ENTER:-1200}
repository=${MASH_ARTIFACT_REPO:-}
repository() {
    [[ -n $repository ]] || repository=$("$gh" repo view --json nameWithOwner --jq .nameWithOwner) ||
        die 'gh could not name this repository; is gh installed and signed in?'
    printf '%s\n' "$repository"
}

# One line about a pull request, tab-separated, "-" for a field that is empty:
# state, merged commit, queue state, queue position, the queue's commit for it,
# whether auto-merge is set on it, the required checks
# of its own that failed ("name  address; ..."), when it last entered and last
# left the queue, the commit it had in the queue when it left, and why it left.
query='
query($owner: String!, $name: String!, $number: Int!) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      state
      mergeCommit { oid }
      mergeQueueEntry { state position headCommit { oid } }
      autoMergeRequest { enabledAt }
      commits(last: 1) { nodes { commit { statusCheckRollup { contexts(first: 100) { nodes {
        ... on CheckRun { name conclusion detailsUrl isRequired(pullRequestNumber: $number) }
        ... on StatusContext { context state targetUrl isRequired(pullRequestNumber: $number) }
      } } } } } }
      added: timelineItems(last: 1, itemTypes: [ADDED_TO_MERGE_QUEUE_EVENT]) {
        nodes { ... on AddedToMergeQueueEvent { createdAt } }
      }
      removed: timelineItems(last: 1, itemTypes: [REMOVED_FROM_MERGE_QUEUE_EVENT]) {
        nodes { ... on RemovedFromMergeQueueEvent { createdAt reason beforeCommit { oid } } }
      }
    }
  }
}'
# shellcheck disable=SC2016 # jq's own variables
fields='.data.repository.pullRequest | [
  .state, .mergeCommit.oid, .mergeQueueEntry.state, .mergeQueueEntry.position, .mergeQueueEntry.headCommit.oid,
  (if .autoMergeRequest then "yes" else null end),
  ([.commits.nodes[0].commit.statusCheckRollup.contexts.nodes[]? | select(.isRequired)
    | select((.conclusion // .state) as $result | ["FAILURE", "CANCELLED", "TIMED_OUT", "ACTION_REQUIRED", "ERROR"] | index($result))
    | "\(.name // .context)  \(.detailsUrl // .targetUrl)"] | join("; ") | if . == "" then null else . end),
  .added.nodes[0].createdAt, .removed.nodes[0].createdAt, .removed.nodes[0].beforeCommit.oid, .removed.nodes[0].reason
] | map(. // "-" | tostring | gsub("[\t\n]"; " ")) | join("\t")'
pull_request_state() {
    local name
    name=$(repository)
    "$gh" api graphql -F "owner=${name%/*}" -F "name=${name#*/}" -F "number=$1" -f "query=$query" --jq "$fields"
}

# The checks that did not pass for a commit, one "name  address" per line.
failed_checks() {
    "$gh" api "repos/$(repository)/commits/$1/check-runs?per_page=100" --jq '
        .check_runs[] | select(.conclusion as $c | ["failure", "cancelled", "timed_out", "action_required"] | index($c))
        | "  \(.name)  \(.html_url)"' 2> /dev/null || true
}
report_failed_checks() {
    local commit=$1 checks
    [[ $commit != - ]] || return 0
    checks=$(failed_checks "$commit")
    if [[ -n $checks ]]; then
        say "Checks that did not pass for ${commit:0:12}:"
        say "$checks"
    else
        say "No failed check is recorded for ${commit:0:12}; see the pull request's page."
    fi
}

# wait PR [SINCE]: SINCE is the time of a removal from the queue that is
# already known, when a pull request is queued again after a repair.
command_wait() {
    [[ $# -ge 1 && $# -le 2 && $1 =~ ^[0-9]+$ ]] || usage
    local number=$1 since=${2:-} started=$SECONDS deadline=$((SECONDS + patience)) last='' unqueued=0 unread=0 queued_commit=-
    local line state merged queue position queue_commit _ failed added removed removed_commit reason note
    while :; do
        (( SECONDS < deadline )) || die "pull request #$number has not merged after ${patience}s; follow it with: tools/merge-queue.sh wait $number"
        # One unanswered question to GitHub must not end a long wait.
        if ! line=$(pull_request_state "$number"); then
            unread=$((unread + 1))
            [[ $unread -le 5 ]] || die "could not read pull request #$number; follow it with: tools/merge-queue.sh wait $number"
            sleep "$poll"
            continue
        fi
        unread=0
        IFS=$'\t' read -r state merged queue position queue_commit _ failed added removed removed_commit reason <<< "$line"
        [[ $queue_commit == - ]] || queued_commit=$queue_commit
        case "$state" in
            MERGED)
                say "Pull request #$number is merged: main has it as ${merged:0:12} ($((SECONDS - started))s after this wait began)."
                return 0 ;;
            CLOSED) die "pull request #$number was closed without being merged." ;;
        esac
        if [[ $queue != - ]]; then
            note="in the merge queue at position $position ($queue)"
            unqueued=0
        # ISO 8601 times in one zone compare as text.
        elif [[ $removed != - && $removed > $since && ( $added == - || ! $removed < $added ) ]]; then
            say "Pull request #$number was removed from the merge queue: ${reason//_/ }. main is unchanged."
            [[ $queued_commit != - ]] || queued_commit=$removed_commit
            report_failed_checks "$queued_commit"
            say 'Fix it on the branch, then land again.'
            return 1
        elif [[ $failed != - ]]; then
            say "Pull request #$number is not queued: a required check failed on the pull request itself, so GitHub will not queue it."
            say "  ${failed//; /$'\n'  }"
            say 'Fix it on the branch, then land again.'
            return 1
        else
            # Not queued. Allow GitHub a few looks to show an entry it has
            # only just accepted.
            unqueued=$((unqueued + 1))
            [[ $unqueued -le 3 ]] || die "pull request #$number is not in the merge queue; land it again."
            note='waiting for GitHub to show it in the queue'
        fi
        [[ $note == "$last" ]] || say "Pull request #$number: $note..."
        last=$note
        sleep "$poll"
    done
}

# Put a pull request in the queue. GitHub accepts it only once the required
# checks have passed on the pull request itself, so this asks until it does.
# (Auto-merge is not used: `gh pr merge --auto` set it, and GitHub then left
# pull requests with passing checks out of the queue.)
enter() {
    local number=$1 url=$2 name id deadline=$((SECONDS + entering)) answer told=no line state queue failed _
    name=$(repository)
    id=$("$gh" pr view "$number" --repo "$name" --json id --jq .id) || die "could not read pull request #$number"
    # shellcheck disable=SC2016 # GraphQL's own variable
    while ! answer=$("$gh" api graphql -f "id=$id" --jq .data.enqueuePullRequest.mergeQueueEntry.state \
        -f 'query=mutation($id: ID!) { enqueuePullRequest(input: {pullRequestId: $id}) { mergeQueueEntry { state } } }' 2>&1); do
        if line=$(pull_request_state "$number"); then
            IFS=$'\t' read -r state _ queue _ _ _ failed _ <<< "$line"
            [[ $state != MERGED && $queue == - ]] || return 0
            if [[ $failed != - ]]; then
                say "Pull request #$number is not queued: a required check failed on the pull request itself ($url)."
                say "  ${failed//; /$'\n'  }"
                exit 1
            fi
        fi
        (( SECONDS < deadline )) ||
            die "GitHub did not accept pull request #$number for the merge queue ($url): $(printf '%s' "$answer" | tail -n 1). When the queue is switched off (tools/github-ruleset.sh show), land with: tools/worktree.sh land NAME --direct"
        [[ $told == yes ]] || say "Pull request #$number: waiting for its own checks before GitHub will queue it..."
        told=yes
        sleep "$poll"
    done
}

command_submit() {
    local wait=no branch=''
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --wait) wait=yes ;;
            -*) usage ;;
            *) [[ -z $branch ]] || usage; branch=$1 ;;
        esac
        shift
    done
    [[ -n $branch ]] || usage
    local name found number url commits title body since=''
    name=$(repository)
    found=$("$gh" pr list --repo "$name" --head "$branch" --base main --state open --limit 1 \
        --json number,url --jq '.[0] | select(.) | "\(.number) \(.url)"') || die 'gh could not list pull requests'
    if [[ -n $found ]]; then
        number=${found%% *}
        url=${found#* }
        say "Reusing pull request #$number for $branch."
        # A removal from the queue before this request is not news.
        since=$(pull_request_state "$number" | cut -f 9) || since=''
        [[ $since != - ]] || since=''
    else
        git rev-parse --verify --quiet "origin/$branch^{commit}" > /dev/null || die "origin has no branch $branch; push it first"
        commits=$(git log --reverse --format=%s "origin/main..origin/$branch")
        [[ -n $commits ]] || die "$branch has no commits beyond origin/main"
        title=$(printf '%s\n' "$commits" | head -n 1)
        body=$(git log -1 --format=%b "origin/$branch")
        [[ $(printf '%s\n' "$commits" | wc -l) -eq 1 ]] || body=$(printf '%s\n' "$commits" | sed 's/^/- /')
        body+=$'\n\n'"$marker Opened by tools/merge-queue.sh to land through the merge queue, which runs the required checks on the commit main will get. This pull request's own run skips them."
        url=$("$gh" pr create --repo "$name" --base main --head "$branch" --title "$title" --body "$body") || die 'gh could not open the pull request'
        url=$(printf '%s\n' "$url" | grep -Eo 'https://[^ ]+/pull/[0-9]+' | tail -n 1)
        number=${url##*/}
        [[ $number =~ ^[0-9]+$ ]] || die 'gh did not print the address of the new pull request'
        say "Opened pull request #$number for $branch."
    fi
    enter "$number" "$url"
    say "Queued: $url"
    printf '%s\n' "$url"
    [[ $wait == no ]] || command_wait "$number" "$since"
}

operation=${1:-}
[[ $# -eq 0 ]] || shift
case "$operation" in
    submit) command_submit "$@" ;;
    wait) command_wait "$@" ;;
    *) usage ;;
esac
