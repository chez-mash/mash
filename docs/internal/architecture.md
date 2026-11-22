# Architecture

[Documentation](../README.md) · [Project home](../../README.md)

## System map

![Clients reach the mash executable's dashboard, CLI and stdio MCP server, which all call one application core. The core reads agent histories, commits silver SQLite, builds and searches Qdrant views, and runs local models. Optionally it publishes to a mash-server with per-machine silver and shared Qdrant views, sends embedding batches to a GPU worker, and the dashboard assistant asks an OpenAI-compatible model.](../../assets/mash-system-map.svg)

Solid arrows run on every installation; dashed ones exist only when configured.
The user-facing version of this picture is in [How Mash works](../architecture.md).

**Entry points.** One `mash` executable serves three front ends: `mash mcp`
(stdio, `crates/mash-mcp`) for agents, the CLI (`cli/`) for the terminal and
the Raycast extension, and `mash dash` (`crates/mash-dash`) for the browser
and the desktop app, which only loads the dashboard. Each translates its input
into a typed request for the core; none calls another. The optional watcher,
`mash index --watch` (`app/watch.rs`), runs as its own process and sends the
same typed indexing requests; the plugin's SessionStart catch-up is a detached
one-shot `mash index`. The dashboard never indexes itself: its setup page
starts a separate `mash index` process.

**Application core.** `app/core.rs` (`mash-app`) owns every operation and
admits one local writer across a silver commit and its view certification.

**Indexing.** Provider adapters in `crates/mash-silver` read agent histories
(bronze) without writing to them, respecting their WAL files. The core
commits chats, messages and branch ancestry to silver SQLite
(`crates/mash-storage`) and derives turns after every write. It then builds
the Qdrant views (`crates/mash-search`) from silver: dialogue text for BM25
word search and embeddings for meaning. Tool calls stay in silver and are
never indexed. Views are replayable from silver, so they can be rebuilt
without rereading the agents' histories.

**Search.** CLI, MCP and the dashboard read local Qdrant directly, without
HTTP. Word and meaning candidates are fused with RRF and reranked within a
bound. Query embeddings and the default reranker run locally on the models
that `mash models install` downloads; Core ML (Apple Silicon) and DirectML
(Windows GPU) accelerate embedding where available.

**Sharing (optional).** With a shared destination configured, the node-side
facade in `crates/mash-sync` publishes exactly the local index to
`mash-server` (`crates/mash-server`, a separate executable). The server
commits each machine's silver SQLite before synchronizing the shared Qdrant
views, and serves shared candidates that the node fuses with its own.
Publication commits are announced on `/v1/events`, which the dashboard relays
for live updates.

**Optional services.** A configured remote GPU worker can compute embedding
batches, and an opt-in reranking worker (not drawn) can score a reranking
pass, with the CPU as the fallback. The dashboard assistant, Mashy, is the
only part of Mash that calls a language model: an OpenAI-compatible service
named in `[assistant]`, with read-only lookups and the key kept server-side.

## Code organization

CLI and MCP are wrappers around the same typed application core in `app/core.rs`.
The core owns indexing, search, chat listing, view, status, forgetting, silver export/import, and model installation.
MCP calls it directly without constructing CLI arguments or invoking CLI handlers.
The watcher also uses typed core requests.

The shared HTTP service starts through `mash-server` and `cli/server_cli.rs`.
The `mash` executable owns local commands, stdio MCP and `mash dash`; it has no
`serve` subcommand. Both executables use the same library and release version,
including the shared storage and search implementation. This separates process
installation and arguments; it does not split the dependency graph into separately
compiled server and client libraries.

`app/core/models.rs` defines the in-memory requests and serialized responses.
Operation handlers live in `app/core/indexing.rs`, `search.rs`, `chats.rs`, `inspection.rs`
and `maintenance.rs`; each returns its concrete result type. The façade resolves
configuration and paths, dispatches the request and wraps the result once.
Indexing owns per-index provider routing, progress and watcher target selection;
inspection owns read-only status and canonical views. Parsing, SQLite and local
inference run on blocking workers, while shared HTTP operations remain asynchronous.

`crates/mash-search/src/config.rs` owns the typed node settings and status models. Its private
`document` module decodes strict TOML and validates provider routing, roots,
identifiers and endpoints; `loading` handles bounded file reads and platform
defaults. CLI, MCP and watcher operations use the same resolved settings.

`crates/mash-sync/src/shared.rs` is the typed shared-index facade. Its private `client` module
owns pooled HTTP requests, authentication, deadlines and response decoding;
`publication` owns durable acknowledgements, routing ownership and streaming
uploads; `retrieval` owns concurrent local/shared candidates, fusion and contextual
ranking. The HTTP service and concrete database backends remain separate from
those node-side modules. These boundaries use existing concrete types rather
than adding transport or provider abstractions.

`crates/mash-server/src/lib.rs` owns Axum routing, authorization, typed HTTP responses
and the listener lifecycle. Its private `configuration` module loads bounded
strict TOML and resolves authorized node identities; `store` initializes Qdrant; `upload` spools bounded
NDJSON streams. Configuration and credentials are validated before connecting
to storage, and backend initialization completes before the listener opens.



## Modules

CLI and MCP translate inputs into typed application-core requests. Implementation
modules are grouped by responsibility:

- `crates/mash-model/`: the vocabulary every layer shares (roles, providers,
  tool activity, attachments, credential masking, progress events, the typed
  error).
- `crates/mash-silver/`: canonical models, typed provider records, history
  discovery, normalization and source reconciliation. It never opens the
  index; `app/silver.rs` coordinates ingestion and checkpoints, and the adapter
  tests that index through storage stay beside it under `app/silver/`.
  Provider-owned history files are bronze inputs.
  `crates/mash-silver/src/dialogue.rs` interprets visible dialogue independently of retrieval;
  canonical messages retain full context alongside an optional `dialogue_text`.
- `app/silver/local.rs` and `crates/mash-search/src/views/local.rs`: local stage boundaries for
  durable canonical ingestion and rebuildable retrieval state. The application
  core wires those stages together.
- `crates/mash-search/src/views/shared.rs`: node-side embedding and publication-payload materialization
  from interpreted silver chats; source scanning stays in the shared
  publication coordinator.
- `crates/mash-search/src/semantic/`, `search_ranking.rs` and `mash-storage`'s `search_*.rs`: views model execution, embedding batches,
  Core ML embeddings, hybrid retrieval and query/ranking policies.
- `app/shared/snapshot/`: backend-independent publication validation; `layers.rs`
  defines persisted silver history and views embeddings within one atomic revision.
- `crates/mash-search/src/views/qdrant/`: common typed REST reads/setup, gRPC document writes, collection schema,
  and dense/BM25 text retrieval. The engine takes an index scope, independently
  of Axum authentication; shared publication owns coordination around it.
  `Store::open` inspects existing collections without writes; `Store::connect`
  explicitly completes setup and then verifies the same reader contract.
- `crates/mash-storage/src/views/documents.rs` and `crates/mash-search/src/views/retrieval.rs`: common document and
  retrieval contracts, also used by the shared HTTP protocol.
- `crates/mash-search/src/views/materialize.rs`: native silver-to-Qdrant text preparation,
  inference and bounded document writes. Callers own silver snapshots, mutation
  coordination and final certification.
- `cli/tests/`: CLI contract tests grouped by behavior, with common setup in
  `support.rs`. CLI and MCP share marker fixtures from `app/test_support.rs`.

Keep assertions in individual tests. Share setup for repeated fixtures, and
split independent cases so each failure names the behavior that broke.
