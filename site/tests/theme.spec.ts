import { expect, test } from "@playwright/test";

// The site has a light and a dark theme. It follows the system until the
// reader picks one in Starlight's theme menu, and the choice holds as pages
// change without a full load.
test("the site follows the system's theme, and the menu overrides it", async ({ browser }) => {
  const context = await browser.newContext({
    colorScheme: "dark",
    viewport: { width: 1200, height: 800 },
  });
  const page = await context.newPage();
  await page.goto("/quick-start/");
  const html = page.locator("html");
  await expect(html).toHaveAttribute("data-theme", "dark");
  const background = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  const dark = await background();
  const menu = page.locator("header starlight-theme-select select");
  await expect(menu).toBeVisible();
  await menu.selectOption("light");
  await expect(html).toHaveAttribute("data-theme", "light");
  expect(await background()).not.toBe(dark);
  await expect(page.locator("header .brand img")).toHaveAttribute("src", /mash-mark/);
  // Moving to another page without a full load keeps the choice.
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Installation", { exact: true }).click();
  await sidebar.locator('a[href="/installation/"]').click();
  await expect(page).toHaveURL(/\/installation\/$/);
  await expect(html).toHaveAttribute("data-theme", "light");
  // And back to the system's.
  await page.locator("header starlight-theme-select select").selectOption("auto");
  await expect(html).toHaveAttribute("data-theme", "dark");
  await context.close();
});

test("a light system gets the light theme", async ({ browser }) => {
  const context = await browser.newContext({ colorScheme: "light" });
  const page = await context.newPage();
  await page.goto("/");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await context.close();
});
