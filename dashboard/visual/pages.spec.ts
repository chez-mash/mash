// The dashboard's visual baselines: one picture of each main page and of a
// few open menus, taken from the demo build and its made-up history, in every
// variant playwright.visual.config.ts lists. A page that drifts fails here.
//
// A picture is only worth comparing if nothing in it changes by itself, so:
// the clock is fixed (the demo dates its history from the browser's clock),
// motion and the caret are off, fonts and images have arrived, and what is
// not the dashboard's own is hidden (still.css) or masked.
import { expect, type Locator, type Page, test } from "@playwright/test";

/** Friday evening, so the overview greets with "Good evening". */
const now = new Date("2026-11-20T19:30:00Z");

interface Shot {
  /** The picture's file name. */
  readonly name: string;
  readonly route: string;
  /** Text the page shows once it has loaded: the picture is of this page and not an error. */
  readonly shows: string | RegExp;
  /** Also taken on a dark phone. */
  readonly phoneDark?: boolean;
  /** What to open or type before the picture. */
  readonly prepare?: (page: Page) => Promise<void>;
}

const narrow = (page: Page): boolean => (page.viewportSize()?.width ?? 0) < 800;

/** Shows a chat from its top; a chat otherwise opens at its latest message. */
async function toChatTop(page: Page): Promise<void> {
  const first = page.getByRole("button", { name: "First message" });
  if (await first.isVisible()) await first.click();
  // Whichever element scrolls the transcript on this screen goes back to its start.
  await page
    .locator("[data-message]")
    .first()
    .evaluate((message) => {
      for (let parent = message.parentElement; parent; parent = parent.parentElement)
        parent.scrollTop = 0;
    });
}

/** Opens the "any agent" filter's menu; a phone folds the filters away first. */
async function openAgentFilter(page: Page): Promise<void> {
  const filters = page.getByRole("button", { name: "Filters", exact: true });
  if (await filters.isVisible()) await filters.click();
  await page.getByRole("button", { name: "any agent" }).click();
  await expect(page.getByRole("menu", { name: "any agent" })).toBeVisible();
}

/** Opens the viewing switcher: in the top bar, or in a phone's navigation drawer. */
async function openViewing(page: Page): Promise<void> {
  if (narrow(page)) await page.getByRole("button", { name: "Open navigation" }).click();
  await page.locator(".vw-b").filter({ visible: true }).click();
  await expect(page.getByRole("menu", { name: "Show chats and memory from" })).toBeVisible();
}

const shots: readonly Shot[] = [
  { name: "overview", route: "#/", shows: "Good evening", phoneDark: true },
  { name: "search", route: "#/search?q=checkout", shows: "Fix the flaky checkout test" },
  { name: "chats", route: "#/search", shows: "Every chat Mash has indexed" },
  {
    name: "chats-filter-open",
    route: "#/search",
    shows: "Every chat Mash has indexed",
    prepare: openAgentFilter,
  },
  {
    name: "chat",
    route: "#/chat/sign-in-tokens?node=laptop",
    shows: "Move sign-in to short-lived tokens",
    phoneDark: true,
    prepare: toChatTop,
  },
  {
    name: "chat-subagents",
    route: "#/chat/release-0-2?node=laptop",
    shows: "Ship the 0.2.0 release",
    prepare: toChatTop,
  },
  { name: "memory", route: "#/records", shows: "What your agents saved", phoneDark: true },
  {
    name: "record",
    route: "#/record/record-sign-in-tokens?node=laptop",
    shows: "Revision history",
  },
  { name: "machines", route: "#/machines", shows: "21 chats on 3 machines" },
  { name: "machine", route: "#/machine/laptop", shows: "Agent setup" },
  { name: "agents", route: "#/agents", shows: "Agent setup on laptop" },
  { name: "agent", route: "#/agent/codex", shows: "Recent chats" },
  { name: "projects", route: "#/projects", shows: "4 projects across 21 chats" },
  {
    name: "project",
    route: `#/project?name=${encodeURIComponent("/home/sam/code/storefront")}`,
    shows: "Recent chats",
  },
  { name: "services", route: "#/services", shows: "How your services connect" },
  { name: "settings", route: "#/settings", shows: "Agent histories" },
  { name: "tools", route: "#/console", shows: "Tool explorer" },
  { name: "export", route: "#/export", shows: "Export chats" },
  { name: "fleet", route: "#/fleet", shows: "Fleet manager" },
  { name: "history", route: "#/history", shows: "Viewing history" },
  { name: "setup-review", route: "#/setup", shows: "Step 4 of 4" },
  { name: "quick-empty", route: "#/quick", shows: "Suggestions" },
  {
    name: "quick-typed",
    route: "#/quick",
    shows: "Suggestions",
    prepare: async (page) => {
      await page.getByPlaceholder("Search Mash").fill("checkout");
      await expect(page.getByText("Fix the flaky checkout test").first()).toBeVisible();
    },
  },
  {
    name: "viewing-open",
    route: "#/",
    shows: "Good evening",
    prepare: openViewing,
  },
  {
    name: "mashy-open",
    route: "#/",
    shows: "Good evening",
    prepare: async (page) => {
      await page.getByRole("button", { name: "Open Mashy" }).click();
      await expect(page.locator("#mashy")).toBeVisible();
    },
  },
];

/** Build and contract hashes change with every release, for no visible reason. */
function variable(page: Page): Locator[] {
  return [page.getByText(/^(build|contract) [0-9a-f]{6,}$/)];
}

/** Waits for what arrives after the page: its fonts, its pictures and its late requests. */
async function settle(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
  await page.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all([...document.images].map((image) => image.decode().catch(() => undefined)));
  });
}

for (const shot of shots) {
  test(shot.name, { tag: shot.phoneDark ? "@phone-dark" : [] }, async ({ page }) => {
    await page.clock.setFixedTime(now);
    await page.goto(shot.route);
    await expect(page.getByText(shot.shows).filter({ visible: true }).first()).toBeVisible();
    await settle(page);
    await shot.prepare?.(page);
    await settle(page);
    // Wherever the last click left it, the pointer rests on nothing: no hover, no tooltip.
    await page.mouse.move(0, 0);
    await expect(page).toHaveScreenshot(`${shot.name}.png`, { mask: variable(page) });
  });
}
