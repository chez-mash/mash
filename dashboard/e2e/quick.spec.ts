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

async function mock(page: Page): Promise<string[]> {
  const typed: string[] = [];
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
  await expect(page.getByRole("menuitem", { name: "Settings…" })).toHaveAttribute(
    "href",
    "#/settings",
  );
  await expect(page.getByRole("link", { name: "Search Everywhere ⌘↵" })).toHaveAttribute(
    "href",
    "#/search",
  );
});
