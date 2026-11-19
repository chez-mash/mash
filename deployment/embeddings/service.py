"""Model loading and request handling shared by the Modal and plain workers."""

from __future__ import annotations

import asyncio
import sys
import threading
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from time import perf_counter
from typing import Protocol

import numpy as np
from fastapi import HTTPException, Request, Response
from tokenizers import Tokenizer

from contract import (
    DIMENSIONS,
    MAX_BODY_BYTES,
    REQUEST_MAGIC,
    REVISION,
    ContractError,
    Input,
    decode_batch,
    decompress_request,
    encode_reply,
)

REPOSITORY = "Xenova/all-MiniLM-L6-v2"
MODEL_FILES = ["onnx/model.onnx", "tokenizer.json"]
MAX_TOKENS = 512
# Amortize network cost with a larger request while bounding the device's
# attention workspace and padding separately for each inference batch.
BATCH_SIZE = 32
CUDA = "CUDAExecutionProvider"
CPU = "CPUExecutionProvider"

# Inference milliseconds, then one (input ID, little-endian f16 vector) per input.
Vectors = tuple[int, list[tuple[int, bytes]]]
Infer = Callable[[list[Input]], Vectors]


class Session(Protocol):
    def run(self, output_names: None, input_feed: dict[str, np.ndarray]) -> Sequence[object]: ...


def download_model(root: Path) -> None:
    """Fetch the pinned model at image build or setup time; a running worker never downloads."""
    from huggingface_hub import snapshot_download

    snapshot_download(
        REPOSITORY, revision=REVISION, local_dir=str(root), allow_patterns=MODEL_FILES
    )


def load_tokenizer(root: Path) -> Tokenizer:
    tokenizer = Tokenizer.from_file(str(root / "tokenizer.json"))
    tokenizer.enable_truncation(max_length=MAX_TOKENS)
    tokenizer.enable_padding(pad_id=0, pad_token="[PAD]")
    return tokenizer


def embed(session: Session, tokenizer: Tokenizer, texts: Sequence[str]) -> list[bytes]:
    """Mean-pooled, L2-normalized vectors as little-endian f16, in input order."""
    vectors: list[bytes] = []
    for start in range(0, len(texts), BATCH_SIZE):
        chunk = texts[start : start + BATCH_SIZE]
        encodings = tokenizer.encode_batch(list(chunk))
        feed = {
            "input_ids": np.asarray([item.ids for item in encodings], dtype=np.int64),
            "attention_mask": np.asarray(
                [item.attention_mask for item in encodings], dtype=np.int64
            ),
            "token_type_ids": np.asarray([item.type_ids for item in encodings], dtype=np.int64),
        }
        outputs = np.asarray(session.run(None, feed)[0], dtype=np.float32)
        mask = feed["attention_mask"][..., None].astype(np.float32)
        pooled = (outputs * mask).sum(axis=1) / mask.sum(axis=1).clip(min=1)
        pooled /= np.linalg.norm(pooled, axis=1, keepdims=True).clip(min=1e-12)
        if pooled.shape != (len(chunk), DIMENSIONS) or not np.isfinite(pooled).all():
            raise ValueError("invalid model output")
        # Halves are enough for the int8 vectors Mash stores, and keep the
        # response five times smaller than JSON float32 text.
        vectors.extend(row.tobytes() for row in pooled.astype("<f2", copy=False))
    return vectors


@dataclass
class Engine:
    session: Session
    tokenizer: Tokenizer
    provider: str
    # One device: overlap request transfer with inference, never inference itself.
    compute: threading.Lock = field(default_factory=threading.Lock)

    def infer(self, inputs: list[Input]) -> Vectors:
        with self.compute:
            started = perf_counter()
            try:
                vectors = embed(self.session, self.tokenizer, [item.text for item in inputs])
            except ValueError:
                raise HTTPException(500, "invalid model output") from None
            inference_milliseconds = round((perf_counter() - started) * 1000)
        return inference_milliseconds, [
            (item.id, vector) for item, vector in zip(inputs, vectors, strict=True)
        ]


def load_engine(root: Path, providers: Sequence[str]) -> Engine:
    """Load the model on the first of `providers` that works, in that order.

    ONNX Runtime drops to CPU by itself when a GPU provider cannot start, so a
    caller that did not list the CPU provider gets an error instead of a slow
    worker that looks healthy.
    """
    import onnxruntime as ort

    installed = set(ort.get_available_providers())
    wanted = [name for name in providers if name in installed]
    if not wanted:
        raise RuntimeError("none of the requested inference providers is installed")
    if sys.platform == "win32" and CUDA in wanted:
        # Windows does not search the folders the NVIDIA Python packages install
        # their libraries into, so CUDA would otherwise be reported unavailable.
        ort.preload_dlls()
    session = ort.InferenceSession(
        str(root / "onnx/model.onnx"), ort.SessionOptions(), providers=wanted
    )
    provider = session.get_providers()[0]
    if provider not in wanted:
        raise RuntimeError(f"{wanted[0]} is unavailable and CPU inference was not requested")
    if provider != CPU:
        # ONNX schedules small shape operations on CPU even with a GPU provider
        # active; this only forbids rerunning a whole failed request on CPU.
        session.disable_fallback()
    engine = Engine(session=session, tokenizer=load_tokenizer(root), provider=provider)
    # Initialize the provider's kernels before the first batch arrives.
    embed(engine.session, engine.tokenizer, ["warm up"])
    return engine


async def answer(request: Request, infer: Infer) -> Response:
    """Decode one bounded request frame, embed it and encode the reply."""
    body = bytearray()
    async for chunk in request.stream():
        if len(body) + len(chunk) > MAX_BODY_BYTES:
            raise HTTPException(413, "embedding request exceeds byte limit")
        body.extend(chunk)
    raw = bytes(body)
    encoding = request.headers.get("content-encoding")
    if encoding is not None:
        if encoding != "zstd":
            raise HTTPException(415, "unsupported content encoding")
        try:
            raw = decompress_request(raw)
        except ContractError:
            raise HTTPException(400, "invalid embedding batch") from None
    if raw[:4] != REQUEST_MAGIC:
        raise HTTPException(400, "invalid embedding batch")
    try:
        batch = decode_batch(raw)
    except ContractError:
        # Diagnostics may echo input text; never return them.
        raise HTTPException(400, "invalid embedding batch") from None
    inference_milliseconds, vectors = await asyncio.to_thread(infer, batch.inputs)
    try:
        reply = encode_reply(batch.request_id, inference_milliseconds, vectors)
    except ContractError:
        raise HTTPException(500, "invalid model output") from None
    return Response(reply, media_type="application/octet-stream")
