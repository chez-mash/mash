"""Growth reviews and headroom: how close source files are to the line limit.

A hard limit alone invites a file to creep up to one line under it and then be
trimmed just enough to pass. Two measures, taken from lockdoc's organization
check, make the approach to the limit visible and deliberate instead:

- A growth review. A file that grows to `REVIEW_AT` lines or more must carry a
  written review in `tools/file-reviews/<path>.json`: what the file is
  responsible for, the split that was considered, what was decided, and the
  evidence for it. The review records a hash of the file, so it covers that
  version only; the next growth needs it reconsidered. Growth is measured
  against a base revision, so a file that stays the same length or shrinks
  never needs one. The expected answer to reaching the threshold is a split
  into parts near half the limit; a review is for a file that is one cohesive
  responsibility and would read worse split.
- Headroom, reported but never enforced: each file costs -ln(1 - lines/limit),
  which is small for a short file and climbs steeply toward the limit (about
  2.3 at 450 of 500 lines). The total, against the base, shows whether a
  change spent the repository's room or made some.
"""

from __future__ import annotations

import hashlib
import json
import math
import subprocess
from pathlib import Path

REVIEW_AT = 450
REVIEWS = Path("tools/file-reviews")
REVIEW_FIELDS = ("responsibility", "alternative", "decision", "evidence")


def barrier(lines: int, limit: int) -> float:
    """The headroom cost of one file: -ln(1 - lines/limit), infinite at the limit."""
    used = lines / limit
    return math.inf if used >= 1 else -math.log1p(-used)


def headroom(measured: dict[str, int], limit: int, debt: dict[str, int]) -> float:
    """The total headroom cost of the files within the limit. Files recorded as
    debt above the limit have no room to measure and are left out."""
    return sum(
        barrier(lines, limit)
        for path, lines in measured.items()
        if path not in debt and lines < limit
    )


def grown(
    measured: dict[str, int], before: dict[str, int], debt: dict[str, int]
) -> list[str]:
    """Files that grew to `REVIEW_AT` lines or more since the base. A file new
    since the base grew from nothing. Debt files have a stricter rule of their
    own: they may not grow at all."""
    return sorted(
        path
        for path, lines in measured.items()
        if path not in debt and lines >= REVIEW_AT and lines > before.get(path, 0)
    )


def review_path(path: str) -> Path:
    return REVIEWS / f"{path}.json"


def digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def review_problem(text: str, review: str | None) -> str | None:
    """Why the review of a grown file does not cover it, or None when it does."""
    if review is None:
        return "has no growth review"
    try:
        fields = json.loads(review)
    except json.JSONDecodeError:
        return "has a growth review that is not JSON"
    if not isinstance(fields, dict):
        return "has a growth review that is not a JSON object"
    if fields.get("source_sha256") != digest(text):
        return "changed since its growth review"
    empty = [name for name in REVIEW_FIELDS if not str(fields.get(name, "")).strip()]
    if empty:
        return f"has a growth review without {', '.join(empty)}"
    return None


def template(path: str, text: str, lines: int) -> str:
    """A review to fill in for `path` as it is now."""
    fields = {
        "source_sha256": digest(text),
        "lines": lines,
        "responsibility": "",
        "alternative": "",
        "decision": "",
        "evidence": "",
    }
    return json.dumps(fields, indent=2) + "\n"


def base_texts(root: Path, revision: str, paths: list[str]) -> dict[str, str] | None:
    """The text of each of `paths` at `revision`, without those it did not
    have; None when the revision cannot be read, as on a new branch's first
    push."""
    probe = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"{revision}^{{commit}}"],
        cwd=root,
        capture_output=True,
    )
    if probe.returncode != 0:
        return None
    texts: dict[str, str] = {}
    for path in paths:
        shown = subprocess.run(
            ["git", "show", f"{revision}:{path}"], cwd=root, capture_output=True
        )
        if shown.returncode == 0:
            texts[path] = shown.stdout.decode("utf-8", errors="replace")
    return texts
