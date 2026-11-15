# An embedding worker for a machine with an NVIDIA GPU:
#   docker build -t mash-embedder deployment/embeddings
#   docker run --gpus all -p 127.0.0.1:8000:8000 -e MASH_EMBEDDING_TOKEN mash-embedder
# The same CUDA base and pinned packages as the Modal worker.
FROM nvidia/cuda:12.4.1-cudnn-runtime-ubuntu22.04

COPY --from=ghcr.io/astral-sh/uv:0.9.9 /uv /usr/local/bin/uv
ENV UV_PYTHON_INSTALL_DIR=/opt/python \
    VIRTUAL_ENV=/opt/venv \
    PATH=/opt/venv/bin:$PATH \
    PYTHONDONTWRITEBYTECODE=1
RUN uv venv --python 3.12 /opt/venv \
    && uv pip install --no-cache \
        "onnxruntime-gpu==1.23.2" \
        "numpy==2.2.6" \
        "tokenizers==0.22.1" \
        "huggingface-hub>=0.35,<1" \
        "fastapi>=0.115,<1" \
        "uvicorn>=0.34,<1" \
        "zstandard>=0.23,<1"

WORKDIR /srv/mash-embedder
COPY contract.py service.py server.py ./
# The pinned model is part of the image; a running worker never downloads.
RUN python -c "from pathlib import Path; from service import download_model; download_model(Path('/opt/mash-model'))"

RUN useradd --system --no-create-home worker
USER worker
EXPOSE 8000
CMD ["uvicorn", "--factory", "server:app_from_environment", "--host", "0.0.0.0", "--port", "8000"]
