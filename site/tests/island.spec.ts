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
  await scrollToPart(page, "#welcome-mashy");
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
    await page.goto("/#welcome-mashy-title");
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    const island = await place(page.locator("header.header .mash-header"));
    await expect
      .poll(() =>
        page.locator("#welcome-mashy-title").evaluate((node) => node.getBoundingClientRect().top),
      )
      .toBeGreaterThanOrEqual(island.bottom);
    await expect(page.locator("#welcome-mashy-title")).toBeInViewport();
  });
}

/** The widest spread of any colour channel in a part of the screen: 0 is one flat colour. */
async function spread(
  page: Page,
  clip: { x: number; y: number; width: number; height: number },
): Promise<number> {
  const picture = (await page.screenshot({ clip })).toString("base64");
  return page.evaluate(async (data) => {
    const image = new Image();
    image.src = `data:image/png;base64,${data}`;
    await image.decode();
    const canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("No canvas to read the picture with");
    context.drawImage(image, 0, 0);
    const pixels = context.getImageData(0, 0, image.width, image.height).data;
    let widest = 0;
    for (let channel = 0; channel < 3; channel += 1) {
      let [low, high] = [255, 0];
      for (let at = channel; at < pixels.length; at += 4) {
        low = Math.min(low, pixels[at] ?? 0);
        high = Math.max(high, pixels[at] ?? 0);
      }
      widest = Math.max(widest, high - low);
    }
    return widest;
  }, picture);
}

test("nothing shows around the island's corners or through it as the page scrolls under", async ({
  page,
}) => {
  await page.setViewportSize(phone);
  await page.goto("/");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  const island = page.locator("header.header .mash-header");
  // The surface is all but opaque, so words under it do not read through it.
  const alpha = await island.evaluate((node) => {
    const colour = getComputedStyle(node).backgroundColor;
    const parts = colour.match(/[\d.]+/g)?.map(Number) ?? [];
    // `rgba(r, g, b, a)` or `color(srgb r g b / a)`; no fourth part is opaque.
    return parts.length > 3 ? (parts[3] ?? 1) : 1;
  });
  expect(alpha).toBeGreaterThanOrEqual(0.9);
  await scrollToPart(page, "#welcome-surfaces");
  await expect(island).toHaveAttribute("data-past-hero", "");
  const box = await place(island);
  const from = await page.evaluate(() => scrollY);
  // The bands' windows and cards start at the island's own gutter: scroll
  // them past it a little at a time and read the strip above the island and
  // the square outside its rounded top-left corner. (Headless WebKit shades
  // the screen's own first pixel on every page, so both start beside it.)
  for (let top = from; top < from + 3600; top += 150) {
    await page.evaluate((y) => scrollTo({ top: y, behavior: "instant" }), top);
    await expect
      .poll(() => spread(page, { x: 2, y: 0, width: phone.width - 2, height: box.top - 1 }), {
        message: `the strip above the island at ${top}`,
      })
      .toBeLessThanOrEqual(4);
    expect(
      await spread(page, { x: 2, y: 0, width: box.left + 1, height: box.top + 3 }),
      `the island's top-left corner at ${top}`,
    ).toBeLessThanOrEqual(4);
  }
});
