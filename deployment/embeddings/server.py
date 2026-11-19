"""A plain embedding worker for a machine or container you run yourself.

Start it with `uvicorn --factory server:app_from_environment`. It speaks the
same contract as the Modal worker and checks the bearer token itself.
"""

from __future__ import annotations

import hmac
import logging
import os
from pathlib import Path

from fastapi import FastAPI, HTTPException, Request, Response

from service import CPU, CUDA, Infer, answer, download_model, load_engine

TOKEN_ENV = "MASH_EMBEDDING_TOKEN"
MODEL_ENV = "MASH_EMBEDDING_MODEL_DIR"
REQUIRE_GPU_ENV = "MASH_EMBEDDING_REQUIRE_GPU"
# Where the container image keeps the model, and where a worker run from this
# directory keeps it instead.
CONTAINER_MODEL_ROOT = Path("/opt/mash-model")
LOCAL_MODEL_ROOT = Path(__file__).resolve().parent / "model"


def model_root() -> Path:
    """The directory named by the environment, else the image's, else this one's."""
    configured = os.environ.get(MODEL_ENV)
    if configured:
        return Path(configured)
    return CONTAINER_MODEL_ROOT if CONTAINER_MODEL_ROOT.exists() else LOCAL_MODEL_ROOT


def create_app(infer: Infer, token: str) -> FastAPI:
    if not token:
        raise ValueError("an embedding worker requires a token")
    expected = f"Bearer {token}".encode()
    # The generated API pages would be reachable without the token.
    app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)

    @app.post("/embed")
    async def embed(request: Request) -> Response:
        supplied = request.headers.get("authorization", "").encode()
        if not hmac.compare_digest(supplied, expected):
            raise HTTPException(401, "invalid embedding token")
        return await answer(request, infer)

    return app


def app_from_environment() -> FastAPI:
    token = os.environ.get(TOKEN_ENV, "").strip()
    if not token:
        raise RuntimeError(f"{TOKEN_ENV} is not set")
    root = model_root()
    require_gpu = os.environ.get(REQUIRE_GPU_ENV) == "1"
    engine = load_engine(root, [CUDA] if require_gpu else [CUDA, CPU])
    logging.getLogger("uvicorn.error").info("embedding with %s", engine.provider)
    return create_app(engine.infer, token)


if __name__ == "__main__":
    # `python server.py` fetches the pinned model, for a worker run without the image.
    download_model(model_root())
