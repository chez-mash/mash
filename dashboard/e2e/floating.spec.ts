// Mashy's circle floats in the screen's bottom right corner. Beside the rail
// a chat keeps a lane for it and for the jumps to either end stacked above
// it. A phone has no lane and shows no jumps, so there the circle tucks to
// the screen's edge while the reader scrolls down, and comes back when they
// scroll up, stop or reach the end.
import { expect, type Page, type Route, test } from "@playwright/test";

/** Whether each control takes taps right after the page scrolls by `by`. */
function scrollBy(page: Page, by: number): Promise<{ fab: string; ends: string }> {
  return page.evaluate(async (by) => {
    // Let any scroll already under way, such as focus moving, report first.
    for (let frame = 0; frame < 2; frame++) await new Promise(requestAnimationFrame);
    const moved = new Promise((done) => addEventListener("scroll", done, { once: true }));
    window.scrollBy({ top: by, behavior: "instant" });
    await moved;
    const taps = (selector: string): string => {
      const element = document.querySelector(selector);
      return element ? getComputedStyle(element).pointerEvents : "missing";
    };
    return { fab: taps("#mashy-fab"), ends: taps(".ends") };
  }, by);
}
const shown = { fab: "auto", ends: "auto" };
const PHONE = 390;
/** A phone's page margin, which is all a tucked circle may show in. */
const MARGIN = 16;

/**
 * Answers for one long chat, and holds each change stream open; `grow` adds
 * a message and ends the next stream saying the chat changed.
 */
async function longChat(page: Page): Promise<{ grow: () => Promise<void> }> {
  let count = 30;
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: [
          { chat_id: "long", node_id: "local", provider: "codex", title: "Long", updated_at: 1 },
        ],
      },
    }),
  );
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: {
        message_count: count,
        messages: Array.from({ length: count }, (_, index) => ({
          id: `m${index}`,
          ordinal: index,
          role: index % 2 ? "assistant" : "user",
          content: `Message number ${index}. ${"Ordinary text to fill the page. ".repeat(12)}`,
        })),
      },
    }),
  );
  const streams: Route[] = [];
  await page.route("**/api/events", (route) => {
    streams.push(route);
  });
  // One more message arrives, and Mash says the chat changed.
  const grow = async (): Promise<void> => {
    count += 1;
    await expect.poll(() => streams.length, { timeout: 15_000 }).toBeGreaterThan(0);
    const change = { node: null, revision: count, chats: ["long"], all: false, records: false };
    await streams.shift()?.fulfill({
      contentType: "text/event-stream",
      body: `event: change\ndata: ${JSON.stringify(change)}\n\n`,
    });
  };
  return { grow };
}

/** Opens the long chat and leaves the reader part way down it. */
async function midChat(page: Page, width: number, height: number) {
  await page.setViewportSize({ width, height });
  const chat = await longChat(page);
  await page.goto("/#/chat/long");
  await expect(page.locator("#message-m29")).toBeInViewport();
  // A jump this far is the page moving itself, so the controls stay.
  await page.evaluate(() => window.scrollTo({ top: 300, behavior: "instant" }));
  await expect(page.locator("#mashy-fab")).toHaveCSS("opacity", "1");
  return chat;
}

test("on a phone Mashy's circle tucks to the edge while scrolling down", async ({ page }) => {
  await midChat(page, PHONE, 844);
  const fab = page.locator("#mashy-fab");
  await expect(page.getByRole("navigation", { name: "Chat ends" })).toBeHidden();
  // At rest it floats in the bottom right corner.
  const resting = await fab.boundingBox();
  expect(resting && resting.x + resting.width > PHONE - 40 && resting.y > 844 - 100).toBe(true);

  expect((await scrollBy(page, 120)).fab).toBe("none");
  // What still shows of it is inside the page's margin, over no words.
  await expect
    .poll(async () => (await fab.boundingBox())?.x ?? 0)
    .toBeGreaterThanOrEqual(PHONE - MARGIN);
  // Still there for a screen reader while tucked.
  await expect(page.getByRole("button", { name: "Open Mashy" })).toHaveCount(1);

  // Scrolling back up brings it back at once.
  expect((await scrollBy(page, -40)).fab).toBe("auto");
  await expect(fab).toHaveCSS("opacity", "1");

  // So does resting a moment.
  expect((await scrollBy(page, 120)).fab).toBe("none");
  await expect(fab).toHaveCSS("pointer-events", "auto");
  await expect(fab).toHaveCSS("opacity", "1");
  await fab.click();
  await expect(page.locator("#mashy")).toBeVisible();
});

test("on a phone the circle comes back at the end of the page", async ({ page }) => {
  await midChat(page, PHONE, 844);
  // Near the end, waiting for the page to say it moved there.
  await page.evaluate(async () => {
    const moved = new Promise((done) => addEventListener("scroll", done, { once: true }));
    window.scrollTo({
      top: document.documentElement.scrollHeight - innerHeight - 300,
      behavior: "instant",
    });
    await moved;
  });
  expect((await scrollBy(page, 100)).fab).toBe("none");
  expect((await scrollBy(page, 400)).fab).toBe("auto");
});

test("a circle the keyboard has reached stays in view", async ({ page }) => {
  await midChat(page, PHONE, 844);
  const fab = page.locator("#mashy-fab");
  // A key press first, so the browser shows focus as it does for a keyboard.
  await page.keyboard.press("Shift");
  await fab.focus();
  expect((await scrollBy(page, 120)).fab).toBe("auto");
  await expect(fab).toHaveCSS("opacity", "1");
});

test("the circle stays while Mashy is open beside the page", async ({ page }) => {
  // Between a phone and a wide screen Mashy is a window over the page, which
  // still scrolls behind it.
  await midChat(page, 760, 900);
  await page.locator("#mashy-fab").click();
  await expect(page.locator("#mashy")).toBeVisible();
  expect((await scrollBy(page, 120)).fab).toBe("auto");
  await expect(page.locator("#mashy-fab")).toHaveCSS("opacity", "1");
});

test("on a wide screen scrolling leaves the floating controls alone", async ({ page }) => {
  await midChat(page, 1440, 900);
  expect(await scrollBy(page, 120)).toEqual(shown);
  await expect(page.locator("#mashy-fab")).toHaveCSS("opacity", "1");
  await expect(page.locator(".ends")).toHaveCSS("opacity", "1");
});

test("on a phone the circle stays while new messages are offered", async ({ page }) => {
  const chat = await midChat(page, PHONE, 844);
  expect((await scrollBy(page, 120)).fab).toBe("none");
  await chat.grow();
  await expect(page.getByRole("button", { name: "1 new message" })).toBeVisible();
  expect((await scrollBy(page, 120)).fab).toBe("auto");
});

test("with reduced motion the circle fades where it is", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await midChat(page, PHONE, 844);
  expect((await scrollBy(page, 120)).fab).toBe("none");
  await expect(page.locator("#mashy-fab")).toHaveCSS("opacity", "0");
  await expect(page.locator("#mashy-fab")).toHaveCSS("translate", "0px");
});
