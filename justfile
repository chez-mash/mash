# Mash's tasks. `just` lists them by group; `just --show RECIPE` prints one.
#
# The recipes call the scripts in tools/ and the packages' own scripts, which
# hold the logic; this file only names them and carries the settings below.
# Install the pinned `just` with `bash tools/install-just.sh` (Linux), or see
# "Task runner" in docs/internal/development.md for macOS and Windows.
#
# Settings. Each takes its value from the environment and can also be given
# before the recipe, so these two are the same:
#
#   DEV_TARGET_DIR=/somewhere just dev-build
#   just DEV_TARGET_DIR=/somewhere dev-build
#
#   DEV_TMPDIR       TMPDIR for Rust builds and tests. The default is Xenia's
#                    disk-backed scratch directory.
#   DEV_TARGET_DIR   the Cargo target directory `iterate`, `dev-build`, `test`,
#                    `clippy` and `validate-dev` share, across worktrees too.
#   DEV_RUN          a launcher put in front of heavy work: `agent-test` where
#                    it is installed (Xenia), nothing elsewhere. `DEV_RUN=`
#                    turns it off, as CI does.
#   CARGO_ARGS       extra Cargo arguments for `build`, `dev-build`, `coverage`.
#   CODEGEN          1 makes `build` regenerate the SQLite queries first.
#   COVERAGE_OUTPUT  where `coverage` writes its JSON report.
#   FOCUS, PACKAGE, TARGETS   `iterate`'s filter, crate and `all`; also its arguments.
#   OWNER            the one test run `test-cleanup` removes containers for.

set minimum-version := '1.55.0'
set shell := ["bash", "-euo", "pipefail", "-c"]

DEV_TMPDIR := env('DEV_TMPDIR', '/home/james/scratch/achi-tmp')
DEV_TARGET_DIR := env('DEV_TARGET_DIR', '/home/james/scratch/achi-targets/integrated')
agent_test := '/home/james/.local/bin/agent-test'
DEV_RUN := env('DEV_RUN', if path_exists(agent_test) == 'true' { agent_test } else { '' })
CARGO_ARGS := env('CARGO_ARGS', '')
CODEGEN := env('CODEGEN', '0')
COVERAGE_OUTPUT := env('COVERAGE_OUTPUT', 'target/llvm-cov/coverage.json')
FOCUS := env('FOCUS', '')
PACKAGE := env('PACKAGE', '')
TARGETS := env('TARGETS', 'lib')
OWNER := env('OWNER', '')

# mold where it is installed, for `dev-build`, `test` and `clippy`: the same
# linker as `iterate`, so they share one warm cache.
dev_linker := if shell('command -v mold 2>/dev/null || true') != '' { 'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"' } else { '' }
dev_dirs := 'TMPDIR="' + DEV_TMPDIR + '" CARGO_TARGET_DIR="' + DEV_TARGET_DIR + '"'
just := quote(just_executable()) + ' --justfile ' + quote(justfile())

[private]
default:
    @{{ just }} --list --unsorted

# ---- develop ----------------------------------------------------------------

# The edit loop: rustfmt, Clippy and the tests FOCUS selects (--package CRATE, --targets all)
[arg('package', long)]
[arg('targets', long)]
[group('develop')]
iterate $focus=FOCUS package=PACKAGE targets=TARGETS:
    @test -n "${focus}" || { echo "Name a nextest filter for the changed behavior: just iterate 'test(changed_behavior)' (or set FOCUS)"; exit 2; }
    env {{ dev_dirs }} CARGO_INCREMENTAL=1 RUSTC_WRAPPER= MASH_ITERATE_LAUNCHER="{{ DEV_RUN }}" MASH_ITERATE_PACKAGE="{{ package }}" bash tools/build-dev-linux.sh --focused "${focus}" {{ if targets == 'all' { '--all-targets' } else { '' } }}

# A debug `mash` to try by hand; prints the binary's path
[group('develop')]
dev-build *cargo_args=CARGO_ARGS:
    {{ DEV_RUN }} env {{ dev_dirs }} CARGO_INCREMENTAL=1 RUSTC_WRAPPER= {{ dev_linker }} cargo build-clif --locked --bin mash {{ cargo_args }}
    @echo "{{ DEV_TARGET_DIR }}/debug/mash"

# The broader gates for a stable release batch, without building a release binary
[group('develop')]
validate-dev:
    {{ DEV_RUN }} env {{ dev_dirs }} CARGO_INCREMENTAL=0 bash tools/build-dev-linux.sh --validate-only

# Check the history skill, then build the workspace (CODEGEN=1 regenerates queries first)
[group('develop')]
build *cargo_args=CARGO_ARGS: check-skills
    @if [[ "{{ CODEGEN }}" == 1 ]]; then {{ just }} generate-queries; fi
    cargo build-clif {{ cargo_args }}

# The dashboard's dev server against a running `mash dash` (dashboard/README.md)
[group('develop')]
[positional-arguments]
dashboard-dev *args:
    pnpm --dir dashboard dev "$@"

# The dashboard's dev server with the demo's made-up history
[group('develop')]
[positional-arguments]
dashboard-demo *args:
    pnpm --dir dashboard dev:demo "$@"

# Serve the dashboard's last production build (`pnpm --dir dashboard build`)
[group('develop')]
[positional-arguments]
dashboard-preview *args:
    pnpm --dir dashboard preview "$@"

# The documentation site's dev server
[group('develop')]
[positional-arguments]
site-dev *args:
    pnpm --dir site dev "$@"

# The site with the Agentation review toolbar, to annotate pages for an agent
[group('develop')]
site-review:
    pnpm --dir site dev:review

# The dashboard's Storybook dev server on loopback
[group('develop')]
[positional-arguments]
storybook *args:
    pnpm --dir dashboard storybook "$@"

# A worktree on branch work/NAME for one task: NAME [BASE] [--deps dashboard,site,raycast]
[group('develop')]
[positional-arguments]
worktree-new name *args:
    bash tools/worktree.sh new "$@"

# Land a worktree's commits on main through the merge queue, and wait for them
[group('develop')]
[positional-arguments]
worktree-land name *flags='--wait':
    bash tools/worktree.sh land "$@"

# Remove a landed worktree and its branch (--force discards unlanded work)
[group('develop')]
[positional-arguments]
worktree-done name *flags:
    bash tools/worktree.sh done "$@"

# Every worktree of this clone
[group('develop')]
worktree-list:
    bash tools/worktree.sh list

# A worktree's dev server on the tailnet: NAME [--site|--storybook] [--local] [--stop]
[group('develop')]
[positional-arguments]
worktree-preview name *flags:
    bash tools/worktree.sh preview "$@"

# The previews that are recorded, and whether each answers
[group('develop')]
worktree-previews:
    bash tools/worktree.sh previews

# ---- test -------------------------------------------------------------------

# One crate's Rust tests from the warm cache: CRATE [NEXTEST ARGUMENTS]
[group('test')]
[positional-arguments]
test crate='mash-app' *nextest_args:
    {{ DEV_RUN }} env {{ dev_dirs }} CARGO_INCREMENTAL=1 RUSTC_WRAPPER= {{ dev_linker }} cargo test-clif --locked -p "$1" "${@:2}"

# The dashboard's unit tests that import what changed since origin/main
[group('test')]
[positional-arguments]
test-changed *args:
    pnpm --dir dashboard test:changed "$@"

# The dashboard browser specs a change can affect: [FILES] [-- PLAYWRIGHT ARGUMENTS]
[group('test')]
[positional-arguments]
e2e-affected *args:
    {{ DEV_RUN }} pnpm --dir dashboard e2e:affected "$@"

# Compare pages with their committed pictures: [dashboard|site|all] [--exact] [--no-build]
[group('test')]
[positional-arguments]
visual package='all' *args:
    bash tools/visual.sh "$@"

# Take the pictures again after a deliberate change: [dashboard|site|all]
[group('test')]
[positional-arguments]
visual-update package='all' *args:
    bash tools/visual.sh "$1" --update "${@:2}"

# The tests of the scripts in tools/, or only those named (justfile, worktree, pre-push, ...)
[group('test')]
[positional-arguments]
tools-tests *names:
    #!/usr/bin/env bash
    set -euo pipefail
    export TMPDIR="${TMPDIR:-{{ DEV_TMPDIR }}}"
    if [[ $# -eq 0 ]]; then
        set -- justfile pre-push worktree dev-build dev-api pr-labels controller-environment run-ansible ansible release
        python3 -m unittest discover -s tools -p 'test_*.py'
    fi
    for name; do
        echo "== tools/test-$name.sh" >&2
        bash "tools/test-$name.sh"
    done

# The cheap checks CI would fail a push on: [BASE [HEAD]] or --worktree
[group('test')]
[positional-arguments]
pre-push *args:
    bash tools/pre-push.sh "$@"

# An LLVM coverage report over every test; needs models, a disposable Qdrant and its binaries
[group('test')]
coverage:
    @test -n "${MASH_TEST_MODELS_DIR:-}" || { echo 'Set MASH_TEST_MODELS_DIR to explicitly installed test models'; exit 1; }
    @test -n "${MASH_TEST_QDRANT_URL:-}" || { echo 'Set MASH_TEST_QDRANT_URL to disposable Qdrant 1.19+'; exit 1; }
    @test -n "${MASH_TEST_QDRANT_GRPC_URL:-}" || { echo 'Set MASH_TEST_QDRANT_GRPC_URL to the same disposable Qdrant instance'; exit 1; }
    @test -n "${MASH_TEST_SERVER_BINARY:-}" || { echo 'Set MASH_TEST_SERVER_BINARY to the coverage-target mash-server binary for database restart contracts'; exit 1; }
    @test -n "${MASH_TEST_CLI_BINARY:-}" || { echo 'Set MASH_TEST_CLI_BINARY to the coverage-target mash binary for MCP probe contracts'; exit 1; }
    cargo llvm-cov clean --workspace
    cargo llvm-cov nextest --no-report --run-ignored all -j 2 -E 'not test(benchmark)' {{ CARGO_ARGS }}
    cargo llvm-cov report --json --ignore-filename-regex '/generated/|/(tests|benchmarks)(/|\.rs$)|/test_support\.rs$|/semantic/evaluation(/|\.rs$)|/semantic/embeddings/cpu/sensitivity\.rs$|/storage/indexing/memory\.rs$' --output-path "{{ COVERAGE_OUTPUT }}"

# Remove containers tests left running (never a real Qdrant): [OWNER] for one run's
[group('test')]
test-cleanup owner=OWNER:
    bash tools/clean-test-containers.sh {{ owner }}

# ---- quality ----------------------------------------------------------------

# rustfmt's check, Biome in dashboard/ and site/, and Raycast's lint (Clippy is `clippy`)
[group('quality')]
lint: lint-rust lint-dashboard lint-site lint-raycast

# rustfmt's check for the workspace and the desktop app
[group('quality')]
lint-rust:
    cargo fmt --check
    cargo fmt --check --manifest-path desktop/src-tauri/Cargo.toml

# Biome's check of dashboard/
[group('quality')]
lint-dashboard:
    pnpm --dir dashboard lint

# Biome's check of site/
[group('quality')]
lint-site:
    pnpm --dir site lint

# Raycast's lint: its manifest, Prettier and ESLint
[group('quality')]
lint-raycast:
    npm --prefix raycast run lint

# Clippy over the whole workspace with warnings as errors
[group('quality')]
clippy:
    {{ DEV_RUN }} env {{ dev_dirs }} CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings

# Rewrite formatting: rustfmt for the workspace and the desktop app, Biome in dashboard/
[group('quality')]
fmt:
    cargo fmt
    cargo fmt --manifest-path desktop/src-tauri/Cargo.toml
    pnpm --dir dashboard format

# Type checks: svelte-check in dashboard/, Astro and tsc in site/
[group('quality')]
typecheck:
    pnpm --dir dashboard check
    pnpm --dir site check

# The dependency policy in deny.toml: advisories, licences, bans and sources
[group('quality')]
deny:
    cargo deny check

# Unused files, exports, types and dependencies in dashboard/, site/ and raycast/
[group('quality')]
knip:
    pnpm --dir dashboard knip
    pnpm --dir site knip
    npm --prefix raycast run knip

# Keep the global stylesheets from growing (--update after removing CSS)
[group('quality')]
[positional-arguments]
css-budget *args:
    python3 tools/check_css_budget.py "$@"

# Keep source files within the line limit (--against BASE reviews growth too)
[group('quality')]
[positional-arguments]
file-lines *args:
    python3 tools/check_file_lines.py "$@"

# Install the pinned TLA+ checker for `protocol-check`
[group('quality')]
protocol-tools:
    bash protocol/check.sh --install-tools

# The bounded protocol models in protocol/ and their expected counterexamples
[group('quality')]
protocol-check:
    {{ DEV_RUN }} bash protocol/check.sh

# Check the history skill's generated files and its examples
[group('quality')]
check-skills:
    bash tools/generate-history-skill.sh --check
    cargo test-clif --locked --lib -E 'test(history_skill_)' --no-tests fail

# Regenerate the history skill's generated files
[group('quality')]
generate-skills:
    bash tools/generate-history-skill.sh

# Rewrite docs/tools.md from the MCP tool definitions
[group('quality')]
generate-docs:
    MASH_WRITE_TOOL_REFERENCE=1 cargo test-clif --locked --lib -E 'test(tool_reference_is_current)' --no-tests fail

# Regenerate the SQLite query models (pinned sqlc and sqlc-gen-rust)
[group('quality')]
generate-queries:
    bash queries/generate.sh

# Regenerate the SQLite query models and fail when the committed ones differ
[group('quality')]
check-generated: generate-queries
    git diff --exit-code -- crates/mash-storage/src/storage/generated crates/mash-sync/src/shared/silver_store/generated

# Install the pinned cargo-mutants for the mutation audits
[group('quality')]
mutation-tools:
    bash tools/mutation-check.sh --install-tools

# The revision and freshness mutation audit: a baseline, then isolated mutations
[group('quality')]
mutation-check:
    {{ DEV_RUN }} bash tools/mutation-check.sh

# The dashboard service authentication and schema mutation audit
[group('quality')]
mutation-services:
    {{ DEV_RUN }} bash tools/mutation-check.sh --services

# The certification mutation audit (Store::certify)
[group('quality')]
mutation-certification:
    {{ DEV_RUN }} bash tools/mutation-check.sh --certification

# The runtime mutation audit
[group('quality')]
mutation-runtime:
    {{ DEV_RUN }} bash tools/mutation-check.sh --runtime

# ---- deploy -----------------------------------------------------------------

# A clean tree at origin/main for deploys and releases; prints its path
[group('deploy')]
deploy-tree:
    bash tools/worktree.sh deploy-tree

# Check services, credentials and install access on TARGETS; installs nothing
[group('deploy')]
[positional-arguments]
preflight +targets:
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/fleet.sh preflight "$@"

# Deploy origin/main to TARGETS from the deploy tree (--source-build always compiles)
[group('deploy')]
[positional-arguments]
deploy +targets:
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/fleet.sh deploy "$@"

# Install a published release's assets on TARGETS: VERSION is vX.Y.Z
[group('deploy')]
[positional-arguments]
deploy-release version +targets:
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/fleet.sh deploy --release "$@"

# Probe the installed build, contract and readiness on TARGETS
[group('deploy')]
[positional-arguments]
check +targets:
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/fleet.sh check "$@"

# Another fleet operation from the deploy tree: doctor, sync, local-views, watch-status, ...
[group('deploy')]
[positional-arguments]
fleet operation +targets:
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/fleet.sh "$@"

# ---- release ----------------------------------------------------------------

# Install the pinned cargo-release that tools/release.sh is written against
[group('release')]
release-tools:
    bash tools/release.sh --install-tools

# Cut release X.Y.Z from the deploy tree (--dry-run rehearses, --resume continues)
[arg('dry_run', long='dry-run', value='--dry-run')]
[arg('resume', long='resume', value='--resume')]
[group('release')]
release version dry_run='' resume='':
    cd "$(bash tools/worktree.sh deploy-tree)" && bash tools/release.sh {{ dry_run }} {{ resume }} "{{ version }}"

# ---- design -----------------------------------------------------------------

# Build and serve main's Storybook on the tailnet, or refresh it (--status, --stop, --refresh)
[group('design')]
[positional-arguments]
storybook-serve *args:
    bash tools/storybook-serve.sh "$@"

# Build the dashboard's static Storybook into dashboard/storybook-static
[group('design')]
storybook-build:
    {{ DEV_RUN }} pnpm --dir dashboard storybook:build

# Take the guides' dashboard screenshots again (assets/screenshots)
[group('design')]
[positional-arguments]
screenshots *args:
    pnpm --dir site screenshots "$@"

# The Raycast extension in a throwaway macOS VM: run, build-image, push-image, list, clean
[group('design')]
[positional-arguments]
mac-vm command='run' *args:
    bash tools/mac-vm.sh "$@"

# The Raycast Store screenshots, taken in the macOS VM: [--mash PATH] [--keep]
[group('design')]
[positional-arguments]
store-shots *args:
    bash tools/mac-vm.sh store-shots "$@"
