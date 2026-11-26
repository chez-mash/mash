# Development and verification

[Documentation](../README.md) · [Project home](../../README.md)

## Build and test

### Task runner

Tasks run through [just](https://just.systems). `just` alone lists every
recipe by group (develop, test, quality, deploy, release, design), and
`just --show RECIPE` prints what one runs. The recipes in the
[justfile](../../justfile) call the scripts in `tools/` and each package's own
scripts; they hold no logic of their own. Install just 1.59.0 or newer:

```bash
bash tools/install-just.sh                 # Linux: the pinned release into ~/.local/bin, no sudo
brew install just                          # macOS
scoop install just                         # Windows
winget install --id Casey.Just --exact     # Windows, without Scoop
```

`tools/install-just.sh` pins the version and the archive's SHA-256 and is what
CI runs. Arguments follow the recipe name (`just iterate 'test(name)'
--package mash-dash`, `just deploy xenia`, `just release --dry-run 0.2.5`).
The machine-specific settings are at the top of the justfile, with Xenia's
values as defaults: `DEV_TMPDIR`, `DEV_TARGET_DIR` and `DEV_RUN` (the
`agent-test` launcher where it is installed). Set one in the environment or
before the recipe: `DEV_TARGET_DIR=/somewhere just dev-build` and
`just DEV_TARGET_DIR=/somewhere dev-build` are the same, and `DEV_RUN=` runs
without the launcher.

The `Makefile` is a shim for the transition: each target it had runs the
recipe of the same name (`make iterate FOCUS=... PACKAGE=...` still works)
and says how to install just when it is missing. Add new tasks to the
justfile only. `tools/test-justfile.sh` dry-runs every recipe and checks the
shim.

### Dashboard browser checks

Run `pnpm run lint`, `pnpm run check`, `pnpm run test`,
`pnpm run test:browser`, and `pnpm run build` in `dashboard/`.
The default browser suite uses mocked API responses to exercise UI behavior.
It does not establish that a running dashboard can search its real index.
While editing, `pnpm test:changed` and `pnpm e2e:affected` run only the tests
a change can affect; see [Fast loops](verification.md#fast-loops).

Before handing over a preview, run the live browser checks against its actual
URL, including its network hostname and protocol. Choose a query that occurs
in that index:

```bash
MASH_DASHBOARD_URL=http://your-host:8790 MASH_DASHBOARD_QUERY=database pnpm run test:browser
```

This mode uses the deployed assets and real API without starting Vite or
mocking requests. It checks both Enter and button submission, successful
results, opening a matching transcript without a document reload, and browser
errors. The target needs current Qdrant views and available query models.
On Xenia, run these workloads through `agent-test` as for other tests.

### Unused front-end code

`pnpm knip` in `dashboard/` and `site/`, and `npm run knip` in `raycast/`, run
[Knip](https://knip.dev) and fail on a file nothing imports, an export or type
nothing uses, and a dependency nothing needs. CI runs them beside each
package's lint. Each package's configuration (`dashboard/knip.ts`,
`site/knip.jsonc`, `raycast/knip.json`) names the entry points Knip cannot
find by itself and gives the reason for every ignored dependency. When it
reports an export, remove the `export` if the file still uses the name and
the code itself if nothing does (`knip --fix --fix-type exports,types` does
the first). Add to an ignore list only for something Knip cannot see, such as
a file named by path in a configuration string, and say why beside it.

`python3 tools/unused_css.py` lists the class and id selectors the
stylesheets define and no source names, and `--prune` removes the rules left
dead; its header explains what it treats as used. Review its list before
pruning, and compare screenshots before and after.

### Storybook

Storybook (`dashboard/.storybook`, Storybook 10 for Svelte and Vite, stories
in Svelte CSF) is the reference for what the dashboard is made of: what
exists, in every state it has. The wireframe canvas is for exploring new
ideas; once an idea is built, its component's stories are where to look.

```bash
cd dashboard
pnpm storybook         # the dev server, on 127.0.0.1:6006 only
pnpm storybook:build   # the static site, to dashboard/storybook-static (ignored by Git)
```

What belongs in it:

| Section | What | Where its stories live |
|---|---|---|
| Foundations | Colour roles (every colour in `tokens.css`, read from the file), type, radii and shadows, the icon weight specimen and state rules, status tones, memory kind tints | `src/stories/foundations/` |
| Kit | Each control in `src/lib/components/ui`, in its states: on and off, disabled, open, focused | Beside the control: `ui/switch/switch.stories.svelte` |
| Blocks | The building blocks pages are made of, in their states | Beside the component: `components/ChatRow.stories.svelte` |
| Composed | The top bar, the rail, Mashy and the quick panel | `src/stories/composed/` |

Whole pages are not stories; the page pictures in `dashboard/visual` cover
those.

A story is drawn with exactly what the dashboard is drawn with: Storybook
uses the dashboard's Vite configuration, and its preview imports
`src/styles.ts`, the same list of stylesheets and typefaces `main.ts` imports.
The toolbar's theme switch sets `data-theme` on the document, as Settings
does, and the viewport menu has the two sizes the baselines use (1280 and
390). The accessibility panel is the a11y addon.

To add a story, write `NAME.stories.svelte` beside the component:

```svelte
<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import StatusBadge from "./StatusBadge.svelte";

const { Story } = defineMeta({
  title: "Blocks/StatusBadge",
  component: StatusBadge,
  args: { tone: "ok", words: "Running" },
});
</script>

<Story name="Default" />

<Story name="Tones" asChild>
  <Cases row>
    <Case label="ok"><StatusBadge tone="ok" words="Running" /></Case>
    <Case label="bad"><StatusBadge tone="bad" words="Failed" /></Case>
  </Cases>
</Story>
```

- **States are shown, not described.** One story per state worth looking at
  alone, and a sheet (`Cases` and `Case` from `src/stories`) that puts the
  states side by side under their names.
- **Made-up data only, never real chat history.** `src/stories/fixtures.ts`
  reads the demo's history (`src/demo`) through the dashboard's own parsers;
  make up what the demo lacks there, in the demo's world.
- **Components that read their own data** need nothing special: every story
  is wrapped in a query cache of its own, and every request is answered in
  the browser by the demo's transport (`src/stories/Frame.svelte`). A story
  that needs an answer the demo never gives sets `parameters.data`
  (`src/stories/answers.ts`): `answers` by route, `route` for where the
  router is, `before` for state kept outside the components. See
  `MachineCard.stories.svelte` and `composed/Mashy.stories.svelte`.
- **Layout is the story's own.** Use scoped `<style>` with token roles, as a
  component does. Tailwind does not scan stories (`@source not` in
  `app.css`), so a utility class that only a story uses does nothing, and
  the dashboard's stylesheet stays byte for byte what it was without them.
- **Tags say how a story is pictured**: `phone` at 390×844, `screen` for a
  subject placed on the screen (a dialog, a menu, the rail, Mashy). A state
  that needs a step first, such as typing, takes it in `play`.

`src/stories/coverage.test.ts` (part of `pnpm test`) fails when a control in
`src/lib/components/ui` or a building block on its list has no story. A new
building block is added to that list with its story. Every story is also
compared with a committed picture in light and dark; see "Visual baselines"
in `verification.md`. `pnpm knip` treats stories and `.storybook` as entry
points. CI builds the static Storybook in the Front ends workflow and keeps
it as the `dashboard-storybook` artifact for a week; it is not published.

To look at it on the tailnet:

```bash
tools/worktree.sh preview NAME --storybook   # a worktree's stories, live while you edit
tools/storybook-serve.sh                     # main's Storybook, built and kept served
```

`tools/storybook-serve.sh` (`just storybook-serve`) builds in a tree of its
own at `origin/main` (`../mash-worktrees/_storybook`, so a build never moves
the deploy tree under a running deploy), copies the static files to
`~/.local/state/mash/storybook/site` and serves them with the user unit
`mash-storybook.service` on this machine's Tailscale address at port 5400,
tailnet only. `tools/worktree.sh land NAME --wait` refreshes it once the
change is on `main`: it runs `storybook-serve.sh --refresh`, which builds
again only on a machine that has the unit, and only when `dashboard/` on
`main` differs from the revision served. A refresh that fails prints a note
and never fails the landing. After a change that landed another way, run
`just storybook-serve` by hand. The old files are served until the new build
finishes, and a failed build leaves them. `--status` says which revision is
served and `--stop` removes the unit.

### Linker

On x86-64 Linux, `just iterate`, `just validate-dev`, `just dev-build` and the
fleet's release builds (`tools/build-dev-linux.sh`) link with
[mold](https://github.com/rui314/mold) when it is installed
(`apt-get install mold`), and with Rust's default `rust-lld` otherwise.
Measured on Xenia on 2026-11-20, linking the debug `mash` binary took about
2 s with mold against 3 s with `rust-lld`. CI and the published releases
keep the default linker. Turning it on or off changes the link flags, so the next build
recompiles everything once.

### Provider snapshots

Each provider-format fixture in `fixtures/` (synthetic, never real history)
is read by its reader in a snapshot test, and the chat and messages it
yields are recorded as JSON with [insta](https://insta.rs): Claude and Codex
in `app/silver/tests/snapshots.rs`, Cursor in `app/silver/cursor`, and
Antigravity in `crates/mash-silver/src/antigravity`. Message IDs appear as
their position (`#0`, `#1`, ...), because some providers derive them from the
file's location. When a reader's output changes on purpose, rerun the test
with `INSTA_UPDATE=always` (or `cargo insta review`, after
`cargo install cargo-insta`) and review the `.snap` diff in the commit. A new
fixture gets a snapshot test beside the reader's other tests.

### File length

Handwritten production source files (Rust, TypeScript, Svelte, Python) stay
at or below 500 lines, the limit gohawk and lockdoc use.
`tools/check_file_lines.py` enforces it in `just iterate`, `just validate-dev`
and CI. Tests, generated files and a Rust file's trailing inline
`#[cfg(test)] mod` block are not counted. Files that were already longer are
listed in `tools/file-lines-baseline.txt` with their length then: they may
shrink but not grow. When one shrinks, run
`python3 tools/check_file_lines.py --update` to lower its entry; the check
fails until you do, so the list only gets shorter. Never add a file to it:
split the file instead.

Functions get the same treatment: suppress `clippy::too_many_lines` or
`clippy::too_many_arguments` with `#[expect(..., reason = "...")]`, never
`#[allow]`, so Clippy fails once a function no longer needs it.

### Global CSS budget

The hand-written global stylesheets only shrink. `tools/css-budget.json`
records the line count of each one: the dashboard's `terminal.css`, `kit.css`,
`semantics.css` and `components/reading.css`, and every file in
`site/src/styles/`. `python3 tools/check_css_budget.py` fails when one is
longer than its number, and when a new stylesheet appears beside them. It
runs in CI next to the line limit and in the pre-push check.

Put new styling in Tailwind utility classes on the element, or in the
component's scoped `<style>`, and take every colour, line, corner and shadow
from a token role. The dashboard's `app.css` and `theme/tokens.css` are
exempt, since a new role adds a line to each.

When a stylesheet shrinks, run `python3 tools/check_css_budget.py --update`
to lower its number; the check fails until you do, so the budget only
tightens. `--update` never raises a number. A rule that can only be global
(a print rule, a third-party element the site cannot put a class on) needs
`--update --allow-growth "REASON"`, which records the file, the change and
the reason under `growth` in the budget file for review.

### Rust checks

[Correctness checks](verification.md) describes generated Proptest
sequences against real SQLite, disposable-backend replay interruptions and
focused cargo-mutants audits. Property tests run with ordinary nextest tests;
mutation audits run explicitly with `just mutation-tools` and `just mutation-check`.
Bounded TLA+ checks for publication, tombstones and shared
view recovery live in [protocol/](../../protocol/README.md). Run `just protocol-tools`
once to install the pinned checker, then `just protocol-check`. These checks
include explicit expected counterexamples, including the historical replay
certification bug; they supplement implementation tests rather than certify
the Rust implementation.

Use Cranelift for development builds and tests. Install the nightly component
and use the Cargo aliases:

```bash
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
cargo build-clif
cargo run-clif -- status
cargo test-clif
```

These aliases select Cranelift for the workspace crates in the development profile. Dependencies
keep LLVM, so their existing build cache can be reused. `test-clif` requires
`cargo-nextest`. Use the usual commands below for standard LLVM verification
and `cargo build --release` for optimized builds. Cranelift is an experimental
nightly component and is available only on supported host platforms; see the
[Cranelift backend documentation](https://github.com/rust-lang/rustc_codegen_cranelift#usage).

```bash
cargo fmt --check
cargo nextest run
cargo clippy --all-targets -- -D warnings
cargo test --doc
```

Run unit tests with nextest, not `cargo test`. Nextest gives each test its own
process. Tracing caches, per process, whether anything listens to each trace
point, so tests that capture traces (such as `mash-server`'s correlation and
admission tests) lose events when another test in the same process has
switched those trace points off. Under `cargo test` they fail at random; under
nextest they pass.

The workspace is the executables' package `mash` (the repository root), their
command lines `mash-cli` (`cli/`), the application library `mash-app` (`app/`)
and the crates under `crates/`. These default commands cover all of them.

| Crate | Holds | Depends on |
| --- | --- | --- |
| `mash-query` | Pure query validation and literal matching | nothing of ours |
| `mash-fleet` | Fleet CLI and readiness probe | nothing of ours |
| `mash-model` | Vocabulary every layer shares: roles, providers, tool activity, attachments, credential masking, platform helpers, the typed error | nothing of ours |
| `mash-silver` | The provider-independent chat model and the native history adapters: discovery, parsing, reconciliation inputs. Never opens the index | `mash-model` |
| `mash-storage` | The silver SQLite index and its migrations, memory records, portable export, and the search filter, result and catalogue types the layers above share. Deploy tooling reads `crates/mash-storage/src/storage/schema.rs` by path | `mash-query`, `mash-model`, `mash-silver` |
| `mash-search` | Configuration, the embedding and reranking models, local and shared Qdrant views, ranking, tracing, and `blocking`. | `mash-model`, `mash-storage` |
| `mash-sync` | The shared index: the node-side client, sync and publication, shared retrieval, and the server's silver store and view synchronization. | `mash-model`, `mash-silver`, `mash-storage`, `mash-search` |
| `mash-app` (`app/`) | The application library: the indexing pipeline, the typed core the CLI, MCP and dashboard call, watching, runtime leases and service setup | `mash-model`, `mash-silver`, `mash-storage`, `mash-search`, `mash-sync` |
| `mash-server` | The authenticated HTTP service for the shared index: routes, authorization, the sync protocol's endpoints and service setup. Its store is in `mash-sync` (`shared::server`) | `mash-app`, `mash-model` |
| `mash-mcp` | The stdio MCP server, its tool contract, the dashboard's console and the generated history skill | `mash-app`, `mash-model`, `mash-silver` |
| `mash-dash` | The local web dashboard served by `mash dash`, and the build script that embeds its front end | `mash-app`, `mash-mcp`, `mash-fleet`, `mash-model`, `mash-silver` |
| `mash-cli` (`cli/`) | Both command lines: argument parsing and output | `mash-app`, `mash-mcp`, `mash-server`, `mash-dash`, `mash-model` |
| `mash` (the root; `build.rs`, `cli/main.rs`, `cli/server_main.rs`, `cli/tests/`) | Both executables, each a few lines over `mash-cli`, the commit stamped into them, the Windows delay-load hook, and the CLI contract tests | `mash-cli` |

The application library is being divided from the bottom up so that an edit recompiles only its
own crate and the ones above it; the measurements under
[The edit loop](#the-edit-loop) say why. The crates under `crates/` are
internal: an item is `pub` there so another workspace crate can use it, not as
a supported API, which is why they allow the Clippy lints that ask for
published-API documentation. A crate lists only the crates it names in its own
code, and one that only its tests use is a development dependency, so the table
is the whole graph. A lower crate never refers to a higher one. When
a function needs a type from above, it belongs with that type: masking a
search result is `ChatMetadata::redact`, next to `ChatMetadata`, not in
`mash-model`'s `redaction`. Inside `mash-app`, `crate::role`, `crate::AppError`
and the like still resolve, through imports in `app/lib.rs`, and
`crate::silver` re-exports `mash-silver` beside the indexing pipeline.

A test belongs to the highest crate it touches. Adapter tests that index
through storage therefore stay in `mash-app` under `app/silver/`, in modules that
mirror the adapter (`silver::copilot::tests`), and reach it through its public
items; tests of an adapter's internals stay in `mash-silver`. Fixtures another
crate's tests need sit behind a `test-support` feature (`mash-model`,
`mash-silver`, `mash-app`), which the crates above enable as a development
dependency; an item gated that way is `pub`, since the library is also built
with the feature on. The CLI lives in `mash`, above
the library, so that crates such as the dashboard can later sit between them;
an item the executables need from `mash-app` is `pub` there, and nothing else is. For an isolated query edit cycle,
use `env CARGO_INCREMENTAL=1 RUSTC_WRAPPER= cargo test-clif -p mash-query`.
See [crate boundaries](https://github.com/mashhq/mash/blob/a49210157b990b23c5ef9f90d5d1d13fd3ce9f11/docs/crate-boundaries.md) for responsibilities and
the integrated checks required when changing search behavior.

#### The edit loop

Use the just recipes while developing; they carry the settings that keep
rebuilds short:

```bash
just iterate 'test(changed_behavior)'                    # fmt, Clippy, mash-app's unit tests
just iterate 'test(assistant)' --package mash-dash       # another crate's unit tests
just iterate 'binary(cli_contract)'                      # the CLI contract tests
just iterate 'test(changed_behavior)' --package all      # every crate's unit tests
just iterate 'test(changed_behavior)' --targets all      # integration targets as well
just dev-build                                           # a debug `mash` to try by hand
```

The test build compiles one crate and the ones below it: `mash-app` unless
`--package` names another, or `all`. A filter that matches no test in that crate
fails, so a wrong `--package` is not silent. Asking for integration targets
(`--targets all`) builds every crate unless `--package` names one. Clippy always checks the whole
workspace.

Measured on Xenia in November 2026, while Mash was still one crate, for a one-line edit in
`app/dash/assistant.rs` (now `crates/mash-dash/src/assistant.rs`), warm caches, four Cargo jobs under `agent-test`, no
other builds running (logs in `mash-artifacts/build-speed/`):

| Step | Time |
| --- | --- |
| `cargo clippy --all-targets` | 20–26 s |
| Test build, all targets (previous `just iterate`) | 55–57 s, 6.0 GiB peak |
| Test build, library tests only (current `just iterate`) | 35–42 s, 3.2–4.0 GiB peak |
| Debug `mash`, incremental (`just dev-build`) | 24–28 s |
| Debug `mash`, raw `cargo build-clif` under the machine-wide Cargo settings | 88 s |
| Whole `just iterate`, all targets, Clippy then tests (original) | about 80 s |
| Whole `just iterate`, library tests, Clippy then tests | 68–71 s |
| Whole `just iterate`, library tests, Clippy beside tests (current) | 56–58 s with Clippy waiting for a slot; the test build alone is 36–41 s |

`just iterate` checks formatting first, then runs Clippy and the test build at
the same time and prints Clippy's output after the tests, so one loop reports
both lint and test failures. Clippy has its own target directory
(`clippy/` under the development target) because Cargo locks a target
directory for a whole build. Each runs as its own `agent-test` job, so a loop
takes two of the four shared slots; in one job they would pass its memory limit.

With the crate split complete, the same one-line edit and `just iterate` with
the crate named (warm caches, November 2026) take:

| Edit in | Loop | Of which test build |
| --- | --- | --- |
| `mash-dash` | 9 s | 4 s |
| `mash-mcp` | 11 s | 4 s |
| `mash-server` | 12 s | 6 s |
| `mash-app` | 21 s | 11 s |
| `mash-sync` | 11 s | 8 s |
| `mash-search` | 15 s | 8 s |
| `mash-storage` | 16 s | 3 s |
| `mash-silver` | 21 s | 5 s |
| `mash-model` | 15 s | 1 s |

Before the split every one of these was the single-crate loop in the table
above. In the lower crates the test build is now a few seconds and the rest is
Clippy rechecking every crate above the edit. `--package all` builds every
crate's unit tests and costs about a minute.

What the time is made of:

- One rustc process compiles the whole `mash` crate. For the library tests a
  one-line edit still costs about 34 s: macro expansion 3.5 s, checks 5 s,
  collecting and partitioning code for generation 12 s, Cranelift itself under
  1 s, saving the incremental cache 3 s, linking 4.5 s. Almost all of it grows
  with the size of the crate, not the size of the edit, so the remaining lever
  is splitting the crate. The parallel compiler front end (`-Zthreads=8`) made
  no difference.
- The all-targets build compiled the library twice (with and without tests) and
  linked four 540 MB executables. Its 6.0 GiB peak is exactly the soft memory
  limit `agent-test` gives a job, at which the kernel slows the job down;
  earlier `just iterate` logs with test builds of two to six minutes all show
  that same peak. Staying under it matters more than the seconds in the table
  when other sessions are building.
- Dependencies are built with line-number debug information only
  (`[profile.dev.package."*"]` in `Cargo.toml`), which takes the debug `mash`
  from 540 MB to 400 MB.
- The machine-wide `~/.cargo/config.toml` on Xenia sets `incremental = false`
  and a compiler cache wrapper. The just recipes override both; a raw Cargo
  command does not, and then every edit recompiles the crate from nothing.
- A commit changes the revision stamped by `build.rs`, so the first build after
  any commit recompiles the `mash` package even when no source changed. That
  package is only the two programs' entry points (and, in a test build, the
  CLI contract tests), which hand the revision to `mash-cli`; the libraries
  read it at run time (`mash_app::build_revision`) and are left alone.

For a raw Cargo command, keep one target directory and enable incremental
compilation explicitly:

```bash
env CARGO_INCREMENTAL=1 RUSTC_WRAPPER= cargo test-clif --lib
# Add a Nextest expression to run the affected unit tests:
env CARGO_INCREMENTAL=1 RUSTC_WRAPPER= cargo test-clif --lib -E 'test(claude_copies::)'
```

On Xenia, warm incremental rebuilds after a small test edit took 7.7–8.6 seconds,
compared with approximately 32–35 seconds without incremental compilation.
The first incremental build took 41.7 seconds and created additional disk cache.
The empty `RUSTC_WRAPPER` bypasses a locally configured compiler wrapper; the
installed wrapper failed on this incremental build. `--lib` selects unit tests
and does not run CLI integration contracts. Keep the full suite, real-model and
shared-backend checks as completion gates. See the
[measured workflow](https://github.com/mashhq/mash/blob/a49210157b990b23c5ef9f90d5d1d13fd3ce9f11/benchmarks/2026-09-30-test-turnaround.md) for scope and limits.

CI also runs the [labeled hybrid relevance matrix](https://github.com/mashhq/mash/blob/a49210157b990b23c5ef9f90d5d1d13fd3ce9f11/benchmarks/2026-10-01-shared-relevance.md)
with explicitly installed models against local and shared Qdrant. The
report retains historical measurements of the removed backends.
The shared tests require their disposable backend endpoint and
`MASH_TEST_MODELS_DIR`. To run these checks together with those variables set:

```bash
cargo test-clif --lib --run-ignored only -j 1 -E 'test(labeled)' --success-output immediate
```

The installed August 2026 nightly can hit a [Clippy compiler crash](https://github.com/rust-lang/rust/issues/161495)
when checking async code. On that toolchain, use
`RUSTFLAGS=-Znext-solver=no cargo clippy --all-targets -- -D warnings`.

The storage integration tests need Qdrant 1.20+ and run with the rest of the
suite; only those that also need installed models, the server binary or
container restarts stay ignored. With Docker available, nothing else is
needed: each test process starts its own
disposable `qdrant/qdrant:v1.20.0` container through
[testcontainers](https://github.com/testcontainers/testcontainers-rs)
(`mash_model::test_support::qdrant`), capped at 2 GiB and two CPUs, labelled
`mash.test=qdrant` and `mash.test.owner`, and removed by a reaper process when
the test process exits, including after a timeout or kill. Starting one takes
about half a second with the image cached. To use a Qdrant you run instead (CI's
service, Windows' native Qdrant), set both `MASH_TEST_QDRANT_URL` and
`MASH_TEST_QDRANT_GRPC_URL`; then no container starts. Without Docker and without
those variables, a Qdrant test fails and says how to provide one. Tests start
real HTTP services and create/remove isolated collections for their owned
groups. The ignored ones that also need models run with:

```bash
MASH_TEST_MODELS_DIR='/path/to/explicitly-installed-models' \
  cargo nextest run --run-ignored only --no-tests fail -j 2 -E 'test(qdrant)'
```

Docker containers run outside `agent-test`'s memory budget, which is why each
one carries its own limit; keep `-j` low on a shared machine. `just
test-cleanup` removes test containers left behind (anything labelled
`mash.test`, never a Qdrant you run, and never one whose test process is still
running); `just test-cleanup ID` removes one run's. Collections a failed
test leaves in a Qdrant you run yourself are not removed automatically.

The real-model integration tests are ignored by default and require
`MASH_TEST_MODELS_DIR` pointing at assets created by `mash models install`.
The shared-backend selections include Claude copy reconciliation with real
embeddings and require that directory too. Selected tests fail when prerequisites
are missing; `--no-tests fail` rejects an accidentally empty selection.

For an instrumented coverage report, install
[cargo-llvm-cov 0.9.1](https://github.com/taiki-e/cargo-llvm-cov/releases/tag/v0.9.1)
and the toolchain's `llvm-tools-preview` component. Set the model directory and
the Qdrant test URL above to installed models and disposable services.

Also set `MASH_TEST_SERVER_BINARY` to the instrumented `mash-server` binary produced
in the coverage target's `debug` directory. With an explicit `CARGO_TARGET_DIR`,
use `$CARGO_TARGET_DIR/llvm-cov-target/debug/mash-server`; without it, the default is
`target/llvm-cov-target/debug/mash-server`. Set `MASH_TEST_CLI_BINARY` to the `mash`
executable beside it for the MCP deployment probe. An explicit `CARGO_LLVM_COV_TARGET_DIR` instead
uses that directory directly, as defined by [cargo-llvm-cov 0.9.1](https://github.com/taiki-e/cargo-llvm-cov/blob/v0.9.1/src/cargo.rs#L63).
The suite builds the binary before running tests, so it need not exist before
the coverage command starts. Restart and snapshot-restore contracts
require Docker and the cached Qdrant image; set `MASH_TEST_DOCKER_SG=1` on Xenia.
They own their containers and API processes and do not restart the supplied
backend test services. Use the instrumented binary; forced API termination can
prevent its final LLVM profile flush, so the contract's runtime checks do not
guarantee that all child-process execution appears in the coverage report.
Snapshot restoration stops all fixture writers and restores both group
collections onto a fresh database after deleting the original database and
volume. See the [backup scope](https://github.com/mashhq/mash/blob/a49210157b990b23c5ef9f90d5d1d13fd3ce9f11/docs/silver-gold-configuration.md#quiescent-backup-and-restore).

Then run:

```bash
rustup component add llvm-tools-preview
just coverage
```

This runs every test except the manual benchmarks, including the installed
model and all three shared-backend contracts. It writes LLVM JSON coverage to
`target/llvm-cov/coverage.json`; set `COVERAGE_OUTPUT` to choose another location.
The coverage task cleans workspace coverage artifacts before execution so older
binaries cannot contribute obsolete source mappings in a persistent target.
On nightly, use `CARGO_ARGS=--branch just coverage` to include
LLVM's experimental branch instrumentation. This does not measure MC/DC.
Generated SQL bindings, separate test files, standalone test helpers and the
test-only relevance evaluator and manual benchmark modules are excluded from
the report.
Tests embedded in production source files still contribute to those files'
coverage, so assess uncovered behavior as well as the aggregate percentage.
Coverage needs LLVM instrumentation; regular development continues to use
Cranelift. On the affected nightly, use `RUSTFLAGS=-Znext-solver=no just coverage`.

For bounded local build and CLI search measurements, use installed models and
a dedicated disposable Qdrant instance. The script copies a bounded sample of
provider histories; captures contain private history and must stay outside Git:

```bash
agent-test uv run --no-project python tools/benchmark-views.py \
  --binary /path/to/mash --models /path/to/installed/models \
  --qdrant-url http://127.0.0.1:6333 \
  --qdrant-grpc-url http://127.0.0.1:6334 \
  --output /home/james/scratch/mash-artifacts/local-views-measurement
```

The script compares lexical/dense replay and search timings while checking that
replay preserves canonical silver. Timings include CLI startup. Process memory
and CPU exclude the Qdrant server; SQLite size reports cover silver only.
Missing memory measurements fail the capture. These measurements do not establish
relevance or fleet performance. The former SQLite scan and ANN replay benchmarks
were removed with SQLite gold.

CPU indexing reserves two workers for long embedding batches and allows short
batches to use all eight workers. It preserves the dynamically quantized model's
batch membership and vector generation. See the
[indexing memory measurements](https://github.com/mashhq/mash/blob/a49210157b990b23c5ef9f90d5d1d13fd3ce9f11/benchmarks/2026-09-30-index-memory.md) for the
memory/throughput tradeoff and the manual worker-budget diagnostic. This is an
inference workspace limit, not a total indexing memory limit.

## Working in parallel

Several agent sessions and people work on this repository at once. Each task
gets its own worktree and short-lived branch, and `main` moves only through
the merge queue, to commits the required checks passed on. Sharing one
checkout is what went wrong before: sessions staged each other's hunks, one
push cancelled the CI run of another commit, and deploys built whatever
half-done files were on disk. Pushing straight to `main` is what went wrong
after that: a commit that failed CI stayed on `main` until someone noticed.

### One worktree per task

```bash
tools/install-hooks.sh                    # once per clone: the pre-push hook
cd "$(tools/worktree.sh new search-fix)"  # ../mash-worktrees/search-fix on work/search-fix
# edit, commit, just iterate ...
tools/worktree.sh land search-fix --wait  # through the merge queue; see below
tools/worktree.sh done search-fix
```

- `new NAME [BASE] [--deps dashboard,site,raycast]` fetches, then creates
  `../mash-worktrees/NAME` on branch `work/NAME` from `origin/main` (or `BASE`)
  and prints the path. A new worktree has no `node_modules`; `--deps` installs
  the named packages (`pnpm install --frozen-lockfile --prefer-offline` in
  `dashboard/` and `site/`, `npm ci` in `raycast/`). Set `MASH_WORKTREES` to
  keep worktrees somewhere else.
- `list` shows every worktree of the clone.
- `land NAME [--wait]` fetches, rebases the branch onto `origin/main` inside
  its worktree, runs `tools/pre-push.sh`, pushes the branch to origin as
  `work/NAME`, opens a pull request for it (or reuses the open one) and puts
  it in the merge queue, which takes about a minute. Without `--wait` it then
  prints the pull request's address and returns; with it, it follows the pull
  request until it is merged, or
  says why it was not and exits non-zero. A rebase conflict is left in the
  worktree to resolve. The worktree stays until `done`. See
  [The merge queue](#the-merge-queue).
- `land NAME --direct` pushes straight to `main`, past the queue. It is for
  emergencies and works only for an administrator.
- `pr NAME` pushes the branch and opens a pull request with
  `gh pr create --fill` without queueing it, for work that wants review or a
  full CI run of its own first. `land NAME` queues that same pull request
  later.
- `preview NAME` and `previews` start, stop and list live dev servers of a
  worktree on the tailnet; see Previews below.
- `deploy-tree [NAME]` prints a clean, detached tree at `origin/main`:
  `_deploy` for deploys and releases, or `_NAME` for a job that must not move
  that one (the Storybook build uses `_storybook`).
- `done NAME` removes the worktree and deletes the branch, and its pushed copy
  once merged if GitHub has not already deleted it. It refuses while the
  worktree holds uncommitted or untracked files, or commits that are not on
  `origin/main`; `--force` discards them. The queue rebases, so the commits
  on `main` are copies under other names: `done` accepts a branch when every
  one of its commits has a commit with the same change on `main`, or when
  GitHub says a pull request merged that exact branch head.

Files Git ignores are not in a new worktree: `node_modules`, `dashboard/dist`
and `.beads/`. `new` links the agent skills (`.agents/skills`) and `.claude`
to the main checkout's copies, so edit them there; run `bd` from the main
checkout too. Do not switch branches, rebase or reset in the
main checkout; keep it on `main`, and update it with `git pull --ff-only` when
nobody is editing it.

Rust builds need no per-worktree setup. The justfile pins one absolute target
directory (`DEV_TARGET_DIR`), so `just iterate`, `just dev-build` and
`just validate-dev` in any worktree reuse the compiled dependencies, and
`tools/build-dev-linux.sh` takes a lock in that directory, so builds from
different worktrees queue instead of running at once. The debug binary at
`$DEV_TARGET_DIR/debug/mash` is whichever worktree built last: pass
`DEV_TARGET_DIR=/home/james/scratch/achi-targets/NAME` to keep one of your
own, at the cost of a cold build. A raw `cargo` command without
`CARGO_TARGET_DIR` builds everything again in the worktree's own `target/`.

### Previews

A preview is a worktree's own dev server, reachable from any device on the
tailnet, so a change can be looked at on a phone seconds after it is saved,
with no build and no deploy:

```bash
tools/worktree.sh preview search-fix          # the dashboard, with the demo's made-up history
tools/worktree.sh preview search-fix --site   # the website
tools/worktree.sh preview search-fix --storybook   # the dashboard's Storybook
tools/worktree.sh previews                    # what is recorded, and whether each answers
tools/worktree.sh preview search-fix --stop   # add --site or --storybook for those
```

`preview` prints the address twice on standard output, by Tailscale address
and by MagicDNS name, and everything else on standard error:

```text
http://100.97.52.20:5242/demo/
http://xenia.example.ts.net:5242/demo/
```

- **Where it listens.** Only on this machine's Tailscale address
  (`tailscale ip -4`), never on every interface. Without Tailscale it refuses;
  `--local` uses `127.0.0.1` instead, for a browser on this machine.
- **Port.** The dashboard uses 5200-5299, the website 5300-5399 and
  Storybook 5400-5499. Port 5400 itself is where main's built Storybook is
  served (`tools/storybook-serve.sh`), so a preview finds it taken.
  The port is the checksum of the worktree's name within the range, so a
  worktree keeps its address from one day to the next; when that port is
  taken the next free one is used.
- **Storybook.** `--storybook` runs `pnpm storybook` (`storybook dev`). Its
  stories are drawn with made-up data only, so `--real` does not apply. Like
  the other previews it listens on the Tailscale address; Storybook also
  listens on `127.0.0.1` on the same port, which no other device reaches.
- **Data.** The dashboard preview runs `pnpm dev:demo`: the demo's entry point
  and made-up history (`dashboard/src/demo`) under `/demo/`, with nothing
  private in it. `--real` runs `pnpm dev` instead and forwards `/api` to this
  machine's `mash dash` (`MASH_DASHBOARD_API_URL`, else the address
  `mash-dash.service` listens on, else `127.0.0.1:8790`). **Anyone on the
  tailnet who opens a `--real` preview reads this machine's real chat history
  without signing in**, so it is never the default and the command says so
  each time. When that dashboard asks for a token, systemd passes
  `~/.mash/dashboard.env` to the dev server and its proxy presents
  `MASH_DASH_TOKEN`; the script never reads or prints it. A `--real` preview
  answers anything but `GET` and `HEAD` under `/api` with 405 itself.
- **How it runs.** As the transient user unit `mash-preview-NAME` (or
  `mash-preview-NAME-site`, `mash-preview-NAME-storybook`) under
  `systemd-run --user`, with `MemoryMax=1500M` and no swap, so it outlives
  the session that started it. A dashboard preview with a page open uses
  about 0.8 GB and a website preview about 0.6 GB. Storybook's dev server
  uses about 1.2 GB once it has started, so its cap is 2500M.
  `MASH_PREVIEW_MEMORY_MAX` sets another cap for any of them. State and logs are in
  `~/.local/state/mash/previews/` (`MASH_PREVIEW_STATE` moves them), outside
  the worktree. `done NAME` stops the worktree's previews before removing it.
  A missing `node_modules` is installed first, as `new --deps` does.
- **Hot updates.** Saving a Svelte, TypeScript or CSS file updates the open
  page in place over the dev server's WebSocket, which uses the page's own
  address and so needs nothing extra on the tailnet. Vite and Astro refuse a
  `Host` header they were not told about: the address always passes, and the
  MagicDNS name is added for the run through
  `__VITE_ADDITIONAL_SERVER_ALLOWED_HOSTS` and `astro dev --allowed-hosts`,
  not in the committed configuration. Storybook checks the header itself as
  well, and reads the name from `MASH_STORYBOOK_HOSTS`
  (`dashboard/.storybook/main.ts`). The website copies `docs/` and the
  README when it starts (`pnpm prepare-content`), so a change to those needs
  the website preview stopped and started again.

Previews are plain HTTP inside the tailnet. Stop one when the work is done;
`previews` shows what is still running.

### The pre-push check

`tools/pre-push.sh` runs the checks that most often turned CI red, only for the
areas a push changes, and takes a few seconds:

| Changed files | Check | Fix |
|---|---|---|
| Rust, TypeScript, Svelte, Python | `python3 tools/check_file_lines.py` | Split the file or write its growth review |
| CSS | `python3 tools/check_css_budget.py` | Use Tailwind utilities or scoped styles; after removing CSS, `--update` |
| `dashboard/`, `site/` | Biome on the changed files | `pnpm exec biome check --write FILES` |
| `dashboard/src/` | `src/theme/theme.test.ts` | Name a role token; remove unused ones |
| Rust | `cargo fmt --check` (workspace, and `desktop/src-tauri`) | `cargo fmt` |
| `raycast/src/` | Prettier and ESLint on the changed files | `npm run fix-lint` |

It runs no test suite and no build. A missing tool or `node_modules` skips its
check with a note; a worktree without `node_modules` borrows the main
checkout's. The hook compares the commits being pushed with what the remote
has; run by hand, `tools/pre-push.sh [BASE [HEAD]]` compares with
`origin/main`. Each failure prints one line with the command that fixes it.

The checks judge the commit, because that is what CI sees. When the checkout
is not exactly that commit (uncommitted or untracked files, or another commit
checked out), the commit is checked out to a temporary directory and checked
there, so someone else's edits on disk can neither fail nor rescue a push.
`tools/pre-push.sh --worktree` checks the files on disk instead, for use
before committing.

`git push --no-verify` skips the hook. Keep it for emergencies, such as
landing a fix while a tool is broken: CI runs the same checks on the commit.

### The merge queue

`main` is protected by a repository ruleset
([main-ruleset.json](main-ruleset.json), applied with
`tools/github-ruleset.sh`). It requires a pull request, the merge queue and
five status checks, refuses force pushes and deletions, and keeps history
linear. No review is required. `land` is the way in:

```bash
tools/worktree.sh land search-fix --wait
```

1. `land` rebases the branch, runs the pre-push check, pushes `work/NAME` and
   opens the pull request. Its description carries the marker `[merge queue]`,
   which makes the pull request's own CI run skip every expensive job: the
   queue is about to run them on the commit that matters. The required checks
   therefore pass on the pull request in about a minute.
2. `land` asks GitHub to queue the pull request (`enqueuePullRequest`) until
   it accepts, which it does once those checks have passed. Auto-merge is not
   used: with it set, GitHub left pull requests with passing checks out of
   the queue.
3. The queue rebases the branch's commits onto `main` plus everything queued
   ahead of it, as the branch `gh-readonly-queue/main/pr-N-...`, and runs the
   workflows on that commit (`on: merge_group`). The merge method is rebase:
   each of the branch's commits arrives on `main` as its own commit, with no
   merge commit, under a new name.
4. When every required check has passed, `main` moves to exactly the commit
   that was tested and GitHub deletes `work/NAME`. When one fails, the pull
   request leaves the queue and `main` does not move.

The required checks are one summary job per workflow. Each passes when the
jobs of its workflow passed or were skipped because the change cannot affect
them (`tools/ci_changes.py`), so a required check is never left waiting for a
job that did not start:

| Check | Workflow | What it covers | Typical time when its jobs run |
|---|---|---|---|
| `ci / required` | `ci.yml` | `file-lines`, `protocol`, `dependencies`, `rust`, `tooling` | 9 minutes |
| `windows / required` | `windows.yml` | the x64 build and its tests, the ARM build, the Linux import | 6 to 11 minutes |
| `front-ends` | `frontends.yml` | the dashboard, its browser shards, Raycast | 3 minutes |
| `visual / required` | `visual.yml` | the dashboard's pictures | 2 minutes |
| `docs / required` | `docs.yml` | the website's build, tests and pictures, never its deploy | 5 minutes |

`desktop.yml` takes about half an hour, so it is not required and does not run
in the queue; it still runs on `main` for the paths it names. `Dev artifacts`
and the website's deploy run only on a push to `main`.

A change is on `main` roughly as long after `land` as its slowest required
check takes, plus two minutes for the pull request's own run and the queue's
bookkeeping: about eleven minutes for a Rust change (up to thirteen when it
changes a crate near the bottom, which Windows ARM then compiles all of ours
on top of; see [The merge queue's caches](#the-merge-queues-caches)), about seven for a
dashboard or website change, about two for `AGENTS.md`. The ruleset's
five-minute wait applies only while fewer changes than its minimum group of
one have passed, so it never delays anything. The queue does not
wait for one change to finish before starting the next. Each queued change is
tested on top of the ones ahead of it, up to three at once, so three changes
landed together are all on `main` after about eleven minutes, not thirty. If
one of them fails, it leaves the queue and the ones behind it are tested
again without it. Up to five that have passed merge in one step.

The push to `main` that the queue makes starts the workflows once more for
the same commit. The `changes` job finds the queue's passing run for that
commit and skips every job that tests, so the run concludes in about a minute
and still counts for a release, which wants a passing `ci.yml` and
`windows.yml` run for its exact commit. The one job that does run there is
`windows.yml`'s `warm`, which saves a cache when the queue needs a new one
(see [The merge queue's caches](#the-merge-queues-caches)).

**Reading a rejection.** `land --wait` prints why and exits 1:

```text
Pull request #12 was removed from the merge queue: failed checks. main is unchanged.
Checks that did not pass for 3f2a9c1d77e0:
  rust  https://github.com/mashhq/mash/actions/runs/123/job/456
  ci / required  https://github.com/mashhq/mash/actions/runs/123/job/789
```

Open the first address, fix the cause in the worktree, commit, and run
`land NAME --wait` again: it reuses the pull request. A failure that looks
unrelated to the change (a runner that died, a download that timed out) is
landed again the same way. Without `--wait`, follow a queued pull request with
`tools/merge-queue.sh wait NUMBER`. When a required check fails on the pull
request itself, GitHub never queues it, and `land --wait` says that instead.

**The pre-push hook** (`tools/install-hooks.sh`) refuses a push to `main` of
`mashhq/mash`. GitHub refuses one as well, except from an administrator, whom
the ruleset lets past: James and every agent on his machines push as one. The
hook is what stops a stray `git push origin HEAD:main` there.

**The emergency path.** When the queue itself is what is broken (GitHub
Actions is down, a required check cannot pass and the fix for it must land),
an administrator can land past it:

```bash
tools/worktree.sh land NAME --direct    # rebase, pre-push check, push to main
bash tools/release.sh --direct VERSION  # the release commit, the same way
```

Nothing has run the test suites on a commit landed like that; watch its runs
on `main`. To switch the queue off for longer, and back on:

```bash
tools/github-ruleset.sh show      # what GitHub enforces now
tools/github-ruleset.sh disable   # main takes a plain push again
tools/github-ruleset.sh apply     # enforce docs/internal/main-ruleset.json again
```

While it is disabled, `land` without `--direct` fails, because there is no
queue to enter. To change a rule, edit `main-ruleset.json`, land it, and run
`apply`. A workflow cannot commit to `main` any more: the token of GitHub
Actions cannot be given a way past a ruleset, and GitHub starts no checks for
a pull request that token opens. That is why a release's Scoop manifest is
landed by `tools/release.sh`, and why the "Update visual baselines" workflow
refuses to run on `main`.

### CI on main

Runs on `main` are not cancelled by a later push
(`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`), because a
release needs a passing run for its exact commit. Pull request branches still
cancel superseded runs. A run that has not started can still be replaced by a
newer one in the same concurrency group. With the merge queue that matters
little: a run on `main` for a commit the queue tested skips every job.

### What CI runs for a change

`ci.yml`, `windows.yml`, `frontends.yml`, `visual.yml` and `docs.yml` start
for every pull request and every merge queue group, the first four for every
push to `main` as well, and skip the jobs a change cannot affect. Their first job,
`changes`, lists the changed files and asks `tools/ci_changes.py` which areas
they reach; each expensive job carries an `if` on the answer. The workflows are
not filtered with `paths:`, because a workflow that does not start has no run,
and a release requires a successful `ci.yml` and `windows.yml` run for its
exact commit. A run whose jobs were skipped still concludes successfully.

| Changed files | `ci.yml` | `windows.yml` | `frontends.yml` | `docs.yml` |
|---|---|---|---|---|
| Rust, `plugins/`, `fixtures/`, `protocol/`, `deployment/`, `Cargo.*` | `protocol`, `dependencies`, `rust`, `tooling` | both builds, the library tests, import | nothing | nothing |
| `docs/`, `README.md` (the binary embeds the guides) | as for Rust | as for Rust | nothing | build, pictures |
| `docs/internal/*.md` (read by no build and no test) | nothing | nothing | nothing | build, pictures |
| `site/` | nothing | nothing | nothing | build, pictures |
| `dashboard/` | nothing | nothing | `dashboard`, four `dashboard-browser` shards | build, pictures (the demo) |
| `raycast/` | nothing | nothing | `raycast` (lint, build, test) | nothing |
| `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CLAUDE.md`, `bucket/` | nothing | nothing | nothing | nothing |
| `.github/`, `tools/`, `justfile`, `Makefile`, anything not listed | everything | everything | everything | build, pictures |

`visual.yml` runs its one job for the rows where `frontends.yml` runs the
dashboard's. The `docs.yml` column is for a pull request and the merge queue.
On a push to `main` that workflow keeps its `paths:` filter, always builds,
and publishes; it does not start for a push that changes nothing it names.

"Nothing" still runs `changes` and, in `ci.yml`, the `file-lines` job, which
counts every area's source. Three files inside the front-end directories also
run the Rust jobs, because the Rust build reads them: `site/public/setup.md`,
`site/package.json` and `dashboard/openapi.json`. A test in
`tools/test_ci_changes.py` searches the Rust source for paths into those
directories and fails when one is not listed. A path no rule claims runs
everything. `desktop.yml`, `spec.yml` and `dependencies.yml` keep their own
`paths:` filters; nothing depends on their having a run. No workflow the
ruleset requires a check from may carry a `paths:` filter on `pull_request`,
and `merge_group` cannot have one: a workflow that does not start reports
nothing, and the change would wait in the queue until it timed out.

On `main` the changed files are those since the last commit the same workflow
passed for, not since the previous push. A skipped job therefore means "these
files are as they were when this job last passed", which is what lets a release
trust a run in which the Rust jobs were skipped. Comparing with the previous
push would let a site-only commit report success on top of a Rust change whose
own run failed, or was replaced while it waited. A pull request is compared
with its base, a merge queue group with the commit it was built on
(`merge_group.base_sha`: `main`, or the group queued ahead of it), and a
manual run (`workflow_dispatch`) runs everything. Two runs answer "nothing"
without comparing: the pull request's own run when `land` opened it for the
queue, and the push to `main` of a commit the same workflow passed for in the
queue.

Each of the five workflows ends with one job that passes when every job
before it passed or was skipped: `front-ends` in `frontends.yml`, and
`ci / required`, `windows / required`, `visual / required` and
`docs / required` in the others. Those are the checks the ruleset for `main`
requires. `front-ends` also merges the browser shards' reports into one HTML
report, kept for a week as the `dashboard-browser-report` artifact.

### The per-commit binary builds

`dev-artifacts.yml` builds `mash` and `mash-server` for every push to `main`
with the release's own steps (`build-binaries.yml`), and deployments wait for
it. Those builds are cached; a release is not. `release.yml` calls the same
steps without `cache`, so a published binary is compiled from nothing and
never depends on what an earlier run left behind.

With `cache: true` a build starts from two caches the last build on `main`
saved, one pair for each target:

| Cache | Holds | Replaced |
|---|---|---|
| `mash-artifacts-v1-<target>-release-…` (`Swatinem/rust-cache`) | The compiled dependencies and Cargo's downloads | When `Cargo.lock`, a manifest or the compiler changes |
| `mash-artifacts-crates-v1-<target>-<lock>-<commit>` | This repository's own compiled crates, with a record of the files they were compiled from | By every build on `main` |

Cargo decides whether one of our crates is current by comparing file times,
and a checkout stamps every file with the present, which on its own would
compile all of them again. Before the build,
`.github/scripts/build-inputs.mjs restore` gives a file its old time back when
its content matches the record saved with the crates, and leaves the rest
alone. The built dashboard (`dashboard/dist`) is in the record as well. A
build therefore compiles the crates the commit changed and the ones built on
them:

| The commit changed | Compiled again |
|---|---|
| Nothing a crate reads (`tools/`, `.github/`, instructions) | `mash` alone: the two entry points, a few lines each, and the programs are linked again |
| The dashboard | `mash-dash`, `mash-cli`, `mash` |
| `mash-cli` | `mash-cli`, `mash` |
| `mash-server` | `mash-server`, `mash-cli`, `mash` |
| `mash-mcp` or `mash-fleet` | That crate, `mash-dash`, `mash-cli`, `mash` |
| `mash-app` | `mash-app`, `mash-mcp`, `mash-server`, `mash-dash`, `mash-cli`, `mash` |
| A crate near the bottom (`mash-model`, `mash-silver`, `mash-query`, `mash-storage`) | Nearly every crate of ours; no dependency |
| `Cargo.lock` | The changed dependencies and every crate of ours |

Measured on GitHub's runners on 26 November 2026, as the `Build` step and
then the whole job, in seconds. The workflow takes as long as its slowest job:

| | Linux x86_64 | Linux aarch64 | macOS arm64 | Windows x64 | Workflow |
|---|---|---|---|---|---|
| Nothing cached: every build before, and a release | 416 / 490 | 420 / 485 | 773 / 816 | 784 / 847 | 14 to 18 min |
| Dependencies cached, all of our crates compiled | 324 / 426 | 272 / 362 | 620 / 685 | 667 / 751 | 12 min 38 s |
| A dashboard-only commit | 139 / 237 | 121 / 202 | 289 / 362 | 206 / 287 | 6 min 8 s |
| A commit that changes no source, while the command lines were in `mash` | 74 / 159 | 63 / 152 | 150 / 222 | 136 / 214 | 3 min 49 s |
| A commit that changes no source, now: only `mash` is compiled | 3 / 105 | 7 / 97 | 14 / 69 | 42 / 132 | 2 min 16 s |
| The same commit again | under 4 / 105 | under 4 / 95 | not measured | not measured | |

The dependencies are a fifth to a third of a build and our own crates the rest.
The second row is therefore the most a commit should cost, and what the first
build after a change to `Cargo.lock` costs. These rows, except the
fifth, were measured before the entry points were boxed (see below); measure
them again before relying on them.

Our crates do not wait long for each other. A crate above can start as soon as
the one below has been checked, which takes 10 to 18 s each; in an optimized
build the rest of a crate's time, nine tenths of it, is generating code, and
all the crates share the same few processors for that. What a build costs is
therefore how much code is generated, not how long the chain of crates is, and
removing a dependency between two crates saves time only when it stops one of
them being compiled again.

What decides how much code is generated is where a future is compiled. An
`async fn` is compiled in the crate that first polls it, not the one that
declares it, together with every `async fn` it awaits. `mash_app::core::execute`
was polled by `mash-mcp`, `mash-dash` and `mash`, so each compiled the whole
indexing and search pipeline again, and `mash` compiled the server, the
dashboard and the MCP server as well, for every commit. So a future that
crosses into another crate is boxed where it is declared, in a plain function
that returns `BoxFuture`: `core::execute` and `execute_with_model_limit`, the
three `run` functions (`mash_server::run`, `mash_dash::run`, `mash_mcp::run`)
and the dashboard console's `call`. Keep that when adding an entry point: an
`async fn` that another crate drives with `block_on`, `spawn` or a request
handler is compiled there, in full. `nm -C -S` on a crate's `.rlib` under the
release target shows what it holds; functions of another crate with
`{closure#0}` after their name are futures compiled in the wrong place.

Optimized rebuilds on Xenia (`cargo build --release --timings`, four jobs
under `agent-test`, other sessions building beside it), before and after
boxing, on 26 November 2026; the reports are in
`mash-artifacts/build-speed/crate-deps/`:

| Rebuild after | Before | After |
|---|---|---|
| A commit that changes no source, while the command lines were still in the `mash` package | 148 s | 55 s |
| A change to `mash-app` | 248 s | 151 s |
| A change to `mash-search` | 301 s | 187 s |

| Crate, when `mash-app` changed | Before | After |
|---|---|---|
| `mash-app` | 106 s | 145 s |
| `mash-mcp` | 147 s | 70 s |
| `mash-server` | 105 s | 126 s |
| `mash-dash` | 169 s | 92 s |
| `mash-cli` (then the `mash` package's library) | 141 s | 65 s |

`mash-app` and `mash-server` now hold the code the crates above them used to
compile, which is why they grew; they run beside the others, so the build ends
when `mash-app` does. That crate is the next thing to shorten. The jobs spend about a minute and
a half outside the build step whatever is cached: starting the container or
runner, installing the toolchain, restoring the caches (about 20 s), building
the dashboard and packaging. Hosted runners vary: the same uncached macOS
build took 773 s, 876 s and 894 s in three runs that day.

Keep it that way: a value that differs for every commit belongs in the `mash`
package, not in a library under it, and that package stays a few lines.
Cargo compiles every target of a package again when its build script's output
changes, and every crate built on a crate that changed, so the stamp can cost
nothing only where nothing else is compiled with it. `MASH_BUILD_REVISION`
used to be compiled into `mash-app`, which recompiled it and the four crates
above it, more than half of a build, for every commit; then into the package
that also held the command lines, which cost 60 to 180 s of every build (the
fourth and fifth rows of the table). What is left is linking the two
programs, which is most of Windows' 42 s. On Xenia, changing only the
revision took 52 s before the command lines moved to `mash-cli` and 1.5 s
after.

Only `main` saves. A manual run on a branch reads `main`'s caches, and
`desktop.yml` reads them for the engine it builds on macOS and Windows. After
each build the `prune-caches` job deletes the caches a newer one replaced, so
the builds hold one pair per target, about 2 GB of the repository's 10 GB, and
GitHub does not evict the test workflows' caches to make room. To start again
from nothing, change `v1` in both keys.

The same job deletes what merge queue groups saved. A group runs on a branch
of its own (`gh-readonly-queue/main/pr-N-…`), a cache saved there can be read
only from that branch, and the branch is deleted when the group merges.
`ci.yml` and `windows.yml` used to save about 1.5 GB there for each group
whose change touched `Cargo.lock` or a manifest; being the newest caches in
the repository, they pushed out ones still in use within hours. Those
workflows no longer save from a group (see [The merge queue's
caches](#the-merge-queues-caches)); what is left is the compiler cache's
entries for the crates a group compiled, some tens of megabytes.

The dependencies' cache is saved when its key is new and never again, so it
must be complete when it is saved. `cache-on-failure` keeps what a run built
when it failed after compiling (the smoke check, packaging); a run that was
cancelled or failed while compiling saves nothing. A build that compiles
dependencies although the action reported a full match has a partial cache:
delete that key (`gh cache delete KEY`) and the next build on `main` saves a
whole one. None of the cache steps can fail a build; when one does not work
the build compiles more.

The workflow builds one commit at a time and, while one runs, keeps only the
newest waiting commit. Each build starts from the crates the build before it
saved, so running them side by side would save nothing and have them write
over each other; and a deployment takes the head of `main`, which is always
built.

### The merge queue's caches

A merge queue group can read a cache only when `main` saved it. A group runs
on a branch of its own, so what it saves is lost with the branch, and on
`main` the test workflows skip their jobs for every commit the queue already
tested, so they save nothing there either. Left like that, the queue's runs
start from whatever `main` last saved before the queue existed: stale after
the next change to `Cargo.lock`, and useless for `ci.yml` after the next
nightly compiler, which is a new one every day.

So nothing in the queue saves, and jobs that run only on `main` save what the
queue reads:

| What a queue run starts from | Read by | Saved on `main` by | When |
|---|---|---|---|
| `mash-windows-v2-x64-…`: the Windows x64 debug dependencies (about 600 MB) | `windows.yml` `build`, `lib-tests` | `windows.yml` `warm` | A new `Cargo.lock`, manifest or stable Rust |
| `mash-artifacts-v1-aarch64-pc-windows-msvc-release-…` and `mash-artifacts-crates-v1-aarch64-pc-windows-msvc-…`: the Windows ARM release dependencies and our own crates (about 550 MB) | `windows.yml` `build-arm64` | `queue-caches.yml` `windows-arm64` | Every commit that is not only `docs/`, `site/` or `raycast/` |
| The compiler cache (`sccache/…`, one small cache per compiled crate) and `mash-Linux-cargo-sources-…` | `ci.yml` `rust` | `queue-caches.yml` `ci-compiler` | A new nightly or `Cargo.lock`; it also runs daily at 01:40 UTC |

Each of the saving jobs first asks whether its cache exists, which takes about
half a minute, and compiles only when it does not. `warm` asks
`Swatinem/rust-cache` itself (`lookup-only`), because the key is that action's
hash of the compiler, `Cargo.lock`, the manifests and the workflow's `CARGO_*`
and `RUST*` variables; that is also why `warm` lives in `windows.yml`, where it
shares those variables with the jobs that read the cache. `ci-compiler` keeps a
marker cache named after the compiler and `Cargo.lock`
(`mash-ci-warmed-v1-…`), since the compiler cache has no single key to ask
for; it carries a copy of `ci.yml`'s variables, and the two must change
together. None of these jobs is part of a required check, and a failure in one
costs the queue compile time, nothing else: `warm` is marked
`continue-on-error`, and no release or deployment looks at `queue-caches.yml`.

After a change to `Cargo.lock` lands, the groups that start before the saving
jobs finish (10 to 15 minutes) restore the previous cache and compile the
dependencies that changed.

`windows.yml` sets `RUSTUP_TOOLCHAIN: stable` for every job. Without it, the
cache action's `rustc -vV` installed the nightly toolchain
`rust-toolchain.toml` names (50 s on x64, 85 s on ARM, in every job), and the
key held that nightly's version, so the cache was abandoned whenever a new
nightly was installed.

The prune job of `dev-artifacts.yml` keeps these within the repository's
10 GB: it deletes every `mash-windows-` and `mash-artifacts-` cache a newer
one of its family replaced, and compiler cache entries nothing has read for
two days (GitHub would keep a retired nightly's for seven).

With these the repository holds about 6 GB: 2 GB for the per-commit binary
builds, 0.6 GB for each of the two Windows rows above, 1.6 GB of pnpm stores
(`setup-node`), and about 0.4 GB of compiler cache for each nightly still
being read. The Linux import is not cached at all: from nothing its build
takes two minutes, and restoring 450 MB of dependencies saved half a minute
of that.

**What it bought.** Measured on GitHub's runners on 26 November 2026, for a
change that runs the Rust jobs, as the time from the group entering the queue
to the required check passing:

| | Before | After |
|---|---|---|
| `windows.yml` | 13 to 17 min (sixteen runs that day) | Not yet measured with its caches saved; with none at all, on a branch: x64 jobs 9 to 10 min, Windows ARM 16 min |
| `ci.yml` | 11 to 15 min (fourteen runs that day) | 8 min 40 s (`rust` 8 min 22 s, `tooling` 3 min) |

Where the time went before, with the dependencies restored from a cache, in
run 45816311226 (`windows.yml`, 15 min 21 s) and its `ci.yml` twin (13 min
8 s):

| Job | Total | Setup and caches | Compiling | Running tests and checks |
|---|---|---|---|---|
| `build` (Windows x64) | 13 min 24 s | 1 min 55 s, 50 s of it installing a nightly nothing used, 22 s saving a cache nothing could read | 4 min 10 s (programs 1 min 36 s, library tests 2 min 8 s, CLI tests 21 s) | 7 min 10 s (library tests 4 min 47 s, CLI tests 2 min 13 s) |
| `build-arm64` | 13 min 13 s | 4 min, 85 s of it the nightly, 1 min 35 s Node and the dashboard | 8 min 29 s, all of it our own crates in the release profile | 10 s |
| `linux-import` | 1 min 38 s, after `build` | 40 s | 52 s | 5 s |
| `rust` (`ci.yml`) | 12 min 20 s | 30 s | about 5 min, with 98% of the compiler's requests answered from its cache | about 3 min 20 s of script suites, 4 min of Rust tests |

What changed, in the order of what it saved:

- The Windows x64 job became four: `build` (the programs and their smoke
  checks, which `linux-import` waits for), `lib-tests` in two halves, and
  `cli-tests`. Each compiles what it needs from the shared dependencies, so
  the tests no longer run one suite after another, nor the import after all
  of them.
- `build-arm64` calls `build-binaries.yml` with `cache: true` and compiles
  only the crates a change touched and those built on them (see [The
  per-commit binary builds](#the-per-commit-binary-builds) for which those
  are). Its Qdrant-under-emulation check needs no build and is the job
  `qdrant-arm64`.
- `ci.yml`'s script suites (deployment, pre-push, worktrees, Ansible,
  releasing) moved from `rust` to `tooling`, which compiles nothing and runs
  beside it.
- No job installs the nightly toolchain to compile with stable, and none
  saves a cache from a group.

A change to `Cargo.lock` still costs a group the dependencies that changed,
and a change to a crate near the bottom (`mash-model`, `mash-storage`) still
compiles nearly all of ours, eight and a half minutes on Windows ARM in the
release profile: that job is then the slowest again.

Not done, and why:

- Caching our own crates for the debug and test builds, as the binary builds
  do. It would save a change that touches a top crate about two minutes in
  each Windows x64 job, for about 1 GB more cache per commit on `main`.
- Building Windows ARM in the debug profile in the queue. It would take two
  minutes instead of up to nine, and would no longer be the build a release
  makes; `release.yml` would be the first to compile that target optimized.
- Pinning `ci.yml`'s nightly. Each new nightly makes the first groups of the
  day compile everything (three to four minutes more) until `ci-compiler`
  has run; a pinned date would make that a deliberate, occasional step, and
  would stop a new nightly's lints from failing unrelated changes.
- Keeping the compiler cache current for our own crates by compiling every
  commit on `main`. Entries for crates a change does not touch would then be
  cache hits in the queue, at about 100 MB of new entries per commit.
- The installed-models cache in `ci.yml` is keyed on `Cargo.lock` and misses
  after every change to it; the miss costs 11 s.

### Deploying and releasing

Deploy and release from a clean tree at `origin/main`, never from a checkout
someone is editing:

```bash
cd "$(tools/worktree.sh deploy-tree)"   # ../mash-worktrees/_deploy, detached at origin/main
bash tools/fleet.sh preflight TARGET...
bash tools/fleet.sh deploy TARGET...
```

`deploy-tree` creates the tree or moves it to the current `origin/main`, and
refuses when it holds local changes. `tools/fleet.sh` builds from the
repository it is run in and checks that `HEAD` did not move during the build,
so nothing else may commit in that tree.

`deploy` does not compile when CI already built the commit. Each push to main
that changes more than `docs/`, `site/` or `raycast/` runs the `Dev artifacts`
workflow, which keeps release-profile `mash` and `mash-server` for Linux,
macOS and Windows for 14 days. `fleet.sh deploy` downloads the ones for its
exact commit, checks them against the checksum the build wrote, and installs
them with the usual steps; with none it compiles and says why.
`deploy --release vX.Y.Z` installs a published release's assets the same way,
and on Macs that ask for it replaces the desktop app from the release's disk
image. `--source-build` always compiles. After installing, a deployment
restarts supervised Mash jobs still running the old binary and lists the
hand-started ones. The details and the inventory settings are in
[deployment/README.md](../../deployment/README.md#deploying-built-artifacts).

A release is one command, also from the deploy tree:

```bash
just release-tools                        # once: the pinned cargo-release
bash tools/release.sh --dry-run 0.2.5     # rehearse in a temporary clone; pushes nothing
bash tools/release.sh 0.2.5
bash tools/release.sh --resume 0.2.5      # after a wait failed or timed out
```

`tools/release.sh` refuses unless the tree is clean, is `origin/main`, and
`ci.yml` and `windows.yml` passed for that commit, the rule `release.yml`
enforces. It then has [cargo-release](https://github.com/crate-ci/cargo-release)
make the "Release X.Y.Z" commit: the workspace version in `Cargo.toml`, the
desktop app's `desktop/src-tauri/Cargo.toml`, and both lockfiles
(`release.toml`; the desktop files move through the replacements in the root
`Cargo.toml`, and crates with their own version carry `release = false`).
Because that commit is new and a tag publishes only a commit CI validated, the
script pushes it as the branch `release/vX.Y.Z` and lands it through the merge
queue like any other change (`tools/merge-queue.sh`). The queue rebases, so
`main` gets a copy of the commit under another name; the script checks that
copy out, confirms `ci.yml` and `windows.yml` passed for it (the queue's runs
count), and only then has cargo-release create `vX.Y.Z` on it, pushes the tag
and waits for `release.yml`. cargo-release's own push step is not used: it
sends the commit and the tag together, to `main`.

When `release.yml` has passed, the script renders the Scoop manifest
(`bucket/mash.json`) from the release's `SHA256SUMS` and lands
"Scoop: mash vX.Y.Z" through the queue as well, which takes a minute or two
because no build reads that file. The release workflow used to push that
commit to `main` itself and no longer can. After a release made without the
script, `tools/release.sh --resume X.Y.Z` lands the manifest.
`tools/release.sh --direct X.Y.Z` pushes both commits straight to `main`
instead, for an administrator in an emergency.

What it leaves to a person, with the commands printed at the end:

- **The notes.** It writes a draft to
  `/home/james/scratch/mash-artifacts/releases/vX.Y.Z-notes.md`
  (`tools/release-notes-draft.sh`: the standard opening paragraph, then the
  commits since the previous tag grouped by the part of Mash they changed) and
  never publishes it. Rewrite it in the reader's terms, then
  `gh release edit vX.Y.Z --notes-file ...`.
- **winget.** It downloads the release run's `winget-manifests` artifact and
  prints how to submit it to `microsoft/winget-pkgs`
  (`packaging/winget/README.md`).
- **The fleet.** `bash tools/fleet.sh deploy --release vX.Y.Z TARGET...`.

When `release.yml` succeeds for a tag, `release-check.yml` installs the
published release the documented ways and runs it:
`gh workflow run release-check.yml -f tag=vX.Y.Z` repeats it for any published
release.

| Job | Runners | What it proves |
|---|---|---|
| `installer` | Linux x64 and ARM64, macOS, Windows x64 (`windows-latest`) and ARM64 (`windows-11-arm`) | `install.sh` or `install.ps1` from `main` installs the tag; `mash` and `mash-server` report its version. On Windows the installed programs must be built for the runner's architecture, because an x64 program also starts on ARM64 and would pass the version check. |
| `homebrew`, `apt`, `scoop` | macOS and Linux; Linux; Windows x64 | The tap, the apt repository and the Scoop bucket offer the tag and install it. |
| `winget-manifests` | Linux | Fills `packaging/winget/*.yaml.in` from the release's `SHA256SUMS`, as `release.yml` does. After a release the templates are the tagged commit's; a manual run uses the branch it was started from (`--ref BRANCH`), which tries a manifest change against a published release before anything is submitted. |
| `winget` | Windows x64 and ARM64 | `winget validate`, then `winget install --manifest` with local manifests enabled, both programs run from the PATH a new terminal would get and checked for the runner's architecture, then `winget uninstall`. |

Every winget command in the `winget` job has its own time limit (12 minutes
inside a 15-minute step), so a command that never returns fails the job and
prints the processes started since it began, instead of holding a runner
until the job is cancelled. The job summary gives each command's duration and
exit code, and winget's own logs are kept as the `winget-logs-RUNNER`
artifacts whether the job passed or not. `windows-11-arm` has no winget, so
the job first installs App Installer there with Microsoft's
`Repair-WinGetPackageManager`. Both runner images already carry the Visual C++
runtime the manifest depends on. On x64 winget finds it and installs nothing;
on ARM64 it still runs Microsoft's runtime installer, which finds the runtime
present and only registers itself. Neither shows how long the dependency takes
on a machine without the runtime.

`--resume` works out where the release stopped from the repository itself (the
release commit, whether main has it, the tag locally and on the remote, the
version in the Scoop manifest on main), so it is safe to run again at any
point. Because it writes the notes draft again, keep an edited draft under
another name until it is published.

## Dependency policy

`deny.toml` is the policy for Rust dependencies, checked with
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny). Install it with
`cargo binstall cargo-deny` and run `just deny` after changing dependencies.

| Check | What fails it | Where it runs |
| --- | --- | --- |
| Licences | A crate with no licence on the allow-list. Only permissive licences are listed; `option-ext` (MPL-2.0) is a named exception | CI, every push |
| Bans | A wildcard version requirement. Duplicate versions of a crate only warn | CI, every push |
| Sources | A crate from anywhere but crates.io | CI, every push |
| Advisories | A RustSec advisory, an unsound or unmaintained crate, or a yanked version | Its own workflow: daily, and when `Cargo.lock` or `deny.toml` changes |

Advisories have their own workflow because a new one can appear without any
commit; it should show as its own failure, not turn an unrelated push red.
When one appears, update the crate if a fixed release exists. Add it to
`ignore` in `deny.toml` only with the reason written beside it, and only when
no fix exists or the affected code is not reachable from Mash.

## Generated SQLite queries

SQLite query models are generated with sqlc 1.31.1 / sqlc-gen-rust 0.1.12.
Generated output is checked in; normal Cargo builds require no generator.
To regenerate as part of a build, install these versions and run
`just CODEGEN=1 build` (add `--release --locked` for release).
`just generate-queries` regenerates without compiling; `just check-generated`
checks for stale output. Generation reads the same SQLite schema SQL used at runtime.
CI checks regeneration against committed output. Dynamic canonical predicates
reuse the generated projection and decoder; schema initialization and writes remain explicit.

## Using Mash in Codex during development

Point Codex at the build output so rebuilding does not require copying or
reinstalling the executable. Build with Cranelift in a stable target directory:

```bash
env CARGO_TARGET_DIR=/absolute/path/to/build cargo build-clif --bin mash
```

Add this to your local `~/.codex/config.toml`, or a trusted project's
`.codex/config.toml`:

```toml
[mcp_servers.mash]
command = "/absolute/path/to/build/debug/mash"
args = ["mcp"]
startup_timeout_sec = 30
tool_timeout_sec = 120
```

Mash uses its normal node configuration, SQLite index and model directory.
To override those paths, put `--config PATH`, `--db PATH` and
`--models-dir PATH` before `mcp` in `args`. Forward the configured token
environment variable using Codex's `env_vars`, or use a local launcher that
loads your private node environment before executing the same build output.
Keep credentials and machine-specific launchers outside Git.

`mash mcp` runs in one process. Restart it after installing a new binary or
changing startup settings, and reload updated plugin instructions. Accepted
mutations are never replayed. Opted-in watching lasts for the process lifetime.

MCP `help` provides live schemas and typed examples without an index or models.
Help/status report the running build revision and contract fingerprint; supply
the installed skill fingerprint as `expected_contract` to detect stale guidance.

The `mash-history` skill's workflow template is `crates/mash-mcp/templates/history_skill.md`.
Its generated files live in `plugins/mash/skills/mash-history/`.
Its small live-help bootstrap and release stamp are generated from Rust with
`just generate-skills`; do not edit the generated `SKILL.md` or `contract.json`.
`just check-skills` checks generation, real argument validation and CLI examples.
Generation compares the contract stamp with the previous release and requires an increased
version in both plugin manifests whenever the contract or workflow changes.
Typed live-help examples also pass the real core validators. Update the installed
plugin and reload its instructions after changing the skill; consult live help
for current parameters instead of duplicating the API reference in the skill.

Development deployment uses `tools/build-dev-linux.sh` to reuse successful gates
for identical source/toolchain/build inputs while still stamping the release
binary for the current commit. Source changes invalidate the gate receipt, and
changes during validation or compilation fail the rollout. The API helper locks
concurrent rollouts and skips restarting an identical healthy release.
Ansible owns fleet selection, builds, API-first rollout, offline skips and
required schema migrations. `mash-fleet probe` verifies the installed build,
contract, query readiness and shared publication over MCP. Ordinary deployment
never ingests history or installs models. A silver upgrade runs one explicit sync
to re-ingest stale interpretation, rebuild local views and publish.

Use `bash tools/fleet.sh deploy shared-api xenia` with a private inventory.
Refresh remains explicit: `bash tools/fleet.sh sync xenia` or
`bash tools/fleet.sh local-views xenia`. See [deployment setup](../../deployment/README.md).
Runner logs and reports stay private. Skipped targets are not installed or ready.

## Work tracking

Agent work is tracked with [Beads](https://github.com/steveyegge/beads). Issue
history lives in a separate, private Dolt remote (`mashhq/mash-beads`), not in this
repository. With `bd` and Dolt installed and access to that remote, run
`bd config set sync.remote git+https://github.com/mashhq/mash-beads.git` and
`bd bootstrap` after cloning, then `bd prime` and `bd ready`. `bd dolt commit` and
`bd dolt push` persist tracking updates. The entire `.beads/` directory, including
configuration, stays local and is ignored by Git.

## Releases and fleet deployment

Ordinary commits do not deploy the fleet. Follow the canonical preflight,
validated build, installation and readiness workflow in
[deployment/README.md](../../deployment/README.md). Database upgrades require a
separate preservation and recovery plan; see [upgrades](../upgrades.md).
