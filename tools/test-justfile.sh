#!/usr/bin/env bash
# Checks the justfile and the Makefile shim without running any real work:
# every recipe parses, is documented and grouped, and evaluates in a dry run;
# the guards refuse what they should; and every target the Makefile used to
# have still exists there and forwards to the recipe of the same name with
# its variables.
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

# The Makefile is a shim: every target it had before the move still exists,
# the justfile has a recipe of that name, and make runs exactly that recipe.
old_targets=(mutation-tools mutation-check mutation-services mutation-certification mutation-runtime
    protocol-tools protocol-check release-tools deny test-cleanup iterate dev-build validate-dev build
    check-skills generate-skills generate-docs generate-queries check-generated coverage site-review)
for target in "${old_targets[@]}"; do
    grep -q "^$target\( \|\$\)" "$scratch/invocations" || fail "the justfile has no recipe for the old Makefile target $target"
    [[ $(make --no-print-directory JUST=echo "$target" 2> "$scratch/output") == "$target" ]] ||
        fail "make $target does not forward to just $target: $(cat "$scratch/output")"
done
# One rule for all of them: a target with commands of its own would be a second source of truth.
[[ $(grep -cE '^[^#[:space:]][^=]*:([^=]|$)' Makefile) == 2 ]] ||
    fail 'the Makefile holds a rule besides the forwarding one; recipes belong in the justfile'
# Variables given to make reach the recipe.
[[ $(make --no-print-directory JUST='just --dry-run' iterate FOCUS='test(a)' PACKAGE=mash-silver TARGETS=all DEV_RUN= DEV_TARGET_DIR=/target 2>&1 | tail -n 1) == \
    *'CARGO_TARGET_DIR="/target" '*'MASH_ITERATE_LAUNCHER="" MASH_ITERATE_PACKAGE="mash-silver" '*'--all-targets' ]] ||
    fail 'make iterate does not pass its variables to just'
[[ $(make --no-print-directory JUST='just --dry-run' build CODEGEN=1 CARGO_ARGS='--release --locked' 2>&1 | tail -n 2) == \
    *'"1" == 1'*'generate-queries; fi'$'\n''cargo build-clif --release --locked' ]] || fail 'make build does not pass CODEGEN and CARGO_ARGS'
[[ $(make --no-print-directory JUST='just --dry-run' test-cleanup OWNER=run-7 2>&1) == 'bash tools/clean-test-containers.sh run-7' ]] ||
    fail 'make test-cleanup does not pass OWNER'
if make --no-print-directory JUST=no-such-just deny > "$scratch/output" 2>&1; then fail 'make ran without just'; fi
grep -q 'tools/install-just.sh' "$scratch/output" || fail 'make without just does not say how to install it'

# The installer pins a version the justfile accepts.
pinned=$(bash tools/install-just.sh --version)
minimum=$(sed -n "s/^set minimum-version := '\(.*\)'$/\1/p" justfile)
[[ -n $minimum && $(printf '%s\n%s\n' "$minimum" "$pinned" | sort -V | head -n 1) == "$minimum" ]] ||
    fail "tools/install-just.sh pins $pinned, older than the justfile's minimum $minimum"
echo "The justfile's $count recipes and the Makefile shim's ${#old_targets[@]} targets passed."
