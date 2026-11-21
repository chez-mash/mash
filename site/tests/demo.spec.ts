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
  await page.getByRole("link", { name: /open the message/ }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
  expect(problems).toEqual([]);
});

test("the quick start opens a sample search that fits a narrow screen", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto("/quick-start/");
  const link = page.getByRole("link", { name: "Try a sample search", exact: true });
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
});

test("the guide to picking up work opens the original decision and shows what is left", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/guides/");
  await page.getByRole("link", { name: "Pick up where you left off or leave a handoff" }).click();
  await expect(page).toHaveURL("/pick-up-work/");
  const example = page.getByRole("link", { name: "Try it with sample chats" });
  await expect(example).toHaveAttribute(
    "href",
    "https://getmash.io/demo/#/record/record-sign-in-tokens?node=laptop",
  );
  const target = new URL((await example.getAttribute("href")) ?? "");
  await page.goto(target.pathname + target.hash);
  await page.getByRole("link", { name: /open the message/ }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
  await expect(page.getByText(/store the refresh token in the device keychain/)).toBeVisible();
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
  await page.getByRole("link", { name: /open the message/ }).click();
  await expect(page.locator("#message-sign-in-tokens-2")).toHaveClass(/\bhit\b/);
});

for (const [language, query, title, message] of [
  ["Spanish", "autenticación", "Depurar el inicio de sesión", "spanish-sign-in-0"],
  ["French", "corrigé", "Valider le correctif de pagination", "french-pagination-0"],
  ["Chinese", "我们更新了身份验证令牌", "更新身份验证令牌", "chinese-token-update-0"],
  [
    "Japanese",
    "認証トークンの更新手順を見直した",
    "認証トークンの更新手順",
    "japanese-token-update-0",
  ],
]) {
  test(`the language guide opens the ${language} example in the demo`, async ({ page }) => {
    await page.goto("/search-languages/");
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

test("the sharing guide leads to remote access and to a chat from another machine", async ({
  page,
}) => {
  await page.goto("/shared-index/");
  await page.getByRole("link", { name: /Open that computer.s dashboard remotely/ }).click();
  await expect(page).toHaveURL("/dashboard/#open-it-from-your-phone-or-another-computer");
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
