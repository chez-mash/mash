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

for (const locale of [
  {
    path: "fr",
    label: "Français",
    lang: "fr",
    quickStart: "Guide de démarrage rapide",
    trialLink: "guide d’essai local",
    trialHeading: "Essayer Mash avec un historique limité",
    privacy: "Historique et confidentialité",
    docs: "Documentation",
    theme: "Mode sombre",
    search: "Rechercher",
    query: "identifiants secrets",
    fallback: "Ce contenu n’est pas encore disponible dans votre langue.",
  },
  {
    path: "zh-cn",
    label: "简体中文",
    lang: "zh-CN",
    quickStart: "快速入门",
    trialLink: "本地试用指南",
    trialHeading: "用少量本地历史试用",
    privacy: "历史记录与隐私",
    docs: "文档",
    theme: "深色模式",
    search: "搜索",
    query: "敏感信息",
    fallback: "此内容尚不支持你的语言。",
  },
]) {
  test(`${locale.label} readers can follow setup, search the docs and see English fallbacks`, async ({
    page,
  }) => {
    await page.goto("/quick-start/");
    await page
      .getByRole("combobox", { name: "Select language" })
      .selectOption({ label: locale.label });
    await expect(page).toHaveURL((url) => url.pathname === `/${locale.path}/quick-start/`);
    await expect(page.locator("html")).toHaveAttribute("lang", locale.lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(locale.quickStart);
    await expect(page.getByRole("link", { name: locale.docs, exact: true })).toHaveAttribute(
      "href",
      `/${locale.path}/guides/`,
    );
    await expect(page.locator("pre").filter({ hasText: "sh -s -- --setup" })).toBeVisible();
    await expect(page.locator('pre[data-language="powershell"]')).toContainText(/ -Setup$/);

    await page.getByRole("link", { name: locale.trialLink }).click();
    await expect(page).toHaveURL(
      (url) => url.pathname === `/${locale.path}/privacy/` && !!url.hash,
    );
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(locale.privacy);
    await expect(page.getByRole("heading", { name: locale.trialHeading })).toBeVisible();
    await expect(page.getByRole("switch", { name: locale.theme })).toBeVisible();

    await page.getByRole("button", { name: locale.search, exact: true }).click();
    await page.getByPlaceholder(locale.search, { exact: true }).fill(locale.query);
    await expect(page.locator(`dialog a[href^="/${locale.path}/privacy/"]`).first()).toBeVisible();
    await page.keyboard.press("Escape");

    await page.locator(`nav.sidebar a[href="/${locale.path}/installation/"]`).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Install Mash");
    await expect(page.getByText(locale.fallback, { exact: true })).toBeVisible();
  });
}
