import { expect, type Locator, type Page, test } from "@playwright/test";

const phone = { width: 390, height: 844 };
const names = ["Agents", "Tools", "Ask"];
const counts = [6, 6, 5];

/** Opens the landing page with the agents band's rails running and on screen. */
async function open(page: Page): Promise<Locator> {
  await page.goto("/");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  const band = page.locator("#welcome-agents");
  await band.locator(".rail-tabs").scrollIntoViewIfNeeded();
  await expect(band.locator(".rail-panel.on [data-rail-go]")).toBeVisible();
  return band;
}

const track = (band: Locator) => band.locator(".rail-panel.on [data-rail-track]");
const left = (band: Locator) => track(band).evaluate((node) => node.scrollLeft);
/** How far the rail is from resting on card `count`, in pixels. */
const off = async (band: Locator, count: number, step: number) =>
  Math.abs((await left(band)) - count * step);

test("the agents band's tabs are a tablist that swaps the rail in place", async ({ page }) => {
  await page.setViewportSize(phone);
  const band = await open(page);
  const tabs = band.getByRole("tablist").getByRole("tab");
  await expect(tabs).toHaveText(names);
  const height = () => band.evaluate((node) => Math.round(node.getBoundingClientRect().height));
  const before = await height();
  for (const [index, name] of names.entries()) {
    await tabs.nth(index).click();
    await expect(tabs.nth(index)).toHaveAttribute("aria-selected", "true");
    const panel = band.getByRole("tabpanel");
    await expect(panel).toHaveCount(1);
    await expect(panel).toHaveAccessibleName(name);
    await expect(panel.getByRole("listitem")).toHaveCount(counts[index] ?? 0);
    expect(await height()).toBe(before);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(phone.width);
  }
  // The arrow keys, Home and End move between the tabs and wrap.
  await page.keyboard.press("ArrowRight");
  await expect(tabs.nth(0)).toBeFocused();
  await expect(tabs.nth(0)).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("End");
  await expect(tabs.nth(2)).toHaveAttribute("aria-selected", "true");
  await page.keyboard.press("ArrowLeft");
  await expect(tabs.nth(1)).toBeFocused();
  await expect(tabs.nth(1)).toHaveAttribute("tabindex", "0");
  await expect(tabs.nth(0)).toHaveAttribute("tabindex", "-1");
  await page.keyboard.press("Home");
  await expect(band.getByRole("tabpanel")).toHaveAccessibleName("Agents");
  // Each tab is tall enough to tap.
  for (const tab of await tabs.all())
    expect((await tab.boundingBox())?.height).toBeGreaterThanOrEqual(44);
});

test("a rail snaps to its cards, shows the next one at the edge, and its buttons step a card", async ({
  page,
}) => {
  await page.setViewportSize(phone);
  const band = await open(page);
  const cards = track(band).getByRole("listitem");
  const [first, second] = [await cards.nth(0).boundingBox(), await cards.nth(1).boundingBox()];
  if (!first || !second) throw new Error("The rail has no cards");
  // The first card starts at the band's gutter and the second shows before the screen ends.
  expect(first.x).toBe(16);
  expect(second.x).toBeLessThan(phone.width - 24);
  expect(second.x + second.width).toBeGreaterThan(phone.width);
  await expect(track(band)).toHaveCSS("scroll-snap-type", "x mandatory");
  await expect(cards.nth(1)).toHaveCSS("scroll-snap-align", "start");
  const step = second.x - first.x;
  const [previous, next] = [
    band.locator(".rail-panel.on").getByRole("button", { name: /^Previous/ }),
    band.locator(".rail-panel.on").getByRole("button", { name: /^Next/ }),
  ];
  await expect(previous).toBeDisabled();
  await next.click();
  await expect.poll(() => off(band, 1, step)).toBeLessThan(2);
  await expect(previous).toBeEnabled();
  await next.click();
  await expect.poll(() => off(band, 2, step)).toBeLessThan(2);
  await previous.click();
  await expect.poll(() => off(band, 1, step)).toBeLessThan(2);
  // A scroll that stops between two cards comes to rest on one.
  await track(band).evaluate(
    (node, by) => node.scrollBy({ left: by, behavior: "instant" }),
    step * 0.4,
  );
  await expect
    .poll(async () => Math.min(await off(band, 1, step), await off(band, 2, step)))
    .toBeLessThan(2);
  // The arrow keys step a card while the rail has the focus.
  await track(band).evaluate((node) => node.scrollTo({ left: 0, behavior: "instant" }));
  await track(band).focus();
  await page.keyboard.press("ArrowRight");
  await expect.poll(() => off(band, 1, step)).toBeLessThan(2);
  await page.keyboard.press("ArrowRight");
  await expect.poll(() => off(band, 2, step)).toBeLessThan(2);
  await page.keyboard.press("ArrowLeft");
  await expect.poll(() => off(band, 1, step)).toBeLessThan(2);
  // At the far end there is no further to go.
  await track(band).evaluate((node) =>
    node.scrollTo({ left: node.scrollWidth, behavior: "instant" }),
  );
  await expect(next).toBeDisabled();
  for (const button of [previous, next]) {
    const box = await button.boundingBox();
    expect(Math.min(box?.width ?? 0, box?.height ?? 0)).toBeGreaterThanOrEqual(44);
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(phone.width);
});

test("every card is one link named by its name and its line, leading to a guide", async ({
  page,
}) => {
  const band = await open(page);
  const tabs = band.getByRole("tab");
  for (const [index] of names.entries()) {
    await tabs.nth(index).click();
    const links = track(band).getByRole("link");
    await expect(links).toHaveCount(counts[index] ?? 0);
    for (const link of await links.all()) {
      await expect(link).toHaveAttribute(
        "href",
        /^\/(agents|plugin|supported-agents|tools|pick-up-work|records)\/(#[\w-]+)?$/,
      );
      await expect(link).toHaveAccessibleName(/^\S.{3,40} .{20,}$/);
      const box = await link.boundingBox();
      expect(box?.height).toBeGreaterThan(300);
    }
    await expect(band.locator(".rail-panel.on .rail-foot a")).toHaveText(
      ["See every agent ›", "All tools ›", "How to ask ›"][index] ?? "",
    );
  }
  await tabs.nth(1).click();
  await expect(track(band).getByRole("link").first()).toHaveAccessibleName(
    /^search Find past chats/,
  );
  // A rail that is not showing is neither drawn nor reachable.
  await expect(band.locator("#agents-rail-agents")).toHaveCSS("content-visibility", "hidden");
  await expect(band.locator("#agents-rail-agents a").first()).toBeHidden();
});

test("the guides the cards lead to have the sections they name", async ({ page }) => {
  for (const target of [
    "/agents/#connect-claude-code-codex-or-copilot",
    "/agents/#ask-your-agent-to-find-past-work",
    "/plugin/#github-copilot-cli",
    "/tools/#tool-recall_records",
    "/tools/#tool-get_attachment",
    "/records/#save-a-conclusion",
  ]) {
    await page.goto(target);
    await expect(page.locator(`[id="${target.split("#")[1]}"]`), target).toHaveCount(1);
  }
});

test("without scripting every rail shows under its name and scrolls by itself", async ({
  browser,
}) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: phone });
  const page = await context.newPage();
  await page.goto("/");
  const band = page.locator("#welcome-agents");
  await expect(band.getByRole("tablist")).toBeHidden();
  await expect(band.locator(".rail-name")).toHaveText(names);
  for (const name of await band.locator(".rail-name").all()) await expect(name).toBeVisible();
  await expect(band.locator("[data-rail-go]").first()).toBeHidden();
  for (const rail of await band.locator("[data-rail-track]").all()) {
    await expect(rail.getByRole("link").first()).toBeVisible();
    expect(await rail.evaluate((node) => node.scrollWidth > node.clientWidth)).toBe(true);
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(phone.width);
  await context.close();
});
