// The landing page's visual baselines. The page moves: the hero makes an
// entrance, each band plays a scene as it scrolls into view, the showcase
// changes surface by itself, and on a wide screen the quick search panel pins
// while the scroll types into it. A picture
// of a page in motion never compares, so this test owns the page's clock. It
// is installed stopped before the page loads, and every scene is then played
// to its end by moving the clock on, which takes no real time and ends in the
// same frame on every run.
import { expect, type Page, test } from "@playwright/test";
import { arrived, settle } from "./settle.ts";

const now = new Date("2026-11-20T19:30:00Z");

/**
 * Moves the clock on four seconds, longer than any scene on the page, in ten
 * frames. The motion library treats a frame more than half a second late as a
 * stall and slows down for it, so each step stays under that.
 */
async function play(page: Page): Promise<void> {
  for (let frame = 0; frame < 10; frame += 1) await page.clock.fastForward(400);
}

/** The bands under the hero, in page order: a picture's name and what it is of. */
const bands = [
  ["agents", ".mash-agents"],
  ["surfaces", "#welcome-surfaces"],
  ["quick-search", "#welcome-shortcuts"],
  ["search", "#welcome-search"],
  ["mashy", "#welcome-mashy"],
  ["memory", "#welcome-memory"],
  ["speed", "#welcome-speed"],
  ["privacy", "#welcome-privacy"],
  ["features", "#welcome-features"],
  ["rust", "#welcome-rust"],
  ["closing", "section.close"],
  ["footer", "footer.mash-footer"],
] as const;

/**
 * Scrolls the page to `top` and plays whatever that sets off to its end. The
 * scenes hear of a scroll through the document's scroll event and act on
 * their next frame. The browser sends that event when it next draws, which
 * could fall between any two of the frames played here, so it is sent by hand
 * first: the scenes then start on the same frame in every run.
 */
async function scrollAndPlay(page: Page, top: number): Promise<void> {
  await page.evaluate((y) => {
    scrollTo({ top: y, behavior: "instant" });
    document.dispatchEvent(new Event("scroll", { bubbles: true }));
  }, top);
  await play(page);
}

const pageTop = (page: Page, selector: string): Promise<number> =>
  page.locator(selector).evaluate((band) => band.getBoundingClientRect().top + scrollY);

/**
 * On a wide screen the quick search panel is held in place while the scroll
 * plays its scene, and its band is as tall as that scroll. Two pictures of the
 * screen stand in for the band: one as the hold begins, before the first key,
 * and one near its end, with the search typed and a row chosen.
 */
async function quickSearchHeld(page: Page): Promise<void> {
  const screen = page.viewportSize()?.height ?? 0;
  // The hold begins when the panel's desk is 12% down the screen and lasts 150% of it.
  const start = (await pageTop(page, "#welcome-shortcuts .pin-spacer")) - 0.12 * screen;
  await scrollAndPlay(page, start);
  await expect.soft(page).toHaveScreenshot("landing-quick-search.png");
  await scrollAndPlay(page, start + 1.4 * screen);
  await expect(page.locator("#welcome-shortcuts .quick-q b")).toHaveText("refresh token");
  await expect.soft(page).toHaveScreenshot("landing-quick-search-held.png");
}

/** Where a band lies on the page, and the screens it takes to show all of it. */
async function screensOf(page: Page, selector: string): Promise<{ top: number; tops: number[] }> {
  const screen = page.viewportSize()?.height ?? 0;
  const top = Math.round(await pageTop(page, selector));
  const height = await page
    .locator(selector)
    .evaluate((band) => band.getBoundingClientRect().height);
  const count = Math.max(1, Math.ceil(height / screen));
  return { top, tops: Array.from({ length: count }, (_, index) => top + index * screen) };
}

/**
 * Plays a band's scenes to their end, a screen at a time. A scene hands its
 * parts back to the stylesheet when it finishes, so a band at rest has no part
 * still holding the opacity or the place a scene gave it. Now and then a scene
 * misses the scroll that should start it, because the browser's own events and
 * the stopped clock are not in step; the band is then played again rather
 * than pictured half way.
 */
async function playBand(page: Page, selector: string): Promise<void> {
  await expect(async () => {
    for (const top of (await screensOf(page, selector)).tops) await scrollAndPlay(page, top);
    const moving = await page.locator(selector).evaluate((band) => {
      const revealed = [...band.querySelectorAll("[data-reveal]")].every((part) =>
        part.classList.contains("shown"),
      );
      // Two parts keep a place of their own: the selection in the quick search
      // panel stays on the chosen row, and Mashy bobs for as long as he is seen.
      const held = [...band.querySelectorAll<HTMLElement>("*")].filter(
        (part) =>
          (part.style.opacity !== "" && part.style.opacity !== "1") ||
          (part.style.transform !== "" && !part.matches(".quick-sel, .mashy-face")),
      );
      return revealed ? held.length : -1;
    });
    expect(moving).toBe(0);
  }).toPass({ timeout: 15_000 });
}

/**
 * Takes a band's pictures: the part of it on screen, one screen after another
 * (`NAME.png`, then `NAME-2.png` and so on for a band taller than the screen).
 * A picture never reaches past the screen. Playwright can take one of a whole
 * tall element, but it resizes and scrolls the page to do so, and the page's
 * scenes answer both by starting over.
 */
async function pictureBand(page: Page, name: string, selector: string): Promise<void> {
  const { tops } = await screensOf(page, selector);
  for (const [index, top] of tops.entries()) {
    await scrollAndPlay(page, top);
    await arrived(page, selector);
    const clip = await page.locator(selector).evaluate((band) => {
      const box = band.getBoundingClientRect();
      const y = Math.max(0, box.top);
      return { x: 0, y, width: innerWidth, height: Math.min(innerHeight, box.bottom) - y };
    });
    const file = index === 0 ? `landing-${name}.png` : `landing-${name}-${index + 1}.png`;
    await expect.soft(page).toHaveScreenshot(file, {
      clip,
      stylePath: ["./visual/still.css", "./visual/band.css"],
    });
  }
}

/** Chooses a surface in the showcase, as a click on its dock button would, without scrolling to it. */
async function chooseSurface(page: Page, id: string): Promise<void> {
  await page
    .locator(`#surface-tab-${id}`)
    .evaluate((tab) =>
      tab.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true })),
    );
  await play(page);
}

/**
 * The showcase's other surfaces, one picture each: the surface, the dock with
 * its button chosen and the caption. The band's own pictures show the first.
 */
async function pictureSurfaces(page: Page): Promise<void> {
  const top = await pageTop(page, "#welcome-surfaces .stage");
  for (const id of ["menu-bar", "raycast", "agents", "terminal"]) {
    await chooseSurface(page, id);
    await scrollAndPlay(page, top - 16);
    await expect(page.locator(`#surface-${id}`)).toHaveClass(/\bon\b/);
    const clip = await page.locator("#welcome-surfaces").evaluate((band) => {
      const from = band.querySelector(".stage")?.getBoundingClientRect().top ?? 0;
      const to = band.querySelector(".captions")?.getBoundingClientRect().bottom ?? innerHeight;
      return {
        x: 0,
        y: Math.max(0, from - 16),
        width: innerWidth,
        height: Math.min(innerHeight, to + 16) - Math.max(0, from - 16),
      };
    });
    await expect.soft(page).toHaveScreenshot(`landing-surface-${id}.png`, {
      clip,
      stylePath: ["./visual/still.css", "./visual/band.css"],
    });
  }
  await chooseSurface(page, "dashboard");
}

test("landing", async ({ page }) => {
  await page.clock.install({ time: now });
  await page.clock.pauseAt(now);
  await page.goto("/");
  // The motion is fetched after the page is drawn and marks the page once it runs.
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  await settle(page);
  // The showcase changes surface by itself; held, it keeps the surface it shows,
  // so every picture finds the same one.
  await page.locator("[data-surfaces]").evaluate((root) => root.setAttribute("data-hold", ""));
  await expect.soft(page).toHaveScreenshot("landing-hero.png");
  // The header island once the hero's buttons have scrolled away under it.
  await scrollAndPlay(page, await pageTop(page, ".mash-agents"));
  await expect(page.locator(".mash-header")).toHaveAttribute("data-past-hero", "");
  await expect.soft(page).toHaveScreenshot("landing-island.png", {
    clip: { x: 0, y: 0, width: page.viewportSize()?.width ?? 0, height: 96 },
  });

  for (const [name, selector] of bands) {
    if (name === "quick-search" && (await page.locator(`${selector} .pin-spacer`).count())) {
      await quickSearchHeld(page);
      continue;
    }
    await playBand(page, selector);
    await pictureBand(page, name, selector);
    if (name === "surfaces") await pictureSurfaces(page);
  }
});
