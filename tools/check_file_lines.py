#!/usr/bin/env python3
"""Keep handwritten production source files at or below 500 lines.

Counts physical lines in tracked Rust, TypeScript, Svelte and Python files.
Tests and generated files are exempt, and so is a Rust file's inline
`#[cfg(test)] mod ... {` block at its end. Files that were already longer
when the limit was introduced are listed in `file-lines-baseline.txt` with
their length then; each may shrink but not grow, and its entry must be
lowered as it shrinks, so the list only gets shorter. `--update` lowers or
drops entries; it never raises one or adds a file.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

LINE_LIMIT = 500
BASELINE = Path("tools/file-lines-baseline.txt")
SUFFIXES = (".rs", ".ts", ".svelte", ".py")
TEST_PATH = re.compile(
    r"(^|/)(tests?|storage_tests)/"
    r"|(^|/)tests?\.rs$|_tests?\.rs$"
    r"|(^|/)test_[^/]*\.py$|_test\.py$"
    r"|\.(test|spec)\.ts$"
)
GENERATED_MARKERS = (
    "DO NOT EDIT",
    "@generated",
    "auto-generated",
    "Do not make direct changes",
)
INLINE_TESTS = re.compile(r"(pub(\([^)]*\))? )?mod \w+ \{")
TEST_DECLARATION = re.compile(r"(pub(\([^)]*\))? )?mod \w+;")


@dataclass(frozen=True)
class Finding:
    path: str
    lines: int
    message: str


def budgeted(path: str, text: str) -> bool:
    """Whether a tracked file is handwritten production source."""
    if not path.endswith(SUFFIXES) or TEST_PATH.search(path) or "/generated/" in path:
        return False
    head = text[:600]
    return not any(marker in head for marker in GENERATED_MARKERS)


def counted_lines(path: str, text: str) -> int:
    """Physical lines, without a Rust file's test modules: a trailing inline
    `#[cfg(test)] mod ... {` block, and `#[cfg(test)] mod ...;` declarations
    of tests kept in their own file."""
    lines = text.splitlines()
    if not path.endswith(".rs"):
        return len(lines)
    count = len(lines)
    for index, line in enumerate(lines[:-1]):
        if not line.startswith("#[cfg(test)]"):
            continue
        following = lines[index + 1]
        if INLINE_TESTS.match(following):
            return index - (len(lines) - count)
        if TEST_DECLARATION.match(following):
            count -= 2
    return count


def read_baseline(text: str) -> dict[str, int]:
    entries: dict[str, int] = {}
    for number, line in enumerate(text.splitlines(), start=1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        count, _, path = line.partition(" ")
        if not count.isdigit() or not path:
            raise ValueError(f"{BASELINE}:{number}: expected 'LINES PATH'")
        entries[path] = int(count)
    return entries


def render_baseline(entries: dict[str, int]) -> str:
    header = (
        "# Files longer than the 500-line limit when it was introduced, with their\n"
        "# length then. Each may shrink but not grow; lower its entry as it shrinks\n"
        "# (python3 tools/check_file_lines.py --update). Do not add files.\n"
    )
    body = "".join(f"{count} {path}\n" for path, count in sorted(entries.items()))
    return header + body


def measure(sources: dict[str, str]) -> dict[str, int]:
    """Counted lines for every budgeted file among `sources` (path to text)."""
    return {
        path: counted_lines(path, text)
        for path, text in sources.items()
        if budgeted(path, text)
    }


def check(
    measured: dict[str, int], baseline: dict[str, int]
) -> tuple[list[Finding], dict[str, int]]:
    """Failures, and the baseline with every entry lowered to the current length."""
    failures: list[Finding] = []
    lowered: dict[str, int] = {}
    for path, lines in sorted(measured.items()):
        allowed = baseline.get(path)
        if allowed is None:
            if lines > LINE_LIMIT:
                failures.append(
                    Finding(
                        path,
                        lines,
                        f"over the {LINE_LIMIT}-line limit by {lines - LINE_LIMIT}; split it",
                    )
                )
        elif lines > allowed:
            failures.append(
                Finding(
                    path,
                    lines,
                    f"grew past its recorded {allowed} lines; split it instead",
                )
            )
        elif lines > LINE_LIMIT:
            lowered[path] = lines
    stale = {path for path, allowed in baseline.items() if lowered.get(path) != allowed}
    for path in sorted(stale):
        if not any(finding.path == path for finding in failures):
            now = lowered.get(path, measured.get(path, 0))
            failures.append(
                Finding(
                    path,
                    now,
                    f"shrank from its recorded {baseline[path]} lines; run with --update to lower its entry",
                )
            )
    return failures, lowered


def tracked_sources(root: Path) -> dict[str, str]:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "--", *(f"*{suffix}" for suffix in SUFFIXES)],
        cwd=root,
        check=True,
        capture_output=True,
    ).stdout.decode()
    sources: dict[str, str] = {}
    for path in filter(None, listed.split("\0")):
        file = root / path
        if file.is_file():
            sources[path] = file.read_text(encoding="utf-8", errors="replace")
    return sources


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="lower or drop baseline entries that shrank",
    )
    args = parser.parse_args(argv)
    baseline_file = args.root / BASELINE
    try:
        baseline = (
            read_baseline(baseline_file.read_text()) if baseline_file.exists() else {}
        )
        measured = measure(tracked_sources(args.root))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"file-lines: cannot inspect sources: {error}", file=sys.stderr)
        return 2
    failures, lowered = check(measured, baseline)
    growth = [finding for finding in failures if "shrank" not in finding.message]
    if args.update:
        if growth:
            for finding in growth:
                print(f"FAIL: {finding.path}: {finding.lines} lines, {finding.message}")
            print("FAIL: --update lowers entries only; split the files above first")
            return 1
        baseline_file.write_text(render_baseline(lowered))
        print(f"Updated {BASELINE}: {len(lowered)} files above the limit remain")
        return 0
    for finding in failures:
        print(f"FAIL: {finding.path}: {finding.lines} lines, {finding.message}")
    if failures:
        print(
            f"FAIL: {len(failures)} files break the {LINE_LIMIT}-line limit or its baseline"
        )
        return 1
    print(
        f"PASS: {len(measured)} handwritten source files; {len(lowered)} recorded files above "
        f"{LINE_LIMIT} lines have not grown"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
