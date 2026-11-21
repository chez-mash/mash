# Contributing

Bug reports, fixes, translations and documentation improvements are welcome.

## Where to start

- **Fix a confusing instruction.** Use **Edit page** on a
  [guide](https://getmash.io/guides/) to open its source on GitHub. Small docs
  fixes can go straight to a pull request.
- **Improve a translation.** The source files for Spanish, French and
  Simplified Chinese live in `docs/es/`, `docs/fr/` and `docs/zh-cn/`.
  Run `pnpm --silent translate-docs --status` from `site/` to see which
  guides need review.
  [Translation workflow](docs/internal/documentation-site.md#translate-a-guide)
- **Fix a bug.** Check the [open issues](https://github.com/mashium/mash/issues)
  for a problem you can reproduce. A short example with made-up chat text is
  enough to get started.

## Report a problem

See [Get help](docs/troubleshooting.md#get-help) for the bug form and which
diagnostics to share. [Report security problems privately](SECURITY.md#report-a-vulnerability).

## Propose a change

For anything larger than a small fix, open an issue first so the approach can
be agreed before you write it. Mash has a deliberately narrow scope: local
agent chat history, searched through one CLI, one MCP server and one
dashboard. Changes that widen that scope are unlikely to be accepted.

## Preview and check the docs

With Node.js 24 and pnpm on your machine, run this from a checkout of Mash.

```sh
cd site
pnpm install --frozen-lockfile
pnpm dev
```

Open the address printed in the terminal and find the guide you changed.
Edit the source in `docs/` or the [README](README.md), then restart `pnpm dev`
to regenerate the preview. The docs site builds without Rust or Qdrant.

From `site/`, run these checks before submitting.

```sh
pnpm lint && pnpm check && pnpm test && pnpm build
```

For changes to the site or demo, see the
[browser checks](docs/internal/documentation-site.md#local-preview-and-checks).

## Build and test code

Use the Rust nightly toolchain from `rust-toolchain.toml` and
[cargo-nextest](https://nexte.st). Start with the tests for the component you
changed. For example, the query unit tests run by themselves.

```sh
cargo fmt --check
cargo nextest run --locked -p mash-query
```

Backend tests need disposable Qdrant. On Unix they can start it with Docker.
To supply a test instance, set both `MASH_TEST_QDRANT_URL` and
`MASH_TEST_QDRANT_GRPC_URL` to that instance. See the
[development checks](docs/internal/development.md#rust-checks) for Clippy,
the focused runner and workspace verification.

For dashboard changes, use Node.js 24 and pnpm.

```sh
cd dashboard
pnpm install --frozen-lockfile
pnpm run lint && pnpm run check && pnpm run test && pnpm run build
```

Use small synthetic chats, private temporary directories and disposable
services for tests. Keep real agent histories and credentials out of fixtures,
logs and screenshots.

## Pull requests

- Keep each pull request to one change, with tests for behaviour it changes.
- Fill in the template with the change, its intent, and the checks you ran for
  it. Include any behavior you could not verify and let the required CI finish.
- Update the guide in `docs/` when you change something a user sees.
- Say which AI tools you used and for what, or "None". AI-assisted changes are
  welcome; you are responsible for understanding and testing what you submit.
- Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
  [Apache License 2.0](LICENSE-APACHE). Unless you say otherwise, anything you
  contribute is offered under both, with no further terms.

When Greptile reviews a pull request, it uses the rules in `.greptile/`.
A maintainer makes the decision.

## Labels

Labels are applied for you; you do not need permission to set them.

| Label | Meaning | Set by |
| --- | --- | --- |
| `awaiting triage` | A new issue no maintainer has read yet | Opening an issue |
| `category:` | The kind of change: `fix`, `feature`, `performance`, `refactor`, `docs`, `tooling` | The box ticked under Intent |
| `area:` | The part of Mash it changes: `cli`, `search`, `history`, `storage`, `server`, `agents`, `dashboard`, `install` | Changed paths |
| `touches:` | A change that needs closer review: `schema`, `contract`, `protocol` | Changed paths |
| `needs:` | Something the description lacks: `intent`, `ai-disclosure` | The description; removed once fixed |

A maintainer's `category:` label is never overwritten. The definitions are in
`.github/labels.tsv`, and `.github/scripts/pr-labels` applies them.
