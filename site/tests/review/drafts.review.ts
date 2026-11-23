import { utimes } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";
import { holdMs } from "../../playwright.review.config.ts";

// Touching a component the docs pages render makes Astro send a full reload,
// as a review worker's edit would. Only the modification time changes.
const component = fileURLToPath(new URL("../../src/components/Footer.astro", import.meta.url));
// Long enough for the dev server to notice the change and send its reload.
const settleMs = 2500;

async function touchComponent(): Promise<void> {
  const now = new Date();
  await utimes(component, now, now);
}

// Answer the review supervisor so the toolbar mounts, and refuse the sync
// server so test notes stay in the browser and never reach a real review.
async function stubReviewServices(page: Page): Promise<void> {
  await page.route("**/__agentation-review/agent", (route) =>
    route.fulfill({ json: { agent: "claude", available: ["claude", "codex"] } }),
  );
  await page.route("**/__agentation-review/events", (route) =>
    route.fulfill({ contentType: "text/event-stream", body: ": idle\n\n" }),
  );
  await page.route("**/__agentation/**", (route) => route.abort());
}

/** Count document loads, so a held reload can be told from one that happened. */
function countLoads(page: Page): () => number {
  let loads = 0;
  page.on("load", () => {
    loads += 1;
  });
  return () => loads;
}

const popup = (page: Page) => page.locator("[data-annotation-popup]");
const noteText = (page: Page) => popup(page).locator("textarea");

async function openNote(page: Page, text: string): Promise<void> {
  await page.locator('[data-agentation-toolbar] > [role="button"]').click();
  await page.locator("main h1").click();
  await expect(noteText(page)).toBeFocused();
  await noteText(page).pressSequentially(text);
}

// A fresh dev server optimizes the toolbar's dependencies on the first visit
// and then reloads the page once. Let that happen before any test counts loads.
test.beforeAll(async ({ browser }) => {
  const page = await browser.newPage();
  await stubReviewServices(page);
  await page.goto("/backup/");
  await page.waitForTimeout(5000);
  await page.goto("/backup/");
  await expect(page.locator('[data-agentation-toolbar] > [role="button"]')).toBeVisible();
  await page.close();
});

test.beforeEach(async ({ page }) => {
  await stubReviewServices(page);
});

test("a full reload waits until the open note is added", async ({ page }) => {
  await page.goto("/backup/");
  const loads = countLoads(page);
  await openNote(page, "Half-typed note");

  await touchComponent();
  await page.waitForTimeout(settleMs);
  expect(loads(), "the reload is held while the note is open").toBe(0);
  await expect(noteText(page)).toHaveValue("Half-typed note");

  await popup(page).getByRole("button", { name: "Add" }).click();
  await expect.poll(loads, { message: "the held reload follows the added note" }).toBe(1);
  await page.waitForLoadState("load");
  await page.waitForTimeout(1000);
  await expect(popup(page), "an added note is not reopened").toHaveCount(0);
});

test("reloading the page reopens the note on the same element", async ({ page }) => {
  await page.goto("/backup/");
  await openNote(page, "Typed before the reload");
  const header = await popup(page).textContent();

  await page.reload();
  await expect(noteText(page)).toHaveValue("Typed before the reload");
  expect(await popup(page).textContent()).toBe(header);

  await popup(page).getByRole("button", { name: "Cancel" }).click();
  await expect(popup(page)).toHaveCount(0);
  await page.reload();
  await page.waitForTimeout(2000);
  await expect(popup(page), "a cancelled note is not reopened").toHaveCount(0);
});

test("a forgotten note releases the held reload after the timeout", async ({ page }) => {
  await page.goto("/backup/");
  const loads = countLoads(page);
  await openNote(page, "Left open");
  const typedAt = Date.now();

  await touchComponent();
  await page.waitForTimeout(settleMs);
  expect(loads(), "the reload is held at first").toBe(0);
  await expect.poll(loads, { timeout: holdMs + 10_000 }).toBe(1);
  expect(Date.now() - typedAt).toBeGreaterThanOrEqual(holdMs);
  await expect(noteText(page), "the note survives the reload").toHaveValue("Left open");
});

test("without an open note, the reload is not held", async ({ page }) => {
  await page.goto("/backup/");
  await expect(page.locator('[data-agentation-toolbar] > [role="button"]')).toBeVisible();
  const loads = countLoads(page);
  await touchComponent();
  await expect.poll(loads, { timeout: settleMs + 2000 }).toBe(1);
});
