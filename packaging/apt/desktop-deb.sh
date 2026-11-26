#!/usr/bin/env bash
# Turn the desktop app's .deb, as Tauri builds it, into the mash-desktop
# package of Mash's apt repository.
#
#   packaging/apt/desktop-deb.sh VERSION BUILT.deb OUTPUT_DIRECTORY
#
# Tauri's package is named after the product (`mash`, which in Debian and
# Ubuntu is an unrelated package) and carries its own copy of the engine as
# /usr/bin/mash, the file mash-cli installs. Two packages must not ship one
# file, and two copies could be two versions, so this package drops its copy
# and depends on mash-cli at exactly its own version: the app runs
# /usr/bin/mash from mash-cli, `apt upgrade` moves both together, and the
# `mash` command is the app's engine by construction. The app does not update
# itself when installed this way; apt does.
#
# Prints the path of the package it wrote.
set -euo pipefail
[[ $# == 3 ]] || { echo 'usage: desktop-deb.sh VERSION BUILT.deb OUTPUT_DIRECTORY' >&2; exit 2; }
version=${1#v}
# As in build-repo.sh: a pre-release sorts before its release only with a tilde.
version=${version/-/\~}
built=$(realpath "$2")
mkdir -p "$3"
output=$(realpath "$3")

scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
dpkg-deb --raw-extract "$built" "$scratch/root"
control=$scratch/root/DEBIAN/control
arch=$(sed -n 's/^Architecture: //p' "$control")
[[ $arch == amd64 || $arch == arm64 ]] || { echo "desktop-deb: unexpected architecture '$arch'" >&2; exit 1; }
[[ -x $scratch/root/usr/bin/mash-desktop ]] || { echo 'desktop-deb: the package has no /usr/bin/mash-desktop' >&2; exit 1; }
rm -f "$scratch/root/usr/bin/mash"

depends=$(sed -n 's/^Depends: //p' "$control")
installed=$(du -sk --exclude=DEBIAN "$scratch/root" | cut -f1)
python3 - "$control" "$version" "$depends" "$installed" <<'PYTHON'
import sys
from pathlib import Path

control, version, depends, installed = sys.argv[1:5]
replaced = {
    "Package": "mash-desktop",
    "Version": version,
    "Depends": ", ".join(filter(None, [f"mash-cli (= {version})", depends])),
    "Installed-Size": installed,
}
lines = []
seen = set()
for line in Path(control).read_text(encoding="utf-8").splitlines():
    field = line.split(":", 1)[0] if line[:1] not in (" ", "\t") else None
    if field in replaced:
        lines.append(f"{field}: {replaced[field]}")
        seen.add(field)
    else:
        lines.append(line)
for field, value in replaced.items():
    if field not in seen:
        lines.insert(1, f"{field}: {value}")
Path(control).write_text("\n".join(lines) + "\n", encoding="utf-8")
PYTHON
# md5sums lists the removed engine; rebuild it from what is left.
(cd "$scratch/root" && find . -type f ! -path './DEBIAN/*' -printf '%P\0' | sort -z | xargs -0 md5sum > DEBIAN/md5sums)

deb="$output/mash-desktop_${version}_${arch}.deb"
dpkg-deb --root-owner-group --build "$scratch/root" "$deb" >/dev/null
echo "$deb"
