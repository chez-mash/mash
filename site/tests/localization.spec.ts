import { expect, test } from "@playwright/test";

test("the Spanish quickstart links to privacy and can search the Spanish guides", async ({
  page,
}) => {
  await page.goto("/quick-start/");
  await page.getByRole("link", { name: "Español", exact: true }).click();
  await expect(page).toHaveURL(/\/es\/quick-start\/$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "es");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Guía de inicio rápido");

  await page
    .locator(".sl-markdown-content")
    .getByRole("link", { name: "Privacidad del historial", exact: true })
    .click();
  await expect(page).toHaveURL(/\/es\/privacy\/$/);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Privacidad del historial");
  await expect(page.getByRole("switch", { name: "Modo oscuro" })).toBeVisible();
  await page.getByRole("button", { name: "Buscar", exact: true }).click();
  await page.getByPlaceholder("Buscar", { exact: true }).fill("credenciales");
  await expect(page.locator('dialog a[href^="/es/privacy/"]').first()).toBeVisible();
  await page.keyboard.press("Escape");

  await page.getByRole("link", { name: "Privacy in English", exact: true }).click();
  await expect(page).toHaveURL((url) => url.pathname === "/privacy/");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.getByRole("button", { name: "Search", exact: true })).toBeVisible();
});

test("the language picker keeps the page and labels English fallbacks", async ({ page }) => {
  await page.goto("/installation/");
  await page.getByRole("combobox", { name: "Select language" }).selectOption({ label: "Español" });
  await expect(page).toHaveURL(/\/es\/installation\/$/);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Install Mash");
  await expect(page.getByText("Esta página aún no está disponible en tu idioma.")).toBeVisible();
});
