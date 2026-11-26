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

// The landing page's header is the logo alone on a phone, so its footer holds
// the theme and language menus.
test("on a phone the landing page's footer changes the theme and the language", async ({
  browser,
}) => {
  const context = await browser.newContext({
    colorScheme: "light",
    viewport: { width: 390, height: 844 },
  });
  const page = await context.newPage();
  await page.goto("/");
  const html = page.locator("html");
  const footer = page.locator("footer.mash-footer");
  const theme = footer.getByRole("combobox", { name: "Select theme" });
  const language = footer.getByRole("combobox", { name: "Select language" });
  await theme.scrollIntoViewIfNeeded();
  await expect(theme).toBeVisible();
  await expect(language).toBeVisible();
  await expect(theme).toHaveValue("auto");
  // Both fit the phone, each tall enough to touch.
  for (const menu of [theme, language]) {
    const box = await menu.boundingBox();
    expect(box?.x).toBeGreaterThanOrEqual(8);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(390);
    expect(box?.height).toBeGreaterThanOrEqual(40);
  }
  // The theme changes at once and is still chosen after a full load.
  await expect(html).toHaveAttribute("data-theme", "light");
  const background = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
  const light = await background();
  await theme.selectOption("dark");
  await expect(html).toHaveAttribute("data-theme", "dark");
  expect(await background()).not.toBe(light);
  await page.reload();
  await expect(html).toHaveAttribute("data-theme", "dark");
  await expect(theme).toHaveValue("dark");
  // The language menu leads to the first page in that language, which keeps
  // the theme. That page is a guide, with both menus in its own menu.
  await language.selectOption({ label: "Español" });
  await expect(page).toHaveURL((url) => url.pathname === "/es/");
  await expect(html).toHaveAttribute("lang", "es");
  await expect(html).toHaveAttribute("data-theme", "dark");
  await page.locator(".sl-menu-button").click();
  await expect(page.locator("nav.sidebar starlight-theme-select select")).toHaveValue("dark");
  // The guides directory shares the footer's line but has the menus in its header.
  await page.goto("/guides/");
  await expect(page.locator("footer.mash-footer")).toBeVisible();
  await expect(page.locator("footer.mash-footer select")).toHaveCount(0);
  await context.close();
});
