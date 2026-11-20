"""The hooks under Copilot CLI's own events: its payload shape, the output it
honours, and saving only when the binary can cite its sessions."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

HOOKS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HOOKS))

import recall_common as common  # noqa: E402

ORIGIN = "https://example.test/repo.git"
# A stand-in Mash: version info from $FAKE_SESSIONS, one search result, no MCP.
FAKE = """
import json, os, sys
if sys.argv[1] == "mcp":
    sys.exit(1)
if sys.argv[1] == "--version-info":
    info = {"record_citations": ["message_id", "session_id"]}
    if os.environ.get("FAKE_SESSIONS"):
        info["session_citations"] = os.environ["FAKE_SESSIONS"].split(",")
    print(json.dumps(info))
elif sys.argv[1] == "search":
    match = {"message_id": "m1", "created_at": 1_793_178_060_000, "snippet": "moved it"}
    chat = {"chat_id": "earlier", "title": "Earlier work", "matches": [match]}
    print(json.dumps({"results": [chat], "next_offset": None}))
else:
    print(json.dumps({"chats": [], "next_offset": None}))
"""


class Shapes(unittest.TestCase):
    def test_copilot_payload_is_read_like_the_others(self) -> None:
        native = {
            "sessionId": "s1",
            "cwd": "/work",
            "toolName": "view",
            "toolArgs": {"path": "/work/a.py", "view_range": [1, 9]},
        }
        payload = common.from_host(native)
        self.assertEqual(payload["session_id"], "s1")
        self.assertEqual(payload["tool_name"], "Read")
        self.assertEqual(payload["tool_input"]["file_path"], "/work/a.py")
        self.assertEqual(payload[common.HOST], common.COPILOT)
        # Another Copilot tool is not mistaken for a file read.
        edited = common.from_host({**native, "toolName": "edit"})
        self.assertEqual(edited["tool_name"], "edit")
        # A prompt event has no tool at all.
        prompt = common.from_host({"sessionId": "s1", "cwd": "/work", "prompt": "hi"})
        self.assertIsNone(prompt["tool_input"]["file_path"])

    def test_other_payloads_pass_through(self) -> None:
        claude = {"session_id": "s1", "tool_name": "Read", "cwd": "/work"}
        self.assertIs(common.from_host(claude), claude)
        self.assertEqual(
            common.from_host(["not", "a", "payload"]), ["not", "a", "payload"]
        )

    def test_each_host_gets_the_output_it_honours(self) -> None:
        copilot = common.from_host({"sessionId": "s1", "cwd": "/work"})
        self.assertEqual(
            json.loads(common.reply("SessionStart", "text", copilot)),
            {"additionalContext": "text"},
        )
        wrapped = json.loads(common.reply("SessionStart", "text", {"session_id": "s1"}))
        self.assertEqual(
            wrapped["hookSpecificOutput"],
            {"hookEventName": "SessionStart", "additionalContext": "text"},
        )


class UnderCopilot(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.repo = self.root / "repo"
        (self.repo / "app").mkdir(parents=True)
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        remote = ["git", "-C", str(self.repo), "remote", "add", "origin", ORIGIN]
        subprocess.run(remote, check=True)
        self.large = self.repo / "app" / "schema.sql"
        self.large.write_text("-- schema\n" * 400)
        (self.root / "fake.py").write_text(FAKE)
        binary = self.root / "mash"
        binary.write_text(
            f'#!/bin/sh\nexec {sys.executable} {self.root}/fake.py "$@"\n'
        )
        binary.chmod(0o755)
        self.env = {**os.environ, "MASH_BIN": str(binary), "MASH_AUTO_INDEX": "off"}
        self.env["MASH_RECALL_STATE_DIR"] = str(self.root / "state")
        self.env.pop("FAKE_SESSIONS", None)

    def run_hook(self, script: str, payload: dict[str, Any]) -> dict[str, Any]:
        command = [sys.executable, str(HOOKS / script)]
        stdin = json.dumps(payload).encode()
        result = subprocess.run(command, input=stdin, capture_output=True, env=self.env)
        self.assertEqual(result.returncode, 0)
        return json.loads(result.stdout) if result.stdout else {}

    def test_file_read_recall_arrives_as_top_level_context(self) -> None:
        payload = {
            "sessionId": "current",
            "cwd": str(self.repo),
            "toolName": "view",
            "toolArgs": {"path": str(self.large)},
            "toolResult": {"resultType": "success", "textResultForLlm": "-- schema"},
        }
        output = self.run_hook("file_context.py", payload)
        self.assertEqual(list(output), ["additionalContext"])
        self.assertIn("app/schema.sql", output["additionalContext"])
        self.assertIn("moved it", output["additionalContext"])
        # Once per file and session, as for the other hosts.
        self.assertEqual(self.run_hook("file_context.py", payload), {})
        # Viewing a directory, or another tool, shows nothing.
        other = {
            **payload,
            "sessionId": "another",
            "toolArgs": {"path": str(self.repo)},
        }
        self.assertEqual(self.run_hook("file_context.py", other), {})

    def test_saving_waits_for_a_binary_that_can_cite_copilot(self) -> None:
        payload = {
            "sessionId": "current",
            "cwd": str(self.repo),
            "prompt": "from now on, always use tabs in this repo",
        }
        # A binary that predates Copilot citations: no instruction it cannot honour.
        self.assertEqual(self.run_hook("prompt_recall.py", payload), {})
        self.env["FAKE_SESSIONS"] = "claude-code,codex"
        self.assertEqual(self.run_hook("prompt_recall.py", payload), {})
        self.env["FAKE_SESSIONS"] = "claude-code,codex,copilot"
        output = self.run_hook("prompt_recall.py", payload)
        self.assertEqual(list(output), ["additionalContext"])
        self.assertIn('"session_id":"current"', output["additionalContext"])
        # The same binary still serves the hosts it always could, by their shape.
        self.env["FAKE_SESSIONS"] = ""
        claude = {
            "session_id": "current",
            "cwd": str(self.repo),
            "prompt": payload["prompt"],
        }
        wrapped = self.run_hook("prompt_recall.py", claude)
        self.assertIn("hookSpecificOutput", wrapped)

    def test_session_start_speaks_copilots_shape(self) -> None:
        self.env["FAKE_SESSIONS"] = "copilot"
        payload = {"sessionId": "current", "cwd": str(self.repo), "source": "new"}
        output = self.run_hook("session_start.py", payload)
        self.assertEqual(list(output), ["additionalContext"])
        self.assertIn("Mash memory:", output["additionalContext"])


if __name__ == "__main__":
    unittest.main()
