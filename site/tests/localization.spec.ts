import { expect, test } from "@playwright/test";

test("the Spanish quickstart links to privacy and can search the Spanish guides", async ({
  page,
}) => {
  const guide = page.locator(".sl-markdown-content");
  await page.goto("/privacy/");
  await expect(guide.getByRole("link", { name: "Español", exact: true })).toHaveCount(0);
  await page.goto("/quick-start/");
  await page.getByRole("combobox", { name: "Select language" }).selectOption({ label: "Español" });
  await expect(page).toHaveURL(/\/es\/quick-start\/$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "es");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Guía de inicio rápido");

  await page
    .locator(".sl-markdown-content")
    .getByRole("link", { name: "guía de prueba local", exact: true })
    .click();
  await expect(page).toHaveURL("/es/privacy/#prueba-con-menos-historial");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Privacidad del historial");
  await page.getByRole("button", { name: "Buscar", exact: true }).click();
  await page.getByPlaceholder("Buscar", { exact: true }).fill("credenciales");
  await expect(page.locator('dialog a[href^="/es/privacy/"]').first()).toBeVisible();
  await page.keyboard.press("Escape");

  await expect(guide.getByRole("link", { name: "Privacy in English", exact: true })).toHaveCount(0);
  await page.locator("header starlight-lang-select select").selectOption({ label: "English" });
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

for (const [locale, language, label, docsLabel, title] of [
  ["es", "es", "Español", "Documentación", "Documentación de Mash"],
  ["fr", "fr", "Français", "Documentation", "Documentation de Mash"],
  ["zh-cn", "zh-CN", "简体中文", "文档", "Mash 文档"],
]) {
  test(`the ${locale} guide directory keeps the selected language`, async ({ page }) => {
    await page.goto("/guides/");
    await page.getByRole("combobox", { name: "Select language" }).selectOption({ label });
    const directory = `/${locale}/guides/`;
    await expect(page).toHaveURL(directory);
    await expect(page.locator("html")).toHaveAttribute("lang", language);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    const guide = page.locator(".sl-markdown-content");
    await expect(guide.locator("#find-past-work")).toBeAttached();
    await expect(guide.locator('a[href="/services/"]')).toBeVisible();
    const memory = `/${locale}/records/`;
    await guide.locator(`a[href="${memory}"]`).click();
    await expect(page).toHaveURL(memory);
    await page.getByRole("link", { name: docsLabel, exact: true }).click();
    await expect(page).toHaveURL(directory);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);

    await page.goto(`/${locale}/`);
    await guide.locator(`a[href="${directory}"]`).click();
    await expect(page).toHaveURL(directory);
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Actualiza Mash"],
  ["fr", "fr", "Mettre Mash à niveau"],
  ["zh-cn", "zh-CN", "升级 Mash"],
]) {
  test(`the ${locale} upgrade guide keeps the install and recovery steps in the same language`, async ({
    page,
  }) => {
    await page.goto(`/${locale}/guides/`);
    const upgrades = `/${locale}/upgrades/`;
    const guide = page.locator(".sl-markdown-content");
    await guide.locator(`a[href="${upgrades}"]`).click();
    await expect(page).toHaveURL(upgrades);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(guide.getByText("staged.sqlite3.attachments", { exact: true })).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: 'replacement.sqlite3" search' }),
    ).toBeVisible();
    const backup = `/${locale}/backup/#restore-without-replacing-your-current-history`;
    await guide.locator(`a[href="${backup}"]`).last().click();
    await expect(page).toHaveURL(backup);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator("#restore-without-replacing-your-current-history")).toBeAttached();
  });
}

for (const [locale, label, lang, title] of [
  ["es", "Español", "es", "Usa la memoria de Mash"],
  ["fr", "Français", "fr", "Utiliser la mémoire de Mash"],
  ["zh-cn", "简体中文", "zh-CN", "使用 Mash 记忆"],
]) {
  test(`the ${locale} memory guide explains saving and backfill in the same language`, async ({
    page,
  }) => {
    await page.goto("/records/");
    await page.getByRole("combobox", { name: "Select language" }).selectOption({ label });
    const records = `/${locale}/records/`;
    await expect(page).toHaveURL(records);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    const guide = page.locator(".sl-markdown-content");
    await expect(guide.locator("#save-a-conclusion")).toBeAttached();
    await expect(guide.locator('a[href="/record-tools/"]')).toBeVisible();
    await expect(guide.locator('a[href^="https://getmash.io/demo/#/record/"]')).toBeVisible();

    await page.goto(`/${locale}/privacy/`);
    await guide.locator(`a[href="${records}#backfill-earlier-memory"]`).click();
    await expect(page).toHaveURL(`${records}#backfill-earlier-memory`);
    await expect(
      guide
        .locator("pre")
        .filter({ hasText: "mash init --yes --no-index --no-agents --backfill-memory codex" }),
    ).toBeVisible();
  });
}

for (const [locale, label, lang, title] of [
  ["es", "Español", "es", "¿Es Mash para ti?"],
  ["fr", "Français", "fr", "Mash est-il fait pour vous ?"],
  ["zh-cn", "简体中文", "zh-CN", "Mash 适合你吗？"],
  ["ja", "日本語", "ja", "Mash はあなたに向いていますか？"],
]) {
  test(`the ${locale} “is Mash for you” guide offers a demo and a localized setup`, async ({
    page,
  }) => {
    const compare = `/${locale}/compare/`;
    const quickStart = `/${locale}/quick-start/`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto("/compare/");
    await page.getByRole("combobox", { name: "Select language" }).selectOption({ label });
    await expect(page).toHaveURL(compare);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    // The page is about Mash alone: it lists no other tools.
    await expect(guide.getByRole("table")).toHaveCount(0);
    await expect(guide.locator('a[href^="https://getmash.io/demo/#/search?"]')).toBeVisible();
    await expect(guide.locator(`a[href="/${locale}/privacy/"]`)).toBeVisible();
    await guide.locator(`a[href="${quickStart}"]`).click();
    await expect(page).toHaveURL(quickStart);
    await page.goto(locale === "ja" ? "/ja/guides/" : `/${locale}/`);
    await guide.locator(`a[href="${compare}"]`).click();
    await expect(page).toHaveURL(compare);
  });
}

for (const [locale, lang, installationTitle] of [
  ["es", "es", "Instala Mash"],
  ["fr", "fr", "Installer Mash"],
  ["zh-cn", "zh-CN", "安装 Mash"],
]) {
  test(`the ${locale} quick start and privacy guide link to localized installation steps`, async ({
    page,
  }) => {
    const install = `/${locale}/installation/`;
    const verify = `/${locale}/downloads/`;
    const uninstall = `/${locale}/uninstall/`;
    const changes = `${install}#what-setup-changes`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${install}"]`).click();
    await expect(page).toHaveURL(install);
    await expect(guide.locator("#set-up-in-a-terminal")).toBeAttached();
    await guide.locator(`a[href="/${locale}/quick-start/#install-it-yourself"]`).click();
    await expect(page).toHaveURL(`/${locale}/quick-start/#install-it-yourself`);
    await guide.locator(`a[href="${changes}"]`).click();
    await expect(page).toHaveURL(changes);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(installationTitle);
    await expect(guide.locator("#what-setup-changes")).toBeAttached();
    await expect(guide.locator('a[href="#skip-agent-connections"]')).toBeVisible();
    await expect(guide.locator(`a[href="${uninstall}"]`).first()).toBeVisible();
    await expect(guide.locator("pre").filter({ hasText: /^mash qdrant install$/ })).toBeVisible();
    await expect(
      guide.getByText("Set up and index without connecting agents", { exact: true }),
    ).toBeVisible();
    await guide.locator(`a[href="${verify}"]`).first().click();
    await expect(page).toHaveURL(verify);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(guide.locator("#verify-the-checksum")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "sha256sum mash-x86_64-unknown-linux-gnu.tar.gz" }),
    ).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: ".\\mash.exe qdrant install" }),
    ).toBeVisible();
    const browserSetup = `/${locale}/dashboard/#set-up-mash-from-the-dashboard`;
    await guide.locator(`a[href="${browserSetup}"]`).click();
    await expect(page).toHaveURL(browserSetup);
    await expect(guide.getByText("no agents", { exact: true })).toBeVisible();

    await page.goto(`/${locale}/privacy/`);
    await guide.locator(`a[href="${uninstall}"]`).last().click();
    await expect(page).toHaveURL(uninstall);
    await expect(page.locator("#remove-mash-and-its-local-copy")).toBeAttached();
    await expect(guide.locator("pre").filter({ hasText: /^mash uninstall --yes$/ })).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: /^mash uninstall --yes --keep-history$/ }),
    ).toBeVisible();
    await expect(guide.locator("#reopen-the-retained-history")).toBeAttached();
    const backup = `/${locale}/backup/#make-a-portable-backup`;
    await guide.locator(`a[href="${backup}"]`).click();
    await expect(page).toHaveURL(backup);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(guide.locator("#make-a-portable-backup")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "list-chats --local --limit 5" }),
    ).toBeVisible();
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Busca conversaciones con Mash"],
  ["fr", "fr", "Rechercher des conversations avec Mash"],
  ["zh-cn", "zh-CN", "用 Mash 搜索对话"],
  ["ja", "ja", "Mash でチャットを検索する"],
]) {
  test(`the ${locale} first terminal search leads to its original message`, async ({ page }) => {
    const search = `/${locale}/search/#your-first-search`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${search}"]`).click();
    await expect(page).toHaveURL(search);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(guide.locator("#your-first-search")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "mash search 'authentication token' --limit 3" }),
    ).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: /^mash view MESSAGE_ID --context 3$/ }),
    ).toBeVisible();
    const example = guide.locator("details").filter({ hasText: '"next_offset": null' });
    await expect(example.locator("pre")).toBeHidden();
    await example.locator("summary").click();
    await expect(example.locator("pre")).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: "--intent related-discussion" }),
    ).toBeVisible();
    const troubleshooting = `/${locale}/troubleshooting/`;
    await guide.locator(`a[href="${troubleshooting}"]`).click();
    await expect(page).toHaveURL(troubleshooting);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator("#search-returns-no-results")).toBeAttached();
  });
}

for (const [locale, lang] of [
  ["", "en"],
  ["/es", "es"],
  ["/fr", "fr"],
  ["/zh-cn", "zh-CN"],
  ["/ja", "ja"],
]) {
  test(`the ${lang} uninstall guide preserves the keep-history flag and pins reinstall`, async ({
    page,
  }) => {
    const guide = page.locator(".sl-markdown-content");
    const uninstall = `${locale}/uninstall/`;
    await page.goto(`${locale}/privacy/#try-a-small-local-setup`);
    await guide.locator(`a[href="${uninstall}"]`).last().click();
    await expect(page).toHaveURL(uninstall);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(guide.locator("#keep-history-for-a-reinstall")).toBeAttached();
    await expect(
      guide
        .locator("p")
        .filter({ has: page.getByText("v0.1.1-alpha", { exact: true }) })
        .filter({ has: page.getByText("--keep-history", { exact: true }) }),
    ).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: /^mash uninstall --yes --keep-history$/ }),
    ).toBeVisible();
    await expect(guide.getByText("sudo apt remove mash-cli", { exact: true })).toBeVisible();
    await expect(guide.getByText("mash --version", { exact: true })).toBeVisible();
    await expect(guide.getByText("MASH_VERSION", { exact: true })).toBeVisible();
    await expect(guide.locator('a[href="https://github.com/mashhq/mash/releases"]')).toBeVisible();
    await guide.locator(`a[href="${locale}/downloads/#verify-the-checksum"]`).click();
    await expect(page).toHaveURL(`${locale}/downloads/#verify-the-checksum`);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Otras formas de instalar Mash"],
  ["fr", "fr", "Autres façons d’installer Mash"],
  ["zh-cn", "zh-CN", "Mash 的其他安装方式"],
  ["ja", "ja", "Mash のほかのインストール方法"],
]) {
  test(`the ${locale} package install journey stays in the same language`, async ({ page }) => {
    const packages = `/${locale}/install-methods/#install-with-a-package-manager`;
    const install = `/${locale}/installation/`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${install}"]`).click();
    await expect(page).toHaveURL(install);
    await guide.locator(`a[href="${packages}"]`).click();
    await expect(page).toHaveURL(packages);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(guide.locator("#install-with-a-package-manager")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: /^brew install mashhq\/tap\/mash$/ }),
    ).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: "sudo apt install mash-cli" }),
    ).toBeVisible();
    await expect(guide).toContainText("Debian 12");
    await expect(guide).toContainText("Windows x64");
    const verify = `/${locale}/downloads/`;
    await guide.locator(`a[href="${verify}"]`).click();
    await expect(page).toHaveURL(verify);
    await expect(guide.locator(`a[href="/${locale}/install-methods/"]`)).toBeVisible();
    await page.goBack();
    const localSearch = `${install}#start-the-local-search-service`;
    await guide.locator(`a[href="${localSearch}"]`).first().click();
    await expect(page).toHaveURL(localSearch);
    await expect(guide.locator("pre").filter({ hasText: /^mash qdrant install$/ })).toBeVisible();
    await page.goBack();
    const browserSetup = `${install}#set-up-in-your-browser`;
    await guide.locator(`a[href="${browserSetup}"]`).click();
    await expect(page).toHaveURL(browserSetup);
    await expect(guide.locator("#set-up-in-your-browser")).toBeAttached();
    await expect(guide.locator("pre").filter({ hasText: /^mash dash$/ })).toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Servicio de búsqueda local (Qdrant)"],
  ["fr", "fr", "Service de recherche local (Qdrant)"],
  ["zh-cn", "zh-CN", "本地搜索服务（Qdrant）"],
  ["ja", "ja", "ローカル検索サービス（Qdrant）"],
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
      `/${locale}/plugin/#github-copilot-cli`,
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

for (const [locale, lang, pluginLabel] of [
  ["es", "es", "Complemento de Mash"],
  ["fr", "fr", "Module Mash"],
  ["zh-cn", "zh-CN", "Mash 插件"],
]) {
  test(`the ${locale} plugin setup and privacy links stay in the reader's language`, async ({
    page,
  }) => {
    const plugin = `/${locale}/plugin/`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/quick-start/`);
    await guide.locator(`a[href="${plugin}#start-with-the-plugin"]`).click();
    await expect(page).toHaveURL(`${plugin}#start-with-the-plugin`);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator('nav.sidebar [aria-current="page"]')).toHaveText(pluginLabel);
    await expect(page.locator("#start-with-the-plugin")).toBeAttached();
    await expect(
      guide.locator("pre").filter({ hasText: "codex plugin add mash@mash" }),
    ).toBeVisible();

    await page.goto(`/${locale}/agents/`);
    await guide.locator(`a[href="${plugin}#finish-setup-in-the-agent"]`).click();
    await expect(page).toHaveURL(`${plugin}#finish-setup-in-the-agent`);
    await expect(page.locator("#finish-setup-in-the-agent")).toBeAttached();

    const privacy = `/${locale}/privacy/#when-history-can-leave-your-machine`;
    await guide.locator(`a[href="${privacy}"]`).click();
    await expect(page).toHaveURL(privacy);
    await expect(page.locator("#when-history-can-leave-your-machine")).toBeAttached();
    await guide.locator(`a[href="${plugin}#what-happens-without-asking"]`).click();
    await expect(page).toHaveURL(`${plugin}#what-happens-without-asking`);
    await expect(page.locator("#what-happens-without-asking")).toBeAttached();
    await expect(guide.getByText("MASH_RECORD_RECALL=off", { exact: true })).toBeVisible();
    await expect(
      guide.locator("pre").filter({ hasText: /^copilot mcp add mash -- mash mcp$/ }),
    ).toBeVisible();
  });
}

for (const [locale, lang, sidebarLabel] of [
  ["es", "es", "Configurar un índice compartido"],
  ["fr", "fr", "Configurer un index partagé"],
  ["zh-cn", "zh-CN", "设置共享索引"],
]) {
  test(`the ${locale} privacy guide leads to shared-index setup and revocation`, async ({
    page,
  }) => {
    const guide = page.locator(".sl-markdown-content");
    const shared = `/${locale}/shared-index/`;
    await page.goto(`/${locale}/privacy/`);
    await guide.locator(`a[href="${shared}#before-sharing"]`).click();
    await expect(page).toHaveURL(`${shared}#before-sharing`);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator('nav.sidebar [aria-current="page"]')).toHaveText(sidebarLabel);
    await expect(guide.locator("#before-sharing")).toBeAttached();
    await guide.locator('a[href="#the-quick-way"]:not(.sl-anchor-link)').click();
    await expect(page).toHaveURL(`${shared}#the-quick-way`);
    await expect(
      guide
        .locator("pre")
        .filter({ hasText: "mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents" }),
    ).toBeVisible();
    const revoke = "#revoke-or-replace-a-computers-credential";
    await guide.locator(`a[href="${revoke}"]:not(.sl-anchor-link)`).click();
    await expect(page).toHaveURL(`${shared}${revoke}`);
    await expect(guide.locator(revoke)).toBeAttached();
    await expect(guide.getByText("server.toml.nodes", { exact: true }).last()).toBeVisible();
  });
}

for (const [locale, lang, title, help] of [
  ["es", "es", "Solución de problemas", "Pide ayuda"],
  ["fr", "fr", "Dépannage", "Obtenir de l’aide"],
  ["zh-cn", "zh-CN", "故障排查", "获取帮助"],
  ["ja", "ja", "トラブルシューティング", "サポートを受ける"],
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
      guide.locator('a[href="https://github.com/mashhq/mash/security/advisories/new"]'),
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
    await expect(page.locator("#linux-without-a-systemd-user-session")).toBeAttached();
  });
}

for (const [locale, lang] of [
  ["", "en"],
  ["/es", "es"],
  ["/fr", "fr"],
  ["/zh-cn", "zh-CN"],
  ["/ja", "ja"],
]) {
  test(`the ${lang} dashboard setup guide leads to agent removal commands`, async ({ page }) => {
    const guide = page.locator(".sl-markdown-content");
    const privacy = `${locale}/privacy/#choose-agent-access`;
    const remove = `${locale}/agents/#disconnect-an-agent`;
    await page.goto(`${locale}/dashboard/#set-up-mash-from-the-dashboard`);
    await guide.locator(`a[href="${privacy}"]`).click();
    await expect(page).toHaveURL(privacy);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(guide.locator("#choose-agent-access")).toBeAttached();
    await expect(guide.getByRole("table")).toContainText("MCP");
    await guide.locator(`a[href="${remove}"]`).click();
    await expect(page).toHaveURL(remove);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(guide.locator("#disconnect-an-agent")).toBeAttached();
    await expect(guide.locator("pre").filter({ hasText: "codex mcp remove mash" })).toContainText(
      "codex plugin remove mash@mash",
    );
  });

  test(`the ${lang} history range guidance leads to deliberate removal`, async ({ page }) => {
    const guide = page.locator(".sl-markdown-content");
    const privacy = `${locale}/privacy/`;
    const range = locale ? `${privacy}#shorten-the-history-range` : "/dashboard-settings/";
    const rangeSection = locale ? "#shorten-the-history-range" : "#change-the-history-range";
    const removeLink = locale ? "#remove-a-chat" : `${privacy}#remove-a-chat`;
    await page.goto(`${locale}/dashboard/#the-rail`);
    await guide.locator(`a[href="${range}"]`).click();
    await expect(page).toHaveURL(range);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator(rangeSection)).toBeAttached();
    await expect(guide.getByText("Save to file", { exact: true })).toBeVisible();
    await guide
      .locator(`:is(.sl-heading-wrapper, p):has(${rangeSection}) ~ p a[href="${removeLink}"]`)
      .click();
    await expect(page).toHaveURL(`${privacy}#remove-a-chat`);
    await expect(page.locator("#remove-a-chat")).toBeAttached();
    await expect(guide.locator("pre").filter({ hasText: /^mash forget CHAT_ID$/ })).toBeVisible();
  });
}

for (const [locale, lang, title] of [
  ["es", "es", "Incluye conversaciones antiguas y busca por significado"],
  ["fr", "fr", "Inclure les conversations plus anciennes et chercher par le sens"],
  ["zh-cn", "zh-CN", "纳入较早的对话并按含义搜索"],
]) {
  test(`the ${locale} search setup explains models and older history in the same language`, async ({
    page,
  }) => {
    const indexing = `/${locale}/indexing/`;
    const meaning = `${indexing}#enable-semantic-search`;
    const older = `${indexing}#add-older-history`;
    const guide = page.locator(".sl-markdown-content");
    await page.goto(`/${locale}/dashboard/#choose-a-search-mode`);
    await guide.locator(`a[href="${meaning}"]`).first().click();
    await expect(page).toHaveURL(meaning);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(title);
    await expect(guide.locator("pre").filter({ hasText: /^mash models install$/ })).toBeVisible();
    await expect(guide.getByText("realized_mode", { exact: true })).toBeVisible();
    await expect(guide.getByText("hybrid", { exact: true })).toBeVisible();
    await guide.locator(`a[href="/${locale}/search-languages/"]`).click();
    await expect(page).toHaveURL(`/${locale}/search-languages/`);

    await page.goto(`/${locale}/installation/#try-your-first-search`);
    await guide.locator(`a[href="${older}"]`).click();
    await expect(page).toHaveURL(older);
    await expect(
      guide.locator("pre").filter({ hasText: /^mash index --all-history$/ }),
    ).toBeVisible();
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
    const resume = `/${locale}/pick-up-work/#reopen-the-original-session`;
    await page.locator(`.sl-markdown-content a[href="${resume}"]`).click();
    await expect(page).toHaveURL(resume);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator("#reopen-the-original-session")).toBeAttached();
    await expect(page.locator("pre").filter({ hasText: "codex resume SESSION_ID" })).toBeVisible();
    await expect(
      page.locator(`.sl-markdown-content a[href="/${locale}/plugin/#finish-setup-in-the-agent"]`),
    ).toBeVisible();
    await openSection("search-languages", "choose-a-search-mode");
    await expect(page.getByText("search: default", { exact: true })).toBeVisible();
    const refresh = `/${locale}/synchronization/#refresh-now`;
    await page.locator(`.sl-markdown-content a[href="${refresh}"]`).click();
    await expect(page).toHaveURL(refresh);
    await expect(page.locator("html")).toHaveAttribute("lang", lang);
    await expect(page.locator("#refresh-now")).toBeAttached();
    await expect(
      page.locator("pre").filter({ hasText: "mash index" }).filter({ hasText: "mash status" }),
    ).toBeVisible();
    await expect(page.locator("pre").filter({ hasText: /^mash index --watch$/ })).toBeVisible();
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
    search: "Rechercher",
    query: "identifiants secrets",
    dashboard: "Tableau de bord",
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
    search: "搜索",
    query: "敏感信息",
    dashboard: "网页面板",
    fallback: "此内容尚不支持你的语言。",
  },
  {
    path: "ja",
    label: "日本語",
    lang: "ja",
    quickStart: "クイックスタート",
    trialLink: "1 つの履歴で試す",
    trialHeading: "1 つの履歴で試す",
    privacy: "履歴とプライバシー",
    docs: "ドキュメント",
    search: "検索",
    query: "プライバシー",
    dashboard: "ダッシュボード",
    fallback: "このコンテンツはまだ日本語訳がありません。",
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
    await expect(page.locator("pre").filter({ hasText: "sh -s -- --dashboard" })).toBeVisible();
    await expect(page.locator('pre[data-language="powershell"]')).toContainText(/ -Dashboard$/);

    await page.getByRole("link", { name: locale.trialLink }).click();
    await expect(page).toHaveURL(
      (url) => url.pathname === `/${locale.path}/privacy/` && !!url.hash,
    );
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(locale.privacy);
    await expect(page.getByRole("heading", { name: locale.trialHeading })).toBeVisible();
    await expect(page.getByText("another number of days", { exact: true })).toBeVisible();
    await expect(page.getByText("no agents", { exact: true })).toBeVisible();
    await expect(
      page.locator(`a[href="/${locale.path}/quick-start/#install-it-yourself"]`),
    ).toBeVisible();

    await page.getByRole("button", { name: locale.search, exact: true }).click();
    await page.getByPlaceholder(locale.search, { exact: true }).fill(locale.query);
    await expect(page.locator(`dialog a[href^="/${locale.path}/privacy/"]`).first()).toBeVisible();
    await page.keyboard.press("Escape");

    const sidebar = page.locator("nav.sidebar");
    await expect(
      sidebar
        .locator(".top-level > li > details > summary")
        .getByText(locale.dashboard, { exact: true }),
    ).toBeVisible();
    await sidebar.locator(`a[href="/${locale.path}/services/"]`).click();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(
      "Inspect and manage Mash services",
    );
    await expect(page.getByText(locale.fallback, { exact: true })).toBeVisible();
  });
}

test("the Japanese guide directory distinguishes available translations", async ({ page }) => {
  const guide = page.locator(".sl-markdown-content");
  await page.goto("/ja/");
  await guide.getByRole("link", { name: "日本語ガイドの目次" }).click();
  await expect(page).toHaveURL("/ja/guides/");
  await page.getByRole("button", { name: "検索", exact: true }).click();
  await page.getByPlaceholder("検索", { exact: true }).fill("プライバシー");
  await expect(page.getByText(/「プライバシー」の検索結果 \d+ 件/)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(guide.locator('a[href="/ja/installation/"]')).toHaveText(
    "Mash をインストールして読み込む履歴を選ぶ",
  );
  await expect(guide.locator('a[href="/ja/install-methods/"]')).toHaveText(
    "パッケージマネージャーでインストールする、またはソースからビルドする",
  );
  await guide
    .getByRole("link", { name: "Mash が読み取れる履歴と接続できるエージェントを確認する" })
    .click();
  await expect(page).toHaveURL("/ja/supported-agents/");
  await guide.getByRole("link", { name: "エージェントを接続", exact: true }).click();
  await expect(page).toHaveURL("/ja/agents/#connect-claude-code-codex-or-copilot");
  await expect(page.locator("#connect-claude-code-codex-or-copilot")).toBeAttached();
  await expect(guide.getByText("Connect an agent", { exact: true })).toBeVisible();
  await expect(guide.getByText("Set up without indexing", { exact: true }).first()).toBeVisible();
});

test("Japanese readers can go from a verified archive to browser setup in Japanese", async ({
  page,
}) => {
  const guide = page.locator(".sl-markdown-content");
  await page.goto("/ja/quick-start/");
  const install = "/ja/installation/";
  const changes = `${install}#what-setup-changes`;
  await guide.locator(`a[href="${changes}"]`).click();
  await expect(page).toHaveURL(changes);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Mash をインストールする");
  await expect(page.locator("#what-setup-changes")).toBeAttached();

  await guide.locator('a[href="/ja/downloads/"]').first().click();
  await expect(page).toHaveURL("/ja/downloads/");
  await expect(page.locator("#verify-the-checksum")).toBeAttached();
  await expect(
    guide.locator("pre").filter({ hasText: "Get-FileHash -Algorithm SHA256" }),
  ).toBeVisible();

  const installers = `${install}#recommended-release-installers`;
  await guide.locator(`a[href="${installers}"]`).click();
  await expect(page).toHaveURL(installers);
  await expect(page.locator("#recommended-release-installers")).toBeAttached();
  await expect(guide.getByText("glibc 2.34", { exact: false })).toBeVisible();

  await page.goto(`${install}#set-up-in-your-browser`);
  await expect(guide.locator("pre").filter({ hasText: /^mash dash$/ })).toBeVisible();
  const browserSetup = "/ja/dashboard/#set-up-mash-from-the-dashboard";
  await guide.locator(`a[href="${browserSetup}"]`).click();
  await expect(page).toHaveURL(browserSetup);
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(page.locator("#set-up-mash-from-the-dashboard")).toBeAttached();

  await page.goto(`${install}#try-your-first-search`);
  const firstChat = "/ja/dashboard/#find-and-read-a-chat";
  await guide.locator(`a[href="${firstChat}"]`).click();
  await expect(page).toHaveURL(firstChat);
  await expect(guide.getByText("Search your history", { exact: true })).toBeVisible();
});

test("the Japanese dashboard guide keeps setup, search, refresh and private access in Japanese", async ({
  page,
}) => {
  const guide = page.locator(".sl-markdown-content");
  const openSection = async (from: string, section: string) => {
    await page.goto(`/ja/${from}/`);
    const destination = `/ja/dashboard/#${section}`;
    await guide.locator(`a[href="${destination}"]`).click();
    await expect(page).toHaveURL(destination);
    await expect(page.locator("html")).toHaveAttribute("lang", "ja");
    await expect(page.locator(`#${section}`)).toBeAttached();
  };

  await openSection("quick-start", "set-up-mash-from-the-dashboard");
  await expect(guide.getByText("Choose agents to connect", { exact: true })).toBeVisible();
  await guide.getByRole("link", { name: "履歴を読み込み直さずにエージェントを接続" }).click();
  await expect(page).toHaveURL("/ja/agents/#connect-claude-code-codex-or-copilot");
  await openSection("search-languages", "choose-a-search-mode");
  await expect(guide.getByText("search: default", { exact: true })).toBeVisible();
  const refresh = "/ja/synchronization/#refresh-now";
  await guide.locator(`a[href="${refresh}"]`).click();
  await expect(page).toHaveURL(refresh);
  await expect(page.locator("#refresh-now")).toBeAttached();
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(
    guide.locator("pre").filter({ hasText: "mash index" }).filter({ hasText: "mash status" }),
  ).toBeVisible();
  await expect(guide.locator("pre").filter({ hasText: /^mash index --watch$/ })).toBeVisible();

  await page.goto("/ja/troubleshooting/#new-chats-are-missing-or-search-is-behind");
  const published = "/ja/synchronization/#across-machines";
  await guide.locator(`a[href="${published}"]`).click();
  await expect(page).toHaveURL(published);
  await expect(page.locator("#across-machines")).toBeAttached();
  await openSection("privacy", "open-it-from-your-phone-or-another-computer");
  await expect(guide.locator("pre").filter({ hasText: /^mash dash remote$/ })).toBeVisible();
});

for (const [locale, menu, demo] of [
  ["es", "Menú", "Prueba la demo"],
  ["fr", "Menu", "Essayer la démo"],
] as const) {
  test(`on a phone the ${locale} menu's links wrap inside it`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`/${locale}/installation/`);
    await page.getByRole("button", { name: menu, exact: true }).click();
    const links = page.locator("nav.sidebar .mash-menu-top a.mash-nav");
    await expect(links.filter({ hasText: demo })).toBeInViewport({ ratio: 1 });
    for (const link of await links.all()) {
      const box = await link.boundingBox();
      expect(box && box.x + box.width).toBeLessThanOrEqual(390);
    }
  });
}

test("the language picker keeps a heading only where the new page has it", async ({ page }) => {
  const picker = page.getByRole("combobox", { name: "Seleccionar idioma" });
  // The Spanish heading's own id is not on the French page, but the English
  // anchor kept beside it is.
  await page.goto("/es/installation/#qué-cambia-al-configurar-mash");
  await picker.selectOption({ label: "Français" });
  await expect(page).toHaveURL("/fr/installation/#what-setup-changes");
  await expect(page.locator("#what-setup-changes")).toBeAttached();

  // A hash the French page does not have is dropped, so the page opens at its
  // top without a stray anchor.
  await page.goto("/es/installation/#no-such-heading");
  await picker.selectOption({ label: "Français" });
  await expect(page).toHaveURL((url) => url.pathname === "/fr/installation/" && url.hash === "");
});

test("search on a translated site finds only pages in its language", async ({ page }) => {
  // The tools reference is not translated into Japanese, so /ja/tools/ shows
  // the English text and must not be in the Japanese search.
  await page.goto("/ja/guides/");
  await expect(page.locator("main")).toHaveAttribute("lang", "ja");
  await page.goto("/ja/tools/");
  await expect(page.locator("main")).toHaveAttribute("lang", "en");
  await expect(page.locator("main[data-pagefind-body]")).toHaveCount(0);
  await page.goto("/ja/search/");
  await expect(page.locator("main[data-pagefind-body]")).toHaveAttribute("lang", "ja");

  await page.getByRole("button", { name: "検索", exact: true }).click();
  await page.getByRole("dialog").getByRole("textbox").fill("mash");
  // Wait for the results, then read every page they lead to.
  await expect(page.locator('dialog a[href^="/ja/"]').first()).toBeVisible();
  const hrefs = await page
    .locator('dialog a[href^="/"]')
    .evaluateAll((links) => links.map((link) => link.getAttribute("href") ?? ""));
  expect(hrefs.filter((href) => !href.startsWith("/ja/"))).toEqual([]);
  expect(hrefs.filter((href) => href.startsWith("/ja/tools/"))).toEqual([]);
});

test("the footer and the missing page lead to the reader's language", async ({ page }) => {
  await page.goto("/fr/guides/");
  const footer = page.getByRole("navigation", { name: "Pied de page" });
  await expect(footer.getByRole("link", { name: "Documentation" })).toHaveAttribute(
    "href",
    "/fr/guides/",
  );
  await expect(footer.getByRole("link", { name: "Confidentialité" })).toHaveAttribute(
    "href",
    "/fr/privacy/",
  );
  await expect(footer.getByRole("link", { name: "Obtenir de l’aide" })).toHaveAttribute(
    "href",
    "/fr/troubleshooting/",
  );

  await page.goto("/ja/no-such-page/");
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("ページが見つかりません");
  const actions = page.locator(".mash-missing");
  await expect(actions.getByRole("link", { name: "ホーム" })).toHaveAttribute("href", "/ja/");
  await expect(actions.getByRole("link", { name: "ドキュメント" })).toHaveAttribute(
    "href",
    "/ja/guides/",
  );
});
