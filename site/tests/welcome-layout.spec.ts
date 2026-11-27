import { expect, test } from "@playwright/test";

for (const [name, side] of [
  ["search", "left"],
  ["mashy", "right"],
  ["search", "below"],
] as const) {
  const stacked = side === "below";
  const placement = stacked ? "below" : `${side} of`;
  test(`the ${name} window sits ${placement} its heading and points`, async ({ page }) => {
    await page.setViewportSize(
      stacked ? { width: 320, height: 568 } : { width: 1280, height: 850 },
    );
    await page.goto(`/#welcome-${name}`);
    await page.evaluate(() => document.fonts.ready);
    const band = page.locator(`#welcome-${name}`);
    const words = band.locator(".words");
    const picture = band.locator("[role=img]");
    await band.locator("[data-reveal]").scrollIntoViewIfNeeded();
    await expect(band.locator("[data-reveal]")).toHaveClass(/\bshown\b/);
    const wordsBox = await words.boundingBox();
    const pictureBox = await picture.boundingBox();
    if (!wordsBox || !pictureBox) throw new Error("Missing welcome band bounds");
    if (stacked) {
      expect(wordsBox.y + wordsBox.height).toBeLessThan(pictureBox.y);
      expect(pictureBox.x + pictureBox.width).toBeLessThanOrEqual(320);
    } else {
      const [first, second] = side === "left" ? [pictureBox, wordsBox] : [wordsBox, pictureBox];
      expect(first.x + first.width).toBeLessThan(second.x);
      expect(first.y).toBeLessThan(second.y + second.height);
      expect(second.y).toBeLessThan(first.y + first.height);
    }
  });
}

test("the privacy band is words alone: its paragraph takes the words' column, or follows the heading", async ({
  page,
}) => {
  const band = page.locator("#welcome-privacy");
  const boxes = async () => {
    await page.goto("/#welcome-privacy");
    await page.evaluate(() => document.fonts.ready);
    await band.locator("[data-reveal]").scrollIntoViewIfNeeded();
    await expect(band.locator("[data-reveal]")).toHaveClass(/\bshown\b/);
    const [heading, paragraph, onward] = await Promise.all(
      ["h2", ".blurb", "a.more"].map((part) => band.locator(part).boundingBox()),
    );
    if (!heading || !paragraph || !onward) throw new Error("Missing privacy band bounds");
    return { heading, paragraph, onward };
  };

  // Wide: the heading where the other bands' windows are, the paragraph under their words.
  await page.setViewportSize({ width: 1280, height: 850 });
  const wide = await boxes();
  const words = await page.locator("#welcome-search .words").boundingBox();
  expect(wide.heading.x + wide.heading.width).toBeLessThan(wide.paragraph.x);
  expect(Math.abs(wide.paragraph.x - (words?.x ?? 0))).toBeLessThan(1);
  expect(wide.paragraph.width).toBeLessThanOrEqual(480);
  expect(wide.paragraph.y + wide.paragraph.height).toBeLessThan(wide.onward.y);

  // Narrow: the heading, the paragraph, then the way onward, as in the bands above.
  await page.setViewportSize({ width: 390, height: 844 });
  const narrow = await boxes();
  expect(narrow.heading.y + narrow.heading.height).toBeLessThan(narrow.paragraph.y);
  expect(narrow.paragraph.y - (narrow.heading.y + narrow.heading.height)).toBeLessThan(24);
  expect(narrow.paragraph.y + narrow.paragraph.height).toBeLessThan(narrow.onward.y);
  expect(narrow.paragraph.x + narrow.paragraph.width).toBeLessThanOrEqual(390);
});
