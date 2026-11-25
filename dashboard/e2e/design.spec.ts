import { expect, type Page, test } from "@playwright/test";
import { recordFixture } from "./record-fixtures";
import { documentLoads, fitsViewport, followRail, rail, searchBox } from "./shell";

const chat = {
  chat_id: "chat-1",
  node_id: "local",
  title: "Investigate the watcher",
  provider: "codex",
  project: "/work/mash",
  updated_at: Date.now(),
};

async function mockHistory(page: Page, searched: () => void): Promise<void> {
  await page.route("**/api/**", (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "local", chats: [chat], truncated: false } });
    if (path === "/api/records")
      return route.fulfill({ json: { records: [recordFixture()], next_offset: null } });
    if (path === "/api/services/access")
      return route.fulfill({ json: { authentication_required: false, local_node: "local" } });
    if (path === "/api/services") return route.fulfill({ json: { machines: [] } });
    if (path === "/api/search") {
      searched();
      return route.fulfill({
        json: {
          results: [{ ...chat, matches: [{ message_id: "message-1", snippet: "watcher result" }] }],
          next_offset: null,
        },
      });
    }
    return route.fulfill({
      json: {
        messages: [
          {
            id: "message-1",
            ordinal: 0,
            role: "user",
            content: `Investigate the watcher\n\n**Agreed** <plan> first\nthen deploy\n\n${"More context.\n\n".repeat(80)}`,
          },
        ],
      },
    });
  });
}

for (const width of [1440, 390]) {
  test(`approved layout navigation, memory and reading at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    const documents = documentLoads(page);
    let searches = 0;
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await mockHistory(page, () => searches++);
    await page.goto("/");
    const main = page.getByRole("main");
    const recentChat = main.getByRole("link", { name: /Investigate the watcher/ });
    await expect(recentChat).toBeVisible();
    for (const name of ["Recent chats", "Recent memories", "Machines", "Agents"])
      await expect(main.getByRole("heading", { name, exact: true })).toBeVisible();
    await expect(main.getByRole("link", { name: /preference fixture on port 8080/ })).toBeVisible();
    const section = (name: string) =>
      main.locator("section").filter({ has: page.getByRole("heading", { name, exact: true }) });
    await section("Agents").getByRole("link", { name: /Codex/ }).click();
    await expect(page).toHaveURL(/#\/agent\/codex$/);
    await expect(page.getByRole("heading", { name: "Codex", exact: true })).toBeVisible();
    await page.goBack();
    await section("Machines").getByRole("link", { name: /local/ }).click();
    await expect(page).toHaveURL(/#\/machine\/local$/);
    await expect(page.getByRole("heading", { name: "local", exact: true })).toBeVisible();
    await page.goBack();
    // The agent's mark is its logo, inline SVG in the brand colour, on an
    // ivory tile: the logo must have a size and a colour, and the tile must
    // not be transparent.
    const logo = recentChat.locator(".mark svg");
    const drawn = await logo.evaluate((svg) => {
      const box = svg.getBoundingClientRect();
      return { width: box.width, colour: getComputedStyle(svg).color };
    });
    expect(drawn.width).toBeGreaterThan(0);
    expect(drawn.colour).not.toBe("rgba(0, 0, 0, 0)");
    const tile = await recentChat
      .locator(".mark")
      .evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(tile).not.toBe("rgba(0, 0, 0, 0)");
    const open = page.getByRole("button", { name: "Open navigation" });
    if (width < 800) {
      // The rail is a drawer on a narrow screen: hidden until asked for.
      await expect(rail(page)).toBeHidden();
      await open.click();
      await expect(rail(page)).toBeVisible();
      await expect(open).toHaveAttribute("aria-expanded", "true");
      await page.getByRole("button", { name: "Close navigation" }).first().click();
      await expect(rail(page)).toBeHidden();
    } else {
      await expect(open).toBeHidden();
      await expect(rail(page)).toBeVisible();
    }
    await followRail(page, "Machines");
    if (width < 800) await expect(rail(page)).toBeHidden();
    await expect(
      page.getByRole("heading", { level: 1, name: "Machines", exact: true }),
    ).toBeVisible();
    await main.getByRole("link", { name: /local/ }).click();
    await main
      .getByRole("navigation", { name: "Summary" })
      .getByRole("link", { name: /^chats/i })
      .click();
    await expect(page).toHaveURL(/#\/search\?machine=local$/);
    await expect(page.getByRole("link", { name: "Remove filter local" })).toBeVisible();
    await expect(recentChat).toBeVisible();
    await searchBox(page).fill("watcher");
    await searchBox(page).press("Enter");
    await expect(page).toHaveURL(/machine=local/);
    await expect(main.getByText("watcher result")).toBeVisible();
    await main.getByRole("link", { name: /Investigate the watcher/ }).click();
    await expect(page.locator("#message-message-1")).toContainText("Investigate the watcher");
    // The reader's own message is formatted, keeps its line breaks, and shows
    // a tag they typed as text.
    await expect(page.locator("#message-message-1 strong")).toHaveText("Agreed");
    await expect(page.locator("#message-message-1")).toContainText("<plan> first");
    await expect(page.locator("#message-message-1 p br")).toHaveCount(1);
    const sessions = page.getByRole("navigation", { name: "Prompts and sessions" });
    const showSessions = page.getByRole("button", { name: "Show session list" });
    if (width <= 700) {
      await expect(sessions).toBeHidden();
      // On a phone the list is in the chat's menu.
      await page.getByRole("button", { name: "More chat actions" }).click();
      await showSessions.click();
      await expect(sessions).toBeVisible();
      await sessions.getByRole("button", { name: "Back to chat" }).click();
      await expect(sessions).toBeHidden();
      await expect(page.locator("#message-message-1")).toBeVisible();
    } else {
      await expect(sessions).toBeVisible();
      await page.getByRole("button", { name: "Hide session list" }).click();
      await expect(sessions).toBeHidden();
      await showSessions.click();
      await expect(sessions).toBeVisible();
    }
    await page.evaluate(() => window.scrollTo(0, 200));
    await page.goBack();
    await expect(searchBox(page)).toHaveValue("watcher");
    await expect(main.getByText("watcher result")).toBeVisible();
    expect(searches).toBe(1);
    await followRail(page, "Viewing history", "Chats");
    await expect(page.getByRole("heading", { name: "Viewing history", exact: true })).toBeVisible();
    await expect(recentChat).toBeVisible();
    await followRail(page, "Memory");
    await page
      .getByRole("navigation", { name: "Memory kind" })
      .getByRole("link", { name: "Preferences", exact: true })
      .click();
    await expect(
      page
        .getByRole("navigation", { name: "Memory kind" })
        .getByRole("link", { name: "Preferences", exact: true }),
    ).toHaveAttribute("aria-current", "page");
    expect(await fitsViewport(page)).toBe(true);
    expect(documents()).toBe(1);
    expect(errors).toEqual([]);
  });
}

// The earlier drawer was a dialog that Escape closed, handing focus back to
// the button that opened it. The rail drawer has no key handler (Rail.svelte,
// App.svelte), so Escape leaves it open over the page and a keyboard reader
// must tab to "Close navigation". Remove fixme once the source handles it.
test("the navigation drawer closes on Escape and returns focus to its button", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await mockHistory(page, () => {});
  await page.goto("/");
  const open = page.getByRole("button", { name: "Open navigation" });
  await open.click();
  await expect(rail(page)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(rail(page)).toBeHidden();
  await expect(open).toBeFocused();
});
