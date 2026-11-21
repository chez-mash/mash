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
    "Ask your agent",
    "Install it yourself",
    "Find your first chat",
  ]);
  await expect(guide.locator("#try-it")).toBeAttached();
  await guide.getByRole("link", { name: "what setup changes", exact: true }).click();
  await expect(page).toHaveURL("/installation/#what-setup-changes");
  await expect(guide.getByRole("link", { name: "Preview removal", exact: true })).toBeVisible();
  await page.goBack();
  await guide.getByRole("link", { name: "package install commands" }).click();
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

test("internal links and browser history keep the current document alive", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  const origin = await page.evaluate(() => performance.timeOrigin);
  await page.locator(".hero").getByRole("link", { name: "Get started", exact: true }).click();
  await expect(page).toHaveURL(/\/quick-start\/$/);
  await expect(page.locator("h1")).toHaveText("Quick start");
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(origin);
  await page.locator('nav.sidebar a[href="/installation/"]').click();
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
  await page.locator('nav.sidebar a[href="/installation/"]').click();
  await expect(page.locator("h1")).toHaveText("Install Mash");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  // The button in the menu switches to the other theme, and the choice holds.
  const before = await page.locator("html").getAttribute("data-theme");
  const other = before === "light" ? "dark" : "light";
  await page.locator(".mash-theme:visible").click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", other);
  await page.locator('nav.sidebar a[href="/synchronization/"]').click();
  await expect(page.locator("h1")).toHaveText("Keep history up to date");
  await expect(page.locator("html")).toHaveAttribute("data-theme", other);
});

test("direct links remain usable without JavaScript", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:8793/installation/");
  await expect(page.locator("h1")).toHaveText("Install Mash");
  await page.locator('nav.sidebar a[href="/synchronization/"]').click();
  await expect(page.locator("h1")).toHaveText("Keep history up to date");
  await context.close();
});
