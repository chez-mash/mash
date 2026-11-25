import process from "node:process";
import { expect, test } from "@playwright/test";
import { documentLoads, rail } from "./shell";

// Run against a disposable Mash database seeded with four Dashboard records
// through the real MCP record tool. No route interception or writes in-browser.
test("live records browse, filter, expose revisions and open the real source chat", async ({
  page,
}) => {
  test.skip(
    process.env.MASH_DASHBOARD_RECORD_FIXTURE !== "1",
    "Requires the disposable record fixture",
  );
  const errors: string[] = [];
  const documents = documentLoads(page);
  page.on("pageerror", (error) => errors.push(error.message));
  const records = page.getByRole("main").getByRole("article");
  await page.goto("/");
  await rail(page).getByRole("link", { name: "Memory", exact: true }).click();
  await expect(records).toHaveCount(4);
  await page.screenshot({
    path: "../../mash-artifacts/dashboard-records-live/list.png",
    fullPage: true,
  });
  await page
    .getByRole("navigation", { name: "Memory kind" })
    .getByRole("link", { name: "preferences", exact: true })
    .click();
  await expect(records).toHaveCount(1);
  // Filters apply only on request, and the button appears once a draft differs.
  await page.getByLabel("Search statements").fill("Dashboard preference");
  await page.getByRole("button", { name: "Apply filters" }).click();
  await expect(page).toHaveURL(/q=Dashboard/);
  await expect(records).toHaveCount(1);
  await records.getByRole("link", { name: /Dashboard preference revised/ }).click();
  await expect(page.getByText(/^Revision 2 \(current\)/)).toBeVisible();
  await expect(page.getByText(/^Revision 1:/)).toBeVisible();
  await page.screenshot({
    path: "../../mash-artifacts/dashboard-records-live/detail.png",
    fullPage: true,
  });
  await page
    .getByRole("link", { name: /open the message/ })
    .first()
    .click();
  await expect(
    page.getByText("Use port 8080. I prefer concise replies. The platform team owns the review.", {
      exact: true,
    }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Memory from this chat" }).click();
  await expect(records).toHaveCount(4);
  await expect(page.getByRole("alert")).toHaveCount(0);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole("heading", { name: "Memory", exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({
    path: "../../mash-artifacts/dashboard-records-live/mobile-list.png",
    fullPage: true,
  });
  expect(documents()).toBe(1);
  expect(errors).toEqual([]);
});
