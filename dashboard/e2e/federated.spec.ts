import { expect, type Page, type Route, test } from "@playwright/test";
import { fitsViewport, openSearch } from "./shell";

// Federated search: one list from every source in the viewing scope, which
// settles and then freezes (see src/lib/federated.ts). These fixtures answer
// `/api/search/stream` directly with the lines the server would send.

const now = Date.now();
const chat = (id: string, title: string, node = "laptop", minutesAgo = 5) => ({
  chat_id: id,
  node_id: node,
  provider: "codex",
  title,
  project: "/work/billing",
  updated_at: now - minutesAgo * 60_000,
});
const catalogue = [
  chat("queue", "Choose a queue for invoice exports"),
  chat("rename", "Rename the billing tables", "laptop", 60),
  chat("retry", "Retry failed invoice exports overnight", "laptop", 600),
  chat("postgres", "Background jobs in a Postgres table", "laptop", 3000),
  chat("pickup", "Pick up invoice exports in Codex", "workstation", 28),
];

const row = (id: string, matchedBy: string[]) => {
  const found = catalogue.find((entry) => entry.chat_id === id);
  if (!found) throw new Error(id);
  return {
    ...found,
    score: 1,
    matches: [
      {
        message_id: `${id}-m`,
        snippet: `about ${found.title.toLowerCase()}`,
        fusion_score: 1,
        role: "user",
        copies: [],
        projection_span: { start_char: 0, end_char: 3 },
      },
    ],
    matched_by: matchedBy,
  };
};

type Source = { source: string; label: string; state: string } & Record<string, unknown>;
const words: Source = { source: "words", label: "laptop", state: "done", count: 3, elapsed_ms: 40 };
const meaningDone: Source = {
  source: "meaning",
  label: "by meaning",
  state: "done",
  count: 2,
  elapsed_ms: 310,
};
const machinesSearching: Source = {
  source: "machines",
  label: "your other machines",
  state: "searching",
};
const machinesFailed: Source = {
  source: "machines",
  label: "build-box",
  state: "failed",
  message: "didn't answer in 3 s",
  elapsed_ms: 3000,
  timed_out: true,
};

function line(results: object[], sources: Source[]) {
  return {
    query: "invoice exports",
    local_node: "laptop",
    done: !sources.some((source) => source.state === "searching"),
    results,
    sources,
  };
}

async function answer(route: Route, lines: object[]) {
  await route.fulfill({
    status: 200,
    contentType: "application/x-ndjson",
    body: lines.map((entry) => `${JSON.stringify(entry)}\n`).join(""),
  });
}

async function mock(page: Page, stream: (asked: URLSearchParams, count: number) => object[]) {
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/chats")
      return route.fulfill({
        json: { local_node: "laptop", truncated: false, joined: true, chats: catalogue },
      });
    if (url.pathname === "/api/viewing") return route.fulfill({ json: { machines: [] } });
    if (url.pathname === "/api/records")
      return route.fulfill({ json: { records: [], next_offset: null } });
    return route.fulfill({ status: 404, json: { error: { message: "not in this test" } } });
  });
  let count = 0;
  await page.route("**/api/search/stream?**", (route) => {
    count += 1;
    return answer(route, stream(new URL(route.request().url()).searchParams, count));
  });
}

async function shoot(page: Page, name: string) {
  const directory = process.env.MASH_SCREENSHOTS;
  if (!directory) return;
  const width = page.viewportSize()?.width ?? 0;
  await page.screenshot({ path: `${directory}/${name}-${width}.png`, fullPage: true });
}

for (const width of [1440, 390]) {
  test.describe(`at ${width}px`, () => {
    test.use({ viewport: { width, height: width > 500 ? 1000 : 844 } });

    test("the results page shows one list while other machines are still searching", async ({
      page,
    }) => {
      await mock(page, () => [
        line(
          [row("queue", ["words", "meaning"]), row("postgres", ["meaning"])],
          [words, meaningDone, machinesSearching],
        ),
      ]);
      await page.goto("/#/search?q=invoice+exports");
      const results = page.locator(".results .chatrow");
      await expect(results).toHaveCount(2);
      // No headings by source; a chat found only by meaning says so.
      await expect(results.nth(1)).toContainText("by meaning");
      await expect(results.nth(0)).not.toContainText("by meaning");
      await expect(
        page.getByText("2 results so far · Searching your other machines…"),
      ).toBeVisible();
      expect(await fitsViewport(page)).toBe(true);
      await shoot(page, "federated-results-searching");
    });

    test("a source that did not answer can be retried, and new results join below", async ({
      page,
    }) => {
      await mock(page, (_asked, count) =>
        count === 1
          ? [line([row("queue", ["words"]), row("rename", ["words"])], [words, machinesFailed])]
          : // Asked again, another machine ranks its chat first.
            [
              line(
                [row("pickup", ["machines"]), row("queue", ["words"]), row("rename", ["words"])],
                [
                  words,
                  {
                    source: "machines",
                    label: "build-box",
                    state: "done",
                    count: 1,
                    elapsed_ms: 900,
                  },
                ],
              ),
            ],
      );
      await page.goto("/#/search?q=invoice+exports");
      const results = page.locator(".results .chatrow");
      await expect(results).toHaveCount(2);
      await expect(page.getByText("build-box didn't answer")).toBeVisible();
      await page.getByRole("button", { name: "Retry" }).click();
      await expect(results).toHaveCount(3);
      // What was shown keeps its place; the late result is added below.
      await expect(results.nth(0)).toContainText("Choose a queue for invoice exports");
      await expect(results.nth(1)).toContainText("Rename the billing tables");
      await expect(results.nth(2)).toContainText("Pick up invoice exports in Codex");
      await expect(page.getByText("build-box didn't answer")).toHaveCount(0);
      await shoot(page, "federated-results-appended");
    });

    test("the search box keeps the list under the pointer while results by meaning arrive", async ({
      page,
    }) => {
      await mock(page, (asked) =>
        asked.get("words_only") === "true"
          ? [
              line(
                [row("queue", ["words"]), row("rename", ["words"])],
                [
                  words,
                  {
                    source: "meaning",
                    label: "by meaning",
                    state: "skipped",
                    reason: "the query is still short",
                  },
                ],
              ),
            ]
          : // By meaning, a chat whose words differ is ranked first.
            [
              line(
                [
                  row("postgres", ["meaning"]),
                  row("queue", ["words", "meaning"]),
                  row("rename", ["words"]),
                ],
                [words, meaningDone],
              ),
            ],
      );
      await page.goto("/#/");
      await (await openSearch(page)).fill("invoice exports");
      const said = page.locator(".drop a.said");
      await expect(said).toHaveCount(2);
      // The reader moves to the list before search by meaning answers.
      await said.first().hover();
      await expect(said).toHaveCount(3);
      await expect(said.nth(0)).toContainText("Choose a queue for invoice exports");
      await expect(said.nth(1)).toContainText("Rename the billing tables");
      await expect(said.nth(2)).toContainText("Background jobs in a Postgres table");
      await expect(said.nth(2)).toContainText("by meaning");
      await shoot(page, "federated-box");
    });
  });
}

test("the quick panel keeps the highlighted row while results arrive", async ({ page }) => {
  await mock(page, (asked) =>
    asked.get("words_only") === "true"
      ? [
          line(
            [row("queue", ["words"]), row("rename", ["words"])],
            [
              words,
              {
                source: "meaning",
                label: "by meaning",
                state: "skipped",
                reason: "the query is still short",
              },
            ],
          ),
        ]
      : [
          line(
            [row("postgres", ["meaning"]), row("queue", ["words"]), row("rename", ["words"])],
            [words, meaningDone, machinesFailed],
          ),
        ],
  );
  await page.goto("/#/quick");
  const field = page.getByRole("textbox", { name: "Search Mash" });
  await field.fill("invoice exports");
  const options = page.getByRole("option");
  await expect(options).toHaveCount(2);
  await field.press("ArrowDown");
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true");
  // Search by meaning answers after the pause; the highlighted row stays.
  await expect(options).toHaveCount(3);
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true");
  await expect(options.nth(1)).toContainText("Rename the billing tables");
  await expect(options.nth(2)).toContainText("by meaning");
  await expect(page.getByText("build-box didn't answer")).toBeVisible();
  await shoot(page, "federated-quick");
});
