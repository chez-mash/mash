// On a phone the controls floating in the corner step aside while the reader
// scrolls down, and come back when they scroll up, stop or reach the end.
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
const tucked = { fab: "none", ends: "none" };

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

test("on a phone the floating controls step aside while scrolling down", async ({ page }) => {
  await midChat(page, 390, 844);
  const fab = page.locator("#mashy-fab");
  const ends = page.getByRole("navigation", { name: "Chat ends" });

  expect(await scrollBy(page, 120)).toEqual(tucked);
  await expect(fab).toHaveCSS("opacity", "0");
  await expect(ends).toHaveCSS("opacity", "0");
  // Still there for a screen reader while out of sight.
  await expect(page.getByRole("button", { name: "Open Mashy" })).toHaveCount(1);
  await expect(ends.getByRole("button")).toHaveCount(2);

  // Scrolling back up brings them back at once.
  expect(await scrollBy(page, -40)).toEqual(shown);
  await expect(fab).toHaveCSS("opacity", "1");

  // So does resting a moment.
  expect(await scrollBy(page, 120)).toEqual(tucked);
  await expect(fab).toHaveCSS("opacity", "1");
  await expect(ends).toHaveCSS("opacity", "1");
  await fab.click();
  await expect(page.locator("#mashy")).toBeVisible();
});

test("on a phone the floating controls come back at the end of the page", async ({ page }) => {
  await midChat(page, 390, 844);
  // Near the end, waiting for the page to say it moved there.
  await page.evaluate(async () => {
    const moved = new Promise((done) => addEventListener("scroll", done, { once: true }));
    window.scrollTo({
      top: document.documentElement.scrollHeight - innerHeight - 300,
      behavior: "instant",
    });
    await moved;
  });
  expect(await scrollBy(page, 100)).toEqual(tucked);
  expect(await scrollBy(page, 400)).toEqual(shown);
});

test("a tucked control comes back when the keyboard reaches it", async ({ page }) => {
  await midChat(page, 390, 844);
  const latest = page.getByRole("button", { name: "Latest message" });
  const first = page.getByRole("button", { name: "First message" });
  await first.focus();
  await page.keyboard.press("Shift+Tab");
  expect(await scrollBy(page, 120)).toEqual(tucked);
  await page.keyboard.press("Tab");
  await expect(first).toBeFocused();
  await expect(page.locator(".ends")).toHaveCSS("opacity", "1");
  await expect(page.locator(".ends")).toHaveCSS("pointer-events", "auto");
  await page.keyboard.press("Tab");
  await expect(latest).toBeFocused();
  await expect(page.locator(".ends")).toHaveCSS("opacity", "1");
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

test("on a phone the floating controls stay while new messages are offered", async ({ page }) => {
  const chat = await midChat(page, 390, 844);
  expect(await scrollBy(page, 120)).toEqual(tucked);
  await chat.grow();
  await expect(page.getByRole("button", { name: "1 new message" })).toBeVisible();
  expect(await scrollBy(page, 120)).toEqual(shown);
});

test("with reduced motion the controls fade where they are", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await midChat(page, 390, 844);
  expect(await scrollBy(page, 120)).toEqual(tucked);
  await expect(page.locator("#mashy-fab")).toHaveCSS("opacity", "0");
  await expect(page.locator("#mashy-fab")).toHaveCSS("translate", "0px");
});
