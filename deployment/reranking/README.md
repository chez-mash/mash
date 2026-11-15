# Reranking worker

This optional worker scores search candidates for Mash's contextual reranking
on an accelerator. Run it on Modal, or, experimentally, on hardware of your
own: as a container on a machine with an NVIDIA card, or on an Apple Silicon
Mac with the `mash-server` you already have. All three speak the same
contract, published in [`openapi.json`](openapi.json), and Mash cannot tell
them apart. Local CPU reranking is the default and the offline path, and
nothing here is used unless you configure it.

## On Modal

It is a separate Modal app from the embedding worker, so searches never queue
behind indexing batches. Deploying it builds an image and downloads the pinned
model; invoking it starts billable GPU work.

From this directory, using your existing Modal account configuration:

```sh
uv sync --locked
uv run modal deploy modal_worker.py
```

Use a Modal proxy token (the embedding worker's token works) as the
`token_env` variable, and the emitted `rerank` endpoint URL in the node's
`[reranking]` configuration (see `docs/configuration.md`). Keep the account API
credential used for deployment separate from this invocation token. See
[Modal proxy authentication](https://modal.com/docs/guide/webhook-proxy-auth).

## On your own machine (experimental)

The container has been built and timed on a cloud GPU, not yet run with
Docker on a machine of our own. Expect rough edges and tell us what you find.

This needs Docker with the NVIDIA Container Toolkit. Build the image from this
directory; the build downloads the pinned model, and a running worker never
downloads anything.

```sh
docker build -t mash-reranker .
export MASH_RERANKING_TOKEN="$(openssl rand -hex 32)"
docker run --gpus all -p 127.0.0.1:8000:8000 -e MASH_RERANKING_TOKEN mash-reranker
```

Point the node's `[reranking]` at `http://127.0.0.1:8000/rerank` with
`token_env = "MASH_RERANKING_TOKEN"`. Mash accepts plain HTTP on loopback only;
put the worker behind HTTPS to reach it from another machine.

- The worker refuses to start without `MASH_RERANKING_TOKEN`, and answers only
  requests carrying it as a bearer token.
- Its start-up log names the engine in use. Without a usable GPU it serves from
  the container's CPU; set `MASH_RERANKING_REQUIRE_GPU=1` to make it fail
  instead.
- `MASH_RERANKING_MODEL_DIR` names a model directory other than the one in the
  image.

## On an Apple Silicon Mac (experimental)

Core ML reranking has been measured on one Mac so far. It is off unless you
turn it on, and its setting and behaviour may change.

For your own searches on a Mac, set `accelerated = true` under `[reranking]`
(see `docs/configuration.md`): Mash then starts and stops a Core ML worker by
itself. The command below runs the same worker by hand, at an address and with
a token you choose, for example to put it behind HTTPS for other machines.

`mash-server` ships with Mash and can serve reranking through Core ML. It
needs the models from `mash models install` and nothing else.

```sh
export MASH_RERANKING_TOKEN="$(openssl rand -hex 32)"
mash-server --reranking-worker 127.0.0.1:8000
```

Point the node's `[reranking]` at `http://127.0.0.1:8000/rerank` with
`token_env = "MASH_RERANKING_TOKEN"`, and set the same variable wherever Mash
searches.

- The first start compiles the model for Core ML, which takes a minute or two.
  Later starts load the compiled model in about twenty seconds. It keeps about
  2 GB of compiled shapes beside the models and holds a similar amount of
  memory while running. The worker prints its
  address when it is ready, and searches rerank on the CPU until then.
- One search process would pay that loading time on every run, which is why
  Core ML reranking lives in a worker and not inside `mash search`.
- If Core ML fails, the worker logs it and serves from the CPU.
- It listens on this machine only. On other platforms the same command serves
  from the CPU, which is useful for testing and no faster than searching
  without a worker.

## Limits and contract

The Modal worker permits one T4 (or L4) container, no warm pool, a 60-second
invocation timeout and no automatic retries. It scales down after 15 idle
minutes, longer than the plugin's ten-minute catch-up interval, and Mash sends
a text-free warm-up when a search is likely. Each request holds one query of at
most 16 KiB and at most 64 documents totalling one MiB of UTF-8 text. Request
bodies and errors never log or echo chat text.

`openapi.json` is generated from the Rust worker and checked against the
Python one by its tests. Requests and replies use versioned binary frames (`MRK1`/`MRR1`); an empty
batch with an empty query is a warm-up and runs no inference. Replies carry one
little-endian f32 raw logit per document ID. Requests above one kilobyte are
zstd compressed and marked `Content-Encoding: zstd`.

The recipe pins `jinaai/jina-reranker-v1-turbo-en` at the revision Mash
installs, FP32 ONNX inference, FastEmbed's pair tokenization (512-token
longest-first truncation, batch-longest padding) and the first logit. Mash
refuses the worker when its installed reranker revision differs, and validates
every reply before using it. On Modal, CUDA execution is required; the worker
fails rather than falling back to CPU.

Local checks do not invoke Modal or need a GPU. Set `MASH_TEST_RERANKER_DIR` to an
installed reranker snapshot to also check tokenization and CPU scoring:

```sh
uv run ruff check .
uv run ty check .
uv run pytest -q
```
