import { expect, type Page, test } from "@playwright/test";
import { fitsViewport, searchBox } from "./shell";

const now = Date.now();
const chats = Array.from({ length: 120 }, (_, index) => ({
  chat_id: `chat-${index}`,
  node_id: "local",
  provider: "codex",
  title: `Conversation ${index}`,
  project: "/work/project",
  updated_at: now - index * 60_000,
}));

async function mockCatalogue(page: Page, history = chats): Promise<void> {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ json: { local_node: "local", truncated: false, chats: history } }),
  );
  await page.route("**/api/chat?**", (route) => route.fulfill({ json: { messages: [] } }));
}

test("the chat list pages through the catalogue and Back keeps the page", async ({ page }) => {
  await mockCatalogue(page);
  const main = page.getByRole("main");
  const rows = main.getByRole("link", { name: /^Conversation \d+/ });
  const pages = page.getByRole("navigation", { name: "Pages" });

  // The earlier address for this list still opens it.
  await page.goto("/#/recent");
  await expect(rows).toHaveCount(10);
  await expect(pages).toContainText("1–10 of 120 chats");
  await expect(page.getByRole("link", { name: "Previous" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Previous" })).toBeDisabled();

  await page.getByRole("link", { name: "Next" }).click();
  await expect(page).toHaveURL(/#\/search\?offset=10$/);
  await expect(pages).toContainText("11–20 of 120 chats");
  await expect(rows).toHaveCount(10);
  await expect(page.getByRole("link", { name: /Conversation 10\b/ })).toBeVisible();

  await page.getByRole("link", { name: /Conversation 13\b/ }).click();
  await expect(page).toHaveURL(/#\/chat\//);
  await page.goBack();
  await expect(pages).toContainText("11–20 of 120 chats");
  await expect(page.getByRole("link", { name: /Conversation 13\b/ })).toBeVisible();

  await page.getByRole("link", { name: "Previous" }).click();
  await expect(pages).toContainText("1–10 of 120 chats");

  await page.goto("/#/search?offset=110");
  await expect(pages).toContainText("111–120 of 120 chats");
  await expect(rows).toHaveCount(10);
  await expect(page.getByRole("link", { name: "Next" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Next" })).toBeDisabled();

  // A page number from the earlier design is not an offset: it opens the list's start.
  await page.goto("/#/recent?page=99");
  await expect(pages).toContainText("1–10 of 120 chats");
});

test("an offset past the end of the chat list shows the last page", async ({ page }) => {
  await mockCatalogue(page);
  await page.goto("/#/search?offset=500");
  await expect(page.getByRole("navigation", { name: "Pages" })).toContainText(
    "111–120 of 120 chats",
  );
  await expect(page.getByRole("heading", { name: /^No (results|chats)/ })).toHaveCount(0);
});

test("a new search from a later page of chats starts at the first result", async ({ page }) => {
  await mockCatalogue(page);
  const offsets: (string | null)[] = [];
  await page.route("**/api/search?**", (route) => {
    offsets.push(new URL(route.request().url()).searchParams.get("offset"));
    return route.fulfill({ json: { results: [], next_offset: null } });
  });
  await page.goto("/#/search?offset=10");
  await expect(page.getByRole("navigation", { name: "Pages" })).toContainText("11–20 of 120 chats");
  await searchBox(page).fill("needle");
  await searchBox(page).press("Enter");
  await expect(page).toHaveURL(/q=needle/);
  await expect.poll(() => offsets).toEqual(["0"]);
  await expect(page).not.toHaveURL(/offset=/);
});

test("clearing filters after an empty search keeps the query, mode, and ranking", async ({
  page,
}) => {
  await mockCatalogue(page);
  const requests: URLSearchParams[] = [];
  await page.route("**/api/search?**", (route) => {
    const params = new URL(route.request().url()).searchParams;
    requests.push(params);
    const filtered = ["provider", "machine", "project", "days"].some((field) => params.get(field));
    return route.fulfill({
      json: {
        results: filtered
          ? []
          : [{ ...chats[0], title: "Recovered result", matches: [{ message_id: "match" }] }],
        next_offset: null,
      },
    });
  });
  const search =
    "/#/search?q=two+words&intent=exact-mention&agent=codex&machine=local&project=%2Fwork%2Fproject&days=7&rank=recent&offset=20";
  await page.goto(search);
  await expect(page.getByRole("heading", { name: /^No results/ })).toBeVisible();
  await searchBox(page).fill("unsubmitted text");
  expect(requests).toHaveLength(1);
  await page.getByRole("link", { name: "Clear filters", exact: true }).click();
  await expect(page.getByRole("link", { name: /Recovered result/ })).toBeVisible();
  expect(Object.fromEntries(requests.at(-1) ?? [])).toEqual({
    q: "two words",
    intent: "exact-mention",
    sort: "recent",
    provider: "",
    machine: "",
    project: "",
    offset: "0",
  });
  await expect(searchBox(page)).toHaveValue("two words");
  await page.goBack();
  await expect(page).toHaveURL(search);
  await expect(page.getByRole("link", { name: "Clear filters", exact: true })).toBeVisible();
});

test("an empty search page can return to the first page and get search help", async ({ page }) => {
  await mockCatalogue(page);
  await page.route("**/api/search?**", (route) =>
    route.fulfill({ json: { results: [], next_offset: null } }),
  );
  await page.goto("/#/search?q=needle&offset=20");
  await expect(
    page.getByRole("heading", { name: "No results on this page", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Find a missing chat", exact: true })).toHaveCount(0);
  await page.getByRole("link", { name: "First page", exact: true }).click();
  await expect(page).toHaveURL("/#/search?q=needle");
  await expect(searchBox(page)).toHaveValue("needle");
  await expect(
    page.getByRole("link", { name: "Find a missing chat", exact: true }),
  ).toHaveAttribute("href", "https://getmash.io/find-missing-chat/");
  await expect(page.getByRole("link", { name: "Troubleshoot search" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Clear filters" })).toHaveCount(0);
});

test("Search again checks for newly indexed matches with the applied query and filters", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockCatalogue(page);
  let indexed = false;
  const requests: string[] = [];
  await page.route("**/api/search?**", (route) => {
    requests.push(route.request().url());
    return route.fulfill({
      json: {
        results: indexed
          ? [{ ...chats[0], title: "Recently indexed chat", matches: [{ message_id: "match" }] }]
          : [],
        next_offset: null,
      },
    });
  });
  const address = "/#/search?q=needle&agent=codex&days=7&intent=exact-mention";
  await page.goto(address);
  await expect(page.getByRole("heading", { name: "No results", exact: true })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Find a missing chat", exact: true }),
  ).toHaveAttribute("href", "https://getmash.io/find-missing-chat/");
  expect(await fitsViewport(page)).toBe(true);

  indexed = true;
  await searchBox(page).fill("unsubmitted draft");
  expect(requests).toHaveLength(1);
  await page.getByRole("button", { name: "Search again", exact: true }).click();
  await expect(page.getByRole("link", { name: /Recently indexed chat/ })).toBeVisible();
  expect(requests).toHaveLength(2);
  expect(requests[1]).toBe(requests[0]);
  await expect(page).toHaveURL(address);
  await expect(searchBox(page)).toHaveValue("unsubmitted draft");
});

test("pressing Enter repeats an unchanged or previously cached search", async ({ page }) => {
  await mockCatalogue(page);
  const requests: string[] = [];
  await page.route("**/api/search?**", (route) => {
    const query = new URL(route.request().url()).searchParams.get("q") ?? "";
    requests.push(query);
    return route.fulfill({
      json: {
        results: [
          {
            ...chats[0],
            title: `${query} result ${requests.length}`,
            matches: [{ message_id: "match" }],
          },
        ],
        next_offset: null,
      },
    });
  });
  await page.goto("/#/search?q=needle");
  await expect(page.getByRole("link", { name: /needle result 1/ })).toBeVisible();
  await searchBox(page).press("Enter");
  await expect(page.getByRole("link", { name: /needle result 2/ })).toBeVisible();
  await expect(page).toHaveURL("/#/search?q=needle");

  await searchBox(page).fill("other");
  await searchBox(page).press("Enter");
  await expect(page.getByRole("link", { name: /other result 3/ })).toBeVisible();
  await page.goBack();
  await expect(page.getByRole("link", { name: /needle result 2/ })).toBeVisible();
  expect(requests).toEqual(["needle", "needle", "other"]);

  await searchBox(page).fill("other");
  await searchBox(page).press("Enter");
  await expect(page.getByRole("link", { name: /other result 4/ })).toBeVisible();
  expect(requests).toEqual(["needle", "needle", "other", "other"]);
});

test("an empty chat list points to history setup", async ({ page }) => {
  await mockCatalogue(page, []);
  await page.goto("/#/search");
  await expect(page.getByRole("heading", { name: "No chats to show" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Check history setup" })).toHaveAttribute(
    "href",
    "https://getmash.io/troubleshooting/#nothing-is-indexed-yet",
  );
  await expect(page.getByRole("link", { name: "Clear filters" })).toHaveCount(0);
});

test("browsing after an empty search keeps the history filters", async ({ page }) => {
  await mockCatalogue(page, [
    ...chats,
    {
      chat_id: "elsewhere",
      node_id: "local",
      provider: "codex",
      title: "Another project",
      project: "/work/other",
      updated_at: now,
    },
  ]);
  const requests: string[] = [];
  await page.route("**/api/search?**", (route) => {
    requests.push(route.request().url());
    return route.fulfill({
      json: { results: [], next_offset: null, intent: "exact-mention" },
    });
  });
  const original = "/#/search?q=needle&project=%2Fwork%2Fproject&rank=recent";
  await page.goto(original);
  await expect(page.getByRole("button", { name: "search: default (words)" })).toBeVisible();
  await page.getByRole("link", { name: "Browse chats", exact: true }).click();
  await expect(page).toHaveURL("/#/search?project=%2Fwork%2Fproject");
  await expect(searchBox(page)).toHaveValue("");
  await expect(page.getByRole("link", { name: /^Conversation 0\b/ })).toBeVisible();
  await expect(page.getByRole("link", { name: /^Another project/ })).toHaveCount(0);
  expect(requests).toHaveLength(1);

  await page.goBack();
  await expect(page).toHaveURL(original);
  await expect(searchBox(page)).toHaveValue("needle");
  await expect(page.getByRole("heading", { name: "No results", exact: true })).toBeVisible();
});

test("search mode applies on request, follows the result page, and resets for a new query", async ({
  page,
}) => {
  await mockCatalogue(page);
  const requests: { intent: string | null; offset: string | null }[] = [];
  await page.route("**/api/search?**", (route) => {
    const params = new URL(route.request().url()).searchParams;
    const intent = params.get("intent");
    const offset = params.get("offset");
    requests.push({ intent, offset });
    return route.fulfill({
      json: {
        intent: intent ?? "related-discussion",
        results: [
          {
            ...chats[0],
            title: intent === "exact-mention" ? "Word result" : "Default result",
            matches: [{ message_id: "match" }],
          },
        ],
        next_offset: 10,
      },
    });
  });
  await page.goto("/#/search?q=needle&offset=10");
  await expect(page.getByRole("link", { name: /Default result/ })).toBeVisible();
  await page.getByRole("button", { name: "search: default (meaning)" }).click();
  await page.getByRole("menuitemradio", { name: "words", exact: true }).click();
  await expect(page.getByRole("link", { name: /Default result/ })).toBeVisible();
  expect(requests).toEqual([{ intent: null, offset: "10" }]);

  await page.getByRole("button", { name: "Apply filters" }).click();
  await expect(page.getByRole("link", { name: /Word result/ })).toBeVisible();
  await expect(page).toHaveURL(/intent=exact-mention/);
  expect(requests.at(-1)).toEqual({ intent: "exact-mention", offset: "0" });
  await page.getByRole("link", { name: "Next", exact: true }).click();
  await expect.poll(() => requests.at(-1)).toEqual({ intent: "exact-mention", offset: "10" });
  await expect(page.getByRole("button", { name: "search: words" })).toBeVisible();

  await searchBox(page).fill("another query");
  await searchBox(page).press("Enter");
  await expect(page).toHaveURL(/#\/search\?q=another\+query$/);
  await expect(page.getByRole("link", { name: /Default result/ })).toBeVisible();
  expect(requests.at(-1)).toEqual({ intent: null, offset: "0" });
  await page.goBack();
  await expect(page.getByRole("button", { name: "search: words" })).toBeVisible();
});
