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
  // Recent chats name no message to jump to.
  await expect(page.getByRole("option").first().getByRole("link")).toHaveAttribute(
    "href",
    "#/chat/c1?node=laptop",
  );
  const more = page.getByRole("button", { name: "More" });
  await more.click();
  // Linux has no Command key: the labels name Ctrl, which is what applies.
  await expect(page.getByRole("menuitem", { name: /Open Mash/ })).toContainText("Ctrl+O");
  await expect(page.getByRole("menuitem", { name: /Settings…/ })).toHaveAttribute(
    "href",
    "#/settings",
  );
  await expect(page.getByRole("link", { name: "Search Everywhere Ctrl+Enter" })).toHaveAttribute(
    "href",
    "#/search",
  );
  // Escape closes the menu from the button, not only from the field.
  await more.press("Escape");
  await expect(page.getByRole("menu")).toHaveCount(0);
  await expect(field).toBeFocused();
  // A click outside the menu closes it too.
  await more.click();
  await expect(page.getByRole("menu")).toBeVisible();
  await page.getByRole("heading", { name: "Recent chats" }).click();
  await expect(page.getByRole("menu")).toHaveCount(0);
});

test("on a Mac the shortcuts are written with Command", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "platform", { get: () => "MacIntel" });
    Object.defineProperty(navigator, "userAgentData", { get: () => undefined });
  });
  await mock(page);
  await page.goto("/#/quick");
  await expect(page.getByRole("link", { name: "Search Everywhere ⌘↵" })).toBeVisible();
  await page.getByRole("button", { name: "More" }).click();
  await expect(page.getByRole("menuitem", { name: /Open Mash/ })).toContainText("⌘O");
  await expect(page.getByRole("menuitem", { name: /Settings…/ })).toHaveAttribute(
    "aria-keyshortcuts",
    "Meta+,",
  );
});

test("copying a row's link says so", async ({ page }) => {
  await page.addInitScript(() => {
    const copies: string[] = [];
    Object.assign(window, { copies });
    Object.defineProperty(navigator, "clipboard", {
      get: () => ({ writeText: async (text: string) => void copies.push(text) }),
    });
  });
  await mock(page);
  await page.goto("/#/quick");
  await expect(page.getByRole("option")).toHaveCount(3);
  await page.getByRole("textbox", { name: "Search Mash" }).press("Control+c");
  await expect(page.getByRole("status")).toHaveText("Link copied");
  expect(await page.evaluate(() => (window as unknown as { copies: string[] }).copies)).toEqual([
    expect.stringMatching(/#\/chat\/c1\?node=laptop$/),
  ]);
  await expect(page.getByRole("status")).toHaveText("Up to date · 1,073 chats");
});

test("ages move on while the panel stays open", async ({ page }) => {
  await page.clock.install({ time: now });
  await mock(page);
  await page.goto("/#/quick");
  const first = page.getByRole("option").first();
  await expect(first).toContainText("2h");
  await page.clock.runFor(3 * 3_600_000);
  await expect(first).toContainText("5h");
});

test("when Mash isn't responding the panel says so, searches still fail visibly, and retry recovers", async ({
  page,
}) => {
  await page.route("**/api/**", (route) => route.abort());
  await page.goto("/#/quick");
  const alert = page.getByRole("alert");
  await expect(alert).toContainText("Mash isn’t responding.");
  await expect(alert.getByRole("link", { name: "Open Mash" })).toHaveAttribute("href", "#/");
  await expect(page.getByRole("status")).toHaveText("Search unavailable");
  await page.getByRole("textbox", { name: "Search Mash" }).fill("deplo");
  await expect(alert).toContainText("Mash couldn’t search.");
  await expect(alert).toContainText("Could not reach Mash.");

  await page.unroute("**/api/**");
  await mock(page);
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(page.getByRole("option")).toHaveCount(2);
  await page.getByRole("textbox", { name: "Search Mash" }).fill("");
  await expect(page.getByRole("option")).toHaveCount(3);
  await expect(page.getByRole("status")).toHaveText("Up to date · 1,073 chats");
});

test("with no chats yet the panel offers the next step", async ({ page }) => {
  const unset = {
    ...ready,
    ready: false,
    chats: 0,
    recommended_action: "init",
    watcher: null,
    configuration: { ...ready.configuration, loaded: false },
  };
  await mock(page, unset);
  await page.unroute("**/api/chats**");
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ json: { local_node: "laptop", truncated: false, chats: [] } }),
  );
  await page.goto("/#/quick");
  await expect(page.getByText("Mash isn’t set up on this machine yet.")).toBeVisible();
  await expect(page.getByRole("link", { name: "Open Mash to set up" })).toHaveAttribute(
    "href",
    "#/setup",
  );
  await page.unroute(statusPath);
  await page.route(statusPath, (route) =>
    route.fulfill({ json: { ...ready, chats: 0, recommended_action: null } }),
  );
  await page.reload();
  await expect(page.getByRole("link", { name: "Connect an agent" })).toHaveAttribute(
    "href",
    "#/setup",
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

test("the quick panel follows the system's dark appearance while the dashboard stays light", async ({
  page,
}) => {
  await mock(page);
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/#/quick");
  await expect(page.getByRole("heading", { name: "Recent chats" })).toBeVisible();
  // The panel offers both schemes, so the system's dark Canvas colours apply.
  expect(
    await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme),
  ).toContain("dark");
  const ground = await page
    .locator(".quick")
    .evaluate((element) => getComputedStyle(element).backgroundColor);
  expect(ground).not.toBe("rgb(255, 255, 255)");
  await page.goto("/#/");
  expect(await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(
    "light",
  );
});

test("the panel follows the viewing scope chosen in the dashboard", async ({ page }) => {
  const elsewhere = {
    chat_id: "w1",
    node_id: "workstation",
    provider: "codex",
    title: "Deploy the workstation build",
    project: "/work/infra",
    updated_at: now - 3_600_000,
  };
  await page.route(statusPath, (route) => route.fulfill({ json: ready }));
  await page.route("**/api/viewing**", (route) => route.fulfill({ json: { machines: [] } }));
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: { local_node: "laptop", truncated: false, joined: true, chats: [...chats, elsewhere] },
    }),
  );
  await page.route("**/api/search/live?**", (route) =>
    route.fulfill({
      json: {
        results: [
          { ...elsewhere, matches: [{ message_id: "m-1", snippet: "deploy on workstation" }] },
          { ...chats[0], matches: [{ message_id: "m-3", snippet: "the deploy step retries" }] },
        ],
        next_offset: null,
      },
    }),
  );
  // The dashboard chose "This machine"; the panel reads the same browser storage.
  await page.addInitScript(() => localStorage.setItem("mash.scope", "local"));
  await page.goto("/#/quick");
  const options = page.getByRole("option");
  await expect(options).toHaveCount(3);
  await expect(page.getByText("Deploy the workstation build")).toHaveCount(0);
  await page.getByRole("textbox", { name: "Search Mash" }).fill("deploy");
  await expect(options).toHaveCount(1);
  await expect(options.first()).toContainText("the deploy step retries");
});
