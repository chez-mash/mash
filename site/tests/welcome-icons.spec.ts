import { expect, test } from "@playwright/test";

for (const [section, tiles] of [
  ["welcome-privacy", 4],
  ["welcome-features", 5],
] as const) {
  test(`the ${section} cards draw their icons as they enter`, async ({ page }) => {
    if (section === "welcome-privacy") await page.setViewportSize({ width: 320, height: 568 });
    await page.goto(`/#${section}`);
    await expect(page.locator(".mash-welcome")).toHaveClass(/\bmotion\b/);
    await page.evaluate(() => document.fonts.ready);
    const scene = page.locator(`#${section}`);
    const cards = scene.locator("[data-icon-tile]");
    const first = cards.locator("svg.ico path").first();
    await expect(cards).toHaveCount(tiles);
    await expect(cards.locator("svg.ico")).toHaveCount(4);
    await scene.evaluate((element) => {
      const top = element.getBoundingClientRect().top + scrollY;
      scrollTo(0, top - 0.4 * innerHeight);
    });
    await expect(first).toHaveCSS("opacity", "0");
    await scene.evaluate((element) => {
      const { top, height } = element.getBoundingClientRect();
      scrollTo(0, scrollY + top + 0.35 * (height - innerHeight));
    });
    await page.waitForFunction(
      (id) => {
        const path = document.querySelector<SVGPathElement>(`#${id} [data-icon-tile] svg.ico path`);
        if (!path) return false;
        const style = getComputedStyle(path);
        const painted = Number.parseFloat(style.strokeDasharray);
        return Number(style.opacity) > 0 && painted > 0.5 && painted < path.getTotalLength() - 0.5;
      },
      section,
      { timeout: 20_000 },
    );
    for (const card of await cards.all()) await expect(card).toHaveCSS("opacity", "1");
    await expect(scene.locator(".show")).toBeInViewport({ ratio: 0.99 });
  });
}
