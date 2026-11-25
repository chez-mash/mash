#!/usr/bin/env python3
"""Write the made-up chats in demo-chats.json into each agent's own history.

Usage: demo_history.py HOME [--now UNIX_SECONDS]

Each chat goes where its agent keeps history on macOS, in that agent's native
format, so Mash finds it by ordinary discovery:

  claude       ~/.claude/projects/<project>/<id>.jsonl
  codex        ~/.codex/sessions/YYYY/MM/DD/rollout-<time>-<id>.jsonl
  copilot      ~/.copilot/session-state/<id>/events.jsonl
  opencode     ~/.local/share/opencode/opencode.db
  cursor       ~/.cursor/chats/<workspace>/<id>/store.db
  antigravity  ~/.gemini/antigravity/conversations/<id>.db

The formats follow the native fixtures under fixtures/ and the parser tests in
app/silver. Only Python's standard library is used, so the script runs inside a
fresh guest.
"""

import argparse
import hashlib
import json
import sqlite3
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent


def iso(ms: int) -> str:
    return (
        datetime.fromtimestamp(ms / 1000, timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%S.%f"
        )[:-3]
        + "Z"
    )


def timed(chat: dict, now_ms: int) -> list[tuple[str, str, int]]:
    """The chat's messages with times: three minutes apart, the last at minutesAgo."""
    messages = chat["messages"]
    end = now_ms - chat["minutesAgo"] * 60_000
    return [
        (role, text, end - (len(messages) - 1 - i) * 180_000)
        for i, (role, text) in enumerate(messages)
    ]


def child_id(chat: dict, position: int) -> str:
    return str(uuid.uuid5(uuid.UUID(chat["id"]), str(position)))


# Protocol buffer encoding: just varints and length-delimited fields.
def varint(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        out.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(out)


def field(tag: int, value) -> bytes:
    if isinstance(value, int):
        return varint(tag << 3) + varint(value)
    if isinstance(value, str):
        value = value.encode()
    return varint(tag << 3 | 2) + varint(len(value)) + value


def claude(home: Path, chat: dict, now_ms: int) -> None:
    project = home / ".claude/projects" / chat["project"].replace("/", "-")
    project.mkdir(parents=True, exist_ok=True)
    lines, parent = [], None
    for i, (role, text, at) in enumerate(timed(chat, now_ms)):
        message = {"role": "user", "content": text}
        if role == "assistant":
            message = {
                "model": "claude-opus-5-5",
                "id": f"msg_{i}",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": text}],
                "stop_reason": "end_turn",
                "stop_sequence": None,
                "usage": {"input_tokens": 1, "output_tokens": 1},
            }
        entry = {
            "parentUuid": parent,
            "isSidechain": False,
            "userType": "external",
            "entrypoint": "cli",
            "cwd": chat["project"],
            "sessionId": chat["id"],
            "version": "2.1.296",
            "gitBranch": "main",
            "type": role,
            "uuid": child_id(chat, i),
            "timestamp": iso(at),
            "message": message,
        }
        parent = entry["uuid"]
        lines.append(json.dumps(entry))
    (project / f"{chat['id']}.jsonl").write_text("\n".join(lines) + "\n")


def codex(home: Path, chat: dict, now_ms: int) -> None:
    messages = timed(chat, now_ms)
    start = messages[0][2]
    day = datetime.fromtimestamp(start / 1000, timezone.utc)
    folder = home / ".codex/sessions" / day.strftime("%Y/%m/%d")
    folder.mkdir(parents=True, exist_ok=True)
    lines = [
        {
            "timestamp": iso(start),
            "type": "session_meta",
            "payload": {
                "id": chat["id"],
                "timestamp": iso(start),
                "cwd": chat["project"],
                "originator": "codex_cli_rs",
                "cli_version": "0.159.2",
                "source": "cli",
                "model_provider": "openai",
            },
        }
    ]
    for role, text, at in messages:
        kind = "input_text" if role == "user" else "output_text"
        lines.append(
            {
                "timestamp": iso(at),
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": role,
                    "content": [{"type": kind, "text": text}],
                },
            }
        )
    name = f"rollout-{day.strftime('%Y-%m-%dT%H-%M-%S')}-{chat['id']}.jsonl"
    (folder / name).write_text("\n".join(json.dumps(line) for line in lines) + "\n")


def copilot(home: Path, chat: dict, now_ms: int) -> None:
    folder = home / ".copilot/session-state" / chat["id"]
    folder.mkdir(parents=True, exist_ok=True)
    messages = timed(chat, now_ms)
    project = chat["project"]
    events = [
        {
            "id": child_id(chat, 100),
            "parentId": None,
            "timestamp": iso(messages[0][2]),
            "type": "session.start",
            "data": {
                "sessionId": chat["id"],
                "version": 1,
                "producer": "copilot-agent",
                "copilotVersion": "1.0.89",
                "startTime": iso(messages[0][2]),
                "context": {"cwd": project, "gitRoot": project},
            },
        }
    ]
    for i, (role, text, at) in enumerate(messages):
        if role == "user":
            data = {"content": text, "source": "user", "attachments": []}
            kind = "user.message"
        else:
            data = {"messageId": child_id(chat, i), "content": text}
            kind = "assistant.message"
        events.append(
            {
                "id": child_id(chat, i),
                "parentId": events[-1]["id"],
                "timestamp": iso(at),
                "type": kind,
                "data": data,
            }
        )
    (folder / "events.jsonl").write_text(
        "\n".join(json.dumps(event) for event in events) + "\n"
    )


def opencode(home: Path, chats: list[dict], now_ms: int) -> None:
    folder = home / ".local/share/opencode"
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / "opencode.db"
    path.unlink(missing_ok=True)
    db = sqlite3.connect(path)
    db.executescript(
        "CREATE TABLE project(id TEXT PRIMARY KEY,worktree TEXT,vcs TEXT);"
        "CREATE TABLE session(id TEXT PRIMARY KEY,project_id TEXT,title TEXT,directory TEXT,"
        "time_created INTEGER,time_updated INTEGER);"
        "CREATE TABLE session_v2(id TEXT PRIMARY KEY,project_id TEXT,title TEXT,directory TEXT,"
        "time_created INTEGER,time_updated INTEGER);"
        "CREATE TABLE message(id TEXT PRIMARY KEY,session_id TEXT,time_created INTEGER,data TEXT);"
        "CREATE TABLE part(id TEXT PRIMARY KEY,message_id TEXT,session_id TEXT,data TEXT);"
        "CREATE TABLE session_message(id TEXT PRIMARY KEY,session_id TEXT,type TEXT,seq INTEGER,"
        "time_created INTEGER,data TEXT);"
    )
    for chat in chats:
        messages = timed(chat, now_ms)
        project = chat["project"]
        db.execute(
            "INSERT OR IGNORE INTO project VALUES(?,?,'git')", (project, project)
        )
        db.execute(
            "INSERT INTO session_v2 VALUES(?,?,?,?,?,?)",
            (
                chat["id"],
                project,
                messages[0][1],
                project,
                messages[0][2],
                messages[-1][2],
            ),
        )
        for i, (role, text, at) in enumerate(messages):
            data = (
                {"text": text}
                if role == "user"
                else {"content": [{"type": "text", "text": text}]}
            )
            db.execute(
                "INSERT INTO session_message VALUES(?,?,?,?,?,?)",
                (child_id(chat, i), chat["id"], role, i + 1, at, json.dumps(data)),
            )
    db.commit()
    db.close()


def cursor(home: Path, chat: dict, now_ms: int) -> None:
    # Cursor's agent store: content-addressed blobs, a protobuf root listing
    # the message blobs, and the root's id in hex-encoded JSON metadata.
    workspace = hashlib.md5(chat["project"].encode()).hexdigest()
    folder = home / ".cursor/chats" / workspace / chat["id"]
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / "store.db"
    path.unlink(missing_ok=True)
    db = sqlite3.connect(path)
    db.executescript(
        "PRAGMA user_version=1;"
        "CREATE TABLE meta(key TEXT PRIMARY KEY,value TEXT); CREATE TABLE blobs(id TEXT PRIMARY KEY,data BLOB);"
    )

    def put(data: bytes) -> bytes:
        digest = hashlib.sha256(data).digest()
        db.execute("INSERT OR REPLACE INTO blobs VALUES(?,?)", (digest.hex(), data))
        return digest

    messages = timed(chat, now_ms)
    root = b"".join(
        field(
            1,
            put(
                json.dumps(
                    {"role": role, "content": text}, separators=(",", ":")
                ).encode()
            ),
        )
        for role, text, _ in messages
    )
    root += field(9, "file://" + chat["project"]) + field(26, messages[0][2])
    metadata = {
        "agentId": chat["id"],
        "latestRootBlobId": put(root).hex(),
        "name": messages[0][1][:60],
        "mode": "default",
        "isRunEverything": False,
        "createdAt": messages[0][2],
    }
    db.execute("INSERT INTO meta VALUES('0',?)", (json.dumps(metadata).encode().hex(),))
    db.commit()
    db.close()


def antigravity_schema() -> Path:
    """The recorded native table definitions: beside this script in a guest, else in the repository."""
    name = "antigravity-native-runtime-2.5.5.json"
    return next(
        path
        for path in (HERE / name, HERE.parent.parent / "fixtures" / name)
        if path.exists()
    )


def antigravity(home: Path, chat: dict, now_ms: int) -> None:
    # One conversation database per cascade: its identity, a metadata blob
    # naming the workspace, and one protobuf Step per message (kind 14 is the
    # user, 15 the assistant; status 3 is done).
    folder = home / ".gemini/antigravity/conversations"
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / f"{chat['id']}.db"
    path.unlink(missing_ok=True)
    schema = json.loads(antigravity_schema().read_text())
    db = sqlite3.connect(path)
    for table in schema["native_tables"].values():
        db.execute(table["create_sql"])
    trajectory = child_id(chat, 200)
    messages = timed(chat, now_ms)

    def stamp(ms: int) -> bytes:
        return field(1, ms // 1000) + field(2, (ms % 1000) * 1_000_000)

    db.execute("INSERT INTO trajectory_meta VALUES(?,?,4,1)", (trajectory, chat["id"]))
    workspace = field(1, "file://" + chat["project"])
    db.execute(
        "INSERT INTO trajectory_metadata_blob VALUES('main',?)",
        (field(1, workspace) + field(2, stamp(messages[0][2])),),
    )
    for i, (role, text, at) in enumerate(messages):
        source = 4 if role == "user" else 2
        metadata = field(1, stamp(at)) + field(3, source)
        if role == "user":
            body = field(19, field(3, field(1, text)))
            kind = 14
        else:
            body = field(20, field(1, text))
            kind = 15
        payload = field(1, kind) + field(4, 3) + field(5, metadata) + body
        db.execute(
            "INSERT INTO steps(idx,step_type,status,metadata,step_payload,step_format) VALUES(?,?,3,?,?,0)",
            (i, kind, metadata, payload),
        )
    db.commit()
    db.close()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("home", type=Path)
    parser.add_argument("--now", type=int, default=int(time.time()))
    args = parser.parse_args()
    now_ms = args.now * 1000
    chats = json.loads((HERE / "demo-chats.json").read_text())["chats"]
    writers = {
        "claude": claude,
        "codex": codex,
        "copilot": copilot,
        "cursor": cursor,
        "antigravity": antigravity,
    }
    for chat in chats:
        if chat["agent"] in writers:
            writers[chat["agent"]](args.home, chat, now_ms)
    opencode(args.home, [chat for chat in chats if chat["agent"] == "opencode"], now_ms)
    print(f"wrote {len(chats)} chats under {args.home}")


if __name__ == "__main__":
    main()
