#!/usr/bin/env python3
"""Keep the hand-written global stylesheets from growing.

`css-budget.json` records how many lines each global stylesheet of the
dashboard and the website has. A file may shrink but not grow, and a new
global stylesheet is refused: new styling belongs in Tailwind utilities or in
a component's scoped `<style>`, using the theme's token roles. When a file
shrinks its number must be lowered, so the budget only ever tightens.

    python3 tools/check_css_budget.py            check
    python3 tools/check_css_budget.py --update   lower the numbers that shrank
    python3 tools/check_css_budget.py --update --allow-growth "why this must be global CSS"
                                                 raise them, recording the reason
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

BUDGET = Path("tools/css-budget.json")
ADVICE = (
    "Put new styling in Tailwind utility classes or in the component's scoped "
    "<style>, using token roles only; the global stylesheets only shrink "
    "(docs/internal/development.md, Global CSS budget)."
)


@dataclass(frozen=True)
class Finding:
    path: str
    message: str
    grows: bool


def measure(root: Path, budget: dict) -> dict[str, int]:
    """Line counts of every stylesheet the budget watches and does not exempt."""
    exempt = budget.get("exempt", {})
    found = {}
    for pattern in budget["watched"]:
        for path in sorted(root.glob(pattern)):
            name = path.relative_to(root).as_posix()
            if path.is_file() and name not in exempt:
                found[name] = len(path.read_text(encoding="utf-8").splitlines())
    return found


def check(measured: dict[str, int], recorded: dict[str, int]) -> list[Finding]:
    findings = []
    for path, lines in sorted(measured.items()):
        allowed = recorded.get(path)
        if allowed is None:
            findings.append(
                Finding(
                    path, f"{lines} lines in a new global stylesheet. {ADVICE}", True
                )
            )
        elif lines > allowed:
            findings.append(
                Finding(
                    path,
                    f"{lines} lines, {lines - allowed} over its budget of {allowed}. {ADVICE}",
                    True,
                )
            )
        elif lines < allowed:
            findings.append(
                Finding(
                    path,
                    f"{lines} lines, shrank from {allowed}; lower its budget with "
                    "python3 tools/check_css_budget.py --update",
                    False,
                )
            )
    for path in sorted(set(recorded) - set(measured)):
        findings.append(
            Finding(
                path,
                "is gone; drop its budget with python3 tools/check_css_budget.py --update",
                False,
            )
        )
    return findings


def updated(
    budget: dict, measured: dict[str, int], reason: str | None
) -> tuple[dict, list[str]]:
    """The budget set to the measured counts, and the paths whose growth
    stopped it. Growth is accepted only with a reason, which is recorded."""
    recorded = budget["files"]
    grown = [path for path, lines in measured.items() if lines > recorded.get(path, 0)]
    if grown and not reason:
        return budget, sorted(grown)
    growth = list(budget.get("growth", []))
    for path in sorted(grown):
        growth.append(
            {
                "file": path,
                "from": recorded.get(path, 0),
                "to": measured[path],
                "reason": reason,
            }
        )
    return {**budget, "files": dict(sorted(measured.items())), "growth": growth}, []


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument(
        "--update", action="store_true", help="lower the numbers that shrank"
    )
    parser.add_argument(
        "--allow-growth",
        metavar="REASON",
        help="with --update, also raise numbers, recording why in the budget file",
    )
    args = parser.parse_args(argv)
    if args.allow_growth is not None and not args.update:
        parser.error("--allow-growth goes with --update")
    if args.allow_growth is not None and len(args.allow_growth.strip()) < 20:
        parser.error("--allow-growth needs the reason, in a sentence")
    budget_file = args.root / BUDGET
    try:
        budget = json.loads(budget_file.read_text(encoding="utf-8"))
        measured = measure(args.root, budget)
    except (OSError, ValueError, KeyError) as error:
        print(f"css-budget: cannot read the budget: {error}", file=sys.stderr)
        return 2
    if args.update:
        reason = args.allow_growth.strip() if args.allow_growth else None
        new, refused = updated(budget, measured, reason)
        if refused:
            for path in refused:
                print(
                    f"FAIL: {path}: {measured[path]} lines, over its budget of "
                    f"{budget['files'].get(path, 0)}"
                )
            print(
                "FAIL: --update only lowers the budget. "
                + ADVICE
                + ' If the rule can only be global, record why: --update --allow-growth "REASON".'
            )
            return 1
        budget_file.write_text(json.dumps(new, indent=2) + "\n", encoding="utf-8")
        print(f"Updated {BUDGET}: {sum(new['files'].values())} lines of global CSS")
        return 0
    findings = check(measured, budget["files"])
    for finding in findings:
        print(f"FAIL: {finding.path}: {finding.message}")
    if findings:
        return 1
    print(
        f"PASS: {len(measured)} global stylesheets, {sum(measured.values())} lines, "
        "none over its budget"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
