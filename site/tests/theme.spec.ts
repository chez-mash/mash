import { expect, test } from "@playwright/test";

// The site has one theme, dark, like the app: there is no theme menu, and a
// light system, a choice stored by an earlier version of the site, and moving
// between pages without a full load all leave it dark.
test("the site is dark on a light system, with no theme menu anywhere", async ({ browser }) => {
  const context = await browser.newContext({
    colorScheme: "light",
    viewport: { width: 1200, height: 800 },
  });
  const page = await context.newPage();
  await page.goto("/");
  const html = page.locator("html");
  await expect(html).toHaveAttribute("data-theme", "dark");
  await expect(page.locator("starlight-theme-select")).toHaveCount(0);
  await page.goto("/quick-start/");
  await expect(html).toHaveAttribute("data-theme", "dark");
  await expect(page.locator("starlight-theme-select")).toHaveCount(0);
  // Moving to another page without a full load stays dark.
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Installation", { exact: true }).click();
  await sidebar.locator('a[href="/installation/"]').click();
  await expect(page).toHaveURL(/\/installation\/$/);
  await expect(html).toHaveAttribute("data-theme", "dark");
  await context.close();
});

test("a light theme stored by an earlier version of the site is ignored", async ({ browser }) => {
  const context = await browser.newContext({
    colorScheme: "light",
    storageState: {
      cookies: [],
      origins: [
        {
          origin: "http://127.0.0.1:8793",
          localStorage: [{ name: "starlight-theme", value: "light" }],
        },
      ],
    },
  });
  const page = await context.newPage();
  await page.goto("/");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await context.close();
});

// The landing page's header is the logo alone on a phone, so its footer holds
// the language menu.
test("on a phone the landing page's footer changes the language and stays dark", async ({
  browser,
}) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto("/");
  const html = page.locator("html");
  const footer = page.locator("footer.mash-footer");
  const language = footer.getByRole("combobox", { name: "Select language" });
  await language.scrollIntoViewIfNeeded();
  await expect(language).toBeVisible();
  await expect(footer.getByRole("combobox", { name: "Select theme" })).toHaveCount(0);
  const box = await language.boundingBox();
  expect(box?.x).toBeGreaterThanOrEqual(8);
  expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(390);
  expect(box?.height).toBeGreaterThanOrEqual(40);
  await language.selectOption({ label: "Español" });
  await expect(page).toHaveURL((url) => url.pathname === "/es/");
  await expect(html).toHaveAttribute("lang", "es");
  await expect(html).toHaveAttribute("data-theme", "dark");
  await context.close();
});
