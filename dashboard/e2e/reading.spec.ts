import { expect, test } from "@playwright/test";

// A long chat reads in stretches as the reader scrolls, with no pages to turn.
const length = 350;
const message = (index: number) => ({
  id: `m${index}`,
  ordinal: index,
  role: index % 2 ? "assistant" : "user",
  content: `Message number ${index}. ${"A line of ordinary text. ".repeat(8)}`,
});

test.beforeEach(async ({ page }) => {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: [
          {
            chat_id: "long",
            node_id: "local",
            provider: "codex",
            title: "Long fixture chat",
            project: "/work/project",
            updated_at: Date.now(),
          },
        ],
      },
    }),
  );
  await page.route("**/api/chat?**", (route) => {
    const url = new URL(route.request().url());
    const offset = Number(url.searchParams.get("offset"));
    const limit = Number(url.searchParams.get("limit"));
    const end = Math.min(length, offset + limit);
    route.fulfill({
      json: {
        message_count: length,
        messages: Array.from({ length: Math.max(0, end - offset) }, (_, index) =>
          message(offset + index),
        ),
      },
    });
  });
  await page.route("**/api/view?**", (route) =>
    route.fulfill({
      json: { messages: Array.from({ length: 51 }, (_, index) => message(175 + index)) },
    }),
  );
});

test("a chat opens at its end and reads earlier messages while the reader scrolls up", async ({
  page,
}) => {
  await page.goto("/#/chat/long");
  await expect(page.locator("#message-m349")).toBeInViewport();
  await expect(page.getByText("350 messages")).toBeVisible();
  await expect(page.getByRole("button", { name: /Next messages|Previous messages/ })).toHaveCount(
    0,
  );

  // Scrolling up reads on, and the message on screen stays where it was.
  for (let step = 0; step < 40; step++) {
    if (await page.locator("#message-m0").count()) break;
    const before = await page.evaluate(() => {
      const element = document.querySelector<HTMLElement>("[data-message]");
      return element
        ? { id: element.dataset.message, top: element.getBoundingClientRect().top }
        : null;
    });
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(150);
    if (before?.id) {
      const top = await page.evaluate(
        (id) => document.getElementById(`message-${id}`)?.getBoundingClientRect().top,
        before.id,
      );
      // The message that was at the top did not jump to the top of the screen.
      if (top !== undefined && (await page.locator("#message-m0").count()) === 0)
        expect(top).toBeGreaterThan(100);
    }
  }
  await expect(page.locator("#message-m0")).toBeAttached();

  await page.getByRole("button", { name: "Latest message" }).click();
  await expect(page.locator("#message-m349")).toBeInViewport();
  await page.getByRole("button", { name: "First message" }).click();
  await expect(page.locator("#message-m0")).toBeInViewport();
});

test("a linked message opens in place and the chat reads on in both directions", async ({
  page,
}) => {
  await page.goto("/#/chat/long?message=m200");
  await expect(page.locator("#message-m200")).toBeInViewport();
  await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
  await expect(page.locator("#message-m240")).toBeAttached();
  await page.evaluate(() => window.scrollTo(0, 0));
  await expect(page.locator("#message-m160")).toBeAttached();
});
