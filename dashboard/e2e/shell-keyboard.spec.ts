import { expect, type Page, test } from "@playwright/test";
import { mockShell, rail } from "./shell";

const focused = (page: Page) =>
  page.evaluate(() => {
    const active = document.activeElement;
    return active ? (active.getAttribute("aria-label") ?? active.textContent?.trim() ?? "") : "";
  });

test("a skip link is the first stop and lands on the page's content", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/projects");
  // Tab only once the page is drawn, or the key lands before the link exists.
  await expect(rail(page).getByRole("link", { name: "Projects", exact: true })).toBeVisible();
  await page.keyboard.press("Tab");
  const skip = page.getByRole("link", { name: "Skip to content" });
  await expect(skip).toBeFocused();
  await expect(skip).toBeInViewport();
  await page.keyboard.press("Enter");
  await expect(page.locator("#main")).toBeFocused();
  // The route stays where it was.
  await expect(page).toHaveURL(/#\/projects$/);
  await page.keyboard.press("Tab");
  expect(
    await page.evaluate(() => document.getElementById("main")?.contains(document.activeElement)),
  ).toBe(true);
});

test("each rail row is one tab stop, and arrows open and close its pages", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/settings");
  const chevron = rail(page).getByRole("button", { name: /^(Expand|Collapse) Chats$/ });
  await expect(chevron).toHaveAttribute("tabindex", "-1");
  const chats = rail(page).getByRole("link", { name: "Chats", exact: true });
  await chats.focus();
  await page.keyboard.press("Tab");
  await expect(rail(page).getByRole("link", { name: "Memory", exact: true })).toBeFocused();
  await chats.focus();
  await page.keyboard.press("ArrowRight");
  await expect(chevron).toHaveAttribute("aria-expanded", "true");
  await expect(rail(page).getByRole("link", { name: "Viewing history" })).toBeVisible();
  await page.keyboard.press("ArrowLeft");
  await expect(chevron).toHaveAttribute("aria-expanded", "false");
  await expect(chats).toBeFocused();
});

test("the phone drawer holds focus while open and returns it to the menu button", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockShell(page);
  await page.goto("/");
  const burger = page.getByRole("button", { name: "Open navigation" });
  await burger.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("button", { name: "Close navigation" })).toBeFocused();
  await expect(page.locator("#col")).toHaveAttribute("inert", "");
  const inDrawer = () =>
    page.evaluate(() => document.getElementById("rail")?.contains(document.activeElement));
  for (let step = 0; step < 25; step++) {
    await page.keyboard.press("Tab");
    expect(await inDrawer(), `after ${step + 1} tabs focus is on ${await focused(page)}`).toBe(
      true,
    );
  }
  for (let step = 0; step < 4; step++) {
    await page.keyboard.press("Shift+Tab");
    expect(await inDrawer()).toBe(true);
  }
  await page.keyboard.press("Escape");
  await expect(burger).toBeFocused();
  await expect(page.locator("#col")).not.toHaveAttribute("inert", "");

  // Closing it with its own button returns focus too.
  await page.keyboard.press("Enter");
  await expect(page.getByRole("button", { name: "Close navigation" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(burger).toBeFocused();
});

test("Escape closes a filter menu and returns focus to its chip", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/search");
  const chip = page.getByRole("button", { name: "any agent", exact: true });
  await chip.focus();
  await page.keyboard.press("Enter");
  await expect(chip).toHaveAttribute("aria-expanded", "true");
  // The menu takes focus and the arrow keys move through its choices.
  await page.keyboard.press("ArrowDown");
  await expect
    .poll(() => page.evaluate(() => document.activeElement?.closest(".fmenu") !== null))
    .toBe(true);
  await page.keyboard.press("Escape");
  await expect(chip).toHaveAttribute("aria-expanded", "false");
  await expect(chip).toBeFocused();
});

test("the search box shows a focus ring", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/");
  const box = page.locator("form.box");
  await expect(box).toHaveCSS("outline-style", "none");
  await page.locator("#q").focus();
  await expect(box).toHaveCSS("outline-style", "solid");
  await expect(box).toHaveCSS("outline-width", "2px");
});

test("the rail marks recent chats without a query and Search with one", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/search");
  const recent = rail(page).getByRole("link", { name: "Recent chats" });
  const chats = rail(page).getByRole("link", { name: "Chats", exact: true });
  const search = rail(page).getByRole("button", { name: "Search every chat" });
  await expect(recent).toHaveAttribute("aria-current", "page");
  await expect(chats).not.toHaveAttribute("aria-current", "page");
  await expect(search).not.toHaveAttribute("aria-current", "page");
  await page.goto("/#/search?q=deploy");
  await expect(search).toHaveAttribute("aria-current", "page");
  await expect(chats).not.toHaveAttribute("aria-current", "page");
});

test("the rail's Search row focuses the search field", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockShell(page);
  await page.goto("/#/settings");
  await rail(page).getByRole("button", { name: "Search every chat" }).click();
  await expect(page.locator("#q")).toBeFocused();
  await expect(page).toHaveURL(/#\/settings$/);
});

test("on a short screen the rail's footer stays in view while its rows scroll", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1024, height: 560 });
  await mockShell(page);
  await page.goto("/#/agents");
  const nav = rail(page);
  await expect(nav).toHaveClass(/fade-below/);
  const update = nav.getByRole("button", { name: "Check for updates" });
  await expect(update).toBeInViewport({ ratio: 1 });
  await nav.evaluate((element) => element.scrollTo(0, element.scrollHeight));
  await expect(update).toBeInViewport({ ratio: 1 });
  await expect(nav.getByRole("link", { name: "Settings", exact: true })).toBeInViewport({
    ratio: 1,
  });
});
