import { expect, type Page, test } from "@playwright/test";
import { searchBox } from "./shell";

const now = Date.now();
const chats = Array.from({ length: 120 }, (_, index) => ({
  chat_id: `chat-${index}`,
  node_id: "local",
  provider: "codex",
  title: `Conversation ${index}`,
  project: "/work/project",
  updated_at: now - index * 60_000,
}));

async function mockCatalogue(page: Page): Promise<void> {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ json: { local_node: "local", truncated: false, chats } }),
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
  await expect(page.getByText("Nothing matched these filters.")).toHaveCount(0);
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
