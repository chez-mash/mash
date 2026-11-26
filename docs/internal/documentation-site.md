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

- **The header** is an island on this page: a rounded bar fixed at the top,
  inset from the screen's edges (and from an iPhone's safe area), on a surface
  that is all but opaque (94%), so its words keep at least 4.5:1 and nothing
  under it reads as broken words, whether or not the browser blurs what is
  behind. `custom.css` places Starlight's header and `Header.astro` draws the
  bar. Once the hero has scrolled away the page's surface covers the strip
  above the island and fades out below it: the bands' windows and cards start
  at the island's own gutter, and without that cover a rounded corner
  scrolling under showed around the island's corner as a stray curve
  (`site/tests/island.spec.ts` reads those pixels). A phone's island is the
  logo alone; once the hero's buttons have scrolled away a short Download
  button fades in, with the hero button's target. Links into the page stop
  under the island (`scroll-padding-top`), and the pinned panel holds below it.
- **The surface showcase** (`Surfaces.astro`, one file per surface in
  `components/surfaces/`) is the first band: one of Mash's surfaces in its
  window (the dashboard, the menu bar panel, the Raycast extension, an
  agent's session with the plugin, the terminal), a dock of icon buttons
  that chooses it, and a caption with a link. The dock is a tablist. As
  served the first surface shows and each button is a link to that
  surface's guide; `site/src/surfaces.ts` turns the links into tabs (click,
  arrow keys, Home and End, a sideways swipe on the picture) and changes
  the surface every five seconds while the picture is on screen and not
  pointed at or focused. A choice rests that for twenty seconds. Every surface lies
  in one cell and every caption in another, so the band keeps its height.
  Each surface shows a different moment from the bands below it: the menu
  bar panel before a search, where the quick search band shows one typed.
  Add a surface by adding its component and its entry in `Surfaces.astro`;
  mark what should type itself `data-typed` and what should rise `data-in`.

- **Bands.** Each idea is one full-width band: a stamped label, a heading,
  a few points with icons, and a small window or set of cards drawn the way
  the dashboard draws them (search results, Mashy's answer, a saved
  decision). The windows are live HTML styled by
  `site/src/styles/welcome-panels.css`, each exposed as one image named for
  what it shows. On wide screens the copy sits beside the window and the side
  alternates; on narrow screens the copy comes first. Bands take the theme's
  surfaces (beige, cream, butter, and lilac only for Mashy).
- **Motion** is nine small scenes built around the product windows
  (`site/src/motion.ts`, with GSAP and ScrollTrigger), drawn first as the
  motion storyboard on the design canvas. `site/src/welcome.ts` fetches that
  script on this page only, after the page is drawn; no other page loads GSAP.
  - The hero's app icon settles and its headline rises a line at a time on
    load; the butter streaks drift a few pixels.
  - The quick search panel pins for a screen and a half while the scroll
    types “refresh token”, brings the results in and moves the selection down
    two rows. This is the only pinned scene, and only on a screen at least
    64rem wide and 47.5rem tall with a mouse.
  - The search results arrive and the highlight sweeps each match; the saved
    decision slides out from under the message it came from; the agents come
    in one by one; the figures count up; Mashy bobs and types his first
    answer; the closing card's icon rises as the hero's did.
  - The surface showcase plays each change (`motion/surfaces.ts`): the
    picture slides out the way the change goes and the next slides in, its
    search types and its rows rise, the chosen dock button grows and lifts,
    and the tag glides to it. Without the motion the change is simply made.
  - Every other band (`data-reveal`) fades up as it scrolls into view and
    gets the `shown` class.

  The rules the scenes keep:
  - The page is complete as served. The script sets each scene's starting
    state, and a scene already on screen when it starts is left alone. The
    stylesheet hides nothing, with one exception: an inline script in
    `Hero.astro` marks the hero `data-motion="pending"` so its copy does not
    show and then hide, and drops the mark after three seconds if the motion
    has not taken over.
  - The motion plays for everyone, including readers who prefer reduced
    motion, so it stays small: no flashing, no parallax, no zooms. (Mashy's
    own carousel still waits for a click under that preference.)
  - Scenes move transform, opacity, clip-path and background-size only, and
    hand each element back to the stylesheet when they finish. Nothing
    changes place: typed text keeps its full width, a counting figure keeps
    its final width, and the quick search list keeps its height while empty.
  - Phones and narrow screens get entrances only: no pin, no loops, no
    typing in Mashy's answer. The quick search panel plays once there.
  - Only the streaks and Mashy's bob loop, and both stop off screen. The
    showcase's own changes also stop off screen, on phones too.
  - The pinned panel adds its scroll distance to the page, so a page opened
    at a link below it goes to the link's target again once the pin exists.

  `site/tests/landing.spec.ts` scrolls the page on a wide screen and a phone
  and checks that every scene ends as the page is served and adds no layout
  shift.
- **Agent marks** under the hero are `<img>` logos in ink on ivory tiles
  (`Agents.astro`). Do not draw them as CSS masks through an inline custom
  property: iPhone Safari rendered those tiles empty.
- **The README pictures** are the current light design. The opening picture
  is `assets/screenshots/dashboard.webp`, taken with the other dashboard
  pictures below. `assets/mashy-answer.webp` is the landing page's Mashy
  window, captured from a site build at 2x in the light scheme (the
  `.mashy-window` element at a 1440 px viewport); capture it again when that
  window changes. `site/public/og.png`, the social preview, is the top of the
  landing page at 1440 by 756 in the light scheme with a Mac user agent,
  scaled to 1200 by 630; capture it again when the hero changes. Use fictional history from the demo.
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
and notes, and crops out the rail and the top bar where the page itself
should be readable.
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

There are two themes, light and dark, with the same roles. `theme.css` sets
the light values on `:root` and the dark ones under `[data-theme="dark"]`;
Starlight's theme menu in the header offers Auto (follow the system, the
default), Light and Dark, and remembers the choice. The typefaces are Geist and
Geist Mono, bundled through `@fontsource`.

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

## Review with Agentation

`just site-review` starts the dev server with the
[Agentation](https://agentation.com) toolbar, so a reviewer can click an
element, leave a note, and submit a batch of notes to an agent. It runs
`agentation-review` (configured in `site/agentation-review.config.mjs`),
which starts three things:

- the Astro dev server at `http://127.0.0.1:4331`;
- the Agentation sync server on port 4757, which stores the notes (reused
  when it is already running);
- the review supervisor on port 4858, which receives a submitted batch and
  runs a Claude Code or Codex worker on it in the repository. Pick the worker
  in the toolbar. Workers do not commit.

The toolbar exists only in `astro dev`: `site/src/agentation.ts` adds it and
the proxy to the two services for the dev command alone, so `pnpm build` and
the published site carry none of it. The ports differ from the Agentation
defaults (4747 and 4848) so the gohawk site's review can run alongside.
Starting the review first stops any background Astro dev server of this site.

A worker edits site files while the reviewer may still be typing the next
note, and Astro answers many edits with a full page reload. Two parts of the
toolbar keep the note:

- The open note's text, the element it is attached to and the page URL are
  saved in the browser's `localStorage` on every keystroke. After a reload, a
  tab crash or a closed tab, the same page opens the note again on the same
  element with the text intact. The saved draft is cleared when the note is
  added or cancelled. Notes already added are kept by Agentation itself. Only a
  note opened by clicking an element is reopened; an edit of an existing note,
  a multi-select note and an area selection are not saved.
- While any tab has a note open, the dev server holds full reloads and sends a
  single one when the last note is added or cancelled, its tab closes, or no
  one has typed in it for two minutes (`AGENTATION_REVIEW_DRAFT_HOLD_MS`
  changes this). CSS and script updates still apply while a reload is held.

Both live in `agentation-review`: `site/src/agentation.ts` adds its
`holdReloadsWhileDrafting()` Vite plugin and mounts the toolbar with
`import.meta.hot`. `pnpm test:review` in `site/` checks them in Chromium: it
serves the one-page site in `site/tests/review/fixture/` from a fresh copy on
port 4339, with a six-second hold, edits only that copy, and answers the
review services in the browser. It never changes a file under `site/src` and
sends no notes, so a running review is not reloaded.

On Xenia the review is also served on the tailnet, for a phone or laptop,
at `https://xenia.taild75e12.ts.net:8449/`, by this one-time setting:

```sh
tailscale serve --bg --https=8449 http://127.0.0.1:4331
```

An agent can also read the notes through the Agentation MCP server. On Xenia
the sync server runs permanently as the user service `agentation-mash`
(`~/.config/systemd/user/agentation-mash.service`), and Claude Code in this
repository has an `agentation-mash` MCP server that reads it:

```sh
claude mcp add --scope local agentation-mash -- node \
  "$PWD/site/node_modules/agentation-mcp/dist/cli.js" \
  server --mcp-only --http-url http://127.0.0.1:4757
```

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
