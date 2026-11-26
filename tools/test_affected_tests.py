import io
import json
import re
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path, PurePosixPath

import affected_tests

REPOSITORY = Path(__file__).resolve().parent.parent
DASHBOARD = REPOSITORY / "dashboard"

# A small dashboard: two pages, a shell that mounts either, and one file of
# each kind the selection has to place.
SOURCES = {
    "src/main.ts": 'import "./app.css";\nimport App from "./App.svelte";\n',
    "src/app.css": '@import "./theme/tokens.css";\n',
    "src/theme/tokens.css": ":root {}\n",
    "src/App.svelte": (
        "<script>\n"
        'import Chat from "./components/Chat.svelte";\n'
        'import Rail from "./components/Rail.svelte";\n'
        'import Search from "./components/Search.svelte";\n'
        'import { session } from "./lib/session";\n'
        "</script>\n"
    ),
    "src/components/Rail.svelte": '<script>import { nav } from "../lib/nav";</script>\n',
    "src/components/Search.svelte": (
        '<script lang="ts">\n'
        "import {\n  rank,\n  type Hit,\n} from \"../lib/search\";\n"
        'import Row from "./Row.svelte";\n'
        "</script>\n"
        '<style>@import "./row.css";</style>\n'
    ),
    "src/components/Chat.svelte": (
        '<script>\nimport Row from "./Row.svelte";\n'
        'const body = () => import("./Body.svelte");\n</script>\n'
    ),
    "src/components/Row.svelte": '<script>import { cn } from "$lib/utils.js";</script>\n',
    "src/components/Body.svelte": "<p>body</p>\n",
    "src/components/Orphan.svelte": "<p>mounted by no mapped route</p>\n",
    "src/components/row.css": ".row {}\n",
    "src/lib/search.ts": 'export { score } from "./score";\nexport type Hit = string;\n',
    "src/lib/score.ts": "export const score = 1;\n",
    "src/lib/score.test.ts": 'import { score } from "./score";\n',
    "src/lib/session.ts": "export const session = 1;\n",
    "src/lib/nav.ts": 'import type { Wire } from "./wire";\nexport const nav = 1;\n',
    "src/lib/wire.ts": "export type Wire = string;\n",
    "src/lib/utils.ts": 'import { clsx } from "clsx";\nexport const cn = clsx;\n',
    "e2e/search.spec.ts": 'import { test } from "@playwright/test";\n',
    "e2e/chat.spec.ts": 'import { test } from "@playwright/test";\n',
    "e2e/both.spec.ts": 'import { test } from "@playwright/test";\n',
    "e2e/shell.ts": "export const rail = 1;\n",
    "e2e/search.live.ts": "export {};\n",
    "package.json": "{}\n",
    "README.md": "notes\n",
}
MAPPING = {
    "everything": ["package.json", "src/theme/*", "src/app.css", "e2e/*.ts"],
    "ignored": ["README.md", "src/*.test.ts", "e2e/*.live.ts"],
    "router": "src/App.svelte",
    "shell": ["src/main.ts", "src/App.svelte", "src/components/Rail.svelte"],
    "routes": {"search": ["src/components/Search.svelte"], "chat": ["src/components/Chat.svelte"]},
    "specs": {"search.spec.ts": ["search"], "chat.spec.ts": ["chat"], "both.spec.ts": ["search", "chat"]},
}
ALL = ["e2e/both.spec.ts", "e2e/chat.spec.ts", "e2e/search.spec.ts"]


class SmallDashboard(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        for name, text in SOURCES.items():
            self.write(name, text)
        self.write(affected_tests.MAPPING, json.dumps(MAPPING))

    def write(self, name: str, text: str) -> None:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def select(self, *changed: str) -> affected_tests.Selection:
        return affected_tests.select(
            changed,
            affected_tests.load_mapping(self.root),
            affected_tests.build_graph(self.root),
            affected_tests.spec_files(self.root),
        )

    def chosen(self, *changed: str) -> list[str]:
        return self.select(*changed).chosen(affected_tests.spec_files(self.root))


class GraphTests(SmallDashboard):
    def test_every_import_form_the_dashboard_uses_is_followed(self):
        graph = affected_tests.build_graph(self.root)
        self.assertEqual(graph.unresolved, {})
        self.assertEqual(
            graph.imports["src/components/Search.svelte"],
            {"src/lib/search.ts", "src/components/Row.svelte", "src/components/row.css"},
        )
        # A re-export, a dynamic import, a type-only import, a stylesheet's
        # @import and the `$lib` alias with a compiled `.js` name.
        self.assertEqual(graph.imports["src/lib/search.ts"], {"src/lib/score.ts"})
        self.assertIn("src/components/Body.svelte", graph.imports["src/components/Chat.svelte"])
        self.assertEqual(graph.imports["src/lib/nav.ts"], {"src/lib/wire.ts"})
        self.assertEqual(graph.imports["src/app.css"], {"src/theme/tokens.css"})
        self.assertEqual(graph.imports["src/components/Row.svelte"], {"src/lib/utils.ts"})
        # A package is not a file of the dashboard's.
        self.assertEqual(graph.imports["src/lib/utils.ts"], frozenset())

    def test_a_query_suffix_and_an_index_file_resolve(self):
        files = frozenset({"src/lib/icons/index.ts", "src/lib/a.svelte.ts", "src/x.svg"})
        importer = PurePosixPath("src/components/A.svelte")
        self.assertEqual(affected_tests.resolve(importer, "../lib/icons", files), "src/lib/icons/index.ts")
        self.assertEqual(affected_tests.resolve(importer, "$lib/a.svelte", files), "src/lib/a.svelte.ts")
        self.assertEqual(affected_tests.resolve(importer, "../x.svg?raw", files), "src/x.svg")
        self.assertIsNone(affected_tests.resolve(importer, "svelte", files))
        self.assertIsNone(affected_tests.resolve(importer, "./missing", files))


class SelectionTests(SmallDashboard):
    def test_a_component_selects_the_specs_whose_routes_import_it(self):
        self.assertEqual(self.chosen("src/components/Search.svelte"), ["e2e/both.spec.ts", "e2e/search.spec.ts"])
        self.assertEqual(self.chosen("src/components/Body.svelte"), ["e2e/both.spec.ts", "e2e/chat.spec.ts"])
        # Through two files, and a stylesheet a component's style imports.
        self.assertEqual(self.chosen("src/lib/score.ts"), ["e2e/both.spec.ts", "e2e/search.spec.ts"])
        self.assertEqual(self.chosen("src/components/row.css"), ["e2e/both.spec.ts", "e2e/search.spec.ts"])
        self.assertEqual(self.chosen("src/components/Row.svelte"), ALL)
        self.assertFalse(self.select("src/components/Row.svelte").everything)

    def test_a_changed_spec_selects_itself(self):
        selection = self.select("e2e/chat.spec.ts")
        self.assertEqual(selection.chosen(ALL), ["e2e/chat.spec.ts"])
        self.assertIn("selects itself", selection.reasons[0])

    def test_selections_add_up_across_files(self):
        self.assertEqual(self.chosen("e2e/chat.spec.ts", "src/lib/score.ts"), ALL)
        self.assertEqual(self.chosen(), [])

    def test_a_shared_foundation_selects_everything_and_says_which(self):
        for path in ("package.json", "src/theme/tokens.css", "src/app.css", "e2e/shell.ts"):
            selection = self.select(path)
            self.assertTrue(selection.everything, path)
            self.assertIn("shared foundation", selection.reasons[0])
            self.assertEqual(selection.chosen(ALL), ALL)

    def test_the_shell_and_what_only_it_imports_select_everything(self):
        for path in ("src/App.svelte", "src/components/Rail.svelte", "src/lib/nav.ts", "src/lib/wire.ts"):
            selection = self.select(path)
            self.assertTrue(selection.everything, path)
            self.assertIn("shell", selection.reasons[0])
        # The router imports each page to mount it; that does not make a page
        # or its files part of the shell.
        self.assertFalse(self.select("src/components/Chat.svelte").everything)

    def test_a_page_file_the_shell_also_imports_selects_everything(self):
        self.write("src/components/Rail.svelte", '<script>import Row from "./Row.svelte";</script>\n')
        selection = self.select("src/components/Row.svelte")
        self.assertTrue(selection.everything)
        self.assertIn("shell", selection.reasons[0])

    def test_a_file_the_tool_cannot_place_selects_everything_and_says_why(self):
        cases = {
            "src/components/Orphan.svelte": "no mapped route imports it",
            "vite.config.ts": "not a file this tool can place",
            "public/logo.svg": "not a file this tool can place",
            "src/components/Deleted.svelte": "not a file this tool can place",
        }
        for path, why in cases.items():
            selection = self.select(path)
            self.assertTrue(selection.everything, path)
            self.assertIn(why, selection.reasons[0])

    def test_unit_tests_notes_and_live_checks_select_nothing(self):
        selection = self.select("src/lib/score.test.ts", "README.md", "e2e/search.live.ts")
        self.assertEqual(selection.chosen(ALL), [])
        self.assertEqual(len(selection.reasons), 3)

    def test_a_spec_missing_from_the_mapping_selects_everything(self):
        self.write("e2e/new.spec.ts", "")
        selection = self.select("src/lib/score.ts")
        self.assertTrue(selection.everything)
        self.assertIn("e2e/new.spec.ts: is not in e2e/affected.json", selection.reasons[0])

    def test_an_import_that_resolves_nowhere_selects_everything(self):
        self.write("src/lib/score.ts", 'import "./gone";\n')
        selection = self.select("src/components/Body.svelte")
        self.assertTrue(selection.everything)
        self.assertIn("'./gone'", selection.reasons[0])

    def test_a_mapping_that_names_an_unknown_route_is_refused(self):
        self.write(affected_tests.MAPPING, json.dumps({**MAPPING, "specs": {"chat.spec.ts": ["chats"]}}))
        with self.assertRaisesRegex(affected_tests.MappingError, "'chats'"):
            affected_tests.load_mapping(self.root)
        self.write(affected_tests.MAPPING, "{")
        with self.assertRaises(affected_tests.MappingError):
            affected_tests.load_mapping(self.root)


class ChangedFilesTests(unittest.TestCase):
    def test_committed_uncommitted_and_untracked_changes_since_the_fork_all_count(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def git(*arguments: str) -> None:
                subprocess.run(["git", "-C", directory, *arguments], check=True, capture_output=True)

            git("init", "-q", "-b", "main")
            git("config", "user.email", "test@example.invalid")
            git("config", "user.name", "Test")
            for name in ("a", "b", "c"):
                (root / name).write_text("1\n")
            git("add", ".")
            git("commit", "-qm", "base")
            git("checkout", "-qb", "work")
            (root / "a").write_text("2\n")
            git("commit", "-qam", "change a")
            # main moves on; what it changed is not this branch's change.
            git("checkout", "-q", "main")
            (root / "c").write_text("2\n")
            git("commit", "-qam", "change c")
            git("checkout", "-q", "work")
            (root / "b").write_text("2\n")
            (root / "new").write_text("1\n")
            self.assertEqual(affected_tests.changed_files(root, "main"), ["a", "b", "new"])


class CommandTests(unittest.TestCase):
    def run_main(self, *arguments: str) -> tuple[int, str, str]:
        printed, noted = io.StringIO(), io.StringIO()
        with redirect_stdout(printed), redirect_stderr(noted):
            status = affected_tests.main(arguments)
        return status, printed.getvalue(), noted.getvalue()

    def test_named_files_print_the_specs_and_the_reasons(self):
        status, printed, noted = self.run_main("dashboard/e2e/quick.spec.ts", "Cargo.toml")
        self.assertEqual(status, 0)
        self.assertEqual(printed, "e2e/quick.spec.ts\n")
        self.assertIn("a changed spec selects itself", noted)
        self.assertIn("1 changed file(s) outside dashboard/", noted)
        self.assertRegex(noted, r"Selected 1 of \d+ specs")

    def test_json_names_everything_when_a_foundation_changed(self):
        status, printed, _ = self.run_main("--json", "dashboard/src/theme/tokens.css")
        answer = json.loads(printed)
        self.assertEqual(status, 0)
        self.assertTrue(answer["everything"])
        self.assertEqual(answer["specs"], affected_tests.spec_files(DASHBOARD))

    def test_a_path_typed_inside_the_dashboard_is_read_from_there(self):
        named = affected_tests.from_root("e2e/quick.spec.ts", REPOSITORY, DASHBOARD)
        self.assertEqual(named, "dashboard/e2e/quick.spec.ts")
        named = affected_tests.from_root("dashboard/package.json", REPOSITORY, DASHBOARD)
        self.assertEqual(named, "dashboard/package.json")


class RepositoryTests(unittest.TestCase):
    """`dashboard/e2e/affected.json` against the dashboard as it is now."""

    @classmethod
    def setUpClass(cls):
        cls.mapping = affected_tests.load_mapping(DASHBOARD)
        cls.graph = affected_tests.build_graph(DASHBOARD)
        cls.specs = affected_tests.spec_files(DASHBOARD)

    def test_every_spec_is_mapped_and_every_mapped_spec_exists(self):
        on_disk = {PurePosixPath(spec).name for spec in self.specs}
        self.assertEqual(
            on_disk - set(self.mapping.specs),
            set(),
            "add these specs to dashboard/e2e/affected.json with the routes they visit",
        )
        self.assertEqual(set(self.mapping.specs) - on_disk, set(), "these specs are gone; remove them")
        for spec, routes in self.mapping.specs.items():
            self.assertTrue(routes, f"{spec} visits no route")

    def test_every_named_file_exists(self):
        named = [self.mapping.router, *self.mapping.shell, *self.mapping.route_entries()]
        self.assertEqual([name for name in named if name not in self.graph.imports], [])

    def test_every_route_the_router_knows_is_in_the_mapping(self):
        router = (DASHBOARD / "src/lib/route.svelte.ts").read_text(encoding="utf-8")
        pages = set(re.findall(r'readonly page: "([a-z-]+)"', router))
        self.assertGreater(len(pages), 15)
        self.assertEqual(pages - set(self.mapping.routes), set(), "add these routes to affected.json")

    def test_every_component_the_shell_mounts_is_a_route_or_part_of_the_shell(self):
        mounted = {name for name in self.graph.imports[self.mapping.router] if name.endswith(".svelte")}
        placed = self.mapping.route_entries() | set(self.mapping.shell)
        self.assertEqual(mounted - placed, set(), "name these under routes or shell in affected.json")

    def test_every_local_import_resolves(self):
        self.assertEqual(dict(self.graph.unresolved), {})

    def test_the_shared_foundations_select_everything(self):
        foundations = (
            "src/theme/tokens.css",
            "src/app.css",
            "src/terminal.css",
            "src/kit.css",
            "src/App.svelte",
            "src/components/Rail.svelte",
            "src/components/TopBar.svelte",
            "src/components/TopSearch.svelte",
            "src/lib/components/ui/switch/switch.svelte",
            "src/lib/components/Icon.svelte",
            "src/lib/api.ts",
            "src/lib/route.svelte.ts",
            "src/lib/routes.ts",
            "src/demo/backend.ts",
            "package.json",
            "pnpm-lock.yaml",
            "playwright.config.ts",
            "e2e/shell.ts",
            "e2e/stream.ts",
            "e2e/affected.json",
            "some/new/file",
        )
        for path in foundations:
            selection = affected_tests.select([path], self.mapping, self.graph, self.specs)
            self.assertTrue(selection.everything, path)

    def test_a_page_component_selects_some_specs_but_not_all(self):
        for path in ("src/components/MemoryRow.svelte", "src/components/ServiceRows.svelte"):
            selection = affected_tests.select([path], self.mapping, self.graph, self.specs)
            self.assertFalse(selection.everything, path)
            self.assertTrue(0 < len(selection.specs) < len(self.specs), path)


if __name__ == "__main__":
    unittest.main()
