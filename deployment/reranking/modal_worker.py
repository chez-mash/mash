"""Explicitly deploy an authenticated GPU reranker; importing does not launch jobs."""

from pathlib import Path

import modal
from fastapi import Request, Response

from service import CUDA, answer, download_model, load_engine

app = modal.App("mash-reranking")
MODEL_ROOT = Path("/opt/mash-reranker")


def download_pinned_model() -> None:
    """Runs during explicit image construction, before a GPU is allocated."""
    download_model(MODEL_ROOT)


image = (
    modal.Image.from_registry("nvidia/cuda:12.4.1-cudnn-runtime-ubuntu22.04", add_python="3.12")
    .pip_install(
        "onnxruntime-gpu==1.23.2",
        "numpy==2.2.6",
        "tokenizers==0.22.1",
        "huggingface-hub>=0.35,<1",
        "fastapi>=0.115,<1",
        "zstandard>=0.23,<1",
    )
    .add_local_file(Path(__file__).with_name("contract.py"), "/root/contract.py", copy=True)
    .add_local_file(Path(__file__).with_name("scoring.py"), "/root/scoring.py", copy=True)
    .add_local_file(Path(__file__).with_name("service.py"), "/root/service.py", copy=True)
    .run_function(download_pinned_model)
)


# A separate app from embedding, so searches never queue behind indexing
# batches. A T4 is ample for a six-layer cross-encoder over at most 64 short
# pairs. Mash warms the worker when a search is likely; fifteen idle minutes
# outlast the plugin's ten-minute catch-up interval.
@app.cls(
    image=image,
    gpu=["T4", "L4"],
    cpu=1,
    memory=4096,
    timeout=60,
    startup_timeout=180,
    max_containers=1,
    min_containers=0,
    scaledown_window=900,
    retries=0,
)
@modal.concurrent(max_inputs=4)
class Worker:
    @modal.enter()
    def load(self) -> None:
        # CUDA only: the worker fails rather than serving from the container's CPU.
        self.engine = load_engine(MODEL_ROOT, [CUDA])

    @modal.fastapi_endpoint(method="POST", requires_proxy_auth=True, docs=False)
    async def rerank(self, request: Request) -> Response:
        return await answer(request, self.engine.infer)
