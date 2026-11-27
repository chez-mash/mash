import { expect, test } from "@playwright/test";

async function emptyHistory(page: import("@playwright/test").Page): Promise<void> {
  await page.route("**/api/**", (route) =>
    route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } }),
  );
}

const scheme = (page: import("@playwright/test").Page) =>
  page.evaluate(() => getComputedStyle(document.documentElement).colorScheme);

// Mash has one theme, dark, whatever the system's appearance, and Settings
// offers no choice of theme.
test("the dashboard is dark on a light system and has no theme setting", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await emptyHistory(page);
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/#/settings");
  await expect(page.locator("#rail .brand img")).toHaveAttribute("src", /mash-mark\.svg$/);
  expect(await scheme(page)).toBe("dark");
  await expect(page.getByRole("group", { name: "Theme" })).toHaveCount(0);
  await expect(page.locator("#set-appearance")).toHaveCount(0);
});

// ⌘K on a Mac, Ctrl+K elsewhere, focuses the search field from anywhere.
test("Control+K focuses the search field", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await emptyHistory(page);
  await page.goto("/#/settings");
  await expect(page.locator("#top .keys")).toBeVisible();
  await page.keyboard.press("Control+k");
  await expect(page.getByRole("textbox", { name: "Search your history" })).toBeFocused();
});
