#!/bin/sh
# Makes the copy of the Raycast extension that goes to raycast/extensions:
# the extension without what only this repository needs (the tests, vitest
# and its test script, and the contributor notes), with its own lockfile.
# Usage: packaging/raycast-store.sh DESTINATION
set -eu
dest=${1:?usage: packaging/raycast-store.sh DESTINATION}
here=$(cd "$(dirname "$0")/.." && pwd)
if [ -e "$dest" ] && [ -n "$(ls -A "$dest")" ]; then
  echo "$dest is not empty" >&2
  exit 1
fi
mkdir -p "$dest"
cd "$here/raycast"
git ls-files | grep -v -e '\.test\.ts$' -e '^CONTRIBUTING\.md$' | while read -r file; do
  mkdir -p "$dest/$(dirname "$file")"
  cp "$file" "$dest/$file"
done
cd "$dest"
node -e '
const fs = require("fs");
const manifest = JSON.parse(fs.readFileSync("package.json", "utf8"));
delete manifest.scripts.test;
delete manifest.devDependencies.vitest;
fs.writeFileSync("package.json", JSON.stringify(manifest, null, 2) + "\n");
'
npm install --package-lock-only --ignore-scripts --no-audit --no-fund >/dev/null
echo "Store copy ready in $dest"
