import json
import math
import subprocess
import tempfile
import unittest
from pathlib import Path

import check_file_lines
import file_growth


def lines(count: int) -> str:
    return "x\n" * count


class HeadroomTests(unittest.TestCase):
    def test_the_cost_is_small_for_short_files_and_climbs_steeply_near_the_limit(self):
        self.assertEqual(file_growth.barrier(0, 500), 0.0)
        self.assertAlmostEqual(file_growth.barrier(250, 500), math.log(2))
        self.assertAlmostEqual(file_growth.barrier(450, 500), math.log(10))
        self.assertEqual(file_growth.barrier(500, 500), math.inf)
        # Ten lines cost far more near the limit than far from it.
        near = file_growth.barrier(495, 500) - file_growth.barrier(485, 500)
        far = file_growth.barrier(110, 500) - file_growth.barrier(100, 500)
        self.assertGreater(near, 20 * far)

    def test_files_recorded_as_debt_have_no_room_to_measure(self):
        total = file_growth.headroom({"a.rs": 250, "big.rs": 900}, 500, {"big.rs": 900})
        self.assertAlmostEqual(total, math.log(2))


class GrowthTests(unittest.TestCase):
    def test_only_growth_to_the_threshold_needs_a_review(self):
        measured = {"same.rs": 470, "shrank.rs": 460, "grew.rs": 451, "new.rs": 455, "low.rs": 449}
        before = {"same.rs": 470, "shrank.rs": 480, "grew.rs": 300, "low.rs": 100}
        self.assertEqual(file_growth.grown(measured, before, {}), ["grew.rs", "new.rs"])
        # A recorded debt file has its own rule: it may not grow at all.
        self.assertEqual(file_growth.grown({"big.rs": 900}, {}, {"big.rs": 900}), [])

    def test_a_review_covers_the_version_it_was_written_for_with_every_field(self):
        text = lines(460)
        review = json.loads(file_growth.template("a.rs", text, 460))
        self.assertEqual(file_growth.review_problem(text, None), "has no growth review")
        self.assertIn("without responsibility", file_growth.review_problem(text, json.dumps(review)))
        for field in file_growth.REVIEW_FIELDS:
            review[field] = "written"
        self.assertIsNone(file_growth.review_problem(text, json.dumps(review)))
        self.assertIn("changed since", file_growth.review_problem(text + "y\n", json.dumps(review)))
        self.assertIn("not JSON", file_growth.review_problem(text, "{"))


class CheckAgainstARevisionTests(unittest.TestCase):
    """The check itself, run on a small repository."""

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        self.write("app/a.rs", lines(440))
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")

    def tearDown(self):
        self.directory.cleanup()

    def git(self, *arguments: str) -> None:
        subprocess.run(["git", *arguments], cwd=self.root, check=True, capture_output=True)

    def write(self, path: str, text: str) -> None:
        file = self.root / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)

    def run_check(self) -> int:
        return check_file_lines.main(["--root", str(self.root), "--against", "HEAD"])

    def test_growth_past_the_threshold_fails_until_reviewed_and_again_when_it_changes(self):
        self.assertEqual(self.run_check(), 0)
        self.write("app/a.rs", lines(460))
        self.assertEqual(self.run_check(), 1)
        sources = {"app/a.rs": (self.root / "app/a.rs").read_text()}
        review = json.loads(file_growth.template("app/a.rs", sources["app/a.rs"], 460))
        for field in file_growth.REVIEW_FIELDS:
            review[field] = "one parser; a split would cut its state machine in two"
        self.write("tools/file-reviews/app/a.rs.json", json.dumps(review))
        self.assertEqual(self.run_check(), 0)
        self.write("app/a.rs", lines(461))
        self.assertEqual(self.run_check(), 1, "the review covered the earlier version")

    def test_a_revision_that_cannot_be_read_skips_the_review(self):
        self.write("app/a.rs", lines(460))
        code = check_file_lines.main(["--root", str(self.root), "--against", "no-such-revision"])
        self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
