import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";

test("the landing page leads to a working demo of the dashboard", async ({ page }) => {
  const problems: string[] = [];
  page.on("pageerror", (error) => problems.push(error.message));
  page.on("response", (response) => {
    if (response.status() >= 400) problems.push(`${response.status()} ${response.url()}`);
  });
  await page.goto("/");
  await page.locator(".hero").getByRole("link", { name: "Try the demo", exact: true }).click();
  await expect(page).toHaveURL(/\/demo\/$/);
  const demo = page.getByRole("complementary", { name: "About this demo" });
  await expect(demo).toContainText("Demo with sample chats");
  await demo.getByRole("link", { name: "Try a search" }).click();
  await expect(page.getByRole("textbox", { name: "Search your history" })).toHaveValue(
    "invoice exports queue",
  );
  await page
    .getByRole("link", { name: /Choose a queue for invoice exports/ })
    .first()
    .click();
  await expect(page.locator("#message-invoice-export-queue-0")).toHaveClass(/\bhit\b/);

  await demo.getByRole("link", { name: "See a decision" }).click();
  await expect(page.getByRole("heading", { name: /fifteen-minute access tokens/ })).toBeVisible();
  await page.getByRole("link", { name: /open the message/i }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
  expect(problems).toEqual([]);
});

test("the guide index opens a sample search that fits a narrow screen", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto("/guides/");
  const link = page.locator(".sl-markdown-content").getByRole("link", {
    name: "Try the demo",
    exact: true,
  });
  await expect(link).toHaveAttribute("href", /^https:\/\/getmash\.io\/demo\/#\/search\?q=/);
  const target = new URL((await link.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  const demo = page.getByRole("complementary", { name: "About this demo" });
  await expect(demo.getByRole("link", { name: "Try a search" })).toBeInViewport({ ratio: 1 });
  await expect(demo.getByRole("link", { name: "See a decision" })).toBeInViewport({ ratio: 1 });
  await expect(demo.getByRole("link", { name: "Install Mash" })).toBeInViewport({ ratio: 1 });
  await expect(
    page.getByRole("link", { name: /Choose a queue for invoice exports/ }).first(),
  ).toBeVisible();

  const query = page.getByRole("textbox", { name: "Search your history" });
  await query.fill("invoice exports zebra");
  await query.press("Enter");
  await expect(page.getByRole("heading", { name: "No results", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Browse chats", exact: true })).toBeVisible();
  // On a phone the refresh lives in the navigation drawer.
  await page.getByRole("button", { name: "Open navigation" }).click();
  await expect(page.getByRole("button", { name: "Check for updates", exact: true })).toBeVisible();
  await page.keyboard.press("Escape");
  const help = page.getByRole("link", { name: "Find a missing chat", exact: true });
  await expect(help).toHaveAttribute("href", "https://getmash.io/find-missing-chat/");
  await expect(page.getByRole("link", { name: "Troubleshoot search" })).toHaveCount(0);
  await page.goto(new URL((await help.getAttribute("href")) ?? "").pathname);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Find a missing chat");
  await expect(page.locator("#the-chat-is-listed-but-search-finds-nothing")).toBeVisible();
  await expect(page.locator("#the-chat-is-on-another-machine")).toBeVisible();
});

test("dashboard settings lead to instructions for changing the history range", async ({ page }) => {
  await page.goto("/demo/#/settings");
  const help = page.getByRole("link", { name: "Settings guide", exact: true });
  await expect(help).toBeVisible();
  await expect(help).toHaveAttribute("href", "https://getmash.io/dashboard-settings/");
  await page.goto(new URL((await help.getAttribute("href")) ?? "").pathname);
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator("#change-the-history-range")).toBeVisible();
  await expect(guide.locator("#save-and-apply-changes")).toBeVisible();

  await page.goto("/privacy/#shorten-the-history-range");
  await expect(guide.locator("#shorten-the-history-range")).toBeAttached();
  await guide.getByRole("link", { name: "Change the history range", exact: true }).click();
  await expect(page).toHaveURL("/dashboard-settings/#change-the-history-range");
});

test("the welcome examples open the checkout source and the cross-agent handoff", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#welcome-search");
  await page.getByRole("link", { name: /read the checkout chat in the demo/i }).click();
  await expect(page.getByRole("heading", { name: "Review of the checkout outage" })).toBeVisible();
  const checkout = page.locator("#message-checkout-outage-review-1");
  await expect(checkout).toHaveClass(/\bhit\b/);
  await expect(checkout).toContainText("Building the index locked the table for writes");
  await expect(
    page.locator("article header").getByText("Claude Code", { exact: true }),
  ).toBeVisible();

  await page.goto("/#welcome-memory");
  await page.getByRole("link", { name: /follow the sign-in decision in the demo/i }).click();
  await page.getByRole("link", { name: /open the message/i }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
  await page.getByRole("link", { name: "See the Codex follow-up", exact: true }).click();
  await expect(page.locator("#message-mobile-keychain-follow-up-1")).toHaveClass(/\bhit\b/);
});

test("the README dashboard picture opens the sign-in decision it shows", async ({ page }) => {
  const readme = await readFile(new URL("../../README.md", import.meta.url), "utf8");
  const link = /\]\(assets\/screenshots\/dashboard\.webp\)\]\(([^)]+)\)/.exec(readme)?.[1];
  if (!link) throw new Error("Missing README dashboard picture link");
  const target = new URL(link);
  expect(target.origin).toBe("https://getmash.io");
  await page.goto(target.pathname + target.hash);
  await expect(
    page.getByRole("heading", { name: "Move sign-in to short-lived tokens" }),
  ).toBeVisible();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
});

test("a copied message link reopens the matching chat inside the demo", async ({
  context,
  page,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/demo/#/chat/checkout-outage-review?node=build-box&shared=true");
  const message = page.locator("#message-checkout-outage-review-1");
  await message.hover();
  await message.getByRole("button", { name: "Copy link to message" }).click();
  const link = new URL(await page.evaluate(() => navigator.clipboard.readText()));
  expect(link.pathname).toBe("/demo/");
  await page.goto(link.href);
  await expect(message).toHaveClass(/\bhit\b/);
  await expect(message).toContainText("Building the index locked the table for writes");
});

test("the sample handoff opens the Claude Code decision and its later Codex chat", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/guides/");
  await page.getByRole("link", { name: "Pick up where you left off or leave a handoff" }).click();
  await expect(page).toHaveURL("/pick-up-work/");
  const example = page.getByRole("link", { name: "Open the saved sign-in decision in the demo" });
  await expect(example).toHaveAttribute(
    "href",
    "https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop",
  );
  const target = new URL((await example.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  const demo = page.getByRole("complementary", { name: "About this demo" });
  await expect(demo).toContainText("Demo with sample chats");
  await page.getByRole("link", { name: /open the message/i }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
  await expect(page.getByText(/store the refresh token in the device keychain/)).toBeVisible();

  const followUp = demo.getByRole("link", { name: "See the Codex follow-up", exact: true });
  const followUpHash =
    "#/chat/mobile-keychain-follow-up?node=workstation&message=mobile-keychain-follow-up-1&shared=true";
  await expect(followUp).toHaveAttribute("href", followUpHash);
  await followUp.click();
  await expect(page).toHaveURL(`/demo/${followUpHash}`);
  const recap = page.locator("#message-mobile-keychain-follow-up-1");
  await expect(recap).toHaveClass(/\bhit\b/);
  await expect(recap).toContainText(
    "OK, go with the tokens. Keep the mobile app on the same scheme.",
  );
  await expect(recap).toContainText("The mobile task is still open");
  await expect(page.getByText(/The remaining check is on two devices/)).toBeVisible();
  await page.reload();
  await expect(recap).toHaveClass(/\bhit\b/);
  await page.goBack();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
});

test("the Japanese guide leads to the same sample handoff", async ({ page }) => {
  const guide = page.locator(".sl-markdown-content");
  await page.goto("/ja/guides/");
  await guide.locator('a[href="/ja/pick-up-work/"]').click();
  await expect(page).toHaveURL("/ja/pick-up-work/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("作業を再開する");
  await expect(page.locator("#follow-a-sample-handoff")).toBeAttached();
  await expect(
    guide.getByRole("link", { name: "デモで保存済みのログイン方針を開く" }),
  ).toHaveAttribute("href", "https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop");
});

test("the memory guide leads from a saved decision to its source message", async ({ page }) => {
  await page.goto("/records/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Use Mash memory");
  const example = page.getByRole("link", { name: "Open a saved decision in the demo" });
  await expect(example).toHaveAttribute(
    "href",
    "https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop",
  );
  const target = new URL((await example.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  await page.getByRole("link", { name: /open the message/i }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
});

for (const [language, query, title, message, guide = "/search-languages/"] of [
  ["Spanish", "autenticación", "Depurar el inicio de sesión", "spanish-sign-in-0"],
  ["French", "corrigé", "Valider le correctif de pagination", "french-pagination-0"],
  ["Chinese", "我们更新了身份验证令牌", "更新身份验证令牌", "chinese-token-update-0"],
  [
    "Japanese",
    "認証トークンの更新手順を見直した",
    "認証トークンの更新手順",
    "japanese-token-update-0",
  ],
  [
    "日本語",
    "認証トークンの更新手順を見直した",
    "認証トークンの更新手順",
    "japanese-token-update-0",
    "/ja/search-languages/",
  ],
]) {
  test(`the language guide opens the ${language} example in the demo`, async ({ page }) => {
    await page.goto(guide);
    const example = page.getByRole("link", { name: language, exact: true });
    const target = new URL((await example.getAttribute("href")) ?? "");
    await page.goto(target.pathname + target.hash);
    await expect(page.getByRole("textbox", { name: "Search your history" })).toHaveValue(query);
    await page
      .getByRole("link", { name: new RegExp(title) })
      .first()
      .click();
    const source = page.locator(`#message-${message}`);
    await expect(source).toHaveClass(/\bhit\b/);
    await expect(source).toContainText(query);
  });
}

test("the Japanese dashboard guide leads to a downloadable sample chat", async ({ page }) => {
  await page.goto("/ja/quick-start/");
  await page
    .locator(".sl-markdown-content")
    .getByRole("link", { name: "ダッシュボードで検索する", exact: true })
    .click();
  await expect(page).toHaveURL("/ja/dashboard/#find-and-read-a-chat");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Mash のダッシュボードを使う");
  await expect(page.getByText("Export to Markdown", { exact: true })).toBeVisible();
  const example = page.getByRole("link", { name: "日本語のサンプルチャットをデモで検索" });
  const target = new URL((await example.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  await page
    .getByRole("link", { name: /認証トークンの更新手順/ })
    .first()
    .click();
  await page.getByRole("button", { name: "Export", exact: true }).click();
  const downloaded = page.waitForEvent("download");
  await page.getByRole("menuitem", { name: "Export to Markdown", exact: true }).click();
  const markdown = await downloaded;
  expect(markdown.suggestedFilename()).toMatch(/\.md$/);
  expect(await readFile(await markdown.path(), "utf8")).toContain(
    "認証トークンの更新手順を見直した",
  );
});

test("the old shared search bookmark leads to remote access and a chat from another machine", async ({
  page,
}) => {
  await page.goto("/shared-index/#search");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator("#search")).toBeAttached();
  await guide.getByRole("link", { name: "Join the existing server" }).click();
  await expect(page).toHaveURL("/shared-index/#join-and-review-each-computer");
  await expect(guide.locator("pre").filter({ hasText: "JOIN_STRING_FROM_SERVER" })).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL("/shared-index/#search");
  await guide.locator('a[href="/search-across-machines/"]').last().click();
  await expect(page).toHaveURL("/search-across-machines/");
  await expect(page.locator('nav.sidebar [aria-current="page"]')).toHaveText(
    "Search across your machines",
  );
  await page.getByRole("link", { name: /Open that computer.s dashboard remotely/ }).click();
  await expect(page).toHaveURL("/dashboard-access/");
  await page.goBack();
  const demo = page.getByRole("link", { name: "Try the sample search", exact: true });
  await expect(demo).toHaveAttribute("href", /^https:\/\/getmash\.io\/demo\/#\/search\?q=/);
  const target = new URL((await demo.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  const result = page.getByRole("link", { name: /Choose a queue for invoice exports/ }).first();
  await expect(result).toHaveAttribute("href", /[?&]node=workstation(?:&|$)/);
  await result.click();
  await expect(
    page.getByRole("heading", { name: "Choose a queue for invoice exports" }),
  ).toBeVisible();
});
