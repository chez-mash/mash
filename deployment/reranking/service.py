"""Model loading and request handling shared by the Modal and plain workers."""

from __future__ import annotations

import asyncio
import sys
import threading
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from time import perf_counter

from fastapi import HTTPException, Request, Response
from tokenizers import Tokenizer

from contract import (
    MAX_BODY_BYTES,
    REPOSITORY,
    REQUEST_MAGIC,
    REVISION,
    ContractError,
    Document,
    decode_batch,
    decompress_request,
    encode_reply,
)
from scoring import MODEL_FILES, Session, load_tokenizer, score

CUDA = "CUDAExecutionProvider"
CPU = "CPUExecutionProvider"

# Inference milliseconds, then one (document ID, raw logit) per document.
Scores = tuple[int, list[tuple[int, float]]]
Infer = Callable[[str, list[Document]], Scores]


def download_model(root: Path) -> None:
    """Fetch the pinned reranker at image build or setup time; a running worker never downloads."""
    from huggingface_hub import snapshot_download

    snapshot_download(
        REPOSITORY,
        revision=REVISION,
        local_dir=str(root),
        allow_patterns=MODEL_FILES,
    )


@dataclass
class Engine:
    session: Session
    tokenizer: Tokenizer
    provider: str
    # One device: overlap request transfer with inference, never inference itself.
    compute: threading.Lock = field(default_factory=threading.Lock)

    def infer(self, query: str, documents: list[Document]) -> Scores:
        with self.compute:
            started = perf_counter()
            try:
                scores = score(
                    self.session, self.tokenizer, query, [item.text for item in documents]
                )
            except ValueError:
                raise HTTPException(500, "invalid model output") from None
            inference_milliseconds = round((perf_counter() - started) * 1000)
        return inference_milliseconds, [
            (item.id, value) for item, value in zip(documents, scores, strict=True)
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
    # Initialize the provider's kernels before the first search arrives.
    score(engine.session, engine.tokenizer, "warm up", ["warm up"])
    return engine


async def answer(request: Request, infer: Infer) -> Response:
    """Decode one bounded request frame, score it and encode the reply."""
    body = bytearray()
    async for chunk in request.stream():
        if len(body) + len(chunk) > MAX_BODY_BYTES:
            raise HTTPException(413, "reranking request exceeds byte limit")
        body.extend(chunk)
    raw = bytes(body)
    encoding = request.headers.get("content-encoding")
    if encoding is not None:
        if encoding != "zstd":
            raise HTTPException(415, "unsupported content encoding")
        try:
            raw = decompress_request(raw)
        except ContractError:
            raise HTTPException(400, "invalid reranking batch") from None
    if raw[:4] != REQUEST_MAGIC:
        raise HTTPException(400, "invalid reranking batch")
    try:
        batch = decode_batch(raw)
    except ContractError as error:
        # Contract messages never contain input text; keep the recipe and
        # limit reasons distinguishable for the client.
        raise HTTPException(400, str(error)) from None
    if not batch.documents:
        # Warm-up: the process and model are loaded; no inference.
        return Response(
            encode_reply(batch.request_id, 0, []), media_type="application/octet-stream"
        )
    inference_milliseconds, scores = await asyncio.to_thread(infer, batch.query, batch.documents)
    try:
        reply = encode_reply(batch.request_id, inference_milliseconds, scores)
    except ContractError:
        raise HTTPException(500, "invalid model output") from None
    return Response(reply, media_type="application/octet-stream")
