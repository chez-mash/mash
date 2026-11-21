import { expect, test } from "@playwright/test";

test("the home page leads to troubleshooting and the right GitHub report forms", async ({
  page,
}) => {
  await page.goto("/");
  await page
    .getByRole("navigation", { name: "Footer" })
    .getByRole("link", { name: "Get help" })
    .click();
  await expect(page).toHaveURL(/\/troubleshooting\/$/);
  const guide = page.locator(".sl-markdown-content");
  await guide.getByRole("link", { name: "Get help", exact: true }).click();
  await expect(page).toHaveURL(/\/troubleshooting\/#get-help$/);
  await expect(guide.getByRole("link", { name: "Open a bug report" })).toHaveAttribute(
    "href",
    "https://github.com/mashium/mash/issues/new?template=bug_report.yml",
  );
  await expect(
    guide.getByRole("link", { name: "GitHub’s private reporting form" }),
  ).toHaveAttribute("href", "https://github.com/mashium/mash/security/advisories/new");
});

test("visitors can check which histories Mash reads and which agents it connects", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Check support for your agent" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Supported agents");
  const support = page.getByRole("table").first();
  const copilot = support.getByRole("row", { name: /GitHub Copilot CLI/ });
  await expect(copilot).toContainText("Development builds");
  await expect(support.getByRole("row", { name: /GitHub Copilot Chat in VS Code/ })).toContainText(
    "Chats saved by VS Code on your computer",
  );
  await page.getByRole("link", { name: "connect an agent", exact: true }).click();
  await expect(page).toHaveURL(/\/agents\/#connect-claude-code-codex-or-copilot$/);
  await expect(page.locator("#connect-claude-code-codex-or-copilot")).toBeAttached();
  await expect(page.getByRole("heading", { name: "Connect an agent after setup" })).toBeVisible();
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.getByRole("heading", { name: "In the browser" })).toBeVisible();
  await expect(guide.getByText("Connect an agent", { exact: true })).toBeVisible();
  await expect(guide.getByText("Set up without indexing", { exact: true }).first()).toBeVisible();
  await expect(guide.getByRole("heading", { name: "In a terminal" })).toBeVisible();
  await page.getByRole("link", { name: "connect Copilot CLI manually" }).click();
  await expect(page).toHaveURL(/\/plugin\/#github-copilot-cli$/);
  await expect(
    page.locator("pre").filter({ hasText: /^copilot mcp add mash -- mash mcp$/ }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Finish setup in the agent" })).toBeVisible();
});

test("the quick start goes from setup to a first chat and links to package installs", async ({
  page,
}) => {
  await page.goto("/quick-start/#quick-start");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator("#quick-start")).toBeAttached();
  await expect(guide.getByRole("heading", { level: 2 })).toHaveText([
    "Install it yourself",
    "Find your first chat",
    "Ask your agent",
  ]);
  await expect(guide.locator('a[href^="https://getmash.io/demo/"]')).toHaveCount(0);
  await expect(guide.locator("#try-it")).toBeAttached();
  await expect(guide.locator("pre").filter({ hasText: "sh -s -- --dashboard" })).toBeVisible();
  await expect(guide.locator('pre[data-language="powershell"]')).toContainText(/ -Dashboard$/);
  await guide.getByRole("link", { name: "what setup changes", exact: true }).click();
  await expect(page).toHaveURL("/installation/#what-setup-changes");
  await expect(guide.getByRole("link", { name: "Preview removal", exact: true })).toBeVisible();
  await page.goBack();
  await guide.getByRole("link", { name: "installation guide", exact: true }).click();
  await expect(page).toHaveURL("/installation/");
  await expect(guide.getByRole("heading", { name: "Set up in a terminal" })).toBeVisible();
  await guide.locator('a[href="/install-methods/#install-with-a-package-manager"]').click();
  await expect(page).toHaveURL("/install-methods/#install-with-a-package-manager");
  await expect(page.locator("#install-with-a-package-manager")).toBeAttached();
  await guide.getByRole("link", { name: "finish setup in the browser" }).click();
  await expect(page).toHaveURL("/installation/#set-up-in-your-browser");
  await expect(guide.getByRole("heading", { name: "Set up in your browser" })).toBeVisible();
  await expect(guide.locator("pre").filter({ hasText: /^mash dash$/ })).toBeVisible();
  await guide.getByRole("link", { name: "See the dashboard setup steps" }).click();
  await expect(page).toHaveURL("/dashboard/#set-up-mash-from-the-dashboard");
  await expect(
    guide.getByRole("heading", { name: "Set up Mash from the dashboard" }),
  ).toBeVisible();
});

test("the old restore section leads to the portable backup guide", async ({ page }) => {
  const section = "#restore-without-replacing-your-current-history";
  await page.goto(`/upgrades/${section}`);
  await expect(page.locator(section)).toBeAttached();
  await page.getByRole("link", { name: "Check a restored copy in a separate database" }).click();
  await expect(page).toHaveURL(`/backup/${section}`);
  await expect(page.locator("pre").filter({ hasText: "mash import" })).toBeVisible();
});

test("the old uninstall section leads to removal and reinstall choices", async ({ page }) => {
  await page.goto("/installation/#uninstall");
  await expect(page.locator("#uninstall")).toBeAttached();
  await page
    .getByRole("link", { name: "See what stays on your computer and how to uninstall" })
    .click();
  await expect(page).toHaveURL("/uninstall/");
  await page.getByRole("link", { name: "Keep Mash’s copy in place" }).click();
  await expect(page).toHaveURL("/uninstall/#keep-history-for-a-reinstall");
  await expect(
    page.locator("pre").filter({ hasText: /^mash uninstall --yes --keep-history$/ }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Reopen the retained history" })).toBeVisible();
});

test("the release check bookmark leads to the right archives and checksum steps", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto("/installation/#check-the-release-download");
  await expect(page.locator("#check-the-release-download")).toBeAttached();
  const guide = page.locator(".sl-markdown-content");
  await guide.locator('a[href="/downloads/"]').last().click();
  await expect(page).toHaveURL("/downloads/");
  const archives = await guide
    .locator("table tbody a")
    .evaluateAll((links) => links.map((link) => link.getAttribute("href")));
  const base = "https://github.com/mashium/mash/releases/latest/download/";
  expect(archives).toEqual(
    [
      "mash-aarch64-apple-darwin.tar.gz",
      "mash-x86_64-unknown-linux-gnu.tar.gz",
      "mash-aarch64-unknown-linux-gnu.tar.gz",
      "mash-x86_64-pc-windows-msvc.zip",
    ].map((name) => base + name),
  );
  await expect(
    guide.locator("pre").filter({ hasText: "Get-FileHash -Algorithm SHA256" }),
  ).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

test("an archive download leads from verification to browser setup", async ({ page }) => {
  await page.goto("/guides/");
  await page
    .getByRole("link", { name: "Download and verify a release archive", exact: true })
    .click();
  await expect(page).toHaveURL("/downloads/");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator("pre").filter({ hasText: "sha256sum mash-" })).toBeVisible();
  await expect(
    guide
      .locator("pre")
      .filter({ hasText: "./mash qdrant install" })
      .filter({ hasText: "./mash dash" }),
  ).toBeVisible();
  await expect(
    guide
      .locator("pre")
      .filter({ hasText: ".\\mash.exe qdrant install" })
      .filter({ hasText: ".\\mash.exe dash" }),
  ).toBeVisible();
  await guide.getByRole("link", { name: "Follow the browser setup steps" }).click();
  await expect(page).toHaveURL("/dashboard/#set-up-mash-from-the-dashboard");
  await expect(guide.getByText("no agents", { exact: true })).toBeVisible();
});

test("internal links and browser history keep the current document alive", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  const origin = await page.evaluate(() => performance.timeOrigin);
  await page.locator(".hero").getByRole("link", { name: "Get started", exact: true }).click();
  await expect(page).toHaveURL(/\/quick-start\/$/);
  await expect(page.locator("h1")).toHaveText("Quick start");
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(origin);
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Installation", { exact: true }).click();
  await sidebar.locator('a[href="/installation/"]').click();
  await expect(page.locator("h1")).toHaveText("Install Mash");
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(origin);
  await page.goBack();
  await expect(page.locator("h1")).toHaveText("Quick start");
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(origin);
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await page.getByPlaceholder("Search", { exact: true }).fill("Qdrant");
  await expect(page.locator("dialog a").first()).toBeVisible();
  await page.locator("dialog a").first().click();
  await expect(page.locator("dialog")).not.toBeVisible();
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(origin);
  await page.keyboard.press("Control+k");
  await expect(page.locator("dialog")).toBeVisible();
  expect(errors).toEqual([]);
});

test("mobile menu and theme still work after navigation", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/quick-start/");
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Installation", { exact: true }).click();
  await sidebar.locator('a[href="/installation/"]').click();
  await expect(page.locator("h1")).toHaveText("Install Mash");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  // The button in the menu switches to the other theme, and the choice holds.
  const before = await page.locator("html").getAttribute("data-theme");
  const other = before === "light" ? "dark" : "light";
  await page.locator(".mash-theme:visible").click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", other);
  await sidebar.locator("summary").getByText("Search", { exact: true }).click();
  await sidebar.locator('a[href="/synchronization/"]').click();
  await expect(page.locator("h1")).toHaveText("Keep history up to date");
  await expect(page.locator("html")).toHaveAttribute("data-theme", other);
});

test("direct links remain usable without JavaScript", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:8793/installation/");
  await expect(page.locator("h1")).toHaveText("Install Mash");
  const sidebar = page.locator("nav.sidebar");
  await sidebar.locator("summary").getByText("Search", { exact: true }).click();
  await sidebar.locator('a[href="/synchronization/"]').click();
  await expect(page.locator("h1")).toHaveText("Keep history up to date");
  await context.close();
});

test("the sidebar groups related guides under two top-level sections", async ({ page }) => {
  await page.goto("/guides/#find-past-work");
  const sidebar = page.locator("nav.sidebar");
  await expect(sidebar.locator(".top-level > li > details > summary")).toHaveText([
    "Start here",
    "Using Mash",
  ]);
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.getByRole("heading", { level: 2 })).toHaveText(["Start here", "Using Mash"]);
  await expect(guide.locator("#find-past-work")).toBeAttached();
  await expect(guide.getByRole("link", { name: "Español", exact: true })).toHaveCount(0);

  const languageSearch = sidebar.getByRole("link", {
    name: "Search in other languages",
    includeHidden: true,
  });
  await expect(languageSearch).toBeHidden();
  await sidebar.locator("summary").getByText("Search", { exact: true }).click();
  await languageSearch.click();
  await expect(page).toHaveURL("/search-languages/");
  await page.reload();
  await expect(languageSearch).toBeVisible();
  await expect(languageSearch).toHaveAttribute("aria-current", "page");
});

test("the architecture guide renders its local and shared search diagram", async ({ page }) => {
  await page.goto("/guides/");
  const guide = page.locator(".sl-markdown-content");
  await guide.locator('a[href="/architecture/"]').click();
  await expect(page).toHaveURL("/architecture/");
  const diagram = guide.getByRole("img", { name: /SQLite.*Qdrant.*Mash server/ });
  await expect(diagram).toHaveAttribute("src", "/mash-architecture.svg");
  await expect
    .poll(() => diagram.evaluate((image: HTMLImageElement) => image.naturalWidth))
    .toBeGreaterThan(0);
});
