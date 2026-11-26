"""Fill a packaging template for one release from its SHA256SUMS file.

    python3 packaging/render.py 0.8.0 dist/SHA256SUMS packaging/homebrew/mash.rb.in mash.rb

Templates name values as @VERSION@ and @SHA256:<archive>@ (lowercase hex) or
@SHA256_UPPER:<archive>@. Standard library only, for stock release runners.

The winget manifests are several packages of three files each, so one command
fills them all, each package into a directory named for its identifier:

    python3 packaging/render.py --winget 0.8.0 dist/SHA256SUMS winget-manifests
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

TOKEN = re.compile(r"@(VERSION|SHA256|SHA256_UPPER)(?::([\w.-]+))?@")


def render(template: str, version: str, sums: str) -> str:
    digests = {}
    for line in sums.splitlines():
        digest, _, name = line.strip().partition("  ")
        digests[name.lstrip("*")] = digest.lower()

    def value(match: re.Match[str]) -> str:
        kind, archive = match.group(1), match.group(2)
        if kind == "VERSION":
            return version
        if archive not in digests:
            raise SystemExit(f"SHA256SUMS lacks {archive}")
        digest = digests[archive]
        return digest.upper() if kind == "SHA256_UPPER" else digest

    return TOKEN.sub(value, template)


def render_winget(templates: Path, version: str, sums: str, output: Path) -> list[Path]:
    """Fill every package under `templates`; returns the manifests written.

    A package is a directory named for its identifier holding
    `<identifier>*.yaml.in`. A template that names another package is refused,
    because winget reads the identifier from the file and the path alike.
    """
    written = []
    for template in sorted(templates.glob("*/*.yaml.in")):
        identifier = template.parent.name
        text = render(template.read_text(), version, sums)
        if (
            not template.name.startswith(f"{identifier}.")
            or f"\nPackageIdentifier: {identifier}\n" not in text
        ):
            raise SystemExit(f"{template} is not a manifest of {identifier}")
        manifest = output / identifier / template.name.removesuffix(".in")
        manifest.parent.mkdir(parents=True, exist_ok=True)
        manifest.write_text(text)
        written.append(manifest)
    if not written:
        raise SystemExit(f"no winget templates under {templates}")
    return written


def main() -> int:
    if sys.argv[1:2] == ["--winget"]:
        version, sums, output = sys.argv[2:5]
        render_winget(
            Path(__file__).parent / "winget",
            version.removeprefix("v"),
            Path(sums).read_text(),
            Path(output),
        )
        return 0
    version, sums, template, output = sys.argv[1:5]
    text = render(
        Path(template).read_text(), version.removeprefix("v"), Path(sums).read_text()
    )
    Path(output).parent.mkdir(parents=True, exist_ok=True)
    Path(output).write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
