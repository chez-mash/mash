import { expect, type Locator, type Page, test } from "@playwright/test";

const labels = ["Dashboard", "Menu bar", "Raycast", "Agent plugin", "Terminal"];
const claims = [
  "Read it all back.",
  "Always within reach.",
  "Stay in your launcher.",
  "Context, before you ask.",
  "Script your history.",
];
const phone = { width: 390, height: 844 };

/** Opens the landing page with the showcase running and brought on screen. */
async function open(page: Page): Promise<Locator> {
  await page.goto("/");
  const band = page.locator("#welcome-surfaces");
  await expect(band.locator("[data-surfaces]")).toHaveAttribute("data-surfaces", "live");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  await band.locator(".stage").scrollIntoViewIfNeeded();
  return band;
}

/** The surface, tab and caption that show now. */
async function shown(
  band: Locator,
): Promise<{ tab: string | null; panels: string[]; claim: string }> {
  const panels = await band
    .locator(".surface")
    .evaluateAll((all) =>
      all
        .filter((panel) => getComputedStyle(panel).visibility === "visible")
        .map((panel) => panel.id),
    );
  const tab = await band.getByRole("tab", { selected: true }).getAttribute("aria-label");
  const claim = await band
    .locator(".caption b")
    .evaluateAll((all) =>
      all
        .filter((part) => getComputedStyle(part).visibility === "visible")
        .map((part) => part.textContent),
    );
  return { tab, panels, claim: claim.join("|") };
}

/** Waits until a change has finished: one surface shows and nothing holds a scene's style. */
async function settled(band: Locator, index: number): Promise<void> {
  await expect
    .poll(async () => {
      const now = await shown(band);
      const moving = await band
        .locator(".leaving, [style*='opacity'], [style*='transform']")
        .count();
      return { ...now, moving };
    })
    .toEqual({
      tab: labels[index],
      panels: [await band.locator(".surface").nth(index).getAttribute("id")],
      claim: claims[index],
      moving: 0,
    });
}

test("the showcase is a tablist: one tab per surface, each with its panel and caption", async ({
  page,
}) => {
  const band = await open(page);
  await expect(band.getByRole("heading", { level: 2 })).toHaveText(
    "One history. Everywhere you work.",
  );
  const list = band.getByRole("tablist", { name: "Where Mash works" });
  const tabs = list.getByRole("tab");
  await expect(tabs).toHaveCount(labels.length);
  for (const [index, label] of labels.entries()) {
    const tab = tabs.nth(index);
    await expect(tab).toHaveAccessibleName(label);
    await expect(tab).toHaveAttribute("aria-selected", String(index === 0));
    // Only the chosen tab is in the tab order; the arrow keys reach the rest.
    await expect(tab).toHaveAttribute("tabindex", index === 0 ? "0" : "-1");
    const panel = page.locator(`#${await tab.getAttribute("aria-controls")}`);
    await expect(panel).toHaveAttribute("role", "tabpanel");
    await expect(panel).toHaveAttribute("aria-labelledby", (await tab.getAttribute("id")) ?? "");
    await expect(panel.locator(".picture[role=img]")).toHaveAttribute("aria-label", /\w{4}/);
    const caption = page.locator(`#${await panel.getAttribute("aria-describedby")}`);
    await expect(caption.locator("b")).toHaveText(claims[index] ?? "");
    await expect(caption.locator("a")).toHaveAttribute(
      "href",
      (await tab.getAttribute("href")) ?? "",
    );
  }
  await expect(band.getByRole("tabpanel")).toHaveCount(1);
  await expect(band.locator(".dock-tag")).toHaveText(labels[0] ?? "");
});

test("a click chooses a surface, and the tag and caption follow it", async ({ page }) => {
  const band = await open(page);
  const tabs = band.getByRole("tab");
  await tabs.nth(2).click();
  await expect(page).toHaveURL(/\/$/);
  await settled(band, 2);
  await expect(band.locator(".dock-tag")).toHaveText("Raycast");
  // The tag sits over the chosen tile.
  const centre = async (part: Locator) => {
    const box = await part.boundingBox();
    return box ? box.x + box.width / 2 : Number.NaN;
  };
  expect(
    Math.abs((await centre(band.locator(".dock-tag span"))) - (await centre(tabs.nth(2)))),
  ).toBeLessThan(2);
  // The chosen tile is larger than the rest and raised above them.
  const [chosen, other] = [await tabs.nth(2).boundingBox(), await tabs.nth(3).boundingBox()];
  expect(chosen?.width ?? 0).toBeGreaterThan((other?.width ?? 0) + 4);
  expect(chosen?.y ?? 0).toBeLessThan((other?.y ?? 0) - 4);
  await tabs.nth(0).click();
  await settled(band, 0);
});

test("the arrow keys, Home and End move between surfaces and wrap", async ({ page }) => {
  const band = await open(page);
  const tabs = band.getByRole("tab");
  await tabs.nth(0).focus();
  await page.keyboard.press("ArrowRight");
  await settled(band, 1);
  await expect(tabs.nth(1)).toBeFocused();
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await settled(band, 4);
  await expect(tabs.nth(4)).toBeFocused();
  await page.keyboard.press("Home");
  await settled(band, 0);
  await page.keyboard.press("End");
  await settled(band, 4);
  await page.keyboard.press("ArrowRight");
  await settled(band, 0);
  // The focus ring is drawn on the tab the keys reached.
  await expect(tabs.nth(0)).toHaveCSS("outline-style", "solid");
});

test("a sideways swipe on the picture changes the surface, and a scroll does not", async ({
  browser,
}) => {
  const context = await browser.newContext({ viewport: phone, hasTouch: true, isMobile: true });
  const page = await context.newPage();
  const band = await open(page);
  const swipe = (dx: number, dy: number) =>
    band.locator(".stage").evaluate(
      (stage, [x, y]) => {
        const box = stage.getBoundingClientRect();
        const at = { x: box.left + box.width / 2, y: box.top + box.height / 2 };
        const touch = (type: string, clientX: number, clientY: number) =>
          stage.dispatchEvent(
            new PointerEvent(type, {
              pointerId: 7,
              pointerType: "touch",
              clientX,
              clientY,
              bubbles: true,
            }),
          );
        touch("pointerdown", at.x, at.y);
        touch("pointerup", at.x + (x ?? 0), at.y + (y ?? 0));
      },
      [dx, dy],
    );
  await swipe(-120, 10);
  await settled(band, 1);
  await swipe(20, 160);
  await swipe(-30, 0);
  await settled(band, 1);
  await swipe(120, -10);
  await settled(band, 0);
  await swipe(120, 0);
  await settled(band, 4);
  await context.close();
});

test("the surfaces change by themselves until the reader chooses one", async ({ page }) => {
  await page.clock.install();
  const band = await open(page);
  // Keep the pointer off the band: pointing at it holds the surface.
  await page.mouse.move(0, 0);
  await settled(band, 0);
  await page.clock.runFor(5200);
  await page.clock.runFor(1500);
  await settled(band, 1);
  await page.clock.runFor(5200);
  await page.clock.runFor(1500);
  await settled(band, 2);
  // Off screen they wait.
  await page.locator("section.close").scrollIntoViewIfNeeded();
  await page.clock.runFor(12_000);
  await band.locator(".stage").scrollIntoViewIfNeeded();
  await settled(band, 2);
  // A choice ends it.
  await band.getByRole("tab").nth(4).click();
  await page.mouse.move(0, 0);
  await page.clock.runFor(1500);
  await settled(band, 4);
  await page.clock.runFor(12_000);
  await settled(band, 4);
});

for (const [name, viewport] of [
  ["a phone", phone],
  ["a wide screen", { width: 1440, height: 900 }],
] as const) {
  test(`on ${name} the showcase fits the screen and keeps its size while it changes`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    const band = await open(page);
    const size = () =>
      band.evaluate((node) => {
        const box = node.getBoundingClientRect();
        return {
          band: Math.round(box.height),
          page: document.documentElement.scrollHeight,
          across: document.documentElement.scrollWidth,
        };
      });
    const before = await size();
    expect(before.across).toBe(viewport.width);
    const tabs = band.getByRole("tab");
    for (const [index] of labels.entries()) {
      await tabs.nth(index).click();
      // Mid-change and at rest, the band is the size it was.
      expect(await size()).toEqual(before);
      await settled(band, index);
      expect(await size()).toEqual(before);
      // The picture, the tag and the dock all lie inside the screen.
      for (const part of [".surface.on .picture > *", ".dock-tag span", ".dock"]) {
        const box = await band.locator(part).first().boundingBox();
        expect(box?.x ?? -1, part).toBeGreaterThanOrEqual(0);
        expect((box?.x ?? 0) + (box?.width ?? 0), part).toBeLessThanOrEqual(viewport.width);
      }
      // Every dock button is large enough to tap.
      const tab = await tabs.nth(index).boundingBox();
      expect(Math.min(tab?.width ?? 0, tab?.height ?? 0)).toBeGreaterThanOrEqual(44);
    }
  });
}

test("without scripting the first surface shows and the dock's buttons are links", async ({
  browser,
}) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: phone });
  const page = await context.newPage();
  await page.goto("/");
  const band = page.locator("#welcome-surfaces");
  await expect(band.locator(".surface.on")).toHaveAttribute("id", "surface-dashboard");
  await expect(band.locator(".surface.on")).toBeVisible();
  await expect(band.locator("#surface-terminal")).toBeHidden();
  await expect(band.locator(".caption.on b")).toHaveText("Read it all back.");
  await band.locator("#surface-tab-terminal").click();
  await expect(page).toHaveURL(/\/cli\/$/);
  await context.close();
});
