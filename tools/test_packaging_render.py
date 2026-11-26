"""The winget manifests packaging/render.py fills for a release: two packages."""

import re
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "packaging"))

import render

ARCHIVES = (
    "mash-x86_64-pc-windows-msvc.zip",
    "mash-aarch64-pc-windows-msvc.zip",
    "mash-desktop-windows-x64-setup.exe",
    "mash-desktop-windows-arm64-setup.exe",
)
# Made-up digests, one per archive: the archive's position repeated.
SUMS = "".join(f"{str(index) * 64}  {name}\n" for index, name in enumerate(ARCHIVES, start=1))
KINDS = ("installer.yaml", "locale.en-US.yaml", "yaml")


def field(text: str, name: str) -> str:
    found = re.search(rf"^{name}: (.+)$", text, re.MULTILINE)
    if not found:
        raise AssertionError(f"no {name} in:\n{text}")
    return found.group(1)


class WingetManifestTests(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.output = Path(scratch.name)
        self.written = render.render_winget(ROOT / "packaging/winget", "1.2.3", SUMS, self.output)

    def manifest(self, identifier: str, kind: str) -> str:
        return (self.output / identifier / f"{identifier}.{kind}").read_text()

    def test_each_package_gets_its_three_manifests_in_its_own_directory(self):
        expected = {
            self.output / identifier / f"{identifier}.{kind}"
            for identifier in ("MashHQ.Mash", "MashHQ.Mash.CLI")
            for kind in KINDS
        }
        self.assertEqual(set(self.written), expected)
        for path in expected:
            text = path.read_text()
            self.assertEqual(field(text, "PackageIdentifier"), path.parent.name)
            self.assertEqual(field(text, "PackageVersion"), "1.2.3")
            self.assertNotRegex(text, r"@[A-Z0-9_]+(:[\w.-]+)?@")
        # One schema version across both packages.
        self.assertEqual(len({field(path.read_text(), "ManifestVersion") for path in expected}), 1)

    def test_the_app_package_installs_the_nsis_installers(self):
        installer = self.manifest("MashHQ.Mash", "installer.yaml")
        self.assertEqual(field(installer, "InstallerType"), "nullsoft")
        self.assertEqual(field(installer, "Scope"), "user")
        self.assertEqual(field(installer, "UpgradeBehavior"), "install")
        self.assertEqual(field(installer, "ProductCode"), "Mash")
        self.assertEqual(field(installer, "  Silent"), "/S")
        self.assertEqual(
            re.findall(r"InstallerUrl: (\S+)\n    InstallerSha256: (\S+)", installer),
            [
                ("https://github.com/mashhq/mash/releases/download/v1.2.3/mash-desktop-windows-x64-setup.exe", "3" * 64),
                ("https://github.com/mashhq/mash/releases/download/v1.2.3/mash-desktop-windows-arm64-setup.exe", "4" * 64),
            ],
        )
        self.assertEqual(re.findall(r"- Architecture: (\w+)", installer), ["x64", "arm64"])
        # The command line is a package of its own and not yet a dependency:
        # only Microsoft's runtime is.
        self.assertEqual(
            re.findall(r"- PackageIdentifier: (\S+)", installer),
            ["Microsoft.VCRedist.2015+.x64", "Microsoft.VCRedist.2015+.arm64"],
        )
        self.assertNotIn("Commands:", installer)
        locale = self.manifest("MashHQ.Mash", "locale.en-US.yaml")
        self.assertEqual(field(locale, "PackageName"), "Mash")
        self.assertEqual(field(locale, "Moniker"), "mash")
        self.assertIn("MashHQ.Mash.CLI", locale)
        self.assertEqual(field(locale, "ReleaseNotesUrl"), "https://github.com/mashhq/mash/releases/tag/v1.2.3")

    def test_the_command_line_package_puts_both_programs_on_the_path(self):
        installer = self.manifest("MashHQ.Mash.CLI", "installer.yaml")
        self.assertEqual(field(installer, "InstallerType"), "zip")
        self.assertEqual(field(installer, "NestedInstallerType"), "portable")
        self.assertEqual(re.findall(r"PortableCommandAlias: (\S+)", installer), ["mash", "mash-server"])
        self.assertEqual(
            re.findall(r"InstallerUrl: (\S+)\n    InstallerSha256: (\S+)", installer),
            [
                ("https://github.com/mashhq/mash/releases/download/v1.2.3/mash-x86_64-pc-windows-msvc.zip", "1" * 64),
                ("https://github.com/mashhq/mash/releases/download/v1.2.3/mash-aarch64-pc-windows-msvc.zip", "2" * 64),
            ],
        )
        locale = self.manifest("MashHQ.Mash.CLI", "locale.en-US.yaml")
        self.assertEqual(field(locale, "PackageName"), "Mash CLI")
        self.assertEqual(field(locale, "Moniker"), "mash-cli")

    def test_the_two_packages_share_no_moniker_or_name(self):
        app, cli = (self.manifest(identifier, "locale.en-US.yaml") for identifier in ("MashHQ.Mash", "MashHQ.Mash.CLI"))
        for name in ("PackageName", "Moniker"):
            self.assertNotEqual(field(app, name), field(cli, name))


class RefusalTests(unittest.TestCase):
    def test_a_release_without_an_installer_is_refused(self):
        with tempfile.TemporaryDirectory() as scratch, self.assertRaises(SystemExit) as refusal:
            render.render_winget(ROOT / "packaging/winget", "1.2.3", f"{'1' * 64}  {ARCHIVES[0]}\n", Path(scratch))
        self.assertIn("SHA256SUMS lacks", str(refusal.exception))

    def test_a_template_in_another_package_directory_is_refused(self):
        with tempfile.TemporaryDirectory() as scratch:
            templates = Path(scratch) / "templates"
            (templates / "Example.Other").mkdir(parents=True)
            (templates / "Example.Other/Example.Other.yaml.in").write_text("# x\nPackageIdentifier: Example.Else\n")
            with self.assertRaises(SystemExit) as refusal:
                render.render_winget(templates, "1.2.3", SUMS, Path(scratch) / "out")
            self.assertIn("is not a manifest of Example.Other", str(refusal.exception))
            with self.assertRaises(SystemExit):
                render.render_winget(Path(scratch) / "none", "1.2.3", SUMS, Path(scratch) / "out")


if __name__ == "__main__":
    unittest.main()
