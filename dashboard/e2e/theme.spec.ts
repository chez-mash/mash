import { expect, test } from "@playwright/test";

async function emptyHistory(page: import("@playwright/test").Page): Promise<void> {
  await page.route("**/api/**", (route) =>
    route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } }),
  );
}

const scheme = (page: import("@playwright/test").Page) =>
  page.evaluate(() => getComputedStyle(document.documentElement).colorScheme);

// The dashboard follows the system's light or dark appearance by default.
test("the dashboard follows the system's appearance", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await emptyHistory(page);
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/");
  await expect(page.locator("#rail .brand img")).toHaveAttribute("src", /mash-mark\.svg$/);
  expect(await scheme(page)).toBe("dark");
  await page.emulateMedia({ colorScheme: "light" });
  expect(await scheme(page)).toBe("light");
});

// Settings can pin one theme in this browser, and the choice survives a reload.
test("a theme chosen in Settings overrides the system and is remembered", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await emptyHistory(page);
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/#/settings");
  const themes = page.getByRole("group", { name: "Theme" });
  await themes.getByRole("radio", { name: "light", exact: true }).click();
  await expect(themes.getByRole("radio", { name: "light", exact: true })).toHaveAttribute(
    "aria-checked",
    "true",
  );
  expect(await scheme(page)).toBe("light");
  await page.reload();
  expect(await scheme(page)).toBe("light");
  await page
    .getByRole("group", { name: "Theme" })
    .getByRole("radio", { name: "match the system" })
    .click();
  expect(await scheme(page)).toBe("dark");
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
