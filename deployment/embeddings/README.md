# Embedding worker

This optional worker generates indexing vectors for Mash on a GPU. Run it on
Modal, or, experimentally, as a container on a machine of your own with an
NVIDIA card; both speak the same contract, published in [`openapi.json`](openapi.json), and Mash
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

The container has been built and timed on a cloud GPU, not yet run with
Docker on a machine of our own or used for a whole index. Expect rough edges.

This needs Docker with the NVIDIA Container Toolkit. Build the image from this
directory; the build downloads the pinned model, and a running worker never
downloads anything.

```sh
docker build -t mash-embedder .
export MASH_EMBEDDING_TOKEN="$(openssl rand -hex 32)"
docker run --gpus all -p 127.0.0.1:8000:8000 -e MASH_EMBEDDING_TOKEN mash-embedder
```

Point the node's `[embeddings]` at `http://127.0.0.1:8000/embed` with
`token_env = "MASH_EMBEDDING_TOKEN"`. Mash accepts plain HTTP on loopback only;
put the worker behind HTTPS to reach it from another machine.

- The worker refuses to start without `MASH_EMBEDDING_TOKEN`, and answers only
  requests carrying it as a bearer token.
- Its start-up log names the engine in use. Without a usable GPU it serves from
  the container's CPU; set `MASH_EMBEDDING_REQUIRE_GPU=1` to make it fail
  instead.
- `MASH_EMBEDDING_MODEL_DIR` names a model directory other than the one in the
  image.
- Mash never falls back to local inference when a configured worker fails, so
  a stopped container stops indexing of batches at or above `local_below`.

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
