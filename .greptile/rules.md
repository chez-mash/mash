# Mash review rules

These rules restate `AGENTS.md`. Mash holds people's private agent chats:
losing, leaking or misreporting that history costs more than a missing feature.

## Findings

- Report a finding only when you can name the input or sequence of events that
  goes wrong. If you cannot, stay silent.
- External input (files, configuration, HTTP, MCP, agent histories) must return
  a typed error and never panic. Flag `unwrap`, `expect` and indexing on data
  that came from outside the process.
- Unsafe Rust is forbidden.

## Product boundary

- Flag changes that add provider traits, registries, plugin systems, a generic
  object model, JSON configuration, alternate output modes or content sources
  other than agent chat history.
- Nothing may download a model or Qdrant outside `mash models install` and
  `mash qdrant install`.
- Tool calls and their outputs are stored for viewing and export but never
  indexed for search.
- The dashboard's history routes are read-only, chat content is rendered as
  text or through the sanitizer, and the assistant has no tool that writes,
  runs commands, reads files or reaches the network.

## Stored data

- A released SQLite migration is immutable. A schema change needs a new
  migration with explicit supported starting versions and tests for an
  interrupted upgrade and for rejecting a newer schema.
- A SQLite change must not invalidate Qdrant data that is still valid.
- Silver is committed before views are synchronised, and views are never
  reported as current unless certified.
- No automatic retry of a Qdrant mutation, a remote inference request or any
  other request whose outcome is ambiguous.

## Scope

- Formatting and lint style belong to `cargo fmt`, Clippy and Biome. Skip them.
- Generated files (`dashboard/openapi.json`, `dashboard/src/lib/contract.ts`,
  lock files) come from their generators. Do not review them, but do flag a
  route or schema change that lacks the regenerated contract.
- A change a user would notice needs its guide under `docs/` updated.
