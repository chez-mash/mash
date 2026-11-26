import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from check_css_budget import BUDGET, check, main, measure, updated


def lines(count: int) -> str:
    return ".a {\n}\n" * (count // 2) + ("\n" if count % 2 else "")


class Tree:
    """A temporary repository with a budget and some stylesheets."""

    def __init__(self, files: dict[str, int], sheets: dict[str, int], exempt=()):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.write_budget(
            {
                "watched": ["app/src/**/*.css", "site/styles/*.css"],
                "exempt": {path: "tokens" for path in exempt},
                "files": files,
                "growth": [],
            }
        )
        for path, count in sheets.items():
            self.sheet(path, count)

    def write_budget(self, budget: dict) -> None:
        path = self.root / BUDGET
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(budget), encoding="utf-8")

    def sheet(self, path: str, count: int) -> None:
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(lines(count), encoding="utf-8")

    def budget(self) -> dict:
        return json.loads((self.root / BUDGET).read_text(encoding="utf-8"))

    def run(self, *arguments: str) -> tuple[int, str]:
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = main(["--root", str(self.root), *arguments])
        return status, output.getvalue()


class CheckTests(unittest.TestCase):
    def test_a_file_at_its_budget_passes(self):
        self.assertEqual(check({"a.css": 10}, {"a.css": 10}), [])

    def test_growth_fails_and_says_where_new_styling_goes(self):
        (finding,) = check({"a.css": 12}, {"a.css": 10})
        self.assertTrue(finding.grows)
        self.assertIn("2 over its budget of 10", finding.message)
        self.assertIn("Tailwind utility classes", finding.message)
        self.assertIn("scoped", finding.message)
        self.assertIn("token roles", finding.message)

    def test_a_new_global_stylesheet_fails(self):
        (finding,) = check({"a.css": 10, "new.css": 4}, {"a.css": 10})
        self.assertEqual(finding.path, "new.css")
        self.assertIn("new global stylesheet", finding.message)

    def test_a_file_that_shrank_or_went_must_have_its_number_lowered(self):
        shrank, gone = check({"a.css": 8}, {"a.css": 10, "old.css": 3})
        self.assertFalse(shrank.grows)
        self.assertIn("--update", shrank.message)
        self.assertEqual(gone.path, "old.css")
        self.assertIn("--update", gone.message)


class MeasureTests(unittest.TestCase):
    def test_watched_patterns_are_counted_and_exempt_files_are_not(self):
        tree = Tree(
            {},
            {
                "app/src/a.css": 6,
                "app/src/deep/b.css": 3,
                "app/src/tokens.css": 40,
                "site/styles/c.css": 2,
                "site/other/d.css": 9,
            },
            exempt=["app/src/tokens.css"],
        )
        self.assertEqual(
            measure(tree.root, tree.budget()),
            {"app/src/a.css": 6, "app/src/deep/b.css": 3, "site/styles/c.css": 2},
        )


class UpdateTests(unittest.TestCase):
    def test_update_lowers_and_drops_but_never_raises(self):
        budget = {"files": {"a.css": 10, "b.css": 5, "old.css": 3}, "growth": []}
        new, refused = updated(budget, {"a.css": 8, "b.css": 5}, None)
        self.assertEqual(refused, [])
        self.assertEqual(new["files"], {"a.css": 8, "b.css": 5})
        same, refused = updated(budget, {"a.css": 8, "b.css": 6}, None)
        self.assertEqual(refused, ["b.css"])
        self.assertIs(same, budget)

    def test_growth_with_a_reason_is_recorded(self):
        budget = {"files": {"a.css": 10}, "growth": [{"file": "x.css"}]}
        new, refused = updated(budget, {"a.css": 12, "new.css": 4}, "Print rules.")
        self.assertEqual(refused, [])
        self.assertEqual(new["files"], {"a.css": 12, "new.css": 4})
        self.assertEqual(
            new["growth"][1:],
            [
                {"file": "a.css", "from": 10, "to": 12, "reason": "Print rules."},
                {"file": "new.css", "from": 0, "to": 4, "reason": "Print rules."},
            ],
        )


class CommandTests(unittest.TestCase):
    def test_the_check_passes_fails_on_growth_and_is_fixed_only_by_removing_it(self):
        tree = Tree({"app/src/a.css": 10}, {"app/src/a.css": 10})
        self.assertEqual(tree.run()[0], 0)
        tree.sheet("app/src/a.css", 14)
        status, output = tree.run()
        self.assertEqual(status, 1)
        self.assertIn("app/src/a.css: 14 lines, 4 over its budget of 10", output)
        status, output = tree.run("--update")
        self.assertEqual(status, 1)
        self.assertIn("--update only lowers the budget", output)
        self.assertEqual(tree.budget()["files"], {"app/src/a.css": 10})

    def test_update_lowers_a_file_that_shrank(self):
        tree = Tree({"app/src/a.css": 10}, {"app/src/a.css": 6})
        self.assertEqual(tree.run()[0], 1)
        self.assertEqual(tree.run("--update")[0], 0)
        self.assertEqual(tree.budget()["files"], {"app/src/a.css": 6})
        self.assertEqual(tree.run()[0], 0)

    def test_allow_growth_raises_the_number_and_keeps_the_reason(self):
        tree = Tree({"app/src/a.css": 10}, {"app/src/a.css": 12})
        reason = "Print rules cannot be written as utilities."
        self.assertEqual(tree.run("--update", "--allow-growth", reason)[0], 0)
        budget = tree.budget()
        self.assertEqual(budget["files"], {"app/src/a.css": 12})
        self.assertEqual(
            budget["growth"],
            [{"file": "app/src/a.css", "from": 10, "to": 12, "reason": reason}],
        )
        self.assertEqual(tree.run()[0], 0)

    def test_allow_growth_needs_update_and_a_real_reason(self):
        tree = Tree({"app/src/a.css": 10}, {"app/src/a.css": 12})
        for arguments in (
            ["--allow-growth", "A long enough reason."],
            ["--update", "--allow-growth", "x"],
        ):
            with (
                self.assertRaises(SystemExit),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                tree.run(*arguments)
        self.assertEqual(tree.budget()["files"], {"app/src/a.css": 10})

    def test_the_repository_budget_matches_its_stylesheets(self):
        root = Path(__file__).resolve().parents[1]
        budget = json.loads((root / BUDGET).read_text(encoding="utf-8"))
        self.assertEqual(check(measure(root, budget), budget["files"]), [])


if __name__ == "__main__":
    unittest.main()
