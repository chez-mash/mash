# Embedding worker

This optional worker generates indexing vectors for Mash on a GPU. Run it on
Modal, or, experimentally, on a machine of your own with an NVIDIA card; both speak the same contract, published in [`openapi.json`](openapi.json), and Mash
cannot tell them apart. Query inference remains local, and no deployment or
model download happens on ordinary Mash startup.

## On Modal

Deploying builds an image and downloads the pinned model; invoking it starts
billable GPU work.

From this directory, using your existing Modal account configuration:

```sh
uv sync --locked
uv run modal deploy modal_worker.py
```

Create a dedicated proxy token through Modal's workspace settings. Export its
ID and secret joined by a period as `MASH_EMBEDDING_TOKEN` in the indexing process.
Use the emitted endpoint URL in the node's `[embeddings]` configuration. Keep the
account API credential used for deployment separate from this invocation token.
See [Modal proxy authentication](https://modal.com/docs/guide/webhook-proxy-auth).

## On your own machine (experimental)

Still experimental, and off unless you configure it. It has been run on one
rented machine per system, each with a Tesla T4 and four CPU cores: a whole
index of 20,000 messages embedded at about 155 messages a second, against
about 18 a second on that machine's CPU.

### Linux, in a container

This needs Docker with the NVIDIA Container Toolkit; `docker info` lists
`nvidia` among its runtimes when both are in place. Build the image from this
directory. The build downloads the pinned model and takes a minute or two; the
image is about 7 GB, and a running worker never downloads anything.

```sh
docker build -t mash-embedder .
export MASH_EMBEDDING_TOKEN="$(openssl rand -hex 32)"
docker run --gpus all -p 127.0.0.1:8000:8000 \
  -e MASH_EMBEDDING_TOKEN -e MASH_EMBEDDING_REQUIRE_GPU=1 mash-embedder
```

The worker logs `embedding with CUDAExecutionProvider` when it is ready.

### Windows, without Docker

Docker is not the route on Windows: Windows containers cannot use CUDA, and
this image is a Linux one. Run the worker directly instead. It needs a current
NVIDIA driver and [uv](https://docs.astral.sh/uv/); the CUDA libraries come
with the Python packages. From this directory, in PowerShell:

```powershell
uv sync --locked --no-dev --group gpu
uv run --no-dev --group gpu python server.py
$env:MASH_EMBEDDING_TOKEN = -join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Maximum 256) })
$env:MASH_EMBEDDING_REQUIRE_GPU = '1'
uv run --no-dev --group gpu uvicorn --factory server:app_from_environment --host 127.0.0.1 --port 8000
```

The second command downloads the pinned model into `model` here, once. The
worker stops when its window closes; keeping it running across logons is up to
you.

### Point Mash at it

Set `[embeddings]` in the node's configuration to
`url = "http://127.0.0.1:8000/embed"` with
`token_env = "MASH_EMBEDDING_TOKEN"`, and set that variable to the same token
wherever `mash index` runs. Mash accepts plain HTTP on loopback only; put the
worker behind HTTPS to reach it from another machine.

- The worker refuses to start without `MASH_EMBEDDING_TOKEN`, and answers only
  requests carrying it as a bearer token.
- `MASH_EMBEDDING_REQUIRE_GPU=1` makes the worker fail when it cannot use the
  card. Without it, a worker that cannot reach the GPU serves from the CPU and
  looks healthy; its start-up log names the engine in use.
- `MASH_EMBEDDING_MODEL_DIR` names a model directory other than the default.
- The [reranking worker](../reranking/README.md) listens on port 8000 too. To
  run both on one machine, give it another port, as its instructions do.
- Mash never falls back to local inference when a configured worker fails, so
  a stopped worker stops indexing of batches at or above `local_below`. The
  next `mash index` after it returns embeds only what is missing.

An Apple Silicon Mac needs no worker: Mash already embeds through Core ML
while indexing.

## Limits and contract

`openapi.json` is generated from Mash's client and checked against this
worker by its tests. The Modal worker permits one L4/A10 container, no warm pool, a 120-second invocation
timeout, and no automatic retries. It retains a loaded model while the container
is warm and scales down after 30 idle seconds. Each request holds at most 256 inputs
and one MiB of UTF-8 text. The worker runs GPU batches of at most 32 inputs to bound
memory use. Request bodies and errors never log or echo chat text.

Requests and replies use versioned binary frames (`MSH1`/`MSR1`). Inputs stay
UTF-8 text with explicit lengths, and reply vectors travel as little-endian f16,
which carries more precision than the int8 vectors Mash stores while keeping the
payload about ten times smaller than JSON. Requests above one kilobyte are zstd
compressed and marked `Content-Encoding: zstd`; smaller bodies stay uncompressed.
The worker accepts only this versioned binary contract.

The recipe pins Xenova's MiniLM model revision, FP32 ONNX inference, 512-token
truncation, attention-mask mean pooling and L2 normalization. Mash validates the
recipe and each vector, then performs its common quantization before storage.
On Modal, CUDA execution is required; the worker rejects unavailable GPU
execution rather than silently falling back to CPU. This deployment is experimental until its
end-to-end benchmark and retrieval checks have been completed.

Local checks do not download models, invoke Modal or need a GPU. Set
`MASH_TEST_EMBEDDING_DIR` to a directory holding the pinned FP32 model to also
check real vectors:

```sh
uv run ruff check .
uv run ty check .
uv run pytest -q
```
