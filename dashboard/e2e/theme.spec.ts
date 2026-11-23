import { expect, test } from "@playwright/test";

// The dashboard has one theme. A reader whose system is set to dark still
// sees it, and there is no switch to look for.
test("the dashboard is light whatever the system prefers", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.route("**/api/**", (route) =>
    route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } }),
  );
  await page.goto("/");
  await expect(page.getByRole("switch", { name: /dark mode/i })).toHaveCount(0);
  const scheme = await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme);
  expect(scheme).toBe("light");
  await expect(page.locator("#rail .brand img")).toHaveAttribute("src", /mash-mark\.svg$/);
});
