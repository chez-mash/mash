import tempfile
import unittest
from pathlib import Path

from css_blocks import names_in, parse
from unused_css import Package, Usage, prune, sheets_of, survey, usage_of


def usage(*words: str, prefixes=(), suffixes=(), third_party=(), kept=None) -> Usage:
    return Usage(set(words), set(prefixes), set(suffixes), set(third_party), kept or {})


class SelectorTests(unittest.TestCase):
    def test_names_are_classes_and_ids_but_not_numbers_or_attribute_values(self):
        self.assertEqual(
            names_in('#rail .nav-row.on > a[href$=".png"]:hover, .w-1\\.5 { }'),
            {("#", "rail"), (".", "nav-row"), (".", "on"), (".", "w-1")},
        )
        self.assertEqual(names_in("margin: 0.5rem"), set())

    def test_a_name_inside_not_is_where_or_has_is_not_required(self):
        selector = ".card:not(.gone) :is(.a, .b) :where(.c):has(.d)"
        self.assertEqual(len(names_in(selector)), 6)
        self.assertEqual(names_in(selector, required_only=True), {(".", "card")})

    def test_blocks_nest_and_strings_and_comments_hide_their_braces(self):
        rules = parse(
            '/* } */ .a::after { content: "}"; } @media (width > 1px) { .b { color: red; } }'
        )
        self.assertEqual(
            [rule.prelude for rule in rules], [".a::after", "@media (width > 1px)"]
        )
        self.assertEqual([child.prelude for child in rules[1].children], [".b"])
        self.assertTrue(rules[0].declarations)
        self.assertFalse(rules[1].declarations)


class VerdictTests(unittest.TestCase):
    def test_a_name_is_used_kept_third_party_in_a_family_or_unused(self):
        known = usage(
            "row",
            prefixes=["tone-"],
            suffixes=["-pill"],
            third_party=["sl-link"],
            kept={"shown": "GSAP"},
        )
        self.assertEqual(
            [
                known.verdict(name)
                for name in (
                    "row",
                    "shown",
                    "sl-link",
                    "tone-ok",
                    "kind-pill",
                    "tone-",
                    "old",
                )
            ],
            ["used", "kept", "third-party", "family", "family", "unused", "unused"],
        )
        self.assertEqual(known.family("tone-ok"), "tone-*")
        self.assertEqual(known.family("kind-pill"), "*-pill")

    def test_a_name_judged_dead_by_hand_is_unused_even_when_its_word_appears(self):
        judged = Usage({"flip"}, set(), set(), set(), {}, frozenset({"flip"}))
        self.assertEqual(judged.verdict("flip"), "unused")


class PruneTests(unittest.TestCase):
    def test_a_rule_goes_only_when_every_selector_is_dead(self):
        css = ".a {\n  color: red;\n}\n.a .old,\n.old > b {\n  color: blue;\n}\n.a,\n.old {\n  margin: 0;\n}\n"
        self.assertEqual(
            prune(css, usage("a")), ".a {\n  color: red;\n}\n.a {\n  margin: 0;\n}\n"
        )

    def test_an_unused_name_in_not_keeps_the_rule(self):
        css = ".a:not(.old) {\n  color: red;\n}\n"
        self.assertEqual(prune(css, usage("a")), css)

    def test_an_at_rule_block_goes_when_emptied_and_stays_otherwise(self):
        css = "@media print {\n  .old {\n    color: red;\n  }\n}\n@media print {\n  .old {\n    color: red;\n  }\n  .a {\n    color: red;\n  }\n}\n"
        self.assertEqual(
            prune(css, usage("a")), "@media print {\n  .a {\n    color: red;\n  }\n}\n"
        )

    def test_keyframes_and_other_opaque_blocks_are_left_alone(self):
        css = "@keyframes spin {\n  to {\n    rotate: 1turn;\n  }\n}\n@theme {\n  --color-*: initial;\n}\n"
        self.assertEqual(prune(css, usage()), css)

    def test_a_nested_rule_dies_with_its_parent_and_alone(self):
        css = ".a {\n  color: red;\n  .old {\n    color: blue;\n  }\n  b {\n    color: green;\n  }\n}\n.old {\n  b {\n    color: red;\n  }\n}\n"
        self.assertEqual(
            prune(css, usage("a")),
            ".a {\n  color: red;\n  b {\n    color: green;\n  }\n}\n",
        )

    def test_a_comment_goes_only_when_every_rule_it_introduces_went(self):
        css = "/* Old things. */\n.old {\n  color: red;\n}\n.old b {\n  color: red;\n}\n/* Mixed. */\n.old i {\n  color: red;\n}\n.a {\n  color: red;\n}\n"
        self.assertEqual(
            prune(css, usage("a")), "/* Mixed. */\n.a {\n  color: red;\n}\n"
        )

    def test_a_comment_inside_a_surviving_rule_stays(self):
        css = ".a {\n  /* Why. */\n  color: red;\n}\n"
        self.assertEqual(prune(css, usage("a")), css)


class PackageTests(unittest.TestCase):
    def test_sources_name_classes_and_styles_do_not(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "app/src").mkdir(parents=True)
            (root / "app/node_modules/kit").mkdir(parents=True)
            (root / "app/src/global.css").write_text(
                ".used { }\n.dead { }\n.tone-ok { }\n.kit-part { }\n.scoped-dead { }\n#shown { }\n",
                encoding="utf-8",
            )
            (root / "app/src/Page.svelte").write_text(
                '<div class="used tone-{tone}" class:shown-never={x}></div>\n<style>\n.scoped-dead { }\n.used b { }\n</style>\n',
                encoding="utf-8",
            )
            (root / "app/node_modules/kit/index.js").write_text(
                'el.className = "kit-part"', encoding="utf-8"
            )
            package = Package("app", ("app/src",), ("app",), ("app/node_modules/kit",))
            known = usage_of(root, package, {"shown": "set by the motion library"})
            report = survey(sheets_of(root, package), known)
            self.assertEqual(sorted(report.unused), [".dead", ".scoped-dead"])
            self.assertEqual(len(report.unused[".scoped-dead"]), 2)
            self.assertEqual(report.unused[".dead"][0][1], 2)
            self.assertEqual(report.families, {"tone-*": {".tone-ok"}})
            self.assertEqual(report.dead_rules, 3)


if __name__ == "__main__":
    unittest.main()
