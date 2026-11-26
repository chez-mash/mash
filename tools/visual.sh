#!/usr/bin/env bash
# Compares the dashboard's and the site's pages with their committed pictures
# (the visual baselines), or takes the pictures again.
#
#   tools/visual.sh [dashboard|site|all] [--update] [--exact] [--no-build] [-- PLAYWRIGHT ARGUMENTS]
#
#   tools/visual.sh                       compare both (what CI runs)
#   tools/visual.sh all --update          take every picture again, after a deliberate change
#   tools/visual.sh dashboard -- -g chat  compare only the dashboard's chat pictures
#   tools/visual.sh all --exact           compare with no tolerance: run it a few times to
#                                         show that new pictures are the same on every run
#
# The same as `pnpm visual` and `pnpm visual:update` in dashboard/ and site/.
#
# Fonts and antialiasing differ from one system to the next, so the browser
# always runs in the Playwright container image that matches the installed
# Playwright version, here and in CI alike, with no network. The package is
# built on the host first (`pnpm build:demo`, `pnpm build`); only the browser
# and the preview server run in the container. Podman is used when installed,
# otherwise Docker; MASH_VISUAL_ENGINE names one explicitly.
#
# A full --update removes the old pictures first, so a renamed or dropped page
# leaves nothing behind. A failed comparison leaves the expected, actual and
# difference pictures in PACKAGE/visual/results and a report to open at
# PACKAGE/visual/report/index.html. See docs/internal/verification.md.
set -euo pipefail

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
say() { printf '%s\n' "$*" >&2; }
die() { say "visual: $*"; exit 1; }

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
packages=()
update=no
exact=''
build=yes
extra=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        dashboard|site) packages+=("$1"); shift ;;
        all) packages=(dashboard site); shift ;;
        --update) update=yes; shift ;;
        --exact) exact=1; shift ;;
        --no-build) build=no; shift ;;
        --) shift; extra=("$@"); break ;;
        *) usage ;;
    esac
done
[[ ${#packages[@]} -gt 0 ]] || packages=(dashboard site)

engine=${MASH_VISUAL_ENGINE:-}
if [[ -z $engine ]]; then
    if command -v podman > /dev/null; then engine=podman
    elif command -v docker > /dev/null; then engine=docker
    else die 'needs podman or docker to run the Playwright container'
    fi
fi

build_package() {
    case "$1" in
        dashboard) (cd "$root/dashboard" && pnpm build:demo) ;;
        site) (cd "$root/site" && pnpm build) ;;
    esac
}

# Runs Playwright for one package inside the container; returns its status.
run_package() {
    local package=$1 directory=$root/$1 version arguments cpus identity=()
    [[ -d $directory/node_modules/@playwright/test ]] ||
        die "$package has no node_modules; run pnpm install --frozen-lockfile in $package/"
    version=$(sed -n 's/^  "version": "\(.*\)",$/\1/p' "$directory/node_modules/@playwright/test/package.json")
    [[ -n $version ]] || die "cannot read the Playwright version installed in $package"
    arguments=(test --config playwright.visual.config.ts)
    if [[ $update == yes ]]; then
        arguments+=(--update-snapshots=changed)
        # Every picture is being taken again: start from none.
        [[ ${#extra[@]} -gt 0 ]] || rm -rf "$directory/visual/baselines"
    fi
    # Rootless Podman already maps the container's root to the caller. Docker
    # does not, and would leave root's files in the checkout.
    [[ $engine == podman ]] || identity=(--user "$(id -u):$(id -g)" --env HOME=/tmp)
    # At most four processors, and never more than the machine has.
    cpus=$(nproc 2> /dev/null || echo 4)
    [[ $cpus -le 4 ]] || cpus=4
    say "visual: $package in mcr.microsoft.com/playwright:v$version-noble ($engine)"
    "$engine" run --rm --init --ipc=host --network=none --memory=4g --cpus="$cpus" \
        "${identity[@]}" \
        --env MASH_VISUAL_CONTAINER=1 --env MASH_VISUAL_EXACT="$exact" --env CI \
        --volume "$directory:/work" --workdir /work \
        "mcr.microsoft.com/playwright:v$version-noble" \
        node node_modules/@playwright/test/cli.js "${arguments[@]}" "${extra[@]}"
}

started=$SECONDS
failed=()
for package in "${packages[@]}"; do
    [[ $build == no ]] || build_package "$package"
    run_package "$package" || failed+=("$package")
done
say "visual: finished in $((SECONDS - started))s"
if [[ ${#failed[@]} -gt 0 ]]; then
    for package in "${failed[@]}"; do
        say "visual: $package differs from its baselines. Open $package/visual/report/index.html,"
        say "        or see the -actual, -expected and -diff pictures under $package/visual/results."
    done
    say 'visual: if the change is intended, run tools/visual.sh all --update, review the pictures and commit them.'
    exit 1
fi
[[ $update == no ]] || say 'visual: pictures taken. Review them with git status and git diff before committing.'
