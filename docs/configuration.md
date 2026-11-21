# Configure Mash

`mash init` creates the configuration file. Run `mash status` to see its
location, or select another file with `mash --config PATH status`. The file
must be TOML. A single machine needs only this:

```toml
version = 1

[providers.codex]
roots = ["/home/you/.codex/sessions"]

[providers.claude-code]
roots = ["/home/you/.claude/projects"]

[index]
since_days = 90
destination = "local"
```

To change history selection, coverage or sharing interactively, rerun
`mash init` in a terminal and choose **Customize setup**. The wizard
preserves your other settings and keeps the previous file as a backup; comments
and formatting are regenerated.

## Choose histories and an age limit

Each provider listed under `[providers]` is indexed. Roots must be absolute
paths to that provider's history, not to the source-code project you want to
search. Use the paths detected by `mash init` or `mash status --installation`;
those above are examples. Original provider files are never modified.

`since_days` defaults to 90. To include all ages, replace it with
`all_history = true`; do not set both. One-off indexing flags such as
`--all-history` override the configured age selection.
[What happens when you shorten the history range](privacy.md#shorten-the-history-range)

## Connected agents

`mash init` records the agents it connected:

```toml
[agents]
connected = ["claude-code", "codex"]
```

`mash status --installation` reports a listed agent as needing repair if its
Mash registration disappears, so remove an agent you disconnect on purpose.
Without this table, an agent is expected only while its own settings register
Mash. `mash init --no-agents` leaves the list as it is.

## Search defaults

```toml
[views.query]
unit = "chats"
limit = 20
intent = "exact-mention"
```

`intent` is `exact-mention` (every word, in any order, best matches first; the
default) or `related-discussion` (by meaning). CLI and MCP arguments override
these for one request. Changes take effect
without rebuilding. To search by meaning, follow
[semantic model setup](indexing.md#enable-semantic-search); changing `intent`
alone does not create dense search data.

## Local embeddings

Semantic search uses the all-MiniLM-L6-v2 model (384 dimensions) for local and
shared search. Indexing and search never download models; run
`mash models install` for the embedding and reranking models. See
[model setup](indexing.md#enable-semantic-search). A server needs the model
assets to rebuild shared views from saved history.

## Embedding inference

Indexing uses the fastest engine available to it, in this order:

1. On an Apple Silicon Mac, Core ML, inside Mash. This is the default there
   and needs no setting.
2. On Windows, your GPU through DirectML, inside Mash. This is the default
   there too; see [below](#gpu-embedding-on-windows).
3. A GPU worker, when you configure one: on Modal, or, experimentally, on
   your own Linux or Windows machine with an NVIDIA card. On Linux, Mash
   cannot use an NVIDIA card without that worker.
4. Otherwise, your CPU.

To send indexing batches to an authenticated Mash embedding worker, add:

```toml
[embeddings]
backend = "remote"
url = "https://YOUR-WORKER.modal.run"
token_env = "MASH_EMBEDDING_TOKEN"
timeout_seconds = 120
```

This sends prepared searchable text to the worker, which may contain sensitive
conversation content. The worker gets no database access. Search-query
embeddings stay local, so semantic search still needs `mash models install`.

- Set the variable named by `token_env` in every process that runs
  `mash index`, including watchers. Use a Modal proxy token in
  `TOKEN_ID.TOKEN_SECRET` form, not your Modal account API key. The value is
  never written to configuration or status.
- `url` must be HTTPS unless the worker is on loopback.
- A failed request returns an error; Mash neither retries it nor switches to
  local inference. A timed-out request may still have incurred GPU cost, so
  check the worker before retrying indexing. Existing matching vectors are
  reused.
- To return to local inference, remove the table or set only
  `backend = "local"`.

The worker must implement Mash's model-specific contract; an arbitrary
embedding API is not interchangeable. Changing the embedding recipe can require
rebuilding search data. The worker runs on Modal. Running it on your own
Linux or Windows machine with an NVIDIA card is experimental. Instructions are
in `deployment/embeddings/README.md` in the source repository. In one test on
a Tesla T4, a worker embedded about 155 messages a second, where the same
machine's four CPU cores managed about 18.

An idle worker can take minutes to start. To keep routine refreshes quick, add
`local_below` to the table:

```toml
local_below = 256
```

Batches with fewer new texts than this use the local model, which needs
`mash models install`; larger ones, such as a first index, go to the worker.
Both produce the same vectors. The maximum is 1024; without the key, every
batch goes to the worker. Each `mash index` result reports the texts the worker
embedded (`remote_inferences`, out of `model_inferences`) and its request time
(`remote_milliseconds`).

## The dashboard assistant

[Mashy](mashy.md) is an optional chat in the corner of the dashboard.
It answers questions about your history using a model service and API key you
provide. On the computer that runs the dashboard, add the service to your Mash
configuration and set the environment variable that holds your API key.

```toml
[assistant]
base_url = "https://openrouter.ai/api/v1"
token_env = "MASH_ASSISTANT_KEY"
model = "anthropic/claude-haiku-5.5"
```

Any service that speaks the OpenAI chat-completions format works: a
provider's own API, a gateway such as [OpenRouter](https://openrouter.ai), or
a model server on your machine. The model must support tool calls and
streaming.

- `base_url` is the API's base; requests go to `chat/completions` beneath it.
  It must use HTTPS, or HTTP on loopback.
- `token_env` names the variable that holds the API key. Set it for the
  process that runs `mash dash`. A dashboard run as a service can instead read
  it from `~/.mash/node.env`, one `NAME=key` line, readable only by you.
- `model` is the model's name as that service knows it.
- `enabled = false` switches Mashy off while keeping the rest of the table.
  It is on when the line is left out.

Mashy is off unless the table is present and the key is set. A key in your
environment does not turn it on by itself. Once the table is there, the
dashboard's **settings** page can switch Mashy off and on and change the
model, choosing from the models the service lists; the service's address and
the key's name are only ever set in this file.

**What is sent.** When Mashy looks something up, the chat text and memories it
reads are sent to that service, with recognizable credentials masked like any
other output. Choose a service you trust with your history. Each question
makes a few requests, which that service bills to your key; a failed request
is never retried.

## GPU embedding on Windows

On Windows, Mash embeds on your GPU while indexing, through DirectML. It works
with any DirectX 12 graphics card (NVIDIA, AMD or Intel), needs nothing
installed beyond the card's normal driver, and is on without any setting. It
has been measured on one machine so far. To keep indexing on the CPU:

```toml
[embeddings]
backend = "local"
accelerated = false
```

- Run `mash models install` once after upgrading, so the model the GPU runs is
  present; it adds about 90 MB to the download on Windows. Until then Mash
  embeds on the CPU and says so.
- `mash index` reports `"embedding_engine": "directml"` when the GPU did the
  work. When Mash cannot use a GPU, it decides so before indexing starts,
  embeds on the CPU, and reports `"embedding_engine": "cpu"` with the reason
  in `embedding_fallback`. A machine with no graphics card, such as most
  virtual machines, reports this on every index; `accelerated = false` stops
  the attempt and the report.
- In one test on a Tesla T4, indexing embedded about 90 messages
  a second, where the same machine's four CPU cores managed about 17. A weak
  integrated GPU may be slower than your CPU; if indexing got slower, turn it
  off.
- DirectML needs the card's ordinary display driver. A data-center card left
  in compute-only mode by its driver is not visible to it; the CPU is used.
- Search-query embeddings stay on the CPU.
- The setting does nothing on other systems.

## Faster reranking on your GPU (experimental)

Searches rerank their best candidates on your CPU by default, on every
machine. Mash can rerank on the GPU instead: through Core ML on an Apple
Silicon Mac, and through DirectML on Windows with any DirectX 12 graphics
card. This is experimental and off unless you turn it on: it has been measured
on few machines, and its setting and behaviour may change.

```toml
[reranking]
backend = "local"
accelerated = true
```

- The first search that reranks starts a background worker and uses the CPU
  meanwhile. Later searches use the worker; nothing needs installing or
  starting by hand.
- The worker leaves by itself after 15 minutes without a search and starts
  again when one needs it, which takes a few seconds.
- On a Mac, the first start ever compiles the model, which takes a minute or
  two and keeps about 2 GB beside the models. While running, the worker holds
  about 1 GB of memory.
- On Windows the gain is smaller than on a Mac or with an NVIDIA worker: in
  one test on a Tesla T4, a search's reranking took 0.2 to 0.6 seconds, where
  the same machine's four CPU cores took 0.4 to 1.1.
- `mash search --diagnostics` reports `reranker_engine` as `remote` when the
  worker answered and `cpu` while it is not up.
- If the GPU cannot be used, searches stay on the CPU and Mash tries again
  after an hour. The worker's messages are in `reranking-worker.log` in the
  models directory.
- The setting does nothing on Linux.

## Remote reranking

To score candidates on an authenticated Mash reranking worker elsewhere, add:

```toml
[reranking]
backend = "remote"
url = "https://YOUR-RERANKING-WORKER.modal.run"
token_env = "MASH_RERANKING_TOKEN"
timeout_milliseconds = 1500
```

This sends the query and the excerpts being ranked to the worker. The worker
runs on Modal. Running it yourself is experimental: on a Linux or Windows
machine with an NVIDIA card, or on an Apple Silicon Mac with
`mash-server --reranking-worker 127.0.0.1:8000`. See
`deployment/reranking/README.md` in the source repository. A worker on the
same machine uses a loopback address such as
`url = "http://127.0.0.1:8000/rerank"`, with the port you started it on. In
one test on a Tesla T4, a worker reranked a search in 30 to 130 milliseconds,
where the same machine's four CPU cores took 0.5 to 2 seconds.

- Set the variable named by `token_env` in every process that searches: MCP
  servers, the dashboard and your shell. A Modal proxy token for the embedding
  worker also works here.
- `timeout_milliseconds` accepts 100 to 10000. A cold worker takes much longer
  than that.
- Keep `mash models install` in place. When the worker does not answer in time
  or the token is missing, that search reranks on your CPU. Failed requests are
  never retried.
- `mash search --diagnostics` reports `reranker_engine` as `remote`, `cpu` or
  `cpu-fallback`, with a `reranker_fallback` reason:

  | Reason | What happened |
  | --- | --- |
  | `timeout` | No reply within `timeout_milliseconds`, typically a worker still starting. |
  | `unavailable` | The connection failed or the worker returned an error. |
  | `credential-missing` | The `token_env` variable is not set in the searching process. |
  | `recipe-mismatch` | The worker's reranking model differs from the installed one. |
  | `invalid-response` | The reply did not match the request. |
  | `input-limit` | The query or candidates exceed the worker's request limits. |

  A worker that is not running reports `unavailable` on Linux. On
  Windows it reports `timeout`, and each search waits that long first.
- When searching by meaning is your default (`intent = "related-discussion"`
  with reranking), MCP and dashboard start, and the agent plugin's
  session-start refresh, send a text-free warm-up request to the worker. With
  word search as the default the worker is left alone until a search by
  meaning needs it. Set `MASH_RERANK_WARMUP=off` to disable warm-ups.

## Memory capture

With the current agent plugin, new outcomes are captured automatically. To
disable capture and explicit backfill:

```toml
[records]
enabled = false
```

Recall of existing memory remains available. The optional `backfill_since` is a
setup-time cutoff in Unix milliseconds; it does not schedule work. Historical
chats are reviewed only during an explicitly requested
[memory backfill](records.md#backfill-earlier-memory).

## Local Qdrant

The defaults are HTTP port 6333 and gRPC port 6334 on localhost. For a service
at another address, edit:

```toml
[qdrant]
url = "http://127.0.0.1:6333"
grpc_url = "http://127.0.0.1:6334"
memory = "durable"
# api_key_env = "MASH_LOCAL_QDRANT_KEY"
```

Both endpoints must reach the same Qdrant 1.20+ instance. If an API key is
required, put it in the named environment variable, not in the URL.

`mash qdrant install` serves the ports configured here, so set them before
running it, or run it again after changing them. It accepts only plain `http`
addresses on this machine without `api_key_env`; any other endpoint is a
service you run yourself.

## Share across your machines

A local-only setup needs no `[shared]` table. To publish as well, follow
[multi-machine setup](shared-index.md), add its connection settings, and set
`[index] destination = "both"`.

Each machine publishes its own indexed history. Use a different node ID and
token on each machine, with the same group ID and server URL.
