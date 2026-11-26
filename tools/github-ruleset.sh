#!/usr/bin/env bash
# The rules GitHub enforces for main: changes arrive through pull requests in
# the merge queue, after the required checks passed on the commit main will
# get. The rules are the file docs/internal/main-ruleset.json, applied as a
# repository ruleset; "The merge queue" in docs/internal/development.md
# explains them.
#
#   tools/github-ruleset.sh show      what GitHub has now, and whether it is the file
#   tools/github-ruleset.sh apply     create or update the ruleset from the file and
#                                     enforce it; delete merged branches automatically
#   tools/github-ruleset.sh disable   stop enforcing it: main takes a plain push again
#                                     (tools/worktree.sh land NAME --direct). The
#                                     ruleset stays, so `apply` turns it back on.
#
# It needs `gh` signed in as an administrator of the repository.
# MASH_ARTIFACT_REPO names another repository (OWNER/NAME), MASH_GH another gh.
set -euo pipefail

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
say() { printf '%s\n' "$*" >&2; }
die() { say "github-ruleset: $*"; exit 1; }

gh=${MASH_GH:-gh}
repo=${MASH_ARTIFACT_REPO:-mashhq/mash}
file=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/docs/internal/main-ruleset.json
[[ -f $file ]] || die "missing $file"
name=$(jq -r .name "$file")

# The number GitHub gave the ruleset of this name; empty when there is none.
ruleset_id() {
    "$gh" api "repos/$repo/rulesets" --paginate --jq ".[] | select(.name == \"$name\") | .id" | head -n 1
}
# What the file and GitHub's answer are compared by: the settings alone, in
# one order. GitHub does not echo an organization administrator's actor_id.
settings='{name, target, enforcement, conditions,
  bypass_actors: [.bypass_actors[]? | {actor_type, bypass_mode}] | sort_by(.actor_type),
  rules: [.rules[]? | {type, parameters}] | sort_by(.type)}'

command_show() {
    local id live
    id=$(ruleset_id)
    if [[ -z $id ]]; then
        say "$repo has no ruleset named \"$name\": main is unprotected. Create it with: tools/github-ruleset.sh apply"
        return 1
    fi
    live=$("$gh" api "repos/$repo/rulesets/$id")
    jq "$settings" <<< "$live"
    say "Ruleset $id is $(jq -r .enforcement <<< "$live"): https://github.com/$repo/rules/$id"
    say "Merged branches are deleted automatically: $("$gh" api "repos/$repo" --jq .delete_branch_on_merge)"
    # GitHub answers with every parameter of a rule, its own defaults
    # included; only the ones the file sets are compared.
    local wanted
    wanted=$(jq -S "$settings" "$file")
    live=$(jq -S --argjson wanted "$wanted" "$settings"' | .rules |= map(. as $rule
        | ($wanted.rules[] | select(.type == $rule.type) | .parameters) as $set
        | if $set then .parameters |= with_entries(select(.key as $key | $set | has($key))) else . end)' <<< "$live")
    if [[ $wanted == "$live" ]]; then
        say 'It matches docs/internal/main-ruleset.json.'
    else
        say 'It DIFFERS from docs/internal/main-ruleset.json (the enforcement, when it is disabled, or more):'
        diff <(printf '%s\n' "$wanted") <(printf '%s\n' "$live") >&2 || true
    fi
}

# write ENFORCEMENT: send the file to GitHub with that enforcement.
write() {
    local id
    id=$(ruleset_id)
    if [[ -n $id ]]; then
        jq --arg enforcement "$1" '.enforcement = $enforcement' "$file" |
            "$gh" api --method PUT "repos/$repo/rulesets/$id" --input - > /dev/null || die 'GitHub refused the ruleset'
    else
        jq --arg enforcement "$1" '.enforcement = $enforcement' "$file" |
            "$gh" api --method POST "repos/$repo/rulesets" --input - > /dev/null || die 'GitHub refused the ruleset'
    fi
}

command_apply() {
    write active
    # A queued pull request's branch is done once it merges.
    "$gh" api --method PATCH "repos/$repo" -F delete_branch_on_merge=true > /dev/null ||
        die 'the ruleset is applied, but GitHub refused the repository setting that deletes merged branches'
    say "main of $repo takes changes only through the merge queue."
    command_show > /dev/null
}

command_disable() {
    [[ -n $(ruleset_id) ]] || die "$repo has no ruleset named \"$name\"; there is nothing to disable"
    write disabled
    say "The rules for main of $repo are no longer enforced: a plain push to main works (tools/worktree.sh land NAME --direct)."
    say 'Pull requests already in the queue stay open; land them again after: tools/github-ruleset.sh apply'
}

[[ $# -eq 1 ]] || usage
case "$1" in
    show) command_show ;;
    apply) command_apply ;;
    disable) command_disable ;;
    *) usage ;;
esac
