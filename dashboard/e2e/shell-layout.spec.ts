import { expect, type Locator, test } from "@playwright/test";
import { recordFixture } from "./record-fixtures";
import { fitsViewport, mockShell, rail } from "./shell";

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

test("the rail opens Chats' own pages only while in Chats, and its last rows stay in view", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/");
  const chats = rail(page).getByRole("button", { name: /^(Expand|Collapse) Chats$/ });
  await expect(chats).toHaveAttribute("aria-expanded", "false");
  await rail(page).getByRole("link", { name: "Chats", exact: true }).click();
  await expect(chats).toHaveAttribute("aria-expanded", "true");
  await expect(rail(page).getByRole("link", { name: "Viewing history" })).toBeVisible();
  // Only Chats has pages of its own; the other sections are single rows.
  await expect(rail(page).getByRole("button", { name: /^(Expand|Collapse) / })).toHaveCount(1);
  for (const bottom of ["Services", "Tools", "Settings", "Export"])
    await expect(rail(page).getByRole("link", { name: bottom, exact: true })).toBeInViewport({
      ratio: 1,
    });
  // Leaving Chats folds its pages.
  await rail(page).getByRole("link", { name: "Services", exact: true }).click();
  await expect(rail(page).locator(".subs")).toHaveCount(0);
});

test("the top bar runs across the screen above the rail", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await mockShell(page);
  await page.goto("/");
  const bar = await box(page.getByRole("banner"));
  const side = await box(page.locator("#rail"));
  expect(bar.x).toBe(0);
  expect(bar.width).toBe(1280);
  expect(side.y).toBeGreaterThanOrEqual(bar.y + bar.height - 1);
  // The bar holds the mark, the search field and the viewing switcher.
  await expect(page.getByRole("banner").getByRole("link", { name: "Mash overview" })).toBeVisible();
  await expect(page.getByRole("banner").locator(".vw-b")).toBeVisible();
  await expect(page.getByPlaceholder("Search every chat")).toBeVisible();
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

test("on a phone the bar keeps the mark, a search button and the menu button", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockShell(page);
  await page.goto("/");
  const bar = page.getByRole("banner");
  await expect(bar.getByRole("link", { name: "Mash overview" })).toBeVisible();
  // The search field waits behind its button, and the switcher in the sheet.
  await expect(page.locator("#q")).toBeHidden();
  await expect(bar.locator(".vw-b")).toBeHidden();
  await bar.getByRole("button", { name: "Show search" }).click();
  await expect(page.locator("#q")).toBeFocused();
  const burger = bar.getByRole("button", { name: "Open navigation" });
  await expect(burger.locator(".menu-ico svg")).toBeVisible();
  await burger.click();
  await expect(rail(page).getByRole("link", { name: "Settings", exact: true })).toBeVisible();
  await expect(page.locator("#rail .vw-b")).toBeVisible();
  expect(await fitsViewport(page)).toBe(true);
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
    // A phone shows no jumps to either end, only Mashy's circle.
    expect(floating.length).toBe(width > 800 ? 3 : 1);
    const actions = await last.locator(".cp").all();
    expect(actions.length).toBeGreaterThan(0);
    for (const target of [last, ...actions]) {
      const area = await box(target);
      for (const control of floating) expect(overlaps(area, control)).toBe(false);
    }
  });
}

test("on a wide screen pages fill the width and Mashy's button floats in the corner", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  const fab = await box(page.locator("#mashy-fab"));
  const bar = await box(page.locator("#top"));
  // Out of the top bar, in the bottom right corner, like a help widget.
  expect(fab.y).toBeGreaterThan(bar.y + bar.height);
  expect(fab.width).toBe(56);
  expect(1440 - (fab.x + fab.width)).toBe(24);
  expect(900 - (fab.y + fab.height)).toBe(24);
  // The page runs to the screen's edge but for its padding, as the boards
  // draw it, with no column set aside for the button; it ends with room to
  // scroll its last rows clear of the button instead.
  const main = await page.locator("#main").evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      right: element.getBoundingClientRect().right - Number.parseFloat(style.paddingRight),
      foot: Number.parseFloat(style.paddingBottom),
    };
  });
  expect(main.right).toBeGreaterThanOrEqual(1440 - 40);
  expect(main.foot).toBeGreaterThanOrEqual(900 - fab.y);
});

// A pane at the right edge scrolls by itself, under the button; it ends with
// room for the button too.
test("a pane at the right edge scrolls its last rows clear of Mashy's button", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [recordFixture()], next_offset: null } }),
  );
  await page.goto("/#/records");
  const pane = page.locator(".mem-p");
  await expect(pane).toBeVisible();
  const fab = await box(page.locator("#mashy-fab"));
  expect(overlaps(await box(pane), fab)).toBe(true);
  const foot = await pane.evaluate((element) =>
    Number.parseFloat(getComputedStyle(element).paddingBottom),
  );
  expect(foot).toBeGreaterThanOrEqual(900 - fab.y);
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
