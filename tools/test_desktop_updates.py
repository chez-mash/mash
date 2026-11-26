import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

import desktop_updates


def order(version: str) -> tuple:
    """Semantic-version precedence, as the app's updater compares versions."""
    core, _, prerelease = version.partition("-")
    numbers = tuple(int(part) for part in core.split("."))
    if not prerelease:
        return (numbers, 1, ())
    identifiers = tuple(
        (0, int(part), "") if part.isdigit() else (1, 0, part)
        for part in prerelease.split(".")
    )
    return (numbers, 0, identifiers)


class MainVersionTests(unittest.TestCase):
    def test_a_build_of_main_sits_between_the_releases_around_it(self):
        build = desktop_updates.main_version("0.2.4", 118)
        self.assertEqual(build, "0.2.5-main.118")
        self.assertLess(order("0.2.4"), order(build))
        self.assertLess(order(build), order("0.2.5"))
        self.assertLess(order(build), order("0.3.0"))

    def test_builds_order_by_number_not_by_text(self):
        ninth, tenth = (
            desktop_updates.main_version("0.2.4", number) for number in (9, 10)
        )
        self.assertLess(order(ninth), order(tenth))
        self.assertLess(
            order(desktop_updates.main_version("0.2.4", 999)), order("0.2.5")
        )
        # The release is cut; the builds after it are newer than it.
        self.assertLess(
            order("0.2.5"), order(desktop_updates.main_version("0.2.5", 1000))
        )

    def test_a_prerelease_keeps_its_place_among_its_builds(self):
        build = desktop_updates.main_version("0.3.0-alpha", 7)
        self.assertEqual(build, "0.3.0-alpha.main.7")
        self.assertLess(order("0.3.0-alpha"), order(build))
        self.assertLess(order(build), order("0.3.0-beta"))
        self.assertLess(order(build), order("0.3.0"))

    def test_refuses_what_is_not_a_version_or_a_build_number(self):
        for version, build in (("v0.2.4", 1), ("0.2", 1), ("0.2.4", 0), ("", 3)):
            with self.assertRaises(ValueError):
                desktop_updates.main_version(version, build)


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.folder = Path(self.enterContext(tempfile.TemporaryDirectory()))

    def signed(self, name: str, signature: str | None = "c2lnbmF0dXJl\n") -> Path:
        file = self.folder / name
        file.write_bytes(b"update")
        if signature is not None:
            (self.folder / f"{name}.sig").write_text(signature, encoding="utf-8")
        return file

    def test_names_each_signed_platform_with_its_address_and_signature(self):
        written = desktop_updates.manifest(
            "0.2.5-main.118",
            "https://github.com/mashhq/mash/releases/download/main-channel/",
            {
                "darwin-aarch64": self.signed(
                    "mash-desktop-macos-arm64-0.2.5-main.118.app.tar.gz"
                ),
                "linux-x86_64": self.signed(
                    "mash-desktop-linux-x86_64-0.2.5-main.118.AppImage"
                ),
                "windows-x86_64": self.signed("unsigned-setup.exe", signature=None),
                "linux-aarch64": self.folder / "never-built.AppImage",
            },
            published=datetime(2026, 11, 26, 9, 30, tzinfo=timezone.utc),
        )
        self.assertEqual(written["version"], "0.2.5-main.118")
        self.assertEqual(written["pub_date"], "2026-11-26T09:30:00Z")
        self.assertEqual(
            written["platforms"],
            {
                "darwin-aarch64": {
                    "signature": "c2lnbmF0dXJl",
                    "url": "https://github.com/mashhq/mash/releases/download/main-channel/mash-desktop-macos-arm64-0.2.5-main.118.app.tar.gz",
                },
                "linux-x86_64": {
                    "signature": "c2lnbmF0dXJl",
                    "url": "https://github.com/mashhq/mash/releases/download/main-channel/mash-desktop-linux-x86_64-0.2.5-main.118.AppImage",
                },
            },
        )

    def test_writes_nothing_when_nothing_was_signed(self):
        output = self.folder / "latest.json"
        unsigned = self.signed("Mash.app.tar.gz", signature=None)
        status = desktop_updates.main(
            [
                "manifest",
                "--version",
                "0.2.5",
                "--base-url",
                "https://example.invalid",
                "--output",
                str(output),
                f"darwin-aarch64={unsigned}",
            ]
        )
        self.assertEqual(status, 3)
        self.assertFalse(output.exists())

    def test_the_command_writes_the_file_the_updater_reads(self):
        output = self.folder / "latest.json"
        signed = self.signed("Mash.app.tar.gz")
        status = desktop_updates.main(
            [
                "manifest",
                "--version",
                "0.2.5",
                "--base-url",
                "https://example.invalid/v0.2.5",
                "--output",
                str(output),
                f"darwin-aarch64={signed}",
            ]
        )
        self.assertEqual(status, 0)
        written = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(
            written["platforms"]["darwin-aarch64"]["url"],
            "https://example.invalid/v0.2.5/Mash.app.tar.gz",
        )
        self.assertEqual(
            desktop_updates.main(
                [
                    "manifest",
                    "--version",
                    "0.2.5",
                    "--base-url",
                    "x",
                    "--output",
                    str(output),
                    "plan9-mips=x",
                ]
            ),
            2,
        )
        self.assertEqual(desktop_updates.main(["main-version", "nope", "1"]), 2)


if __name__ == "__main__":
    unittest.main()
