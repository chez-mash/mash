#!/usr/bin/env bash
# Checks the justfile without running any real work: every recipe parses, is
# documented and grouped, and evaluates in a dry run; and the guards refuse
# what they should.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repository"
fail() { echo "test-justfile: $*" >&2; exit 1; }
command -v just > /dev/null || fail 'just is not installed; run: bash tools/install-just.sh'
scratch=$(mktemp -d "${TMPDIR:-/tmp}/justfile-test.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
# Settings from the caller's environment would change what the recipes print.
unset DEV_TMPDIR DEV_TARGET_DIR DEV_RUN CARGO_ARGS CODEGEN COVERAGE_OUTPUT FOCUS PACKAGE TARGETS OWNER MAKEFLAGS MAKELEVEL
unset MASH_TEST_MODELS_DIR MASH_TEST_QDRANT_URL

# The front door lists every group, and bare `just` is that list.
just --list > "$scratch/list"
for group in develop test quality deploy release design; do
    grep -qx "    \[$group\]" "$scratch/list" || fail "just --list has no [$group] group"
done
just > "$scratch/default"
grep -q '^    iterate ' "$scratch/default" || fail 'bare `just` does not list the recipes'

# Every public recipe has a doc comment and a group, and a dry run of it
# evaluates: an undefined variable or a bad expression fails here.
just --dump --dump-format json > "$scratch/dump.json"
python3 - "$scratch/dump.json" > "$scratch/invocations" << 'PYTHON'
import json
import sys

recipes = json.load(open(sys.argv[1]))["recipes"]
problems = []
for name, recipe in sorted(recipes.items()):
    if recipe["private"]:
        continue
    if not recipe["doc"]:
        problems.append(f"{name} has no doc comment")
    if not any(isinstance(attribute, dict) and "group" in attribute for attribute in recipe["attributes"]):
        problems.append(f"{name} has no group")
    # One placeholder for each parameter that has no default.
    required = [
        parameter
        for parameter in recipe["parameters"]
        if parameter["default"] is None and parameter["kind"] != "star" and not parameter.get("long")
    ]
    print(" ".join([name, *("placeholder" for _ in required)]))
if problems:
    sys.exit("\n".join(problems))
PYTHON
count=0
while read -r -a invocation; do
    just --dry-run "${invocation[@]}" > /dev/null 2> "$scratch/dry-run" ||
        fail "just --dry-run ${invocation[*]} failed: $(cat "$scratch/dry-run")"
    count=$((count + 1))
done < "$scratch/invocations"
[[ $count -ge 60 ]] || fail "expected at least 60 recipes, found $count"

# Arguments and settings reach the commands.
dry() { just --dry-run "$@" 2>&1; }
[[ $(dry DEV_RUN= DEV_TARGET_DIR=/target iterate 'test(a)' --package mash-dash --targets all | tail -n 1) == \
    *'CARGO_TARGET_DIR="/target" CARGO_INCREMENTAL=1 RUSTC_WRAPPER= MASH_ITERATE_LAUNCHER="" MASH_ITERATE_PACKAGE="mash-dash" bash tools/build-dev-linux.sh --focused "${focus}" --all-targets' ]] ||
    fail 'iterate does not pass its package, targets and settings'
[[ $(DEV_RUN=launcher dry protocol-check) == 'launcher bash protocol/check.sh' ]] || fail 'DEV_RUN from the environment is not used'
[[ $(DEV_RUN='' dry protocol-check) == ' bash protocol/check.sh' ]] || fail 'an empty DEV_RUN does not turn the launcher off'
[[ $(dry build --release --locked | tail -n 1) == 'cargo build-clif --release --locked' ]] || fail 'build does not pass Cargo arguments'
[[ $(dry release --dry-run 1.2.3) == *'bash tools/release.sh --dry-run  "1.2.3"' ]] || fail 'release --dry-run is not passed on'
[[ $(dry release --resume 1.2.3) == *'bash tools/release.sh  --resume "1.2.3"' ]] || fail 'release --resume is not passed on'

# Guards run before any work: no filter, and coverage without its services.
if just DEV_RUN= iterate > "$scratch/output" 2>&1; then fail 'iterate ran without a filter'; fi
grep -q 'Name a nextest filter' "$scratch/output" || fail "iterate without a filter does not say what to do: $(cat "$scratch/output")"
if just coverage > "$scratch/output" 2>&1; then fail 'coverage ran without its test services'; fi
grep -q 'Set MASH_TEST_MODELS_DIR' "$scratch/output" || fail 'coverage does not name the missing setting'

# The installer pins a version the justfile accepts.
pinned=$(bash tools/install-just.sh --version)
minimum=$(sed -n "s/^set minimum-version := '\(.*\)'$/\1/p" justfile)
[[ -n $minimum && $(printf '%s\n%s\n' "$minimum" "$pinned" | sort -V | head -n 1) == "$minimum" ]] ||
    fail "tools/install-just.sh pins $pinned, older than the justfile's minimum $minimum"
echo "The justfile's $count recipes passed."
