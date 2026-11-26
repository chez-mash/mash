#!/usr/bin/env python3
"""What the desktop app's update channels publish.

`main-version VERSION BUILD` prints the version of a build of the main
branch: a prerelease of the release that will follow VERSION, numbered by
BUILD. After release 0.2.4, build 118 is `0.2.5-main.118`, which by
semantic-version order is newer than 0.2.4, older than build 119 and older
than 0.2.5 (or any later release), so an app on either channel always moves
forward. While VERSION is itself a prerelease (`0.3.0-alpha`), the build is
`0.3.0-alpha.main.118`, which sorts after it and before `0.3.0-beta`.

`manifest` writes `latest.json`, the file the app's updater reads: the
version and, for each platform, the address of its update file and the text
of that file's signature. Each platform is named `TARGET=FILE`; the
signature is read from `FILE.sig` beside it, and a platform whose file or
signature is missing is left out, so a build that signed nothing publishes
no manifest (exit status 3) instead of one the app would refuse.

Dependency-free and written for Python 3.9 on: it runs on the release and
desktop workflow runners, whatever their `python3` is.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections.abc import Sequence
from datetime import datetime, timezone
from pathlib import Path

VERSION = re.compile(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$")
TARGETS = (
    "darwin-aarch64",
    "darwin-x86_64",
    "linux-aarch64",
    "linux-x86_64",
    "windows-aarch64",
    "windows-x86_64",
)


def main_version(version: str, build: int) -> str:
    """The version of build number `build` of main, after release `version`."""
    match = VERSION.match(version)
    if match is None:
        raise ValueError(f"{version!r} is not a version like 1.2.3 or 1.2.3-alpha")
    if build < 1:
        raise ValueError("the build number starts at 1")
    major, minor, patch, prerelease = match.groups()
    if prerelease:
        return f"{major}.{minor}.{patch}-{prerelease}.main.{build}"
    return f"{major}.{minor}.{int(patch) + 1}-main.{build}"


def manifest(
    version: str,
    base_url: str,
    files: dict[str, Path],
    notes: str = "",
    published: datetime | None = None,
) -> dict[str, object]:
    """The updater manifest for the platforms whose file and signature exist."""
    if VERSION.match(version) is None:
        raise ValueError(f"{version!r} is not a version")
    platforms: dict[str, dict[str, str]] = {}
    for target, file in sorted(files.items()):
        if target not in TARGETS:
            raise ValueError(
                f"unknown platform {target!r}; expected one of {', '.join(TARGETS)}"
            )
        signature = file.with_name(file.name + ".sig")
        if not file.is_file() or not signature.is_file():
            continue
        platforms[target] = {
            "signature": signature.read_text(encoding="utf-8").strip(),
            "url": f"{base_url.rstrip('/')}/{file.name}",
        }
    return {
        "version": version,
        "notes": notes,
        "pub_date": (published or datetime.now(timezone.utc)).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        ),
        "platforms": platforms,
    }


def parse(argv: Sequence[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    main = commands.add_parser("main-version", help="the version of a build of main")
    main.add_argument("version")
    main.add_argument("build", type=int)
    write = commands.add_parser("manifest", help="write latest.json")
    write.add_argument("--version", required=True)
    write.add_argument(
        "--base-url", required=True, help="where the update files are downloaded from"
    )
    write.add_argument("--notes", default="")
    write.add_argument("--output", type=Path, required=True)
    write.add_argument("files", nargs="+", metavar="TARGET=FILE")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parse(argv)
    try:
        if arguments.command == "main-version":
            print(main_version(arguments.version, arguments.build))
            return 0
        files = {}
        for entry in arguments.files:
            target, separator, file = entry.partition("=")
            if not separator:
                raise ValueError(f"{entry!r} is not TARGET=FILE")
            files[target] = Path(file)
        written = manifest(
            arguments.version, arguments.base_url, files, arguments.notes
        )
    except ValueError as error:
        print(f"desktop_updates: {error}", file=sys.stderr)
        return 2
    if not written["platforms"]:
        print(
            "desktop_updates: no signed update file was found; no manifest written",
            file=sys.stderr,
        )
        return 3
    arguments.output.write_text(json.dumps(written, indent=2) + "\n", encoding="utf-8")
    print(
        f"{arguments.output}: {arguments.version} for {', '.join(written['platforms'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
