# Correctness checks

Mash combines bounded [protocol models](../../protocol/README.md), tests
against real storage, targeted failure regressions, and focused mutation audits.
Each checks a different claim; none proves the whole distributed system correct.

The entry points are:

| Check | Command | Setup |
| --- | --- | --- |
| Bounded protocol exploration | `just protocol-check` | Explicit `just protocol-tools` installation |
| Revision/freshness mutation audit | `just mutation-check` | Explicit `just mutation-tools` installation |
| Dashboard service auth/schema audit | `just mutation-services` | Explicit `just mutation-tools` installation |
| Visual baselines of the dashboard and the site | `bash tools/visual.sh` | Podman or Docker, and `node_modules` in `dashboard/` and `site/` |

Mutation audits remain explicit because they rebuild and retest each variant.
On Xenia the just recipes use `agent-test` for heavy work. No check installs
embedding models or uses production backend data.

## Fast loops

Two commands in `dashboard/` run only the tests a change can affect. They are
for the edit loop; neither replaces the checks before a change lands.

```bash
pnpm test:changed    # Vitest: the unit tests that import what changed since origin/main
pnpm e2e:affected    # Playwright: the browser specs for the pages that changed
```

On Xenia run the second through `agent-test` (`agent-test pnpm e2e:affected`),
as for any browser run. Both compare with `origin/main` and count uncommitted
and untracked files. `pnpm e2e:affected --base HEAD~1` compares with another
revision, `pnpm e2e:affected src/components/MemoryRow.svelte` answers for
named files instead, and anything after `--` goes to Playwright
(`-- --headed`). Without `--run`, `python3 tools/affected_tests.py` prints the
selection and the reason for each file and runs nothing.

Vitest follows imports from each test, so `test:changed` needs no upkeep. The
browser specs drive the running dashboard and import none of its code, so
nothing can be read from them. `tools/affected_tests.py` joins two things
instead: the import graph of `dashboard/src` (every `import`, `export … from`
and stylesheet `@import`), and `dashboard/e2e/affected.json`, which states the
component each route mounts and the routes each spec visits. A changed file
selects the specs that visit a route whose component imports it, directly or
through other files.

| Change | Selects |
| --- | --- |
| A spec | That spec |
| A file some routes import | The specs that visit those routes |
| A shared foundation (`everything` in `affected.json`): `src/theme/`, the top-level stylesheets, `App.svelte`, `main.ts`, the rail, top bar and its search, the assistant's button, `src/lib/components/` (the kit and the icon), `lib/api.ts`, `lib/route*`, `src/demo/`, `package.json`, the lockfile, the Playwright, Vite, Svelte and TypeScript configuration, `index.html`, `public/`, the specs' shared helpers and `affected.json` itself | Every spec |
| A file the shell imports (the rail and top bar reach about forty files under `src/lib`, `queries.ts` among them) | Every spec |
| A file no mapped route imports, a spec missing from `affected.json`, an import that resolves to no file, or anything else the tool cannot place | Every spec, with the reason printed |
| A unit test, a `*.live.ts` check, `README.md`, the Biome and shadcn configuration | Nothing |
| A file outside `dashboard/` | Nothing: the specs answer every API call themselves |

Measured on Xenia on 2026-11-26, one file changed each time, against the whole
suite of 225 tests in 32 specs:

| Changed file | Specs | Tests | Time |
| --- | --- | --- | --- |
| The whole suite (`pnpm test:browser`) | 32 | 225 | 5.8 to 7.2 min |
| `src/components/MemoryRow.svelte` | 16 | 131 | 3.0 min |
| `src/components/SearchRow.svelte` | 11 | 94 | 2.2 min |
| `src/components/ServiceRows.svelte` | 7 | 40 | 1.3 min |
| `src/theme/tokens.css` | 32 | 225 | 7.2 min |
| `e2e/quick.spec.ts` | 1 | 13 | 26 s |

The machine was busy with other sessions' tests, so the times are slower than
an idle run and comparable only with each other. Of the 227 source files, 128
select part of the suite (ten specs at the median) and the rest select all of
it.

The whole suite is still required:

- before landing a change that the table says selects every spec. The command
  runs the whole suite for those by itself, so this only means: do not skip it;
- before landing a change to a spec's routes that `affected.json` does not
  yet describe;
- always in CI. `frontends.yml` runs every spec, in four shards, on every push
  to `main` and every pull request that touches `dashboard/`, whatever this
  selection says. The selection is never used there.

`tools/pre-push.sh` runs no browser spec, selected or not.

### Keeping `affected.json` true

The file is reviewed by hand, and the selection is only as good as it is.

- **A new spec** must be added under `specs` with every route it reaches, by
  address (`goto("/#/services")`) or by clicking through to it. Until then
  `tools/test_affected_tests.py` fails and every selection is the whole suite.
- **A spec that starts visiting another route** needs that route added. No
  test can see this, so check it when reviewing a spec change. Listing a route
  too many only costs time; a missing one skips the spec when it matters.
- **A new route** needs an entry under `routes` naming the components
  `App.svelte` mounts for it; a test compares the list with the router's.
  `assistant` and `sign-in` are not addresses: they stand for the assistant's
  panel and the sign-in screen, which `App.svelte` mounts outside the router.
- **A component `App.svelte` mounts on every page** belongs under `shell`, and
  under `everything` if its own change should select every spec.
- Names under `everything` and `ignored` are patterns in which `*` also
  crosses directories.

The first version was taken from a run of the suite that recorded, for each
spec, the addresses the page went to and the source files whose code ran, and
then checked that changing any file a spec had run would select that spec.
Routes reached only by clicking came from that run, not from reading the
specs.

## Replay interruption

`qdrant_replay_interruption_never_leaves_current_metadata_and_retry_repairs`
uses disposable Qdrant and per-store test hooks after invalidation and document
deletion. A fresh service handle must report stale views, refuse dense search,
preserve silver, and repair through replay. It is included in the existing CI
Qdrant test selection. It does not terminate a process or simulate power loss;
the separate persisted-restart contract covers process termination.

Run locally through `agent-test`. With Docker available, each test starts a
disposable, memory-limited Qdrant 1.20 container itself; otherwise set
`MASH_TEST_QDRANT_URL` and `MASH_TEST_QDRANT_GRPC_URL` to one you run (see the
development guide):

```bash
agent-test env TMPDIR=/home/james/scratch/achi-tmp \
  CARGO_TARGET_DIR=/home/james/scratch/achi-targets/integrated \
  CARGO_INCREMENTAL=1 RUSTC_WRAPPER= \
  cargo test-clif --locked --lib --run-ignored only \
  -E 'test(qdrant_replay_interruption)' --no-tests fail --test-threads 1
```

The local release validation runner limits the broad nextest suite to four
concurrent tests, as the focused runner does. SQLite-heavy tests otherwise
compete for disk access at host CPU-count concurrency and can exceed their
existing deadlines even inside the per-job memory budget.

## Direct local Qdrant acceptance

Local CLI/MCP calls Qdrant directly; it does not start an Axum service. Run the
local lifecycle tests against a disposable Qdrant: a container each test
starts when Docker is available, or one instance named by both
`MASH_TEST_QDRANT_URL` (REST reads/setup) and `MASH_TEST_QDRANT_GRPC_URL`
(point mutations):

```bash
agent-test env TMPDIR=/home/james/scratch/achi-tmp \
  CARGO_TARGET_DIR=/home/james/scratch/achi-targets/integrated \
  CARGO_INCREMENTAL=1 RUSTC_WRAPPER= \
  cargo test-clif --locked --lib --run-ignored only \
  -E 'test(local_qdrant) or test(qdrant_status_rejects)' \
  --no-tests fail --test-threads 2
```

These tests cover committed silver before replay, stale status, hidden/deleted
records, failed inference, and namespace isolation after
copying or replacing silver. Public CLI/MCP acceptance adds first-run setup,
readiness, pagination, evidence, privacy, and streamed rebuild progress. Tests
using real embedding models require an explicit `MASH_TEST_MODELS_DIR`.

The former SQLite vector scan and ANN replay benchmarks were removed with
SQLite gold. Their historical captures do not describe the Qdrant implementation.
Measure end-to-end indexing and search separately from transport benchmarks;
transport byte counts and latency alone do not establish indexing throughput.

## Mutation audits

[cargo-mutants](https://mutants.rs/) changes expressions or function bodies in
an isolated copy and asks whether selected tests reject the changed behavior.
The default scope is silver revision validation and provider
timestamp-provenance validation, with their corresponding tests
in `.cargo/mutants.toml`. The enrichment selection is limited to
`Timestamps::is_current_for`; Codex paginated-history tests cover native metadata
origins alongside the existing JSONL message-boundary evidence tests.
This is a narrow test-quality audit, not a workspace coverage score.

```bash
just mutation-tools                  # Explicit pinned Linux x86_64 download.
bash tools/mutation-check.sh --list  # Review scope without building.
just mutation-check                  # Baseline first; then isolated mutations.
```

For native chat certification, set the disposable `MASH_TEST_QDRANT_URL`, review
`bash tools/mutation-check.sh --list-certification`, and run
`just mutation-certification`. Its separate scope covers `Store::certify` with
real-backend coverage-merge, stale-head and parallel-baseline regressions, plus
common Qdrant freshness predicates and local reader identity checks. Set
`MASH_MUTATION_DIFF` to an existing unified diff to bound any audit to changed
hunks; the same filter applies to its list mode. Retain that diff with the
results and check that the selected list is nonempty. This does not disable
the baseline, source isolation, job limits or timeouts.

The installer pins cargo-mutants 27.1.0 and verifies its release archive SHA-256.
Requires Bash, curl, tar, sha256sum, flock, the repository nightly with Cranelift,
and cargo-nextest. On Xenia `just` runs the audit through `agent-test`. Audits use
one mutant at a time and four build tasks, a ten-minute build deadline and a
two-minute test deadline per scenario. No automatic download happens during a
check. Artifacts and a persistent build cache stay outside Git under
`/home/james/scratch/mash-artifacts/mutations` on Xenia or the user cache elsewhere;
`MASH_MUTATION_ARTIFACTS` overrides this. A lock serializes access to that cache.

Inspect the generated `mutants.out` outcomes, logs and diffs. A survivor is a
question to investigate: is a contract assertion missing, is the selected test
set wrong, or is the mutation behaviorally equivalent? Add meaningful regression
assertions for real gaps. Review unviable mutations and timeouts separately;
they do not demonstrate detection. Keep baseline failures visible. Changing the
scope requires reviewing the selected tests too. Run audits manually for relevant
changes or periodic test reviews; ordinary CI runs the unit and backend
regressions without paying mutation build costs on every push.

## Visual baselines

The dashboard's and the site's main pages, and every story in the
dashboard's Storybook, are kept as pictures in the repository, and a test compares what the pages look like now with those
pictures (Playwright's `toHaveScreenshot`). A page that drifts, such as a
changed colour, a moved control or a font that stopped loading, fails that
test instead of waiting for someone to notice. The pictures show the layout
and the theme; the ordinary browser tests still own behaviour.

### What is covered

| | Pages | Variants | Pictures |
| --- | --- | --- | --- |
| Dashboard (`dashboard/visual/`) | Overview, search results for "checkout", the chat list, a chat from its top, a chat with subagents, memory, a record, machines, a machine, agents, an agent, projects, a project, services, settings, tools, export, the fleet manager, viewing history, setup's review step, the quick panel empty and typed, and three open states: a filter menu, the viewing switcher and Mashy's panel | 1280×860 and 390×844 | 50 |
| Stories (`dashboard/visual/stories.spec.ts`) | Every story in the static Storybook, found through its `index.json`: foundations, kit controls, building blocks and composed pieces, each in its states | 1280×860; a story tagged `phone` at 390×844 instead | 1 per story |
| Site (`site/visual/`) | The landing page (the hero, the header island once the page has scrolled, each band at rest, each surface of the showcase, and the pinned quick search panel before and after its scene), a guide (Search across your machines), downloads, supported agents and the 404 page | 1440×900 and 390×844 | 52 |

The dashboard pictures come from the demo build (`pnpm build:demo`) and its
made-up history, so they need no index and no `mash dash`. The story
pictures come from the static Storybook (`pnpm storybook:build`), which
`tools/visual.sh dashboard` builds after the demo and serves beside it; they
are in `baselines/stories`, named by the
story's id. A story's picture is of its whole page, or of the screen when it
is tagged `screen` (a dialog, a menu, the rail, Mashy). A new story is
pictured without being named anywhere: add it, run
`bash tools/visual.sh dashboard --update -- --project stories`
and commit its picture. A picture is of
the screen, not of the whole page: apart from the landing page, which is
taken band by band to its footer, what is below the first screen is not
compared.

### Run it

```bash
bash tools/visual.sh              # compare both; `pnpm visual` in dashboard/ or site/ compares one
bash tools/visual.sh all --update # take every picture again; `pnpm visual:update` for one package
bash tools/visual.sh dashboard -- -g chat   # only the pictures whose name matches
bash tools/visual.sh dashboard -- --project stories   # only the stories
```

It needs `node_modules` in the package (`pnpm install --frozen-lockfile`) and
Podman or Docker. Both packages together take about five minutes on Xenia,
builds included; the stories' two pictures each are most of that. It is not part of `tools/pre-push.sh`, which stays at a few
seconds; CI runs it.

Fonts and antialiasing differ between systems, so a picture taken on a Mac
never matches one taken on Linux. The browser therefore always runs in the
Playwright container image for the installed Playwright version
(`mcr.microsoft.com/playwright:vVERSION-noble`), locally and in CI, without
network access and with a memory limit. The package is built on the host
first; only the browser and the preview server run in the container. The
Playwright configurations (`playwright.visual.config.ts`) refuse to run
outside it. Upgrading `@playwright/test` changes the browser and so the
pictures: update the baselines in the same change.

### What keeps a picture the same

A picture is only worth comparing if nothing in it changes by itself:

- The clock is fixed. The demo dates its history from the browser's clock,
  so "11m" and "Good evening" stay as they are. Locale and time zone are
  fixed too, at one device pixel per CSS pixel.
- CSS animations and transitions are off and the caret is hidden. Fonts and
  images have arrived before a picture is taken.
- A story is pictured once Storybook says it has drawn and its `play`
  function has finished; a story that throws, or whose `play` does, fails
  instead of being pictured. A menu or dialog that is open from the start
  lets go of the focus first (`src/stories/play.ts`), so it is drawn as the
  pointer opens it, without the ring a script's focus gets in a browser
  nobody has clicked in.
- The landing page's scenes are scripted motion, which no setting switches
  off. Its test installs a stopped clock before the page loads and plays
  each scene to its end by moving that clock, band by band; the pinned quick
  search panel is taken at the start of its hold and near its end. A band is
  pictured only once none of its parts still holds the opacity or the place a
  scene gave it, and is played again if a scene missed its scroll. A band
  taller than the screen is taken a screen at a time (`landing-mashy.png`,
  `landing-mashy-2.png`): a picture of a whole tall element makes Playwright
  resize and scroll the page, and the scenes answer both by starting over.
- What is not Mash's own is left out: scrollbars, the demo's banner, and the
  build and contract hashes on the tools page (masked, drawn as a pink box).

The container gives the same pixels on every run, so the tolerance only
absorbs rounding: a pixel counts as changed past a colour distance of 0.02
(a few levels of one channel), and a picture fails once forty of its pixels
changed. A slightly retinted surface or one changed icon fails. Do not loosen
either number to get a picture through. When adding a page, run
`bash tools/visual.sh all --exact` a few times: it allows no difference at
all, and a picture that fails it has something in it that moves.

### When a change is meant to look different

1. Run `bash tools/visual.sh all --update` (or `pnpm visual:update` in the
   package). A full update removes the old pictures first, so a renamed page
   leaves nothing behind.
2. Look at the changed pictures (`git status`, then open them, or compare in
   the pull request). Every changed picture should be explained by the
   change you made.
3. Commit the pictures with the change.

Without a container at hand, run the **Update visual baselines** workflow on
the branch (`gh workflow run visual-update.yml --ref BRANCH`): it takes the
pictures again in CI and commits them to that branch. A commit made by a
workflow does not start other workflows, so the comparison runs again with
the next push.

Never update the pictures to silence a difference you cannot explain. That
difference is the finding.

### Reading a failure

Locally the script names the report: open
`PACKAGE/visual/report/index.html`, or look in `PACKAGE/visual/results`,
where each failed picture has an `-expected`, an `-actual` and a `-diff`
image (the changed pixels in red). In CI the dashboard is compared by the
**Visual baselines** workflow, when the dashboard changed since that workflow
last passed, and the site by the `visual` job of **Documentation**, under
that workflow's path filter; a failed run keeps the same report and pictures as an
artifact named `dashboard-visual-differences` or `site-visual-differences`.
The site's comparison does not hold back publishing.

A difference in one picture usually means the page changed. The same small
difference in every picture points at something shared: a token, the font,
the rail, or a new Playwright version.

The pictures are ordinary files in Git, about 13 MB. Each update adds the
changed pictures to the history, so keep updates to deliberate changes.
