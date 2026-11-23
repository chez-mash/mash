import { expect, type Page, test } from "@playwright/test";

const now = Date.now();
const chats = [
  {
    chat_id: "c1",
    node_id: "laptop",
    provider: "claude-code",
    title: "Fix the deploy retry loop",
    project: "/work/infra",
    updated_at: now - 2 * 3_600_000,
  },
  {
    chat_id: "c2",
    node_id: "laptop",
    provider: "codex",
    title: "Draft the release notes",
    project: "/work/mash",
    updated_at: now - 5 * 3_600_000,
  },
  {
    chat_id: "c3",
    node_id: "laptop",
    provider: "cursor",
    title: "Speed up the search page",
    project: "/work/web",
    updated_at: now - 86_400_000,
  },
];

const statusPath = (url: URL) => url.pathname === "/api/status";

const ready = {
  ready: true,
  chats: 1073,
  messages: 40_000,
  searchable_messages: 39_000,
  query_models: "available",
  local_qdrant: { state: "reachable", url: "http://127.0.0.1:6333" },
  realized_mode: "hybrid",
  recommended_action: null,
  watcher: { definition: "/u/mash-watch.service", running: true },
  configuration: {
    path: "/config/mash/config.toml",
    loaded: true,
    providers: {},
    index: { since_days: null, destination: "local" },
    embeddings: { backend: "local" },
    qdrant: { url: "http://127.0.0.1:6333", vector_search: { algorithm: "hnsw" } },
    views: { query: { ranking: "contextual-rerank" } },
    records: { enabled: false },
    redaction: { output: true, publication: true },
    shared: null,
  },
};

async function mock(page: Page, status: object = ready): Promise<string[]> {
  const typed: string[] = [];
  await page.route(statusPath, (route) => route.fulfill({ json: status }));
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ json: { local_node: "laptop", truncated: false, chats } }),
  );
  await page.route("**/api/search/live?**", (route) => {
    typed.push(new URL(route.request().url()).searchParams.get("q") ?? "");
    return route.fulfill({
      json: {
        results: [
          {
            ...chats[0],
            matches: [{ message_id: "m-3", snippet: "the deploy step retries forever" }],
          },
          {
            ...chats[2],
            matches: [{ message_id: "m-9", snippet: "deploy the search page behind a flag" }],
          },
        ],
        next_offset: null,
      },
    });
  });
  return typed;
}

test("the quick panel lists recent chats, searches as you type and opens a match", async ({
  page,
}) => {
  const typed = await mock(page);
  await page.goto("/#/quick");
  const field = page.getByRole("textbox", { name: "Search Mash" });
  await expect(field).toBeFocused();
  // The panel stands alone: no rail or dashboard search box.
  await expect(page.getByRole("navigation", { name: "Main" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Recent chats" })).toBeVisible();
  const options = page.getByRole("option");
  await expect(options).toHaveCount(3);
  await expect(options.first()).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("status")).toHaveText("Up to date · 1,073 chats");

  await field.fill("deplo");
  await expect(page.getByRole("heading", { name: "Chats on this machine" })).toBeVisible();
  await expect(options).toHaveCount(2);
  await expect(options.first()).toContainText("the deploy step retries forever");
  expect(typed).toEqual(["deplo"]);

  await field.press("ArrowDown");
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true");
  await field.press("ArrowDown");
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true");
  await field.press("Enter");
  // In a browser a match opens the dashboard's own chat page.
  await expect(page).toHaveURL(/#\/chat\/c3\?node=laptop&message=m-9$/);
});

test("Escape clears the query before anything else, and the menu reaches the main pages", async ({
  page,
}) => {
  await mock(page);
  await page.goto("/#/quick");
  const field = page.getByRole("textbox", { name: "Search Mash" });
  await field.fill("deplo");
  await field.press("Escape");
  await expect(field).toHaveValue("");
  await page.getByRole("button", { name: "More" }).click();
  await expect(page.getByRole("menuitem", { name: /Open Mash/ })).toContainText("⌘O");
  await expect(page.getByRole("menuitem", { name: /Settings…/ })).toHaveAttribute(
    "href",
    "#/settings",
  );
  await expect(page.getByRole("link", { name: "Search Everywhere ⌘↵" })).toHaveAttribute(
    "href",
    "#/search",
  );
});

test("the footer is honest while a new machine indexes, and when search is down", async ({
  page,
}) => {
  await mock(page, { ...ready, ready: false, chats: 0, recommended_action: "index" });
  await page.goto("/#/quick");
  await expect(page.getByRole("status")).toHaveText("Indexing…");
  await page.unroute(statusPath);
  await page.route(statusPath, (route) =>
    route.fulfill({ status: 503, json: { error: { message: "down" } } }),
  );
  await page.reload();
  await expect(page.getByRole("status")).toHaveText("Search unavailable");
});

test("Control+, opens settings from the field", async ({ page }) => {
  await mock(page);
  await page.goto("/#/quick");
  await page.getByRole("textbox", { name: "Search Mash" }).press("Control+,");
  await expect(page).toHaveURL(/#\/settings$/);
});
