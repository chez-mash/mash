import { expect, type Page, test } from "@playwright/test";
import { fitsViewport, mockShell } from "./shell";

// The desktop app's update banner, with the app stood in for: the page is
// marked as running in the app and given the app's three commands.
const offer = {
  current: "1.4.0",
  version: "1.5.0",
  way: "download",
  stage: "available",
  progress: null,
  reason: null,
  command: null,
  downloads: "https://example.invalid/mash/downloads",
};

type Asked = { asked: string[] };

/** Stands in for the desktop app, which offers `first` until told otherwise. */
async function inApp(page: Page, first: object | null): Promise<void> {
  await page.addInitScript((answer) => {
    const app = { answer, asked: [] as string[] };
    Object.assign(window, {
      app,
      __MASH_DESKTOP__: true,
      __TAURI_INTERNALS__: {
        invoke: async (command: string) => {
          app.asked.push(command);
          if (command === "update_later") app.answer = null;
          if (command === "update_install" && app.answer)
            app.answer = { ...app.answer, stage: "installing" };
          return command === "update_banner" ? app.answer : null;
        },
      },
    });
  }, first);
}

const asked = (page: Page): Promise<string[]> =>
  page.evaluate(() => (window as unknown as { app: Asked }).app.asked);

test("the app's banner offers a newer version and goes away on Later", async ({ page }) => {
  await inApp(page, offer);
  await mockShell(page);
  await page.goto("/#/");
  const banner = page.getByRole("complementary", { name: "Mash update" });
  await expect(banner).toContainText("Mash 1.5.0 is available");
  await expect(banner).toContainText("You have 1.4.0.");
  await expect(banner.getByRole("link", { name: "Download update" })).toHaveAttribute(
    "href",
    offer.downloads,
  );
  expect(await fitsViewport(page)).toBe(true);
  // It stays while the reader moves between pages.
  await page.goto("/#/settings");
  await expect(banner).toBeVisible();
  await banner.getByRole("button", { name: "Later" }).click();
  await expect(banner).toHaveCount(0);
  expect(await asked(page)).toContain("update_later");
  expect(await asked(page)).not.toContain("update_install");
});

test("Update and restart asks the app to install and shows it at work", async ({ page }) => {
  await inApp(page, { ...offer, way: "install" });
  await mockShell(page);
  await page.goto("/#/");
  const banner = page.getByRole("complementary", { name: "Mash update" });
  await expect(banner.getByRole("button", { name: "Update and restart" })).toBeVisible();
  expect(await asked(page)).not.toContain("update_install");
  await banner.getByRole("button", { name: "Update and restart" }).click();
  await expect(banner).toContainText("Installing Mash 1.5.0…");
  await expect(banner.getByRole("button")).toHaveCount(0);
  expect(await asked(page)).toContain("update_install");
});

test("the banner fits a narrow window", async ({ page }) => {
  await page.setViewportSize({ width: 420, height: 700 });
  await inApp(page, {
    ...offer,
    way: "install",
    stage: "failed",
    reason: "the download did not pass its signature check (invalid signature)",
  });
  await mockShell(page);
  await page.goto("/#/");
  const banner = page.getByRole("complementary", { name: "Mash update" });
  await expect(banner).toContainText("Mash couldn’t update to 1.5.0");
  await expect(banner).toContainText("signature check");
  await expect(banner.getByRole("link", { name: "Download instead" })).toBeVisible();
  expect(await fitsViewport(page)).toBe(true);
});

test("a browser shows no banner", async ({ page }) => {
  // The commands alone are not the app: the page must be marked by it.
  await page.addInitScript((answer) => {
    Object.assign(window, { __TAURI_INTERNALS__: { invoke: async () => answer } });
  }, offer);
  await mockShell(page);
  await page.goto("/#/");
  await expect(page.getByRole("navigation", { name: "Main" })).toBeVisible();
  await expect(page.getByRole("complementary", { name: "Mash update" })).toHaveCount(0);
});
