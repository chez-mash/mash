import { expect, type Locator, type Page, test } from "@playwright/test";

const phone = { width: 390, height: 844 };
const wide = { width: 1440, height: 900 };

/** Where the landing page's header island lies on the screen. */
async function place(
  island: Locator,
): Promise<{ top: number; bottom: number; left: number; right: number }> {
  return island.evaluate((node) => {
    const box = node.getBoundingClientRect();
    return { top: box.top, bottom: box.bottom, left: box.left, right: box.right };
  });
}

/** Scrolls until `selector` is at the top of the screen, as a reader would reach it. */
async function scrollToPart(page: Page, selector: string): Promise<void> {
  await page.locator(selector).evaluate((node) => {
    scrollTo({ top: node.getBoundingClientRect().top + scrollY, behavior: "instant" });
  });
}

test("on a phone the island stays at the top, and offers the download once the hero's buttons are gone", async ({
  page,
}) => {
  await page.setViewportSize(phone);
  await page.goto("/");
  const island = page.locator("header.header .mash-header");
  const download = island.locator("a.download");
  const before = await place(island);
  // An inset, rounded bar: clear of the screen's edges, with the logo alone.
  expect(before.top).toBeGreaterThanOrEqual(8);
  expect(before.left).toBeGreaterThanOrEqual(12);
  expect(before.right).toBeLessThanOrEqual(phone.width - 12);
  await expect(island).toHaveCSS("backdrop-filter", /blur/);
  await expect(island.locator("a.brand")).toBeVisible();
  await expect(download).toBeHidden();
  await expect(island.getByRole("link")).toHaveCount(1);

  await scrollToPart(page, "#welcome-surfaces");
  await expect(download).toBeVisible();
  await expect(download).toHaveCSS("opacity", "1");
  await expect(download).toHaveAccessibleName("Download");
  // It leads where the hero's own download button does.
  const target = await page.locator(".hero .actions a.download").getAttribute("href");
  await expect(download).toHaveAttribute("href", target ?? "");
  // The island has not moved, and the page gained no sideways scroll.
  expect(await place(island)).toEqual(before);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(phone.width);
  // The button answers a tap across at least 44 pixels of height.
  const box = await download.boundingBox();
  if (!box) throw new Error("The download button has no box");
  for (const y of [box.y + box.height / 2 - 21, box.y + box.height / 2 + 21]) {
    const hit = await page.evaluate(
      ([x, at]) => document.elementFromPoint(x ?? 0, at ?? 0)?.closest("a")?.className ?? "",
      [box.x + box.width / 2, y],
    );
    expect(hit).toContain("download");
  }

  // Back at the top the button goes again.
  await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
  await expect(download).toBeHidden();
});

test("on a wide screen the island keeps its links while the page scrolls under it", async ({
  page,
}) => {
  await page.setViewportSize(wide);
  await page.goto("/");
  const island = page.locator("header.header .mash-header");
  const before = await place(island);
  await scrollToPart(page, "#welcome-search");
  expect(await place(island)).toEqual(before);
  await expect(island.getByRole("link", { name: "Docs", exact: true })).toBeVisible();
  await expect(island.getByRole("link", { name: "GitHub", exact: true })).toBeVisible();
  await expect(
    island.getByRole("link", { name: /^Download for (Mac|Windows|Linux)$/ }),
  ).toBeVisible();
  // The island is the topmost thing where it lies.
  const top = await page.evaluate(
    ([x, y]) => Boolean(document.elementFromPoint(x ?? 0, y ?? 0)?.closest(".mash-header")),
    [(before.left + before.right) / 2, (before.top + before.bottom) / 2],
  );
  expect(top).toBe(true);
});

test("the pinned search panel holds below the island", async ({ page }) => {
  await page.setViewportSize(wide);
  await page.goto("/");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  const desk = page.locator("#welcome-shortcuts .desk");
  await expect(page.locator("#welcome-shortcuts .pin-spacer")).toHaveCount(1);
  const start = await page
    .locator("#welcome-shortcuts .pin-spacer")
    .evaluate((node) => node.getBoundingClientRect().top + scrollY);
  await page.evaluate((top) => scrollTo({ top, behavior: "instant" }), start);
  const island = await place(page.locator("header.header .mash-header"));
  await expect
    .poll(() => desk.evaluate((node) => node.getBoundingClientRect().top))
    .toBeGreaterThan(island.bottom + 16);
});

for (const [name, viewport] of [
  ["a phone", phone],
  ["a wide screen", wide],
] as const) {
  test(`on ${name} a link into the page lands its heading below the island`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await page.goto("/#welcome-memory-title");
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    const island = await place(page.locator("header.header .mash-header"));
    await expect
      .poll(() =>
        page.locator("#welcome-memory-title").evaluate((node) => node.getBoundingClientRect().top),
      )
      .toBeGreaterThanOrEqual(island.bottom);
    await expect(page.locator("#welcome-memory-title")).toBeInViewport();
    // The install commands under the hero, reached from the hero's own link.
    await page.goto("/");
    await page.locator('.hero a[href="#install"]').click();
    await expect
      .poll(() => page.locator("#install").evaluate((node) => node.getBoundingClientRect().top))
      .toBeGreaterThanOrEqual(island.bottom);
  });
}
