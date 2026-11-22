# Documentation website

The documentation at [getmash.io](https://getmash.io) uses Astro and Starlight,
published as static files on GitHub Pages. Cloudflare manages the domain's DNS.
Internal links use Astro's client router to change pages without a full reload.
Browser back/forward, direct URLs, and ordinary links without JavaScript still
work. Search stays mounted across navigation, preserving its query and shortcut.

## Editing content

Edit the Markdown guides in `docs/`. The homepage is `docs/home.mdx`, and the
quick start comes from the repository README. There is no second maintained
copy: `site/scripts/content.ts` prepares ignored build inputs and rewrites
relative Markdown links for the website. Links to repository source stay on
GitHub. Each generated guide has an edit link to its original file.
The English docs directory at `/guides/` comes from `docs/README.md`.
Each translated directory comes from `docs/<locale>/guides.md`. Its welcome
page comes from that locale's `index.md`.

Add a guide to `docs/README.md` and, when appropriate, the sidebar in
`site/astro.config.mjs`. Keep **Start here** and **Using Mash** as the two
top-level sections. Put related pages under a collapsible topic and use
the same hierarchy in the guide index. New Markdown guides are included
in builds and search automatically. The build includes top-level `docs/*.md` files and Markdown
under configured language folders. Keep engineering and maintenance notes in
`docs/internal/`, outside the website and its search.
Re-run the development command after editing source guides.

## Translate a guide

From `site/`, list the guides that need review.

```sh
pnpm --silent translate-docs --status
pnpm --silent translate-docs --status fr
```

The report covers guides supported by the drafting tool. It shows missing
translations, changed English sources, and translations without a recorded
source revision. It runs offline with no Codex sign-in. The Documentation
workflow includes the same report in its job summary.

With a signed-in Codex CLI, run this from `site/` to draft a French guide.
Use a supported language tag in place of `fr`. The help command lists the
guides that the tool can draft.

```sh
pnpm --silent translate-docs fr quick-start
pnpm --silent translate-docs --help
```

The command sends the selected public English guide as context and returns a
Markdown draft on stdout. It inserts the translated prose into the original
structure and preserves command examples and inline code. It uses the
translation’s spacing around links, emphasis, and inline code. The draft keeps the
English section IDs, so links like `#find-and-read-a-chat` can still be used.
It links to a translated guide when that guide contains the requested section.
Links to missing translations or sections go to the English guide. After a
successful draft, the command lists each distinct English fallback destination
on stderr. Mark those links as English during review. The draft stays on stdout.

Review the prose and links before saving under `docs/fr/` at the repository
root. For an existing translation, use the draft to help update the existing
copy and keep reviewed wording where it still fits. Keep the section aliases
and record the new source marker only after checking the translation against
that English source.

To publish a language, add it to `site/scripts/locales.ts`. Add a welcome
page at `docs/<locale>/index.md` and a guide directory at
`docs/<locale>/guides.md`. For an example, see the [Japanese welcome page](../ja/index.md).
Add sidebar labels in `site/astro.config.mjs` and header labels in
`site/src/content/i18n/` using Starlight's language code. Mark links to
untranslated guides as English. The language picker and sidebar can open
those guides in the locale with an English fallback notice.
Commands and buttons in Mash itself still appear in English.

## The welcome page

The homepage is the hero (`site/src/components/Hero.astro`), the agents' marks
(`Agents.astro`) and everything below them (`Welcome.astro`, styled by
`site/src/styles/welcome.css`).

- **Bands.** Each idea is one full-width band: a stamped label, a heading,
  a few points with icons, and a small window or set of cards drawn the way
  the dashboard draws them (search results, Mashy's answer, a saved
  decision). The windows are live HTML styled by
  `site/src/styles/welcome-panels.css`, each exposed as one image named for
  what it shows. On wide screens the copy sits beside the window and the side
  alternates; on narrow screens the copy comes first. Bands take the theme's
  surfaces (beige, cream, butter, and lilac only for Mashy).
- **Motion** is a gentle fade and rise as a band scrolls into view
  (`site/src/welcome.ts`, an `IntersectionObserver`). Without scripting, or
  when the reader prefers reduced motion, every band is simply shown: the
  stylesheet hides a band only once the page has the `reveal` class and only
  under `prefers-reduced-motion: no-preference`.
- **Agent marks** under the hero are `<img>` logos in ink on ivory tiles
  (`Agents.astro`). Do not draw them as CSS masks through an inline custom
  property: iPhone Safari rendered those tiles empty.
- **The README snapshots** (`assets/dashboard-demo.svg` and
  `site/public/welcome/welcome-mashy.svg`) are SVG exports with outlined text
  from editable Figma frames. Use fictional history from the demo.
- **The performance figures** are measurements, with their conditions folded
  under them (“How these were measured”). The GPU figures come from the README and
  `docs/configuration.md`; DirectML on Windows is on by default, the NVIDIA
  worker and GPU reranking are experimental and opt-in, and the page says which. The 66 ms search was one exact-phrase search timed
  by Mash's own diagnostics on a development machine on 2026-11-20 (1,137
  chats, about 103,000 searchable messages); a common word took 0.8 to 1 s
  there. The 20 MB is the resident memory of idle `mash mcp` processes on the
  same machine that day (16 to 23 MB each; 220 to 330 MB once one had loaded
  the models). The local Qdrant held 2.4 to 2.9 GB for that index of about
  795,000 messages, so the page makes no claim about memory in use overall.
  Re-measure before changing a figure, and keep the small print true.

## Dashboard pictures

The guides show real dashboard pages from the demo's sample history. They are
WebP files in `assets/screenshots/`, committed so the guides render on GitHub
too, and the site build copies them to `/screenshots/`. After a visible
dashboard change, build the demo (`pnpm build:demo` in `dashboard/`), run
`pnpm screenshots` in `site/`, and look at every picture before committing.
`site/scripts/screenshots.ts` lists the pages, hides the demo's own banner
and notes, and crops out the rail where the page itself should be readable.
Write alt text that says what the picture shows.

## Theme

Every colour, line width, corner, shadow and typeface the site draws with is
a role in `site/src/styles/theme.css` (`--surface-page`, `--line-outline`,
`--action-bg`, `--text-muted` and so on), which also sets Starlight's
`--sl-color-*` and `--sl-font*` variables from those roles. The other
stylesheets and the components' styles use the roles only;
`site/scripts/theme.test.ts` fails when a literal colour, typeface, corner or
shadow, or an unknown role, appears anywhere else. To change the look, change
the values in `theme.css` and the bundled fonts listed in
`site/astro.config.mjs`. The dashboard keeps the same arrangement in
`dashboard/src/theme/`.

## Local preview and checks

From `site/`, with Node 24 and pnpm installed:

```sh
pnpm install --frozen-lockfile
pnpm dev
```

Before publishing, run `pnpm lint`, `pnpm check`, `pnpm test`, and `pnpm build`.
Then run `pnpm test:browser` (install Chromium with
`pnpm exec playwright install chromium` on first use).
On Xenia, run builds and tests through `agent-test`. The static output goes to
`site/dist/`; Starlight builds the Pagefind search index during the build.

## Publishing

The Documentation workflow validates pull requests and publishes changes on
`main`. It can also be dispatched manually. A completed Release workflow
rebuilds the site so newly published apt packages are included.

The release workflow owns the signed apt repository on the `gh-pages` branch.
Documentation builds copy that branch's `apt/` directory into the Pages artifact
without modifying it. The documentation workflow never rewrites that branch.
If fetching an existing apt branch fails, publishing fails rather than dropping
packages. Before the first apt release, publishing only the documentation is
expected.

GitHub Pages must use GitHub Actions as its publishing source and `getmash.io`
as its custom domain. DNS points the domain at GitHub Pages; no Cloudflare token
is required in GitHub Actions. Enable HTTPS enforcement once GitHub has issued
the domain certificate.

## The dashboard demo

`getmash.io/demo/` is the dashboard itself, built with `pnpm build:demo` in
`dashboard/` and copied into the site by `pnpm add-demo` in `site/`. The
documentation workflow runs both, so the demo is republished whenever the
dashboard changes.

The demo sends no requests. Its entry point (`dashboard/src/demo/main.ts`)
installs a transport that answers in the browser from a made-up history
(`dashboard/src/demo/history.ts`), in the shapes `mash dash` sends. Nothing
outside `dashboard/src/demo/` knows the demo exists, and the real build
carries none of it.

- **Keeping it honest.** Every demo answer is typed against the dashboard's
  API contract (see below), so a route or field the server changes stops
  `backend.ts` compiling until the demo follows. `backend.test.ts` also runs
  each answer through the code that turns it into what a page shows.
- **Search.** The demo requires each exact word in the same message or an
  exact quoted phrase. The results page describes this behavior.
- **The tool list.** `dashboard/src/demo/tools.json` is a copy of what a
  running dashboard serves at `/api/console/tools`. Refresh it with
  `curl -s http://127.0.0.1:8790/api/console/tools > dashboard/src/demo/tools.json`.
- **History.** Every project, machine and conversation is invented. Never add
  real history to it.

## The dashboard's API contract

`mash dash` describes its own API. Its routes are registered with `aide`
(`crates/mash-dash/src/lib.rs`, `crates/mash-dash/src/services.rs`), which builds the router and an
OpenAPI 3.1 document together from the handlers' extractors and the Rust types
each route answers with. A running dashboard serves the document at
`/api/openapi.json`.

- `dashboard/openapi.json` is the committed copy. The Rust test
  `committed_contract_matches_the_routes` fails when it is out of date;
  rewrite it by running that test with `MASH_WRITE_CONTRACT=1`.
- `dashboard/src/lib/contract.ts` holds the TypeScript types generated from
  it with `pnpm contract` in `dashboard/`. A dashboard test fails when they
  are out of date.
- The dashboard's request functions (`read()` in `dashboard/src/lib/api.ts`,
  `get()` and `post()` in `services.svelte.ts`) take their paths, bodies and
  answer types from those types. Answers are not checked again at run time:
  the dashboard is built into the binary that serves it, so the two share one
  contract. The exception is `pnpm dev` pointed at a `mash dash` from another
  build, where a changed field shows up as a rendering error, not a message.
- What remains by hand turns an answer into what a page shows, and keeps the
  few checks that are about meaning, such as a memory page whose record lacks
  its current revision.
- A test that needs only part of an answer wraps its fixture in `trimmed()`
  (`dashboard/src/lib/trimmed.ts`).
- A type that answers a route needs `schemars::JsonSchema`. Types the MCP
  tools also publish must keep their schema names: renaming one changes the
  MCP contract.
- `aide` is pinned to a pre-release (`0.16.0-alpha.4`), the first line that
  reads `schemars` 1 schemas. Move to the stable release when there is one.

The shared server is described the same way. Its handlers already take and
return typed values, so `aide` reads everything from them. The result is kept
as a reference at `docs/internal/shared-server-openapi.json`; the Rust test
`committed_server_contract_matches_the_routes` fails when it is out of date
and rewrites it with `MASH_WRITE_CONTRACT=1`. The synchronization route
upgrades to a WebSocket, and its frames are outside what the document
describes.

The embedding worker's contract is `deployment/embeddings/openapi.json`,
kept the same way from `crates/mash-search/src/semantic/remote.rs` by
`committed_embedding_worker_contract_matches_the_code`.

The reranking worker's contract is `deployment/reranking/openapi.json`. Its
bodies are binary frames, so the document is written out in
`crates/mash-search/src/semantic/remote_reranker/worker.rs` from the same
constants the code uses; `committed_worker_contract_matches_the_code` fails
when it is out of date and rewrites it with `MASH_WRITE_CONTRACT=1`. The
Python workers' tests check their limits, route and statuses against it.

Several types share a short name, such as `Request` and `Scope`, and the MCP
tools publish them under those names. `app/openapi.rs` gives each a distinct
name inside the generated documents without renaming the types.
