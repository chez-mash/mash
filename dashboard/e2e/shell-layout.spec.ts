import { expect, type Locator, test } from "@playwright/test";
import { mockShell, rail } from "./shell";

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
  for (const section of ["Chats", "Agents", "Machines", "Memory", "Projects"])
    await rail(page).getByRole("link", { name: section, exact: true }).click();
  const expanded = (section: string) =>
    rail(page).getByRole("button", { name: new RegExp(`^(Expand|Collapse) ${section}$`) });
  await expect(expanded("Projects")).toHaveAttribute("aria-expanded", "true");
  // Memory's kinds are tabs on its page, so it has no rows to open here.
  await expect(expanded("Memory")).toHaveCount(0);
  for (const section of ["Chats", "Agents", "Machines"])
    await expect(expanded(section)).toHaveAttribute("aria-expanded", "false");
  for (const bottom of ["Services", "Tool explorer", "Settings"])
    await expect(rail(page).getByRole("link", { name: bottom, exact: true })).toBeInViewport({
      ratio: 1,
    });

  // A section the reader opens stays open while they stay where they are.
  await expanded("Agents").click();
  await expect(expanded("Agents")).toHaveAttribute("aria-expanded", "true");
  // Leaving for a section without pages folds them all.
  await rail(page).getByRole("link", { name: "Services", exact: true }).click();
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
  await expect(rail(page).getByRole("link", { name: "Settings", exact: true })).toBeVisible();
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

// The floating controls' column must not crowd the transcript out: at a
// tablet's width the list beside it starts hidden and the actions fit on
// one row, and on a phone a reply runs as wide as a prompt.
test("a chat keeps a readable measure at 1024px", async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await mockShell(page);
  await page.goto("/#/chat/chat-0?node=machine-0");
  await expect(page.locator("#message-m11")).toBeAttached();
  const log = await box(page.locator(".log"));
  expect(log.width).toBeGreaterThanOrEqual(600);
  const tops = await page
    .locator(".chat-h .acts > *")
    .evaluateAll(
      (items) => new Set(items.map((item) => Math.round(item.getBoundingClientRect().top))).size,
    );
  expect(tops).toBe(1);
  // The list is a click away, and beside the transcript it leaves it room.
  await page.getByRole("button", { name: "Show session list" }).click();
  await expect(page.locator(".sess")).toBeVisible();
  expect((await box(page.locator(".log"))).width).toBeGreaterThanOrEqual(360);
});

test("on a phone a reply runs as wide as a prompt", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockShell(page);
  await page.goto("/#/chat/chat-0?node=machine-0");
  const rightEdge = (id: string) =>
    page.locator(`#message-${id}`).evaluate((element) => {
      const text = element.querySelector("p") ?? element;
      const range = document.createRange();
      range.selectNodeContents(text);
      return Math.max(...[...range.getClientRects()].map((line) => line.right));
    });
  const prompt = await box(page.locator("#message-m0"));
  const reply = await rightEdge("m1");
  // Text wraps short of the box by up to a word; a gutter would be 76px.
  expect(prompt.x + prompt.width - reply).toBeLessThan(60);
});
