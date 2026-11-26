#!/usr/bin/env python3
"""Decide which areas of CI a set of changed files needs.

CI's workflows always start, because a release requires a successful `ci.yml`
and `windows.yml` run for its exact commit. A `changes` job runs this script
and the expensive jobs carry an `if` on its answer, so a job that a change
cannot affect is skipped and the run still concludes successfully.

Reads changed paths, one per line, from standard input and prints one
`area=true|false` line per area, the form `$GITHUB_OUTPUT` takes. `--all`
answers true for every area without reading anything, and `--none` false.

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

AREAS = ("rust", "dashboard", "raycast", "docs")

RUST = frozenset({"rust"})
# The website's build and its pictures (docs.yml): the site, the guides it
# publishes, and the dashboard, which it serves as the demo.
DOCS = frozenset({"docs"})
NONE: frozenset[str] = frozenset()
# The first pattern a path matches names its areas. `fnmatch` lets `*` cross
# directory separators, so `site/*` covers the whole tree.
CLAIMS: tuple[tuple[str, frozenset[str]], ...] = (
    # What the Rust build or its jobs read from the front-end directories: the
    # setup page and the API contract are compiled into the binary or compared
    # by a Rust test, and the Windows job takes its pnpm version from the
    # site's manifest.
    ("site/public/setup.md", RUST | DOCS),
    ("site/package.json", RUST | DOCS),
    ("dashboard/openapi.json", frozenset({"rust", "dashboard", "docs"})),
    # The mapping from dashboard source to browser specs is checked by a tools
    # test, which the dashboard workflow runs as well.
    ("tools/affected_tests.py", frozenset({"rust", "dashboard"})),
    ("tools/test_affected_tests.py", frozenset({"rust", "dashboard"})),
    ("site/*", DOCS),
    ("dashboard/*", frozenset({"dashboard", "docs"})),
    ("raycast/*", frozenset({"raycast"})),
    # The development notes are prose for people: no build embeds them and no
    # test reads them, so they need only the website's link and build checks.
    # The other files there, such as the server's contract copy that a Rust
    # test compares, fall through to the rule below.
    ("docs/internal/*.md", DOCS),
    # The user guides are Rust's as well as the website's, because the binary
    # embeds them.
    ("docs/*", RUST | DOCS),
    ("README.md", RUST | DOCS),
    # No front end reads these.
    *(
        (pattern, RUST)
        for pattern in (
            "crates/*", "app/*", "cli/*", "fixtures/*", "queries/*", "protocol/*", "plugins/*",
            "deployment/*", "packaging/*", "Cargo.toml", "Cargo.lock",
            "rust-toolchain.toml", "deny.toml", "sqlc.yaml", ".cargo/*", ".config/*",
        )
    ),
    # The Scoop manifest a release renders from a template under packaging/:
    # Scoop reads it from main, and no build or test does.
    ("bucket/*", NONE),
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
    answers = parser.add_mutually_exclusive_group()
    answers.add_argument("--all", action="store_true", help="answer true for every area")
    answers.add_argument("--none", action="store_true", help="answer false for every area")
    known = parser.parse_args(arguments)
    if known.all:
        chosen = frozenset(AREAS)
    elif known.none:
        chosen = NONE
    else:
        chosen = affected(line.strip() for line in sys.stdin if line.strip())
    for area in AREAS:
        print(f"{area}={'true' if area in chosen else 'false'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
