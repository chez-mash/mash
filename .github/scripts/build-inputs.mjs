// Lets a cached target directory survive a fresh checkout.
//
// Cargo decides whether one of this repository's crates is up to date by
// comparing the modification times of its source files with the time it was
// last compiled. A checkout stamps every file with the current time, so every
// crate looks changed and is compiled again, however little the commit
// touched. `save` records each build input's content hash and time beside the
// compiled crates; `restore`, run on a later checkout before the build, gives
// a file its recorded time back when its content is the same. A file that
// changed, is new, or is not in the record keeps the time the checkout gave
// it, which is newer than anything in the cache, so Cargo compiles what
// depends on it.
//
//   node build-inputs.mjs save RECORD      after a build
//   node build-inputs.mjs restore RECORD   before one; a missing record is not an error
//
// The inputs are the files Git tracks and the built dashboard
// (dashboard/dist), which crates/mash-dash/build.rs embeds and watches as a
// directory tree, so its directories are recorded too. Times are kept in whole
// seconds, rounded down: an older source file is never mistaken for a change,
// and the hash, not the time, is what says a file is the same.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, utimesSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

const GENERATED = ['dashboard/dist'];

const [action, record] = process.argv.slice(2);
if (!['save', 'restore'].includes(action) || !record) {
  console.error('usage: build-inputs.mjs save|restore RECORD');
  process.exit(2);
}

function digest(path, directory) {
  const hash = createHash('sha256');
  // A directory is the same while it lists the same names.
  hash.update(directory ? readdirSync(path).sort().join('\0') : readFileSync(path));
  return hash.digest('hex');
}

function walk(root, found) {
  if (!existsSync(root)) return;
  found.push([root, true]);
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = `${root}/${entry.name}`;
    if (entry.isDirectory()) walk(path, found);
    else if (entry.isFile()) found.push([path, false]);
  }
}

function inputs() {
  // The Linux builds run as root in a container on a checkout the runner's
  // user owns, which Git refuses to read unless told the directory is safe.
  const tracked = execFileSync('git', ['-c', 'safe.directory=*', 'ls-files', '-z'], { encoding: 'utf8', maxBuffer: 1 << 28 })
    .split('\0')
    .filter(Boolean);
  const found = [];
  for (const path of tracked) {
    // A deleted file, a submodule or a symbolic link has no content of its own.
    const stat = lstatSync(path, { throwIfNoEntry: false });
    if (stat?.isFile()) found.push([path, false]);
  }
  for (const root of GENERATED) walk(root, found);
  return found;
}

if (action === 'save') {
  const files = {};
  for (const [path, directory] of inputs()) {
    files[path] = [digest(path, directory), Math.floor(lstatSync(path).mtimeMs / 1000)];
  }
  mkdirSync(dirname(record), { recursive: true });
  writeFileSync(record, JSON.stringify({ version: 1, files }));
  console.log(`Recorded ${Object.keys(files).length} build inputs in ${record}.`);
} else {
  if (!existsSync(record)) {
    console.log(`No ${record}: nothing was restored, so every crate of this repository compiles.`);
    process.exit(0);
  }
  const { version, files } = JSON.parse(readFileSync(record, 'utf8'));
  if (version !== 1) {
    console.log(`${record} is version ${version}, which this script does not read; nothing restored.`);
    process.exit(0);
  }
  const changed = [];
  let same = 0;
  // Deepest paths first: a directory's time is set after its entries'.
  const current = inputs().sort(([a], [b]) => b.length - a.length);
  for (const [path, directory] of current) {
    const recorded = files[path];
    if (recorded && recorded[0] === digest(path, directory)) {
      utimesSync(path, recorded[1], recorded[1]);
      same += 1;
    } else {
      changed.push(path);
    }
  }
  console.log(`${same} build inputs are as they were when the cache was made; ${changed.length} changed or new.`);
  for (const path of changed.slice(0, 200)) console.log(`  ${path}`);
  if (changed.length > 200) console.log(`  ... and ${changed.length - 200} more`);
}
