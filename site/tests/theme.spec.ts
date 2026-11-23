import { expect, test } from "@playwright/test";

// The site has one theme. A reader whose system is set to dark still sees it,
// and there is no switch to look for.
test("the site is light whatever the system prefers", async ({ browser }) => {
  const context = await browser.newContext({
    colorScheme: "dark",
    viewport: { width: 1200, height: 800 },
  });
  const page = await context.newPage();
  await page.goto("/quick-start/");
  const html = page.locator("html");
  await expect(html).toHaveAttribute("data-theme", "light");
  await expect(page.getByRole("switch", { name: /dark mode/i })).toHaveCount(0);
  await expect(page.locator("starlight-theme-select, .mash-theme")).toHaveCount(0);
  await expect(page.locator(".site-title img:visible")).toHaveAttribute("src", /mash-mark/);
  // Moving to another page without a full load stays light.
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Installation", { exact: true }).click();
  await sidebar.locator('a[href="/installation/"]').click();
  await expect(page).toHaveURL(/\/installation\/$/);
  await expect(html).toHaveAttribute("data-theme", "light");
  await context.close();
});
