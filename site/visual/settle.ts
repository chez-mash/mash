import type { Page } from "@playwright/test";

/**
 * Waits for what arrives after the page: its fonts and its pictures. A picture
 * the browser has put off until it is nearly on screen (`loading="lazy"`) is
 * not waited for, because it would never come; `arrived` waits for those once
 * their part of the page is on screen.
 */
export async function settle(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
  await page.evaluate(async () => {
    await document.fonts.ready;
    const due = [...document.images].filter((image) => image.complete || image.loading !== "lazy");
    await Promise.all(due.map((image) => image.decode().catch(() => undefined)));
  });
}

/** Waits for the pictures inside `selector`, which is on screen, to be drawn. */
export async function arrived(page: Page, selector: string): Promise<void> {
  await page.locator(selector).evaluate(async (part) => {
    const pictures = [...part.querySelectorAll("img")];
    await Promise.all(pictures.map((image) => image.decode().catch(() => undefined)));
  });
}
