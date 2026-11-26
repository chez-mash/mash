#!/usr/bin/env python3
"""Decide which areas of CI a set of changed files needs.

CI's workflows always start, because a release requires a successful `ci.yml`
and `windows.yml` run for its exact commit. A `changes` job runs this script
and the expensive jobs carry an `if` on its answer, so a job that a change
cannot affect is skipped and the run still concludes successfully.

Reads changed paths, one per line, from standard input and prints one
`area=true|false` line per area, the form `$GITHUB_OUTPUT` takes. `--all`
answers true for every area without reading anything.

The rule is fail-safe: a path belongs to every area unless a pattern in
`CLAIMS` says otherwise. The claims name what the Rust build reads from the
front-end directories, and `test_ci_changes.py` checks them against the
source, so embedding another file there fails a test instead of skipping CI.
"""

from __future__ import annotations

import argparse
import fnmatch
import sys
from collections.abc import Iterable, Sequence

AREAS = ("rust", "dashboard", "raycast")

RUST = frozenset({"rust"})
NONE: frozenset[str] = frozenset()
# The first pattern a path matches names its areas. `fnmatch` lets `*` cross
# directory separators, so `site/*` covers the whole tree.
CLAIMS: tuple[tuple[str, frozenset[str]], ...] = (
    # What the Rust build or its jobs read from the front-end directories: the
    # setup page and the API contract are compiled into the binary or compared
    # by a Rust test, and the Windows job takes its pnpm version from the
    # site's manifest.
    ("site/public/setup.md", RUST),
    ("site/package.json", RUST),
    ("dashboard/openapi.json", frozenset({"rust", "dashboard"})),
    # The mapping from dashboard source to browser specs is checked by a tools
    # test, which the dashboard workflow runs as well.
    ("tools/affected_tests.py", frozenset({"rust", "dashboard"})),
    ("tools/test_affected_tests.py", frozenset({"rust", "dashboard"})),
    # Published by docs.yml, which has its own path filter.
    ("site/*", NONE),
    ("dashboard/*", frozenset({"dashboard"})),
    ("raycast/*", frozenset({"raycast"})),
    # No front end reads these. The user guides are Rust's because the binary
    # embeds them; docs.yml publishes them under its own filter.
    *(
        (pattern, RUST)
        for pattern in (
            "crates/*", "app/*", "cli/*", "fixtures/*", "queries/*", "protocol/*", "plugins/*",
            "deployment/*", "packaging/*", "docs/*", "README.md", "Cargo.toml", "Cargo.lock",
            "rust-toolchain.toml", "deny.toml", "sqlc.yaml", ".cargo/*", ".config/*",
        )
    ),
    # Instructions for people and agents, read by no build and no test.
    *((name, NONE) for name in ("AGENTS.md", "CLAUDE.md", "CONTRIBUTING.md", "SECURITY.md")),
)


def areas_for(path: str) -> frozenset[str]:
    """The areas one changed path can affect; every area when nothing claims it."""
    for pattern, areas in CLAIMS:
        if fnmatch.fnmatchcase(path, pattern):
            return areas
    return frozenset(AREAS)


def affected(paths: Iterable[str]) -> frozenset[str]:
    return frozenset(area for path in paths for area in areas_for(path))


def main(arguments: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--all", action="store_true", help="answer true for every area")
    known = parser.parse_args(arguments)
    paths = [] if known.all else [line.strip() for line in sys.stdin if line.strip()]
    chosen = frozenset(AREAS) if known.all else affected(paths)
    for area in AREAS:
        print(f"{area}={'true' if area in chosen else 'false'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
