// The site's visual baselines: one picture of the top of each main page, in
// every variant playwright.visual.config.ts lists. The landing page has its
// own file, because its scenes have to be played to their end first.
import { expect, test } from "@playwright/test";
import { settle } from "./settle.ts";

const pages = [
  { name: "docs", path: "/search-across-machines/", shows: "Search across your machines" },
  { name: "downloads", path: "/downloads/", shows: "Download and verify Mash" },
  { name: "supported-agents", path: "/supported-agents/", shows: "Supported agents" },
  { name: "not-found", path: "/no-such-page/", shows: "Page not found" },
] as const;

for (const entry of pages) {
  test(entry.name, async ({ page }) => {
    await page.goto(entry.path);
    await expect(page.locator("h1").filter({ visible: true }).first()).toContainText(entry.shows);
    await settle(page);
    await expect(page).toHaveScreenshot(`${entry.name}.png`);
  });
}
