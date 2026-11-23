import type { Locator, Page } from "@playwright/test";

/** The rail's links. Below 800px the rail is a drawer and its links are hidden until opened. */
export function rail(page: Page): Locator {
  return page.getByRole("navigation", { name: "Main" });
}

export function searchBox(page: Page): Locator {
  return page.getByRole("textbox", { name: "Search your history" });
}

/** Opens the drawer on a narrow screen; the rail is always shown on a wide one. */
export async function openRail(page: Page): Promise<void> {
  const burger = page.getByRole("button", { name: "Open navigation" });
  if ((await burger.isVisible()) && (await burger.getAttribute("aria-expanded")) !== "true")
    await burger.click();
}

/**
 * Follows a rail link the way a reader would: opening the drawer when the
 * screen is narrow, and expanding `section` when the link is one of its children.
 */
export async function followRail(page: Page, label: string, section?: string): Promise<void> {
  await openRail(page);
  const link = rail(page).getByRole("link", { name: label, exact: true });
  if (section && !(await link.isVisible()))
    await rail(page)
      .getByRole("button", { name: `Expand ${section}`, exact: true })
      .click();
  await link.click();
}

export function fitsViewport(page: Page): Promise<boolean> {
  return page.evaluate(() => document.documentElement.scrollWidth <= innerWidth);
}

// Enough chats that every section of the rail has several pages under it.
const agents = ["codex", "claude-code", "cursor", "opencode", "copilot"];
const chats = Array.from({ length: 12 }, (_, index) => ({
  chat_id: `chat-${index}`,
  node_id: `machine-${index % 4}`,
  provider: agents[index % agents.length],
  title: `Fixture chat ${index}`,
  project: `/work/project-${index % 6}`,
  updated_at: Date.now() - index * 60_000,
}));
const message = (index: number) => ({
  id: `m${index}`,
  ordinal: index,
  role: index % 2 ? "assistant" : "user",
  content: `Message number ${index}. ${"A line of ordinary text. ".repeat(6)}`,
});

/** Answers the dashboard's reads with a dozen chats across agents, machines and projects. */
export async function mockShell(page: Page): Promise<void> {
  await page.route("**/api/**", (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/records") return route.fulfill({ json: { records: [], next_offset: null } });
    if (path === "/api/assistant") return route.fulfill({ json: { enabled: true, model: "m" } });
    if (path === "/api/chat")
      return route.fulfill({
        json: { message_count: 12, messages: Array.from({ length: 12 }, (_, i) => message(i)) },
      });
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "machine-0", truncated: false, chats } });
    return route.fulfill({ json: [] });
  });
}

/**
 * Counts the documents the page loads, leaving out a reload that follows a
 * module that failed to arrive. index.html makes those reloads when a network
 * change on the host aborts the page's own code while it loads (see its
 * load-failure handler); they recover a blank page and are not navigation.
 */
export function documentLoads(page: Page): () => number {
  let loads = 0;
  let lostModule = false;
  page.on("requestfailed", (request) => {
    if (request.resourceType() === "script") lostModule = true;
  });
  page.on("request", (request) => {
    if (!request.isNavigationRequest() || request.frame() !== page.mainFrame()) return;
    if (!lostModule) loads++;
    lostModule = false;
  });
  return () => loads;
}
