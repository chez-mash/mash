#!/usr/bin/env python3
"""Select the dashboard browser specs a change can affect.

The specs in `dashboard/e2e` drive the running dashboard and import none of
its code, so no tool can tell from a spec which components it exercises.
`dashboard/e2e/affected.json` states it instead, and is reviewed by hand: the
component each route mounts, and the routes each spec visits. This module
joins that statement to the import graph of `dashboard/src`:

- a changed spec selects itself;
- a changed source file selects every spec that visits a route whose
  component imports the file, directly or through other files;
- a change to a shared foundation, to the shell every page is drawn in, or to
  a file this module cannot place selects every spec, and says why.

The selection is for the edit loop. It is wrong wherever the mapping has
fallen behind the specs, so CI always runs the whole suite; see "Fast loops"
in docs/internal/verification.md.

    tools/affected_tests.py                   # changes since origin/main
    tools/affected_tests.py --base HEAD~1
    tools/affected_tests.py dashboard/src/components/MemoryRow.svelte
    tools/affected_tests.py --run -- --reporter=line
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath

DASHBOARD = "dashboard"
MAPPING = "e2e/affected.json"
SPEC_GLOB = "e2e/*.spec.ts"
SOURCE_SUFFIXES = (".svelte", ".ts", ".css")
# The order a bundler tries when an import names no extension. `./x.js` may
# name `x.ts`: TypeScript's bundler resolution allows the compiled name.
RESOLVE_SUFFIXES = ("", ".ts", ".svelte.ts", ".svelte", ".js", ".css", "/index.ts", "/index.js")

# Static `import … from "x"`, `import "x"`, `export … from "x"` and dynamic
# `import("x")`. The clause before `from` may span lines, but holds no quote
# or semicolon, which keeps one match from running into the next statement.
SCRIPT_IMPORT = re.compile(
    r"""(?:^|[\s;}>])(?:import|export)\s+(?:type\s+)?(?:[^'";]*?\s+from\s+)?["']([^"']+)["']"""
    r"""|\bimport\(\s*["']([^"']+)["']\s*\)"""
)
STYLE_IMPORT = re.compile(r"""@import\s+(?:url\(\s*)?["']([^"']+)["']""")


class MappingError(ValueError):
    """`affected.json` is missing, malformed or names something that is not there."""


@dataclass(frozen=True)
class Coverage:
    """The reviewed statement in `affected.json`, paths relative to `dashboard/`."""

    everything: tuple[str, ...]
    ignored: tuple[str, ...]
    router: str
    shell: tuple[str, ...]
    routes: Mapping[str, tuple[str, ...]]
    specs: Mapping[str, tuple[str, ...]]

    def entries(self, spec: str) -> tuple[str, ...]:
        """The components the routes a spec visits mount."""
        return tuple(entry for route in self.specs[spec] for entry in self.routes[route])

    def route_entries(self) -> frozenset[str]:
        return frozenset(entry for entries in self.routes.values() for entry in entries)


@dataclass(frozen=True)
class Graph:
    """What each source file imports, and the local imports that name no file."""

    imports: Mapping[str, frozenset[str]]
    unresolved: Mapping[str, tuple[str, ...]]

    def reachable(self, roots: Iterable[str]) -> frozenset[str]:
        """The roots and every file they import, directly or through others."""
        seen: set[str] = set()
        pending = [root for root in roots if root in self.imports]
        while pending:
            current = pending.pop()
            if current in seen:
                continue
            seen.add(current)
            pending.extend(self.imports[current] - seen)
        return frozenset(seen)

    def without(self, importer: str, targets: frozenset[str]) -> Graph:
        """The same graph with `importer` no longer importing `targets`."""
        kept = self.imports.get(importer, frozenset()) - targets
        return Graph(imports={**self.imports, importer: kept}, unresolved=self.unresolved)


@dataclass
class Selection:
    """The specs to run and the reason for each decision, in the order found."""

    everything: bool = False
    specs: set[str] = field(default_factory=set)
    reasons: list[str] = field(default_factory=list)

    def chosen(self, all_specs: Sequence[str]) -> list[str]:
        return list(all_specs) if self.everything else sorted(self.specs)


def load_mapping(root: Path) -> Coverage:
    path = root / MAPPING
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise MappingError(f"{path}: {error}") from error
    try:
        routes = {name: tuple(entries) for name, entries in raw["routes"].items()}
        specs = {name: tuple(visited) for name, visited in raw["specs"].items()}
        mapping = Coverage(
            everything=tuple(raw["everything"]),
            ignored=tuple(raw["ignored"]),
            router=raw["router"],
            shell=tuple(raw["shell"]),
            routes=routes,
            specs=specs,
        )
    except (KeyError, TypeError, AttributeError) as error:
        raise MappingError(f"{path}: expected everything, ignored, router, shell, routes and specs") from error
    for spec, visited in specs.items():
        for route in visited:
            if route not in routes:
                raise MappingError(f"{path}: {spec} visits {route!r}, which is not under routes")
    return mapping


def specifiers(path: PurePosixPath, text: str) -> list[str]:
    """Every module a source file names, in the forms the dashboard uses."""
    found = [match.group(1) for match in STYLE_IMPORT.finditer(text)]
    if path.suffix != ".css":
        found += [first or second for first, second in SCRIPT_IMPORT.findall(text)]
    return found


def resolve(importer: PurePosixPath, specifier: str, files: frozenset[str]) -> str | None:
    """The source file a specifier names; None for a package or a missing file."""
    target = specifier.split("?", 1)[0]
    if target.startswith("$lib/") or target == "$lib":
        base = PurePosixPath("src/lib") / target[5:]
    elif target.startswith("."):
        base = importer.parent / target
    else:
        return None
    normal = os.path.normpath(base.as_posix())
    stem = normal[:-3] if normal.endswith(".js") else None
    for candidate in [normal + suffix for suffix in RESOLVE_SUFFIXES] + (
        [stem + ".ts", stem + ".svelte.ts"] if stem else []
    ):
        if candidate in files:
            return candidate
    return None


def is_local(specifier: str) -> bool:
    return specifier.startswith((".", "$lib"))


def build_graph(root: Path) -> Graph:
    """Read every source file under `src` and `e2e` once and resolve its imports."""
    paths = sorted(
        path.relative_to(root).as_posix()
        for directory in ("src", "e2e")
        for path in (root / directory).rglob("*")
        if path.is_file()
    )
    files = frozenset(paths)
    imports: dict[str, frozenset[str]] = {}
    unresolved: dict[str, tuple[str, ...]] = {}
    for name in paths:
        if not name.endswith(SOURCE_SUFFIXES):
            imports[name] = frozenset()
            continue
        path = PurePosixPath(name)
        text = (root / name).read_text(encoding="utf-8")
        resolved: set[str] = set()
        missing: list[str] = []
        for specifier in specifiers(path, text):
            target = resolve(path, specifier, files)
            if target is not None:
                resolved.add(target)
            elif is_local(specifier):
                missing.append(specifier)
        imports[name] = frozenset(resolved)
        if missing:
            unresolved[name] = tuple(missing)
    return Graph(imports=imports, unresolved=unresolved)


def matches(path: str, patterns: Iterable[str]) -> str | None:
    """The first pattern a path matches. `**` and `*` both cross directories."""
    return next((pattern for pattern in patterns if fnmatch.fnmatchcase(path, pattern)), None)


def spec_files(root: Path) -> list[str]:
    return sorted(path.relative_to(root).as_posix() for path in root.glob(SPEC_GLOB))


def select(changed: Iterable[str], mapping: Coverage, graph: Graph, all_specs: Sequence[str]) -> Selection:
    """Decide which specs the changed files, relative to `dashboard/`, can affect.

    Every branch that cannot name the affected specs selects all of them: a
    skipped spec is the only wrong answer this function can give.
    """
    selection = Selection()

    def everything(path: str, why: str) -> None:
        selection.everything = True
        selection.reasons.append(f"{path}: {why}; selects every spec")

    # The router imports every route's component in order to mount one of
    # them. Leaving those imports out keeps a page's own files out of the
    # shell, unless the shell also reaches them some other way.
    shell = graph.without(mapping.router, mapping.route_entries()).reachable(mapping.shell)
    reach = {spec: graph.reachable(mapping.entries(spec)) for spec in mapping.specs}
    if graph.unresolved:
        name, missing = next(iter(sorted(graph.unresolved.items())))
        everything(name, f"imports {missing[0]!r}, which resolves to no file, so the graph is incomplete")
    for spec in all_specs:
        if PurePosixPath(spec).name not in mapping.specs:
            everything(spec, f"is not in {MAPPING}")
    for path in sorted(set(changed)):
        name = PurePosixPath(path).name
        if path in all_specs and name in mapping.specs:
            selection.specs.add(path)
            selection.reasons.append(f"{path}: a changed spec selects itself")
        elif pattern := matches(path, mapping.ignored):
            selection.reasons.append(f"{path}: ignored ({pattern}); no browser spec depends on it")
        elif pattern := matches(path, mapping.everything):
            everything(path, f"shared foundation ({pattern})")
        elif path in shell:
            everything(path, "part of the shell every page is drawn in")
        elif path in graph.imports and path.startswith("src/"):
            users = sorted(spec for spec, files in reach.items() if path in files)
            if not users:
                everything(path, "no mapped route imports it")
                continue
            selection.specs.update(f"e2e/{spec}" for spec in users)
            selection.reasons.append(f"{path}: imported by the routes of {', '.join(users)}")
        else:
            everything(path, "not a file this tool can place")
    return selection


def changed_files(repository: Path, base: str) -> list[str]:
    """Paths changed since the branch left `base`, uncommitted and untracked work included."""

    def git(*arguments: str) -> list[str]:
        result = subprocess.run(
            ["git", "-C", str(repository), *arguments], check=True, capture_output=True, text=True
        )
        return [line for line in result.stdout.splitlines() if line]

    fork = git("merge-base", base, "HEAD")[0]
    return sorted({*git("diff", "--name-only", fork), *git("ls-files", "--others", "--exclude-standard")})


def within_dashboard(paths: Sequence[str]) -> tuple[list[str], int]:
    """Paths under `dashboard/`, made relative to it, and how many lay elsewhere."""
    prefix = DASHBOARD + "/"
    inside = [path.removeprefix(prefix) for path in paths if path.startswith(prefix)]
    return inside, len(paths) - len(inside)


def report(selection: Selection, chosen: Sequence[str], total: int, outside: int) -> str:
    lines = list(selection.reasons)
    if outside:
        lines.append(f"{outside} changed file(s) outside dashboard/ do not affect its browser specs")
    lines.append(f"Selected {len(chosen)} of {total} specs" + (" (the whole suite)" if selection.everything else ""))
    return "\n".join(lines)


def from_root(named: str, repository: Path, directory: Path) -> str:
    """A path given on the command line, relative to the repository root.

    `pnpm e2e:affected` runs in `dashboard/`, where a path is most naturally
    typed from there, so a name that is not a file from the root is read from
    the current directory.
    """
    if (repository / named).exists():
        return PurePosixPath(named).as_posix()
    return Path(os.path.relpath(directory / named, repository)).as_posix()


def main(arguments: Sequence[str] | None = None) -> int:
    given = list(sys.argv[1:] if arguments is None else arguments)
    # Everything after `--` is Playwright's; argparse would read it as files.
    split = given.index("--") if "--" in given else len(given)
    parser = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    parser.add_argument("files", nargs="*", help="changed files (default: ask git what changed since --base)")
    parser.add_argument("--base", default="origin/main", help="revision to compare with (default: origin/main)")
    parser.add_argument("--json", action="store_true", help="print the selection as JSON")
    parser.add_argument(
        "--run", action="store_true", help="run the selected specs with Playwright; its options follow --"
    )
    known = parser.parse_args(given[:split])
    playwright_arguments = given[split + 1 :]

    repository = Path(__file__).resolve().parent.parent
    root = repository / DASHBOARD
    try:
        mapping = load_mapping(root)
        paths = [from_root(named, repository, Path.cwd()) for named in known.files] or changed_files(
            repository, known.base
        )
    except (MappingError, subprocess.CalledProcessError) as error:
        detail = error.stderr.strip() if isinstance(error, subprocess.CalledProcessError) else str(error)
        print(f"affected-tests: {detail}", file=sys.stderr)
        return 2
    inside, outside = within_dashboard(paths)
    all_specs = spec_files(root)
    selection = select(inside, mapping, build_graph(root), all_specs)
    chosen = selection.chosen(all_specs)
    summary = report(selection, chosen, len(all_specs), outside)

    if known.json:
        print(json.dumps({"everything": selection.everything, "specs": chosen, "reasons": selection.reasons}))
        return 0
    if not known.run:
        print(summary, file=sys.stderr)
        print("\n".join(chosen))
        return 0
    print(summary, flush=True)
    if not chosen:
        print("No browser spec to run.")
        return 0
    targets = [] if selection.everything else chosen
    return subprocess.run(["pnpm", "exec", "playwright", "test", *targets, *playwright_arguments], cwd=root).returncode


if __name__ == "__main__":
    sys.exit(main())
