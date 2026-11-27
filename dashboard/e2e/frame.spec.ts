import { expect, type Page, test } from "@playwright/test";

const chat = {
  chat_id: "chat-1",
  node_id: "laptop",
  title: "Investigate the watcher",
  provider: "codex",
  project: "/work/mash",
  updated_at: Date.now(),
};

// The desktop app tells the page which system it is on before the page
// loads (desktop/src-tauri/src/frame.rs); a browser says nothing.
async function inShell(page: Page, shell?: string): Promise<void> {
  if (shell)
    await page.addInitScript((name) => Object.assign(window, { __MASH_SHELL__: name }), shell);
  await page.route("**/api/**", (route) =>
    route.fulfill({ json: { local_node: "laptop", chats: [chat], truncated: false } }),
  );
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
}

test("in a browser the top bar keeps the mark and is no drag region", async ({ page }) => {
  await inShell(page);
  await expect(page.locator("html")).toHaveAttribute("data-shell", "browser");
  const bar = page.locator("#top");
  await expect(bar.locator(".brand")).toBeVisible();
  await expect(bar.locator(".lights")).toHaveCount(0);
  await expect(bar).not.toHaveAttribute("data-tauri-drag-region");
});

test("in the Mac app the top bar is the title bar, with the traffic lights over the sidebar", async ({
  page,
}) => {
  await inShell(page, "macos");
  await expect(page.locator("html")).toHaveAttribute("data-shell", "macos");
  const bar = page.locator("#top");
  await expect(bar).toHaveAttribute("data-tauri-drag-region", "");
  await expect(bar.locator(".brand")).toHaveCount(0);
  const lights = bar.locator(".lights");
  await expect(lights).toHaveAttribute("data-tauri-drag-region", "");
  // The cell is as wide as the sidebar under it, in its colour, and meets it.
  const [cell, rail] = await Promise.all([
    lights.boundingBox(),
    page.locator("#rail").boundingBox(),
  ]);
  if (!cell || !rail) throw new Error("Missing the traffic lights' cell or the sidebar");
  expect(cell.x).toBe(0);
  expect(Math.abs(cell.width - rail.width)).toBeLessThan(1);
  expect(Math.abs(cell.y + cell.height - rail.y)).toBeLessThan(1.5);
  const colour = (selector: string) =>
    page.locator(selector).evaluate((node) => getComputedStyle(node).backgroundColor);
  expect(await colour("#top .lights")).toBe(await colour("#rail"));
  // The search field stays centred over the page beside the sidebar.
  const search = await bar.locator(".search").boundingBox();
  const main = await page.locator("#main").boundingBox();
  if (!search || !main) throw new Error("Missing the search field or the page");
  expect(Math.abs(search.x + search.width / 2 - (main.x + main.width / 2))).toBeLessThan(2);
});

test("on Windows and Linux the system draws the title bar and the mark stays", async ({ page }) => {
  for (const shell of ["windows", "linux"]) {
    await inShell(page, shell);
    await expect(page.locator("html")).toHaveAttribute("data-shell", shell);
    await expect(page.locator("#top .brand")).toBeVisible();
    await expect(page.locator("#top")).toHaveAttribute("data-tauri-drag-region", "");
  }
});

test("rows and buttons show the arrow; a link out of Mash shows the hand", async ({ page }) => {
  await inShell(page);
  const cursor = (selector: string) =>
    page
      .locator(selector)
      .first()
      .evaluate((node) => getComputedStyle(node).cursor);
  expect(await cursor("#rail .nav-link")).toBe("default");
  expect(await cursor("#top button, #rail button")).toBe("default");
  await page.evaluate(() => {
    const out = document.createElement("a");
    out.href = "https://getmash.io/";
    out.id = "out";
    out.textContent = "docs";
    document.body.append(out);
  });
  expect(await cursor("#out")).toBe("pointer");
  // The page does not bounce past its ends, and the frame's labels do not select.
  expect(await page.evaluate(() => getComputedStyle(document.body).overscrollBehaviorY)).toBe(
    "none",
  );
  expect(await page.locator("#rail").evaluate((node) => getComputedStyle(node).userSelect)).toBe(
    "none",
  );
});

test("in a browser a right-click on a chat opens the dashboard's own menu", async ({ page }) => {
  await inShell(page);
  const row = page.locator(".oc-row").first();
  await row.click({ button: "right" });
  const menu = page.getByRole("menu");
  await expect(menu.getByRole("menuitem")).toHaveText(["Open", "Copy Link"]);
  await expect(menu.getByRole("menuitem", { name: "Open" })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(menu).toHaveCount(0);
  await row.click({ button: "right" });
  await page.getByRole("menuitem", { name: "Open" }).click();
  await expect(page).toHaveURL(/#\/chat\//);
  await expect(page.getByRole("menu")).toHaveCount(0);
});

test("in the app a right-click with no menu of Mash's opens nothing, but fields and selected text keep theirs", async ({
  page,
}) => {
  /** Right-clicks the first element matching `selector`; true when the page held the system's menu back. */
  const held = (selector: string) =>
    page
      .locator(selector)
      .first()
      .evaluate((node) => {
        const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
        node.dispatchEvent(event);
        return event.defaultPrevented;
      });
  await page.addInitScript(() =>
    Object.assign(window, { __MASH_DESKTOP__: true, __MASH_SHELL__: "macos" }),
  );
  await inShell(page);
  expect(await held("#main h1")).toBe(true);
  expect(await held("#rail .nav-link")).toBe(true);
  expect(await held("#q")).toBe(false);
  await page.locator("#main h1").first().selectText();
  expect(await held("#main h1")).toBe(false);
});

test("in a browser a right-click keeps the browser's menu", async ({ page }) => {
  await inShell(page);
  const held = await page
    .locator("#main h1")
    .first()
    .evaluate((node) => {
      const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
      node.dispatchEvent(event);
      return event.defaultPrevented;
    });
  expect(held).toBe(false);
});
