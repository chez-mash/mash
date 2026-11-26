#!/usr/bin/env python3
"""List class and id selectors that a front end's stylesheets define and
nothing in its sources names.

A name is "used" when its literal appears as a whole word anywhere in the
package's sources outside a stylesheet: markup, `class:name`, `classList`,
`cn(...)`, template strings, tests and scripts alike. That is deliberately
generous, so a hit is a strong sign the rule is dead. Three kinds of name are
never reported as unused, each for a stated reason:

- a member of a family built at run time (`tone-${x}`, `class="rank-{n}"`),
  listed separately so a person can check the family's members;
- a name found in the third-party code that sets it (Starlight, Expressive
  Code, Pagefind), when that code is installed;
- a name in `tools/unused-css-keep.json`, with the reason it stays.

A selector is dead when it needs an unused name to match; names inside
`:not()`, `:is()`, `:where()` and `:has()` never make it dead. `--prune`
removes dead selectors, the rules and at-rule blocks left without any, and
the comments that introduced only removed rules. Review the list first, then
run the formatter over the result.

    python3 tools/unused_css.py                list, exit 1 when any is unused
    python3 tools/unused_css.py dashboard      one package
    python3 tools/unused_css.py --prune        rewrite the stylesheets
    python3 tools/unused_css.py dashboard --dead flip,cell --prune
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

from css_blocks import (
    Rule,
    blank_comments,
    names_in,
    parse,
    split_selectors,
    style_rules,
)

KEEP = Path("tools/unused-css-keep.json")
SKIPPED_DIRECTORIES = frozenset(
    {
        "node_modules",
        "dist",
        "dist-demo",
        "test-results",
        "playwright-report",
        ".astro",
        "target",
        "gen",
    }
)
SOURCE_SUFFIXES = {
    ".svelte",
    ".astro",
    ".ts",
    ".tsx",
    ".js",
    ".mjs",
    ".cjs",
    ".html",
    ".md",
    ".mdx",
    ".json",
    ".rs",
}
SKIPPED_FILES = {"pnpm-lock.yaml", "package-lock.json", "openapi.json"}


@dataclass(frozen=True)
class Package:
    """One front end: where its styles are, and every place that may name a class."""

    name: str
    styles: tuple[str, ...]
    sources: tuple[str, ...]
    third_party: tuple[str, ...] = ()


PACKAGES = (
    Package(
        "dashboard",
        styles=("dashboard/src",),
        # The website's screenshot script and tests drive the demo build, and
        # the desktop app loads the dashboard, so both may name its classes.
        sources=(
            "dashboard",
            "site/scripts",
            "site/tests",
            "desktop/ui",
            "desktop/src-tauri/src",
        ),
    ),
    Package(
        "site",
        styles=("site/src",),
        # The guides are written in docs/ and copied in at build time, and the
        # MCP reference's markup is written by the Rust that documents the tools.
        sources=("site", "docs", "README.md", "crates/mash-mcp/src/reference.rs"),
        third_party=(
            "site/node_modules/@astrojs/starlight",
            "site/node_modules/.pnpm/astro-expressive-code@*/node_modules/astro-expressive-code",
            "site/node_modules/.pnpm/@expressive-code+*/node_modules/@expressive-code/*",
            "site/node_modules/.pnpm/@pagefind+default-ui@*/node_modules/@pagefind/default-ui",
        ),
    ),
)

WORD = re.compile(r"[A-Za-z_][\w-]*")
STYLE_BLOCK = re.compile(r"(<style\b[^>]*>)(.*?)(</style>)", re.S)
# `tone-${x}`, `tone-{x}` and "tone-" + x build a family of names by prefix;
# `{x}-pill` builds one by suffix.
BUILT_PREFIX = re.compile(r"([A-Za-z_][\w-]*-)(?:\$?\{|[\"'`]\s*\+)")
BUILT_SUFFIX = re.compile(r"(?:\}|\+\s*[\"'`])(-[\w-]*[A-Za-z0-9])")


@dataclass(frozen=True)
class Sheet:
    path: Path
    offset: int  # where the CSS starts inside the file
    css: str

    def line(self, position: int) -> int:
        return (
            self.path.read_text(encoding="utf-8").count("\n", 0, self.offset + position)
            + 1
        )


def walk(root: Path, base: str, skipped: frozenset[str] = SKIPPED_DIRECTORIES):
    """The files under each directory a glob pattern names, or the file itself."""
    for start in sorted(root.glob(base)):
        if start.is_file():
            yield start
            continue
        for path in sorted(start.rglob("*")):
            if path.is_file() and not skipped.intersection(
                path.relative_to(start).parts
            ):
                yield path


def sheets_of(root: Path, package: Package) -> list[Sheet]:
    """Every stylesheet and every `<style>` block of a component."""
    found = []
    for base in package.styles:
        for path in walk(root, base):
            if path.suffix == ".css":
                found.append(Sheet(path, 0, path.read_text(encoding="utf-8")))
            elif path.suffix in {".svelte", ".astro"}:
                for block in STYLE_BLOCK.finditer(path.read_text(encoding="utf-8")):
                    found.append(Sheet(path, block.start(2), block.group(2)))
    return found


def reference_text(
    root: Path, bases: tuple[str, ...], third_party: bool = False
) -> str:
    """Everything that may name a class: the sources, without their styles.
    Third-party code is read whole, its built files and stylesheets too."""
    suffixes = SOURCE_SUFFIXES | {".css"} if third_party else SOURCE_SUFFIXES
    parts = []
    for base in bases:
        for path in walk(
            root,
            base,
            frozenset({"node_modules"}) if third_party else SKIPPED_DIRECTORIES,
        ):
            if path.suffix not in suffixes or path.name in SKIPPED_FILES:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            parts.append(STYLE_BLOCK.sub(" ", text))
    return "\n".join(parts)


@dataclass
class Usage:
    words: set[str]
    prefixes: set[str]
    suffixes: set[str]
    third_party: set[str]
    kept: dict[str, str]
    dead: frozenset[str] = frozenset()

    def family(self, name: str) -> str | None:
        for prefix in self.prefixes:
            if name.startswith(prefix) and len(name) > len(prefix):
                return f"{prefix}*"
        for suffix in self.suffixes:
            if name.endswith(suffix) and len(name) > len(suffix):
                return f"*{suffix}"
        return None

    def verdict(self, name: str) -> str:
        """`used`, `kept`, `third-party`, `family` or `unused`."""
        if name in self.dead:
            return "unused"
        if name in self.words:
            return "used"
        if name in self.kept:
            return "kept"
        if name in self.third_party:
            return "third-party"
        return "family" if self.family(name) else "unused"


def usage_of(
    root: Path,
    package: Package,
    kept: dict[str, str],
    dead: frozenset[str] = frozenset(),
) -> Usage:
    text = reference_text(root, package.sources)
    return Usage(
        words=set(WORD.findall(text)),
        prefixes=set(BUILT_PREFIX.findall(text)),
        suffixes=set(BUILT_SUFFIX.findall(text)),
        third_party=set(
            WORD.findall(reference_text(root, package.third_party, third_party=True))
        ),
        kept=kept,
        dead=dead,
    )


def dead_selector(selector: str, usage: Usage) -> bool:
    return any(
        usage.verdict(name) == "unused"
        for _, name in names_in(selector, required_only=True)
    )


def dead_rule(rule: Rule, usage: Usage) -> bool:
    """Whether no element can match: the rule's own selectors are all dead,
    or it is nested in a rule that is."""
    if rule.at_rule:
        return False
    if all(
        dead_selector(selector, usage) for selector in split_selectors(rule.prelude)
    ):
        return True
    return rule.parent is not None and dead_rule(rule.parent, usage)


@dataclass
class Report:
    unused: dict[str, list[tuple[Path, int]]] = field(default_factory=dict)
    families: dict[str, set[str]] = field(default_factory=dict)
    dead_rules: int = 0


def survey(sheets: list[Sheet], usage: Usage) -> Report:
    report = Report()
    for sheet in sheets:
        for rule in style_rules(parse(sheet.css)):
            report.dead_rules += dead_rule(rule, usage)
            for sigil, name in sorted(names_in(rule.prelude)):
                verdict = usage.verdict(name)
                if verdict == "unused":
                    report.unused.setdefault(sigil + name, []).append(
                        (sheet.path, sheet.line(rule.start))
                    )
                elif verdict == "family":
                    report.families.setdefault(usage.family(name) or "", set()).add(
                        sigil + name
                    )
    return report


def removals(css: str, usage: Usage) -> list[tuple[int, int, str]]:
    """(start, end, replacement) edits that take the dead selectors, rules and
    emptied at-rule blocks out of a stylesheet."""
    edits: list[tuple[int, int, str]] = []

    def visit(rule: Rule) -> bool:
        """Records the edits inside a block; returns whether the block goes."""
        if rule.opaque:
            return False
        if not rule.at_rule:
            selectors = split_selectors(rule.prelude)
            alive = [
                selector for selector in selectors if not dead_selector(selector, usage)
            ]
            if not alive:
                return True
            if len(alive) < len(selectors):
                edits.append((rule.start, rule.open, ",\n".join(alive) + " "))
        gone = [child for child in rule.children if visit(child)]
        if rule.children and len(gone) == len(rule.children) and not rule.declarations:
            return True
        edits.extend((child.start, child.end, "") for child in gone)
        return False

    edits.extend((rule.start, rule.end, "") for rule in parse(css) if visit(rule))
    return edits


def prune(css: str, usage: Usage) -> str:
    """The stylesheet without its dead rules and the comments that introduced
    only dead rules."""
    edits = sorted(removals(css, usage))
    if not edits:
        return css
    removed = [(start, end) for start, end, replacement in edits if not replacement]
    # A comment introduces the rules up to the next comment or the end of its
    # block. It goes when rules followed it and every one of them went.
    blanked = blank_comments(css)
    comments = list(re.finditer(r"/\*.*?\*/", css, re.S))
    starts = {comment.start() for comment in comments}
    for comment in comments:
        span_end = _span_end(blanked, starts, comment.end())
        rest = blanked[comment.end() : span_end]
        covered = [
            (start, end)
            for start, end in removed
            if comment.end() <= start and end <= span_end
        ]
        for start, end in reversed(covered):
            rest = rest[: start - comment.end()] + rest[end - comment.end() :]
        if covered and not rest.strip():
            edits.append((comment.start(), comment.end(), ""))
    out = css
    for start, end, replacement in sorted(edits, reverse=True):
        if not replacement:
            start, end = _whole_lines(out, start, end)
        out = out[:start] + replacement + out[end:]
    return re.sub(r"\n{3,}", "\n\n", out)


def _whole_lines(text: str, start: int, end: int) -> tuple[int, int]:
    """A removed span widened over its indentation and its line break, when
    nothing else shares its lines."""
    line_start = start
    while line_start > 0 and text[line_start - 1] in " \t":
        line_start -= 1
    line_end = end
    while line_end < len(text) and text[line_end] in " \t":
        line_end += 1
    alone = (line_start == 0 or text[line_start - 1] == "\n") and (
        line_end == len(text) or text[line_end] == "\n"
    )
    return (line_start, min(line_end + 1, len(text))) if alone else (start, end)


def _span_end(blanked: str, comment_starts: set[int], position: int) -> int:
    """Where the rules a comment introduces stop: at the next comment beside
    it, or at the end of the block they share."""
    depth = 0
    for index in range(position, len(blanked)):
        if depth == 0 and index in comment_starts:
            return index
        if blanked[index] == "{":
            depth += 1
        elif blanked[index] == "}":
            if depth == 0:
                return index
            depth -= 1
    return len(blanked)


def read_keep(root: Path) -> dict[str, dict[str, str]]:
    path = root / KEEP
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("packages", nargs="*", help="dashboard, site (default: both)")
    parser.add_argument("--prune", action="store_true", help="remove the dead rules")
    parser.add_argument(
        "--dead",
        default="",
        metavar="NAME,NAME",
        help="names judged unused by hand although the word appears in the sources (a variable, a comment)",
    )
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parent.parent
    )
    arguments = parser.parse_args()
    keep = read_keep(arguments.root)
    failed = False
    for package in PACKAGES:
        if arguments.packages and package.name not in arguments.packages:
            continue
        dead = frozenset(filter(None, arguments.dead.split(",")))
        usage = usage_of(arguments.root, package, keep.get(package.name, {}), dead)
        sheets = sheets_of(arguments.root, package)
        report = survey(sheets, usage)
        print(
            f"{package.name}: {len(report.unused)} unused names, {report.dead_rules} dead rules"
        )
        for name, places in sorted(report.unused.items()):
            where = ", ".join(
                f"{path.relative_to(arguments.root)}:{line}"
                for path, line in places[:4]
            )
            print(f"  {name}  {where}{' ...' if len(places) > 4 else ''}")
        if report.families:
            print("  built at run time, check each family's members by hand:")
            for family, names in sorted(report.families.items()):
                print(f"    {family}  {' '.join(sorted(names))}")
        failed = failed or bool(report.unused)
        if arguments.prune:
            for path in sorted({sheet.path for sheet in sheets}):
                text = path.read_text(encoding="utf-8")
                for sheet in sorted(
                    (s for s in sheets if s.path == path), key=lambda s: -s.offset
                ):
                    pruned = prune(sheet.css, usage)
                    text = (
                        text[: sheet.offset]
                        + pruned
                        + text[sheet.offset + len(sheet.css) :]
                    )
                path.write_text(text, encoding="utf-8")
    return 1 if failed and not arguments.prune else 0


if __name__ == "__main__":
    sys.exit(main())
