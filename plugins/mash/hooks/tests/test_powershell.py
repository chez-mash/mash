"""Copilot CLI runs hooks through PowerShell on Windows. The `powershell`
commands in copilot.json must hand the payload on stdin to Python, return what
the hook prints, and exit 0 whatever happens."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

HOOKS = Path(__file__).resolve().parent.parent
SHELLS = [shell for shell in ("powershell", "pwsh") if shutil.which(shell)]


@unittest.skipUnless(sys.platform == "win32", "Copilot uses PowerShell on Windows only")
class PowerShellCommands(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.work = directory.name
        self.env = {
            **os.environ,
            "COPILOT_PLUGIN_ROOT": str(HOOKS.parent),
            # No Mash at all: the hooks must still exit cleanly.
            "MASH_BIN": str(Path(self.work) / "absent-mash.exe"),
            "MASH_RECALL_STATE_DIR": str(Path(self.work) / "state"),
            "MASH_AUTO_INDEX": "off",
        }
        self.env.pop("MASH_PROMPT_RECALL", None)
        definition = json.loads((HOOKS / "copilot.json").read_text(encoding="utf-8"))
        self.commands = {
            event: entries[0]["powershell"]
            for event, entries in definition["hooks"].items()
        }

    def run_hook(self, shell: str, event: str, payload: dict[str, Any]) -> str:
        command = [
            shell,
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            self.commands[event],
        ]
        result = subprocess.run(
            command,
            input=json.dumps(payload).encode(),
            capture_output=True,
            env=self.env,
            timeout=120,
        )
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
        return result.stdout.decode(errors="replace").strip()

    def test_a_shell_is_present(self) -> None:
        self.assertTrue(SHELLS, "no PowerShell on PATH")

    def test_the_prompt_hook_reads_stdin_and_answers_in_copilots_shape(self) -> None:
        payload = {
            "sessionId": "windows-probe",
            "cwd": self.work,
            "prompt": "what did we decide about the watermark location?",
        }
        for shell in SHELLS:
            with self.subTest(shell=shell):
                output = json.loads(
                    self.run_hook(shell, "userPromptSubmitted", payload)
                )
                self.assertEqual(list(output), ["additionalContext"])
                self.assertIn("Mash", output["additionalContext"])

    def test_every_hook_exits_cleanly_without_mash(self) -> None:
        start = {"sessionId": "windows-probe", "cwd": self.work, "source": "new"}
        read = {
            "sessionId": "windows-probe",
            "cwd": self.work,
            "toolName": "view",
            "toolArgs": {"path": str(HOOKS / "session_start.py")},
        }
        for shell in SHELLS:
            with self.subTest(shell=shell):
                self.assertEqual(self.run_hook(shell, "sessionStart", start), "")
                self.assertEqual(self.run_hook(shell, "postToolUse", read), "")


if __name__ == "__main__":
    unittest.main()
