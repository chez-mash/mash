# Reranking worker

This optional worker scores search candidates for Mash's contextual reranking
on an accelerator. Run it on Modal, or, experimentally, on hardware of your
own: on a machine with an NVIDIA card, or on an Apple Silicon Mac with the
`mash-server` you already have. All three speak the same
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

Still experimental, and off unless you configure it. It has been run on one
rented machine per system, each with a Tesla T4 and four CPU cores: a search's
reranking took 30 to 130 milliseconds, against 0.5 to 2 seconds on that
machine's CPU, with results in the same order.

The commands below use port 8001 so this worker can run beside the
[embedding worker](../embeddings/README.md), which uses 8000.

### Linux, in a container

This needs Docker with the NVIDIA Container Toolkit; `docker info` lists
`nvidia` among its runtimes when both are in place. Build the image from this
directory. The build downloads the pinned model and takes a minute or two; the
image is about 7 GB, and a running worker never downloads anything.

```sh
docker build -t mash-reranker .
export MASH_RERANKING_TOKEN="$(openssl rand -hex 32)"
docker run --gpus all -p 127.0.0.1:8001:8000 \
  -e MASH_RERANKING_TOKEN -e MASH_RERANKING_REQUIRE_GPU=1 mash-reranker
```

The worker logs `reranking with CUDAExecutionProvider` when it is ready.

### Windows, without Docker

Docker is not the route on Windows: Windows containers cannot use CUDA, and
this image is a Linux one. Run the worker directly instead. It needs a current
NVIDIA driver and [uv](https://docs.astral.sh/uv/); the CUDA libraries come
with the Python packages. From this directory, in PowerShell:

```powershell
uv sync --locked --no-dev --group gpu
uv run --no-dev --group gpu python server.py
$env:MASH_RERANKING_TOKEN = -join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Maximum 256) })
$env:MASH_RERANKING_REQUIRE_GPU = '1'
uv run --no-dev --group gpu uvicorn --factory server:app_from_environment --host 127.0.0.1 --port 8001
```

The second command downloads the pinned model into `model` here, once. The
worker stops when its window closes; keeping it running across logons is up to
you.

### Point Mash at it

Set `[reranking]` in the node's configuration to
`url = "http://127.0.0.1:8001/rerank"` with
`token_env = "MASH_RERANKING_TOKEN"`, and set that variable to the same token
wherever Mash searches. Mash accepts plain HTTP on loopback only; put the
worker behind HTTPS to reach it from another machine.

- The worker refuses to start without `MASH_RERANKING_TOKEN`, and answers only
  requests carrying it as a bearer token.
- `MASH_RERANKING_REQUIRE_GPU=1` makes the worker fail when it cannot use the
  card. Without it, a worker that cannot reach the GPU serves from the CPU and
  looks healthy; its start-up log names the engine in use.
- `MASH_RERANKING_MODEL_DIR` names a model directory other than the default.
- While the worker is stopped, searches rerank on the CPU. On Windows each of
  those searches first waits out `timeout_milliseconds`, because a closed
  local port is not refused at once there.

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
  Later starts load the compiled model in about two seconds. It keeps about
  2 GB of compiled shapes beside the models and holds about 1 GB of memory
  while running. The worker prints its
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
