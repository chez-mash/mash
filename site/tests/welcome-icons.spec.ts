import { expect, test } from "@playwright/test";

for (const [section, tiles, drawing, drawn] of [
  ["welcome-privacy", 4, 0.24, 0.45],
  ["welcome-features", 5, 0.4, 0.76],
] as const) {
  test(`the ${section} icons draw and undraw with scroll position`, async ({ page }) => {
    if (section === "welcome-privacy") await page.setViewportSize({ width: 320, height: 568 });
    await page.goto(`/#${section}`);
    await expect(page.locator(".mash-welcome")).toHaveClass(/\bmotion\b/);
    await page.evaluate(() => document.fonts.ready);
    const scene = page.locator(`#${section}`);
    const cards = scene.locator("[data-icon-tile]");
    const paths = cards.locator("svg.ico path");
    const first = paths.first();
    await expect(cards).toHaveCount(tiles);
    await expect(cards.locator("svg.ico")).toHaveCount(4);
    const jump = (share: number) =>
      scene.evaluate(async (element, part) => {
        const stage = element.querySelector<HTMLElement>(".stage");
        if (!stage) throw new Error("Missing welcome stage");
        const { top, height } = element.getBoundingClientRect();
        const pinnedAt = Number.parseFloat(getComputedStyle(stage).top);
        scrollTo({
          top: scrollY + top - pinnedAt + part * (height - stage.offsetHeight),
          behavior: "instant",
        });
        await new Promise<void>((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
        );
      }, share);
    const ink = () =>
      first.evaluate((path: SVGPathElement) => {
        const style = getComputedStyle(path);
        return {
          opacity: Number(style.opacity),
          dash: Number.parseFloat(style.strokeDasharray),
          length: path.getTotalLength(),
        };
      });

    await jump(0);
    await expect(first).toHaveCSS("opacity", "0");
    await jump(drawing);
    const partial = await ink();
    expect(partial.opacity).toBeGreaterThan(0);
    expect(partial.dash).toBeGreaterThan(0.5);
    expect(partial.dash).toBeLessThan(partial.length - 0.5);

    await jump(drawn);
    for (const card of await cards.all()) await expect(card).toHaveCSS("opacity", "1");
    const fullyDrawn = await paths.evaluateAll((strokes) =>
      strokes.every((path) => {
        if (!(path instanceof SVGPathElement)) return false;
        const { opacity, strokeDasharray } = getComputedStyle(path);
        return (
          opacity === "1" &&
          (strokeDasharray === "none" ||
            Number.parseFloat(strokeDasharray) >= path.getTotalLength() - 0.5)
        );
      }),
    );
    expect(fullyDrawn).toBe(true);
    await expect(scene.locator(".show")).toBeInViewport({ ratio: 0.99 });
    await jump(drawing);
    expect(await ink()).toEqual(partial);
  });
}
