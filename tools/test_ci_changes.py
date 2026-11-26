import io
import os
import re
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

import ci_changes

ROOT = Path(__file__).resolve().parent.parent
# A relative path in a Rust string that climbs out of its crate: what
# `include_str!` and a test's `join("../../…")` both look like.
CLIMBING_PATH = re.compile(r'"((?:\.\./)+[^"\s]+)"')


def answer(*paths: str) -> set[str]:
    return set(ci_changes.affected(paths))


class AreaTests(unittest.TestCase):
    def test_a_change_inside_one_front_end_reaches_only_its_own_jobs(self):
        self.assertEqual(answer("site/src/pages/index.astro", "site/pnpm-lock.yaml"), {"docs"})
        # The website serves the dashboard as its demo.
        self.assertEqual(answer("dashboard/src/components/Rail.svelte"), {"dashboard", "docs"})
        self.assertEqual(answer("dashboard/e2e/quick.spec.ts"), {"dashboard", "docs"})
        self.assertEqual(answer("raycast/src/search.tsx", "raycast/package-lock.json"), {"raycast"})

    def test_a_rust_change_runs_no_front_end_job(self):
        self.assertEqual(answer("crates/mash-dash/src/lib.rs", "Cargo.lock", "app/core.rs"), {"rust"})
        # The binary embeds the user guides, so they are Rust's as well as the website's.
        self.assertEqual(answer("docs/search.md", "README.md", "docs/internal/development.md"), {"rust", "docs"})

    def test_instructions_for_contributors_run_nothing(self):
        self.assertEqual(answer("AGENTS.md", "CONTRIBUTING.md"), set())
        # Nor does the Scoop manifest a release lands.
        self.assertEqual(answer("bucket/mash.json"), set())
        # A front end's own notes stay with that front end.
        self.assertEqual(answer("dashboard/README.md"), {"dashboard", "docs"})

    def test_files_the_rust_build_reads_from_a_front_end_still_run_rust(self):
        self.assertEqual(answer("site/public/setup.md"), {"rust", "docs"})
        self.assertEqual(answer("site/package.json"), {"rust", "docs"})
        self.assertEqual(answer("dashboard/openapi.json"), {"rust", "dashboard", "docs"})

    def test_anything_unclaimed_runs_everything(self):
        everything = set(ci_changes.AREAS)
        unclaimed = (
            ".github/workflows/docs.yml", "tools/visual.sh", "tools/ci_changes.py", "justfile", "Makefile",
            "new-directory/file", "sitemap",
        )
        for path in (*unclaimed, "LICENSE-MIT"):
            self.assertEqual(answer(path), everything, path)
        self.assertEqual(answer("site/a.ts", "tools/worktree.sh"), everything)

    def test_the_selection_tool_and_its_test_run_with_the_dashboard(self):
        self.assertEqual(answer("tools/affected_tests.py"), {"rust", "dashboard"})

    def test_the_output_is_one_line_per_area(self):
        printed = io.StringIO()
        with mock.patch("sys.stdin", io.StringIO("raycast/src/a.ts\n\n")), redirect_stdout(printed):
            self.assertEqual(ci_changes.main([]), 0)
        self.assertEqual(printed.getvalue(), "rust=false\ndashboard=false\nraycast=true\ndocs=false\n")
        printed = io.StringIO()
        with redirect_stdout(printed):
            ci_changes.main(["--all"])
        self.assertEqual(printed.getvalue(), "rust=true\ndashboard=true\nraycast=true\ndocs=true\n")
        printed = io.StringIO()
        with redirect_stdout(printed):
            ci_changes.main(["--none"])
        self.assertEqual(printed.getvalue(), "rust=false\ndashboard=false\nraycast=false\ndocs=false\n")


class RepositoryTests(unittest.TestCase):
    """The exceptions above, checked against what the Rust source really reads."""

    def test_every_front_end_file_a_rust_source_names_still_runs_rust(self):
        skipped: list[str] = []
        sources = [
            path
            for path in ROOT.rglob("*.rs")
            if not {"target", "node_modules"} & set(path.relative_to(ROOT).parts)
        ]
        self.assertGreater(len(sources), 100)
        for source in sources:
            for climbing in CLIMBING_PATH.findall(source.read_text(encoding="utf-8")):
                named = Path(os.path.normpath(source.parent / climbing))
                # A manifest-relative path climbs from the crate, not the file.
                for base in (named, *(Path(os.path.normpath(up / climbing)) for up in source.parents)):
                    if base.is_file() and ROOT in base.parents:
                        relative = base.relative_to(ROOT).as_posix()
                        if "rust" not in ci_changes.areas_for(relative):
                            skipped.append(f"{source.relative_to(ROOT)} reads {relative}")
        self.assertEqual(skipped, [], "claim these for rust in tools/ci_changes.py")


if __name__ == "__main__":
    unittest.main()
