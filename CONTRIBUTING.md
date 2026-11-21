# Contributing

Bug reports, fixes and documentation improvements are welcome.

## Report a problem

See [Get help](docs/troubleshooting.md#get-help) for the bug form and which
diagnostics to share. [Report security problems privately](SECURITY.md#report-a-vulnerability).

## Propose a change

For anything larger than a small fix, open an issue first so the approach can
be agreed before you write it. Mash has a deliberately narrow scope: local
agent chat history, searched through one CLI, one MCP server and one
dashboard. Changes that widen that scope are unlikely to be accepted.

## Build and test

You need the Rust nightly toolchain that `rust-toolchain.toml` selects,
[cargo-nextest](https://nexte.st), and Node.js 24 with pnpm for the dashboard.

```sh
git clone https://github.com/mashium/mash.git
cd mash
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run
```

Dashboard changes:

```sh
cd dashboard
pnpm install --frozen-lockfile
pnpm run lint && pnpm run check && pnpm run test && pnpm run build
```

Documentation lives in `docs/` and is published to
[getmash.io](https://getmash.io); the site is generated from those files, so
edit them rather than anything under `site/src/content`.

## Pull requests

- Keep each pull request to one change, with tests for behaviour it changes.
- Fill in the template: what changed, which kind of change it is, and what
  you ran. Formatting, Clippy and the test suite must pass.
- Update the guide in `docs/` when you change something a user sees.
- Say which AI tools you used and for what, or "None". AI-assisted changes are
  welcome; you are responsible for understanding and testing what you submit.
- Mash is licensed under either the [MIT licence](LICENSE-MIT) or the
  [Apache License 2.0](LICENSE-APACHE). Unless you say otherwise, anything you
  contribute is offered under both, with no further terms.

An automated reviewer (Greptile) comments on new pull requests using the rules
in `.greptile/`. Its comments are advice; a maintainer makes the decision.

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
