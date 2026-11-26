#!/usr/bin/env bash
# Checks the copy packaging/raycast-store.sh makes for raycast/extensions: it
# keeps the extension and leaves out what only this repository needs, in the
# files, the manifest and the lockfile. The script resolves the copy's own
# lockfile with npm, so this needs the npm registry or a warm npm cache.
set -euo pipefail
cd "$(dirname "$0")/.."

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
copy=$work/mash

fail() {
	echo "FAIL $1" >&2
	exit 1
}

packaging/raycast-store.sh "$copy" >/dev/null

# What the Store builds from is there.
for kept in package.json package-lock.json README.md CHANGELOG.md eslint.config.js tsconfig.json src/search.tsx; do
	[[ -f $copy/$kept ]] || fail "the copy lacks $kept"
done

# No tests, test runner configuration, knip configuration or contributor notes.
extra=$(cd "$copy" && find . -type f \( -name '*.test.*' -o -name 'knip.json*' -o -name 'vitest.config.*' -o -name CONTRIBUTING.md \))
[[ -z $extra ]] || fail "the copy carries development-only files: $extra"

# Neither tool is a dependency, a script or a locked package.
node - "$copy" <<'NODE' || fail 'the copy still names development-only tooling'
const fs = require("fs");
const path = require("path");
const copy = process.argv[2];
const read = (name) => JSON.parse(fs.readFileSync(path.join(copy, name), "utf8"));
const manifest = read("package.json");
const lock = read("package-lock.json");
const tools = ["knip", "vitest"];
const problems = [];
for (const tool of tools) {
  for (const group of ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"]) {
    if (manifest[group]?.[tool]) problems.push(`package.json ${group} has ${tool}`);
    if (lock.packages[""][group]?.[tool]) problems.push(`package-lock.json root ${group} has ${tool}`);
  }
  for (const [name, command] of Object.entries(manifest.scripts)) {
    if (name === tool || new RegExp(`\\b${tool}\\b`).test(command)) problems.push(`package.json script ${name} uses ${tool}`);
  }
  for (const location of Object.keys(lock.packages)) {
    if (location === `node_modules/${tool}` || location.endsWith(`/node_modules/${tool}`)) {
      problems.push(`package-lock.json locks ${location}`);
    }
  }
}
// The scripts the Store's own checks run are still there.
for (const script of ["build", "lint"]) {
  if (!manifest.scripts[script]) problems.push(`package.json lost its ${script} script`);
}
for (const problem of problems) console.error(problem);
process.exit(problems.length === 0 ? 0 : 1);
NODE

# A destination with something in it is refused, untouched.
if packaging/raycast-store.sh "$copy" >/dev/null 2>&1; then
	fail 'a destination that is not empty was accepted'
fi

echo 'raycast-store: ok'
