import { expect, type Page, test } from "@playwright/test";
import { rail } from "./shell";

// Pinned projects come first on the projects page and in the sidebar, and
// the pins are the dashboard's, saved through /api/pins.

const hour = 3_600_000;
const chats = ["billing", "storefront", "docs"].map((name, index) => ({
  chat_id: `chat-${name}`,
  provider: "codex",
  title: `Work on ${name}`,
  node_id: "laptop",
  project: `/home/sam/code/${name}`,
  updated_at: Date.now() - (index + 1) * hour,
}));

async function mock(page: Page, options: { editable: boolean; refuse?: boolean }) {
  const saved: string[][] = [];
  let pins: string[] = [];
  await page.route("**/api/**", (route) => {
    const request = route.request();
    const url = new URL(request.url());
    if (url.pathname === "/api/chats")
      return route.fulfill({ json: { local_node: "laptop", truncated: false, chats } });
    if (url.pathname === "/api/pins" && request.method() === "POST") {
      if (options.refuse)
        return route.fulfill({ status: 403, json: { error: { message: "Not here" } } });
      pins = (request.postDataJSON() as { projects: string[] }).projects;
      saved.push(pins);
      return route.fulfill({ json: { projects: pins, editable: true } });
    }
    if (url.pathname === "/api/pins")
      return route.fulfill({ json: { projects: pins, editable: options.editable } });
    return route.fulfill({ status: 404, json: { error: { message: "Not in this fixture" } } });
  });
  return saved;
}

const cards = (page: Page) => page.locator(".sums .card.sum b").first();

test("a pinned project comes first on the projects page and in the sidebar", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  const saved = await mock(page, { editable: true });
  await page.goto("/#/projects");
  await expect(cards(page)).toHaveText("billing");
  await page.getByRole("button", { name: "Pin docs" }).click();
  await expect(cards(page)).toHaveText("docs");
  expect(saved).toEqual([["/home/sam/code/docs"]]);
  await expect(page.getByRole("button", { name: "Unpin docs" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  // On the projects page the sidebar's projects section is already open.
  const first = rail(page)
    .locator("a.sub")
    .filter({ hasText: /docs|billing|storefront/ })
    .first();
  await expect(first).toHaveText("docs");
  await expect(first.getByLabel("Pinned")).toBeVisible();
  // Unpinning puts it back in its own place.
  await page.getByRole("button", { name: "Unpin docs" }).click();
  await expect(cards(page)).toHaveText("billing");
  expect(saved.at(-1)).toEqual([]);
});

test("a refused pin goes back, and a dashboard that takes no changes offers none", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mock(page, { editable: true, refuse: true });
  await page.goto("/#/projects");
  await page.getByRole("button", { name: "Pin docs" }).click();
  await expect(page.getByRole("button", { name: "Pin docs" })).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await expect(cards(page)).toHaveText("billing");
  await page.unrouteAll();
  await mock(page, { editable: false });
  await page.reload();
  await expect(cards(page)).toHaveText("billing");
  await expect(page.getByRole("button", { name: /^Pin / })).toHaveCount(0);
});
