#!/usr/bin/env python3
"""Record the made-up decisions in demo-chats.json through `mash mcp`.

Usage: demo_records.py [MASH]

Each record cites its chat by session_id and an exact quote, so it works
before the message ids are known. Run it after `mash index` has read the demo
histories that demo_history.py wrote.
"""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main() -> None:
    mash = sys.argv[1] if len(sys.argv) > 1 else "mash"
    records = json.loads((HERE / "demo-chats.json").read_text())["records"]
    server = subprocess.Popen(
        [mash, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True
    )

    def send(message: dict) -> None:
        server.stdin.write(json.dumps(message) + "\n")
        server.stdin.flush()

    def call(number: int, method: str, params: dict) -> dict:
        send({"jsonrpc": "2.0", "id": number, "method": method, "params": params})
        while True:
            reply = json.loads(server.stdout.readline())
            if reply.get("id") == number:
                return reply

    call(
        0,
        "initialize",
        {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "mac-vm-demo", "version": "1"},
        },
    )
    send({"jsonrpc": "2.0", "method": "notifications/initialized"})
    failed = 0
    for number, record in enumerate(records, start=1):
        arguments = {key: value for key, value in record.items() if key != "evidence"}
        arguments.update(
            {
                "action": "create",
                "state": "accepted",
                "review": "established",
                "recorded_by": "demo",
                "on_duplicate": "reuse",
                "evidence": [
                    {
                        "session_id": quote["chat"],
                        "quote": quote["quote"],
                        "purpose": "endorsement",
                    }
                    for quote in record["evidence"]
                ],
            }
        )
        reply = call(number, "tools/call", {"name": "record", "arguments": arguments})
        result = reply.get("result", {})
        if "error" in reply or result.get("isError"):
            failed += 1
            print(
                f"{record['operation_id']}: {json.dumps(reply)[:400]}", file=sys.stderr
            )
    server.stdin.close()
    server.wait(timeout=30)
    print(f"recorded {len(records) - failed} of {len(records)} demo decisions")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
