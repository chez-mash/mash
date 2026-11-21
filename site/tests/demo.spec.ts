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
});
