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
  await page.goto("/configuration/");
  await page.getByRole("combobox", { name: "Select language" }).selectOption({ label: "Español" });
  await expect(page).toHaveURL(/\/es\/configuration\/$/);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Configure Mash");
  await expect(page.getByText("Esta página aún no está disponible en tu idioma.")).toBeVisible();
});

for (const [locale, lang, installationTitle] of [
  ["es", "es", "Instala Mash"],
  ["fr", "fr", "Installer Mash"],
  ["zh-cn", "zh-CN", "安装 Mash"],
]) {
  test(`the ${locale} quick start and privacy guide link to localized installation steps`, async ({
    page,
  }) => {
    const install = `/${locale}/installation/`;
    const verify = `${install}#check-the-release-download`;
    const uninstall = `${install}#uninstall`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${verify}"]`).click();
    await expect(page).toHaveURL(verify);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(installationTitle);
    await expect(page.locator("#check-the-release-download")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "sha256sum mash-x86_64-unknown-linux-gnu.tar.gz" }),
    ).toBeVisible();
    await guide.locator('a[href="#start-the-local-search-service"]').last().click();
    await expect(page).toHaveURL(`${install}#start-the-local-search-service`);
    await expect(guide.locator("pre").filter({ hasText: /^mash qdrant install$/ })).toBeVisible();
    await expect(
      guide.getByText("Set up and index without connecting agents", { exact: true }),
    ).toBeVisible();

    await page.goto(`/${locale}/privacy/`);
    await guide.locator(`a[href="${uninstall}"]`).click();
    await expect(page).toHaveURL(uninstall);
    await expect(page.locator("#uninstall")).toBeAttached();
    await expect(guide.locator("pre").filter({ hasText: "mash uninstall --yes" })).toBeVisible();
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Servicio de búsqueda local (Qdrant)"],
  ["fr", "fr", "Service de recherche local (Qdrant)"],
  ["zh-cn", "zh-CN", "本地搜索服务（Qdrant）"],
]) {
  test(`the ${locale} setup keeps Qdrant alternatives and restart instructions in the same language`, async ({
    page,
  }) => {
    const search = `/${locale}/local-search/`;
    const systemd = `${search}#linux-without-a-systemd-user-session`;
    const restart = `${search}#check-or-restart-the-service`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${systemd}"]`).click();
    await expect(page).toHaveURL(systemd);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(page.locator("#linux-without-a-systemd-user-session")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "systemctl --user show-environment" }),
    ).toBeVisible();
    await guide.locator('a[href="#optional-qdrant-docker-image"]').first().click();
    await expect(page).toHaveURL(`${search}#optional-qdrant-docker-image`);
    await expect(
      guide.locator("pre").filter({ hasText: "docker run -d --name mash-local-qdrant" }),
    ).toBeVisible();

    await page.goto(`/${locale}/installation/#start-the-local-search-service`);
    await guide.locator(`a[href="${restart}"]`).click();
    await expect(page).toHaveURL(restart);
    await expect(page.locator("#check-or-restart-the-service")).toBeAttached();
    await guide.locator('a[href="#where-qdrant-stores-data"]').click();
    await expect(page).toHaveURL(`${search}#where-qdrant-stores-data`);
    await expect(guide.locator("table").first()).toContainText(
      "journalctl --user --unit mash-qdrant.service",
    );
  });
}

for (const [locale, language, supportTitle, quickStartTitle, agentsTitle] of [
  ["es", "es", "Agentes compatibles", "Guía de inicio rápido", "Usa Mash con tu agente"],
  ["fr", "fr", "Agents compatibles", "Guide de démarrage rapide", "Utiliser Mash avec votre agent"],
  ["zh-cn", "zh-CN", "支持的代理", "快速入门", "在你的代理中使用 Mash"],
]) {
  test(`the ${locale} support guide leads to setup in the same language`, async ({ page }) => {
    await page.goto(`/${locale}/`);
    await page.locator(`.sl-markdown-content a[href="/${locale}/supported-agents/"]`).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(supportTitle);
    await expect(page.locator("html")).toHaveAttribute("lang", language);
    const copilot = page
      .getByRole("table")
      .first()
      .getByRole("row", { name: /GitHub Copilot CLI/ });
    await expect(copilot.getByRole("link", { name: /v0\.1\.1-alpha/ })).toHaveAttribute(
      "href",
      "/plugin/#github-copilot-cli",
    );

    await page.locator(`.sl-markdown-content a[href="/${locale}/quick-start/"]`).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(quickStartTitle);
    await expect(page).toHaveURL(`/${locale}/quick-start/`);

    const connection = `/${locale}/agents/#connect-claude-code-codex-or-copilot`;
    await page.locator(`.sl-markdown-content a[href="${connection}"]`).click();
    await expect(page).toHaveURL(connection);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(agentsTitle);
    await expect(page.locator("#connect-claude-code-codex-or-copilot")).toBeAttached();
    await expect(page.locator("pre").filter({ hasText: "mash init --no-index" })).toBeVisible();
    await expect(page.getByText("Use existing settings", { exact: true })).toBeVisible();
    const verify = `/${locale}/agents/#confirm-the-connection`;
    await page.locator(`.sl-markdown-content a[href="#confirm-the-connection"]`).click();
    await expect(page).toHaveURL(verify);
    await expect(page.locator("#confirm-the-connection")).toBeAttached();
    await expect(
      page.locator("pre").filter({ hasText: /^mash status --installation$/ }),
    ).toBeVisible();
  });
}

for (const [locale, lang, title, help] of [
  ["es", "es", "Solución de problemas", "Pide ayuda"],
  ["fr", "fr", "Dépannage", "Obtenir de l’aide"],
  ["zh-cn", "zh-CN", "故障排查", "获取帮助"],
]) {
  test(`the ${locale} setup and search guides open troubleshooting in the same language`, async ({
    page,
  }) => {
    const trouble = `/${locale}/troubleshooting/`;
    const missing = `${trouble}#mash-command-not-found`;
    const empty = `${trouble}#search-returns-no-results`;
    const guide = page.locator(".sl-markdown-content");

    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${missing}"]`).click();
    await expect(page).toHaveURL(missing);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(page.locator("#mash-command-not-found")).toBeAttached();
    await expect(guide.locator('pre[data-language="powershell"]')).toContainText("mash --version");
    await guide.getByRole("link", { name: help, exact: true }).click();
    await expect(page).toHaveURL(`${trouble}#get-help`);
    await expect(guide.locator("#get-help")).toBeAttached();
    await expect(guide.getByRole("heading", { name: help, exact: true })).toBeVisible();
    await expect(
      guide.locator('a[href="https://github.com/mashium/mash/security/advisories/new"]'),
    ).toBeVisible();

    await page.goto(`/${locale}/dashboard/#find-and-read-a-chat`);
    await guide.locator(`a[href="${empty}"]`).click();
    await expect(page).toHaveURL(empty);
    await expect(page.locator("#search-returns-no-results")).toBeAttached();
    await expect(guide.getByText("Browse chats", { exact: true })).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: "mash list-chats --title-contains" }),
    ).toBeVisible();

    await guide.locator('a[href="#qdrant-connection-failed"]').click();
    await expect(page).toHaveURL(`${trouble}#qdrant-connection-failed`);
    await expect(guide.getByText("Save configuration only", { exact: true })).toBeVisible();
    const systemd = `/${locale}/local-search/#linux-without-a-systemd-user-session`;
    await guide.locator(`a[href="${systemd}"]`).click();
    await expect(page).toHaveURL(systemd);
  });
}

for (const [locale, lang, dashboardTitle] of [
  ["es", "es", "Usa el panel web de Mash"],
  ["fr", "fr", "Utiliser le tableau de bord de Mash"],
  ["zh-cn", "zh-CN", "使用 Mash 网页面板"],
]) {
  test(`the ${locale} dashboard guide keeps the language and opens the requested sections`, async ({
    page,
  }) => {
    const guide = `/${locale}/dashboard/`;
    const openSection = async (from: string, section: string) => {
      await page.goto(`/${locale}/${from}/`);
      const destination = `${guide}#${section}`;
      await page.locator(`.sl-markdown-content a[href="${destination}"]`).click();
      await expect(page).toHaveURL(destination);
      await expect(page.locator("html")).toHaveAttribute("lang", lang);
      await expect(page.getByRole("heading", { level: 1 })).toHaveText(dashboardTitle);
      await expect(page.locator(`#${section}`)).toBeAttached();
    };

    await openSection("quick-start", "find-and-read-a-chat");
    await expect(page.getByText("Search your history", { exact: true })).toBeVisible();
    await expect(page.getByText("Clear filters", { exact: true })).toBeVisible();
    await openSection("search-languages", "choose-a-search-mode");
    await expect(page.getByText("search: default", { exact: true })).toBeVisible();
    await openSection("privacy", "open-it-from-your-phone-or-another-computer");
    await expect(page.locator("pre").filter({ hasText: /^mash dash remote$/ })).toBeVisible();
  });
}

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

    await page.locator(`nav.sidebar a[href="/${locale.path}/upgrades/"]`).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(
      "Back up, restore, and upgrade",
    );
    await expect(page.getByText(locale.fallback, { exact: true })).toBeVisible();
  });
}
