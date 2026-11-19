"""The plain worker's HTTP boundary, without a GPU, a download or a network."""

import json
import os
import struct
from pathlib import Path

import numpy as np
import pytest
import zstandard
from fastapi.testclient import TestClient

import contract
import server
from contract import DIMENSIONS, MAX_BODY_BYTES, REQUEST_ID_BYTES, VECTOR_BYTES, Input
from server import create_app
from service import CPU, CUDA, Vectors, load_engine
from test_contract import REQUEST_ID, pack

TOKEN = "worker-token"
AUTHORIZED = {"authorization": f"Bearer {TOKEN}"}
HEADER_BYTES = 4 + REQUEST_ID_BYTES + 8


def basis(index: int) -> bytes:
    vector = np.zeros(DIMENSIONS, dtype="<f2")
    vector[index] = 1.0
    return vector.tobytes()


class Lengths:
    """Stands in for the model: each input's vector points along its length."""

    def __init__(self) -> None:
        self.calls = 0

    def infer(self, inputs: list[Input]) -> Vectors:
        self.calls += 1
        return 7, [(item.id, basis(len(item.text))) for item in inputs]


def client(model: Lengths | None = None) -> TestClient:
    return TestClient(create_app((model or Lengths()).infer, TOKEN))


def unpack(reply: bytes) -> tuple[int, dict[int, int]]:
    """Inference milliseconds, and for each input ID the axis its vector points along."""
    assert reply[:4] == b"MSR1"
    assert reply[4 : 4 + REQUEST_ID_BYTES] == REQUEST_ID
    milliseconds, count = struct.unpack_from("<II", reply, 4 + REQUEST_ID_BYTES)
    assert len(reply) == HEADER_BYTES + count * (4 + VECTOR_BYTES)
    axes = {}
    for index in range(count):
        offset = HEADER_BYTES + index * (4 + VECTOR_BYTES)
        (input_id,) = struct.unpack_from("<I", reply, offset)
        vector = np.frombuffer(reply, dtype="<f2", count=DIMENSIONS, offset=offset + 4)
        axes[input_id] = int(vector.argmax())
    return milliseconds, axes


def test_a_worker_cannot_be_created_without_a_token() -> None:
    with pytest.raises(ValueError):
        create_app(Lengths().infer, "")


@pytest.mark.parametrize(
    "headers",
    [
        {},
        {"authorization": "Bearer wrong"},
        {"authorization": TOKEN},
        {"authorization": f"Bearer {TOKEN}x"},
        {"authorization": f"Basic {TOKEN}"},
    ],
)
def test_requests_without_the_exact_bearer_token_never_reach_the_model(
    headers: dict[str, str],
) -> None:
    model = Lengths()
    response = client(model).post("/embed", content=pack([(0, "x")]), headers=headers)
    assert response.status_code == 401
    assert model.calls == 0


def test_vectors_come_back_keyed_by_input_id() -> None:
    response = client().post(
        "/embed", content=pack([(1, "four"), (0, "sixsix")]), headers=AUTHORIZED
    )
    assert response.status_code == 200
    assert response.headers["content-type"] == "application/octet-stream"
    assert unpack(response.content) == (7, {1: 4, 0: 6})


def test_a_compressed_request_is_embedded_like_a_plain_one() -> None:
    body = zstandard.ZstdCompressor(level=3).compress(pack([(0, "東京")]))
    response = client().post(
        "/embed", content=body, headers={**AUTHORIZED, "content-encoding": "zstd"}
    )
    assert unpack(response.content) == (7, {0: 2})


def test_malformed_oversized_and_unsupported_requests_are_refused() -> None:
    model = Lengths()
    worker = client(model)
    cases = [
        (b"not a frame", {}, 400),
        (pack([(0, "x")], recipe="another-model"), {}, 400),
        (pack([]), {}, 400),
        (b"x" * (MAX_BODY_BYTES + 1), {}, 413),
        (pack([(0, "x")]), {"content-encoding": "gzip"}, 415),
        (b"not zstd", {"content-encoding": "zstd"}, 400),
    ]
    for body, extra, status in cases:
        response = worker.post("/embed", content=body, headers={**AUTHORIZED, **extra})
        assert response.status_code == status
        # A refusal never echoes what was sent.
        assert response.json() == {"detail": response.json()["detail"]}
        assert "another-model" not in response.text
    assert model.calls == 0


def test_a_vector_of_the_wrong_size_is_an_error_not_a_reply() -> None:
    def short(inputs: list[Input]) -> Vectors:
        return 1, [(item.id, b"\x00" * 10) for item in inputs]

    worker = TestClient(create_app(short, TOKEN))
    assert worker.post("/embed", content=pack([(0, "x")]), headers=AUTHORIZED).status_code == 500


def test_only_the_embed_route_exists() -> None:
    worker = client()
    for path in ["/", "/docs", "/openapi.json", "/redoc"]:
        assert worker.get(path, headers=AUTHORIZED).status_code == 404


def test_the_worker_matches_the_published_contract() -> None:
    """openapi.json is generated from Mash's client; this worker answers to it."""
    # Plain json: the file is a build artifact read once, not a hot path.
    document = json.loads(Path(__file__).with_name("openapi.json").read_text())
    assert document["x-mash-contract"] == {
        "recipe": contract.RECIPE,
        "request_magic": contract.REQUEST_MAGIC.decode(),
        "response_magic": contract.RESPONSE_MAGIC.decode(),
        "request_id_bytes": contract.REQUEST_ID_BYTES,
        "dimensions": contract.DIMENSIONS,
        "max_inputs": contract.MAX_INPUTS,
        "max_input_bytes": contract.MAX_INPUT_BYTES,
        "max_body_bytes": contract.MAX_BODY_BYTES,
    }
    routes = {
        (path, method.lower())
        for route in create_app(Lengths().infer, TOKEN).routes
        for path, methods in [(getattr(route, "path", ""), getattr(route, "methods", None) or [])]
        for method in methods
    }
    published = {
        (path, method) for path, operations in document["paths"].items() for method in operations
    }
    assert routes == published
    statuses = set(document["paths"]["/embed"]["post"]["responses"])
    assert {"200", "400", "401", "413", "415", "500"} == statuses


def model_root() -> Path | None:
    root = os.environ.get("MASH_TEST_EMBEDDING_DIR")
    return Path(root) if root else None


@pytest.mark.skipif(model_root() is None, reason="set MASH_TEST_EMBEDDING_DIR")
def test_the_real_model_places_similar_sentences_together_on_cpu() -> None:
    root = model_root()
    assert root is not None
    engine = load_engine(root, [CUDA, CPU])
    worker = TestClient(create_app(engine.infer, TOKEN))
    texts = [
        "A cat sleeps on the sofa.",
        "A kitten rests on a couch.",
        "A database transaction commits its changes to disk.",
    ]
    reply = worker.post("/embed", content=pack(list(enumerate(texts))), headers=AUTHORIZED).content
    vectors = [
        np.frombuffer(
            reply, dtype="<f2", count=DIMENSIONS, offset=HEADER_BYTES + i * (4 + VECTOR_BYTES) + 4
        ).astype(np.float32)
        for i in range(3)
    ]
    assert float(vectors[0] @ vectors[1]) > float(vectors[0] @ vectors[2])
    assert all(abs(float(np.linalg.norm(vector)) - 1.0) < 1e-2 for vector in vectors)


def test_the_model_directory_comes_from_the_environment_first(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv(server.MODEL_ENV, str(tmp_path / "chosen"))
    assert server.model_root() == tmp_path / "chosen"


def test_the_model_directory_is_the_image_one_only_where_it_exists(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.delenv(server.MODEL_ENV, raising=False)
    monkeypatch.setattr(server, "CONTAINER_MODEL_ROOT", tmp_path)
    assert server.model_root() == tmp_path
    monkeypatch.setattr(server, "CONTAINER_MODEL_ROOT", tmp_path / "absent")
    assert server.model_root() == server.LOCAL_MODEL_ROOT
