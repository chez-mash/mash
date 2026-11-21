import { expect, test } from "@playwright/test";

for (const [name, side] of [
  ["search", "left"],
  ["mashy", "right"],
  ["memory", "left"],
  ["search", "below"],
] as const) {
  test(`the ${name} snapshot sits ${side} of its heading and captions`, async ({ page }) => {
    const stacked = side === "below";
    await page.setViewportSize(
      stacked ? { width: 320, height: 568 } : { width: 1280, height: 850 },
    );
    await page.goto(`/#welcome-${name}`);
    await expect(page.locator(".mash-welcome")).toHaveClass(/\bmotion\b/);
    await page.evaluate(async () => {
      await document.fonts.ready;
    });
    const scene = page.locator(`#welcome-${name}`);
    await scene.evaluate((element) => {
      const { top, height } = element.getBoundingClientRect();
      scrollTo(0, scrollY + top + 0.4 * (height - innerHeight));
    });

    const copy = scene.locator(".snapshot-copy");
    const picture = scene.locator(".scene-image");
    await expect(scene.locator(".show")).toHaveCSS("opacity", "1");
    await expect(copy).toBeInViewport({ ratio: 0.98 });
    await expect(picture).toBeInViewport({ ratio: 0.98 });
    const copyBox = await copy.boundingBox();
    const pictureBox = await picture.boundingBox();
    if (!copyBox || !pictureBox) throw new Error("Missing welcome scene bounds");
    if (stacked) {
      expect(copyBox.y + copyBox.height).toBeLessThan(pictureBox.y);
    } else {
      const [first, second] = side === "left" ? [pictureBox, copyBox] : [copyBox, pictureBox];
      expect(first.x + first.width).toBeLessThan(second.x);
      expect(first.y).toBeLessThan(second.y + second.height);
      expect(second.y).toBeLessThan(first.y + first.height);
    }
  });
}
