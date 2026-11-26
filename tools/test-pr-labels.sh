#!/usr/bin/env bash
# Checks .github/scripts/pr-labels against made-up pull requests, offline.
set -euo pipefail
cd "$(dirname "$0")/.."

body() {
	printf '## Summary\n\nx\n\n## Intent\n\n- [%s] **Fix**: a\n- [%s] **Documentation**: b\n\n## AI assistance\n\n<!-- hint -->\n%s\n\n## Verification\n' "$1" "$2" "$3"
}
check() {
	local name=$1 expected=$2 files=$3 text=$4 existing=${5-}
	local actual
	actual=$(PR_LABELS_FILES=$files PR_LABELS_BODY=$text PR_LABELS_EXISTING=$existing \
		.github/scripts/pr-labels 1 | sort | tr '\n' ' ')
	if [[ $actual != "$expected" ]]; then
		printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$name" "$expected" "$actual" >&2
		exit 1
	fi
}

check 'ticked fix in the dashboard' \
	'label: area:dashboard label: category:fix label: touches:contract ' \
	$'dashboard/openapi.json\ncrates/mash-dash/src/lib.rs' "$(body x ' ' None)"
check 'untouched template, docs only' \
	'label: category:docs label: needs:ai-disclosure label: needs:intent ' \
	$'docs/search.md\nREADME.md' "$(body ' ' ' ' '')"
check 'no description, tooling' \
	'label: area:install label: category:tooling label: needs:ai-disclosure label: needs:intent ' \
	$'.github/workflows/release.yml\njustfile\nMakefile' ''
check 'two boxes ticked sets no category' \
	'label: area:storage label: touches:schema ' \
	'crates/mash-storage/src/storage/silver.sql' "$(body x x 'Claude Code wrote the tests')"
check 'a maintainer category wins and stale needs are removed' \
	'label: area:cli unlabel: needs:ai-disclosure unlabel: needs:intent ' \
	'cli/src/main.rs' "$(body x ' ' None)" $'category:feature\nneeds:intent\nneeds:ai-disclosure'

# Every label the script can apply is defined.
for label in $(grep -oE '(area|touches|category|needs):[a-z-]+' .github/scripts/pr-labels | sort -u); do
	cut -f1 .github/labels.tsv | grep -qxF "$label" || { echo "FAIL undefined label $label" >&2; exit 1; }
done
echo 'pr-labels: ok'
