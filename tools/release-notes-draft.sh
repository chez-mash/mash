#!/usr/bin/env bash
# Draft release notes from the commits since the previous release, grouped by
# the part of Mash each one mostly changed. The draft is a starting point in
# the shape of the published notes (the standard opening paragraph, "What's
# new", bold section leads with bullets); the commit subjects still need
# rewriting in the reader's terms before anyone publishes them.
#
#   release-notes-draft.sh VERSION [PREVIOUS_TAG [HEAD]] > notes.md
set -euo pipefail
[[ $# -ge 1 && $# -le 3 ]] || { echo 'usage: release-notes-draft.sh VERSION [PREVIOUS_TAG [HEAD]]' >&2; exit 2; }
version=$1
head=${3:-HEAD}
previous=${2:-$(git describe --tags --abbrev=0 --match 'v[0-9]*' "$head^" 2>/dev/null || true)}
[[ -n $previous ]] || { echo 'No previous release tag found; name one.' >&2; exit 2; }
repo=${MASH_ARTIFACT_REPO:-mashhq/mash}

# Section order is the order of the published notes: what people see first.
sections=(dashboard desktop engine memory raycast site fixes)
title() {
    case "$1" in
        dashboard) echo 'The dashboard.' ;;
        desktop) echo 'The desktop app.' ;;
        engine) echo 'The command line, search and indexing.' ;;
        memory) echo 'Memory and the agent plugin.' ;;
        raycast) echo 'The Raycast extension.' ;;
        site) echo 'Website and guides.' ;;
        fixes) echo 'Fixes.' ;;
    esac
}
# The section a path belongs to; empty for work readers do not see (CI,
# deployment, tooling, tests).
area() {
    case "$1" in
        dashboard/*|crates/mash-dash/*) echo dashboard ;;
        desktop/*) echo desktop ;;
        raycast/*) echo raycast ;;
        site/*|docs/internal/*) [[ $1 == docs/internal/* ]] || echo site ;;
        docs/*|README.md) echo site ;;
        plugins/*) echo memory ;;
        app/*|cli/*|crates/*|install.sh|install.ps1) echo engine ;;
    esac
}
scratch=$(mktemp -d "${TMPDIR:-/tmp}/release-notes.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
while IFS=$'\t' read -r commit subject; do
    case "$subject" in "Release "[0-9]* | 'Scoop: '* | 'apt: '*) continue ;; esac
    section=$(git show --name-only --format= "$commit" | while IFS= read -r path; do area "$path"; done |
        sort | uniq -c | sort -rn | awk 'NR == 1 { print $2 }')
    if [[ -z $section ]]; then section=internal
    elif [[ $subject =~ ^(Fix|Repair|Correct|Stop|Keep)\  ]]; then section=fixes
    fi
    printf -- '- %s\n' "$subject" >> "$scratch/$section"
done < <(git log --no-merges --reverse --format='%H%x09%s' "$previous..$head")

cat <<NOTES
<!-- DRAFT from $(git rev-list --count --no-merges "$previous..$head") commits, $previous..$(git rev-parse --short "$head").
     Rewrite each section in plain language: what changed for the person using
     Mash, most noticeable first, with a bold lead sentence per theme. Drop
     what nobody would notice, name any schema upgrade or "upgrade every
     machine together" step, then delete these comments. -->
Mash searches the chats your coding agents already saved (Claude Code, Codex, GitHub Copilot, Cursor, OpenCode and Antigravity) on every machine you use, so you can find the message behind a decision and pick up work in another agent. It is open source and self-hosted: indexing and search stay on your machine and need no account. [Try the demo](https://getmash.io/demo/) · [Quick start](https://github.com/$repo#quick-start) · [Docs](https://getmash.io/guides/)

## What's new in $version

NOTES
for section in "${sections[@]}"; do
    [[ -f "$scratch/$section" ]] || continue
    printf '**%s**\n' "$(title "$section")"
    cat "$scratch/$section"
    echo
done
cat <<NOTES
## Install or upgrade

- **The app:** [download it for macOS, Windows or Linux](https://getmash.io/downloads/). The Mac app is signed and notarized.
- **Homebrew:** \`brew install mashhq/tap/mash\`
- **Everything else:** see [install methods](https://getmash.io/install-methods/) and [upgrading](https://getmash.io/upgrades/).

**Full Changelog**: https://github.com/$repo/compare/$previous...v$version
NOTES
if [[ -f "$scratch/internal" ]]; then
    printf '\n<!-- Not for the notes (CI, deployment, tooling, tests):\n'
    sed 's/--/- -/g' "$scratch/internal"
    printf -- '-->\n'
fi
