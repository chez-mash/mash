import { readFile, utimes, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";
import { holdMs } from "../../playwright.review.config.ts";

// The fixture copy the dev server serves (see playwright.review.config.ts).
// Touching its component makes Astro send a full reload, as a review worker's
// edit would; changing its style is a CSS hot update.
const component = fileURLToPath(new URL(".fixture/src/components/Note.astro", import.meta.url));
// Long enough for the dev server to notice the change and send its reload.
const settleMs = 2500;

async function touchComponent(): Promise<void> {
  const now = new Date();
  await utimes(component, now, now);
}

async function recolourComponent(colour: string): Promise<void> {
  const source = await readFile(component, "utf8");
  await writeFile(component, source.replace(/color: [^;]+;/, `color: ${colour};`));
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
  await page.locator(".note").click();
  await expect(noteText(page)).toBeFocused();
  await noteText(page).pressSequentially(text);
}

// A fresh dev server optimizes the toolbar's dependencies on the first visit
// and then reloads the page once. Let that happen before any test counts loads.
test.beforeAll(async ({ browser }) => {
  const page = await browser.newPage();
  await stubReviewServices(page);
  await page.goto("/");
  await page.waitForTimeout(5000);
  await page.goto("/");
  await expect(page.locator('[data-agentation-toolbar] > [role="button"]')).toBeVisible();
  await page.close();
});

test.beforeEach(async ({ page }) => {
  await stubReviewServices(page);
});

test("a full reload waits until the open note is added", async ({ page }) => {
  await page.goto("/");
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
  await page.goto("/");
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
  await page.goto("/");
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

test("style updates still apply while a note is open", async ({ page }) => {
  await page.goto("/");
  const loads = countLoads(page);
  await openNote(page, "Watching the colour");
  await recolourComponent("rgb(200, 0, 0)");
  await expect(page.locator(".note")).toHaveCSS("color", "rgb(200, 0, 0)");
  await page.waitForTimeout(settleMs);
  expect(loads(), "a style update does not reload the page").toBe(0);
  await expect(noteText(page)).toHaveValue("Watching the colour");
});

// Seen on the live review: a style hot update whose module fails to load
// ("Importing a module script failed" in Safari). Vite's client logs the
// failure, and any full reload the server sends after it is held.
test("a failed style update does not reload the page while a note is open", async ({ page }) => {
  await page.goto("/");
  const loads = countLoads(page);
  await openNote(page, "Despite a failed update");
  let failedUpdates = 0;
  await page.route(/Note\.astro\?t=\d+&astro&type=style/, (route) => {
    failedUpdates += 1;
    return route.abort();
  });
  await recolourComponent("rgb(0, 0, 200)");
  await expect.poll(() => failedUpdates, { message: "the style update was attempted" }).toBe(1);
  await touchComponent();
  await page.waitForTimeout(settleMs);
  expect(loads(), "the page is not reloaded").toBe(0);
  await expect(noteText(page)).toHaveValue("Despite a failed update");
  await popup(page).getByRole("button", { name: "Cancel" }).click();
  await expect.poll(loads, { message: "the held reload follows the cancelled note" }).toBe(1);
});

test("without an open note, the reload is not held", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator('[data-agentation-toolbar] > [role="button"]')).toBeVisible();
  const loads = countLoads(page);
  await touchComponent();
  await expect.poll(loads, { timeout: settleMs + 2000 }).toBe(1);
});
