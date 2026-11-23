import { expect, type Locator, type Page, test } from "@playwright/test";
import { rail } from "./shell";

// Enough chats that every section of the rail has several pages under it.
const agents = ["codex", "claude-code", "cursor", "opencode", "copilot"];
const chats = Array.from({ length: 12 }, (_, index) => ({
  chat_id: `chat-${index}`,
  node_id: `machine-${index % 4}`,
  provider: agents[index % agents.length],
  title: `Fixture chat ${index}`,
  project: `/work/project-${index % 6}`,
  updated_at: Date.now() - index * 60_000,
}));
const message = (index: number) => ({
  id: `m${index}`,
  ordinal: index,
  role: index % 2 ? "assistant" : "user",
  content: `Message number ${index}. ${"A line of ordinary text. ".repeat(6)}`,
});

async function mockShell(page: Page): Promise<void> {
  await page.route("**/api/**", (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/records") return route.fulfill({ json: { records: [], next_offset: null } });
    if (path === "/api/assistant") return route.fulfill({ json: { enabled: true, model: "m" } });
    if (path === "/api/chat")
      return route.fulfill({
        json: { message_count: 12, messages: Array.from({ length: 12 }, (_, i) => message(i)) },
      });
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "machine-0", truncated: false, chats } });
    return route.fulfill({ json: [] });
  });
}

async function box(locator: Locator) {
  const found = await locator.boundingBox();
  if (!found) throw new Error("not on screen");
  return found;
}
function overlaps(
  a: { x: number; y: number; width: number; height: number },
  b: { x: number; y: number; width: number; height: number },
): boolean {
  return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

test("the rail keeps only the current section open, so its last rows stay in view", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/");
  for (const section of ["chats", "agents", "machines", "projects", "memory"])
    await rail(page).getByRole("link", { name: section, exact: true }).click();
  const expanded = (section: string) =>
    rail(page).getByRole("button", { name: new RegExp(`^(Expand|Collapse) ${section}$`) });
  await expect(expanded("memory")).toHaveAttribute("aria-expanded", "true");
  for (const section of ["chats", "agents", "machines", "projects"])
    await expect(expanded(section)).toHaveAttribute("aria-expanded", "false");
  for (const bottom of ["services", "tool explorer", "settings"])
    await expect(rail(page).getByRole("link", { name: bottom, exact: true })).toBeInViewport({
      ratio: 1,
    });

  // A section the reader opens stays open while they stay where they are.
  await expanded("agents").click();
  await expect(expanded("agents")).toHaveAttribute("aria-expanded", "true");
  // Leaving for a section without pages folds them all.
  await rail(page).getByRole("link", { name: "services", exact: true }).click();
  await expect(rail(page).locator(".subs")).toHaveCount(0);
});

test("a rail taller than the screen fades at the edge with more rows", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 420 });
  await mockShell(page);
  await page.goto("/#/agents");
  const nav = rail(page);
  await expect(nav).toHaveClass(/fade-below/);
  await expect(nav).not.toHaveClass(/fade-above/);
  await nav.evaluate((element) => element.scrollTo(0, element.scrollHeight));
  await expect(nav).toHaveClass(/fade-above/);
  await expect(nav).not.toHaveClass(/fade-below/);

  // A rail that fits does not fade.
  await page.setViewportSize({ width: 1280, height: 1000 });
  await page.goto("/#/settings");
  await expect(nav).not.toHaveClass(/fade-/);
});

test("on a phone the navigation button shows a menu icon beside the mark", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockShell(page);
  await page.goto("/");
  const burger = page.getByRole("button", { name: "Open navigation" });
  await expect(burger.locator(".menu-ico svg")).toBeVisible();
  await expect(burger.locator("img")).toBeVisible();
  await burger.click();
  await expect(rail(page).getByRole("link", { name: "settings", exact: true })).toBeVisible();
});

for (const [width, height] of [
  [1440, 900],
  [390, 844],
] as const) {
  test(`floating controls leave a chat's last message clear at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await mockShell(page);
    await page.goto("/#/chat/chat-0?node=machine-0");
    const last = page.locator("#message-m11");
    await expect(last).toBeInViewport();
    await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
    const floating = [
      await box(page.locator("#mashy-fab")),
      ...(await Promise.all(
        (
          await page.getByRole("navigation", { name: "Chat ends" }).getByRole("button").all()
        ).map(box),
      )),
    ];
    expect(floating.length).toBe(3);
    const actions = await last.locator(".cp").all();
    expect(actions.length).toBeGreaterThan(0);
    for (const target of [last, ...actions]) {
      const area = await box(target);
      for (const control of floating) expect(overlaps(area, control)).toBe(false);
    }
  });
}

test("on a wide screen page content keeps clear of Mashy's button at any scroll", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/search");
  await expect(page.getByText("Fixture chat 0").first()).toBeVisible();
  const fab = await box(page.locator("#mashy-fab"));
  const main = await page.locator("#main").evaluate((element) => {
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    return rect.right - Number.parseFloat(style.paddingRight);
  });
  expect(main).toBeLessThanOrEqual(fab.x);
});
