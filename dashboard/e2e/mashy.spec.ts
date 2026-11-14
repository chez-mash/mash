import { expect, test } from "@playwright/test";

const answer = (events: readonly object[]): string =>
  events.map((event) => `data: ${JSON.stringify(event)}\n\n`).join("");

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
            chat_id: "ports",
            node_id: "local",
            provider: "codex",
            title: "Port decision",
            project: "/work/project",
            updated_at: Date.now(),
          },
        ],
      },
    }),
  );
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: {
        message_count: 1,
        messages: [{ id: "m0", ordinal: 0, role: "user", content: "Which port?" }],
      },
    }),
  );
});

test("there is no Mashy on a dashboard without a model", async ({ page }) => {
  await page.route("**/api/assistant**", (route) =>
    route.fulfill({ json: { enabled: false, model: null } }),
  );
  await page.goto("/");
  await expect(page.getByRole("link", { name: /Port decision/ }).first()).toBeVisible();
  await expect(page.getByRole("button", { name: "Open Mashy" })).toHaveCount(0);
});

test("Mashy answers about the open chat, links its sources and reports failures", async ({
  page,
}) => {
  const asked: { messages: { role: string; content: string }[]; open_chat?: { id: string } }[] = [];
  let reply = answer([
    { type: "searching" },
    { type: "text", delta: "You chose port 8080 in " },
    { type: "text", delta: "[Port decision](chat:ports). <img src=x onerror=alert(1)>" },
    { type: "done", partial: false },
  ]);
  await page.route("**/api/assistant**", async (route) => {
    if (route.request().method() === "GET")
      return route.fulfill({ json: { enabled: true, model: "test/model" } });
    asked.push(route.request().postDataJSON());
    return route.fulfill({ contentType: "text/event-stream", body: reply });
  });
  await page.goto("/#/chat/ports");
  await page.getByRole("button", { name: "Open Mashy" }).click();
  const mashy = page.getByRole("region", { name: "Mashy" });
  await expect(mashy.getByText("About: Port decision")).toBeVisible();
  await mashy.getByRole("button", { name: "Summarise this chat" }).click();
  await expect(mashy.getByText("You chose port 8080 in")).toBeVisible();
  // What the model wrote is shown as text and sanitised markdown, never as markup.
  await expect(mashy.locator("img[src='x']")).toHaveCount(0);
  expect(asked[0]?.open_chat?.id).toBe("ports");
  expect(asked[0]?.messages).toEqual([{ role: "user", content: "Summarise this chat" }]);
  // The chat it names is a chip that opens it in the dashboard.
  const chip = mashy.locator("a.mashy-chip");
  await expect(chip).toHaveText("Port decision");
  await expect(chip).toHaveAttribute("href", /^#\/chat\/ports\?/);

  // A follow-up carries the conversation so far.
  reply = answer([
    { type: "failed", reason: "limited", message: "The model’s service is limiting requests." },
  ]);
  await mashy.getByRole("textbox").fill("and why?");
  await mashy.getByRole("textbox").press("Enter");
  await expect(mashy.getByRole("alert")).toContainText("The model’s service is busy");
  expect(asked[1]?.messages.map((turn) => turn.role)).toEqual(["user", "assistant", "user"]);
  reply = answer([
    { type: "text", delta: "Because 8080 was free." },
    { type: "done", partial: true },
  ]);
  await mashy.getByRole("button", { name: "Retry" }).click();
  await expect(mashy.getByText("Because 8080 was free.")).toBeVisible();
  await expect(mashy.getByText("There is more than I could read in one go")).toBeVisible();
  // The retry asked the same question again, without the failure in between.
  expect(asked[2]?.messages).toEqual(asked[1]?.messages);

  // Closed, the circle stays; opening it again shows the same conversation.
  await mashy.getByRole("button", { name: "Close Mashy" }).click();
  await expect(mashy).toHaveCount(0);
  await page.getByRole("button", { name: "Open Mashy" }).click();
  await expect(mashy.getByText("Because 8080 was free.")).toBeVisible();
});

test("on a phone the Back button closes Mashy and returns to it from a linked chat", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 700 });
  await page.route("**/api/assistant**", (route) =>
    route.request().method() === "GET"
      ? route.fulfill({ json: { enabled: true, model: "test/model" } })
      : route.fulfill({
          contentType: "text/event-stream",
          body: answer([
            { type: "text", delta: "See [Port decision](chat:ports)." },
            { type: "done", partial: false },
          ]),
        }),
  );
  await page.goto("/#/");
  const mashy = page.getByRole("region", { name: "Mashy" });
  await page.getByRole("button", { name: "Open Mashy" }).click();
  await expect(mashy).toBeVisible();
  await page.goBack();
  await expect(mashy).toHaveCount(0);
  await expect(page).toHaveURL(/#\/$/);

  await page.getByRole("button", { name: "Open Mashy" }).click();
  await mashy.getByRole("textbox").fill("which port?");
  await mashy.getByRole("textbox").press("Enter");
  await mashy.locator("a.mashy-chip").click();
  // The sheet steps aside for the chat, and Back brings the answer back.
  await expect(mashy).toHaveCount(0);
  await expect(page).toHaveURL(/#\/chat\/ports/);
  await page.goBack();
  await expect(mashy.getByText("See")).toBeVisible();
  // Closing it with its own button leaves the page where it was.
  await mashy.getByRole("button", { name: "Close Mashy" }).click();
  await expect(mashy).toHaveCount(0);
  await expect(page).toHaveURL(/#\/$/);
});

test("the Mashy window can be dragged by its header and stays on screen", async ({ page }) => {
  await page.route("**/api/assistant**", (route) =>
    route.fulfill({ json: { enabled: true, model: "test/model" } }),
  );
  await page.setViewportSize({ width: 1200, height: 800 });
  await page.goto("/");
  await page.getByRole("button", { name: "Open Mashy" }).click();
  const panel = page.locator("#mashy");
  const before = await panel.boundingBox();
  if (!before) throw new Error("Mashy did not open");
  await page.mouse.move(before.x + 120, before.y + 20);
  await page.mouse.down();
  await page.mouse.move(before.x - 180, before.y - 80, { steps: 5 });
  await page.mouse.up();
  const after = await panel.boundingBox();
  expect(after?.x).toBeCloseTo(before.x - 300, 0);
  expect(after?.y).toBeCloseTo(before.y - 100, 0);

  // Dragged far past a corner, the whole window is still visible.
  await page.mouse.move((after?.x ?? 0) + 120, (after?.y ?? 0) + 20);
  await page.mouse.down();
  await page.mouse.move(-500, -500, { steps: 5 });
  await page.mouse.up();
  const corner = await panel.boundingBox();
  expect(corner?.x).toBe(8);
  expect(corner?.y).toBe(8);

  // The place is kept across a reload, and a double click puts it back.
  await page.reload();
  await page.getByRole("button", { name: "Open Mashy" }).click();
  expect((await panel.boundingBox())?.x).toBe(8);
  await page.locator("#mashy > header strong").dblclick();
  expect((await panel.boundingBox())?.x).toBeCloseTo(before.x, 0);
});

test("while Mashy looks, a ring turns and what it says changes", async ({ page }) => {
  await page.route("**/api/assistant**", async (route) => {
    if (route.request().method() === "GET")
      return route.fulfill({ json: { enabled: true, model: "test/model" } });
    // Long enough for the second phrase to come round.
    await new Promise((resolve) => setTimeout(resolve, 3200));
    return route.fulfill({
      contentType: "text/event-stream",
      body: answer([
        { type: "text", delta: "Port 8080." },
        { type: "done", partial: false },
      ]),
    });
  });
  await page.goto("/#/chat/ports");
  await page.getByRole("button", { name: "Open Mashy" }).click();
  const mashy = page.getByRole("region", { name: "Mashy" });
  await mashy.getByRole("button", { name: "Summarise this chat" }).click();
  await expect(mashy.locator(".mashy-ring")).toBeVisible();
  // A screen reader hears one steady line, not every change.
  await expect(mashy.getByRole("status")).toHaveText("Searching your history…");
  await expect(mashy.locator(".mashy-phrase")).toHaveText("Searching your history…");
  await expect(mashy.locator(".mashy-phrase")).toHaveText("Reading old chats…");
  await mashy.screenshot({ path: process.env.MASHY_SHOT ?? "test-results/mashy-waiting.png" });
  await expect(mashy.getByText("Port 8080.")).toBeVisible();
  await expect(mashy.locator(".mashy-ring")).toHaveCount(0);
});

test("an answer can quote, point to places, offer the full search and a command to copy", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.route("**/api/assistant**", async (route) => {
    if (route.request().method() === "GET")
      return route.fulfill({ json: { enabled: true, model: "test/model" } });
    return route.fulfill({
      contentType: "text/event-stream",
      body: answer([
        {
          type: "text",
          delta:
            "Yes, in [project](project:/work/project) on [local](machine:local) with [Codex](agent:codex).\n\n" +
            "> “Which port?”\n>\n> — You, 13 Nov 19:00, [Port decision](chat:ports?message=m0)\n\n" +
            "> “Port 8080.”\n>\n> — Codex, 13 Nov 19:01, [Port decision](chat:ports?message=m0)\n\n" +
            "Run `mash index` to pick up newer chats.\n\n" +
            '[See all for "port"](search:port)\n\n§ Which chats mention ports?',
        },
        { type: "done", partial: false },
      ]),
    });
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Open Mashy" }).click();
  const mashy = page.getByRole("region", { name: "Mashy" });
  await mashy.getByRole("button", { name: "What did I work on yesterday?" }).click();
  await expect(mashy.locator("a.mashy-chip.place")).toHaveText(["project", "local", "Codex"]);
  await expect(mashy.locator("a.mashy-chip.place").first()).toHaveAttribute(
    "href",
    "#/project?name=%2Fwork%2Fproject",
  );
  // The reader's own words are marked apart from the agent's.
  await expect(mashy.locator("blockquote.mashy-quote")).toHaveCount(2);
  await expect(mashy.locator("blockquote.mashy-quote.own")).toHaveCount(1);
  await expect(mashy.locator("blockquote.mashy-quote a.mashy-chip").first()).toHaveAttribute(
    "href",
    "#/chat/ports?node=&message=m0",
  );
  await expect(mashy.locator("a.mashy-all")).toHaveAttribute("href", "#/search?q=port");
  await mashy.getByRole("button", { name: "Copy", exact: true }).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe("mash index");
  // A follow-up question carries its icon.
  await expect(mashy.locator(".mashy-sugg.next button svg")).toHaveCount(1);
  await mashy.screenshot({ path: process.env.MASHY_SHOT_RICH ?? "test-results/mashy-rich.png" });
});

test("selected text in a transcript can be asked about", async ({ page }) => {
  const asked: { passage?: string }[] = [];
  await page.route("**/api/assistant**", async (route) => {
    if (route.request().method() === "GET")
      return route.fulfill({ json: { enabled: true, model: "test/model" } });
    asked.push(route.request().postDataJSON());
    return route.fulfill({
      contentType: "text/event-stream",
      body: answer([
        { type: "text", delta: "It asks which port to use." },
        { type: "done", partial: false },
      ]),
    });
  });
  await page.goto("/#/chat/ports");
  const said = page.locator(".chat .hitbox").getByText("Which port?").first();
  await expect(said).toBeVisible();
  await said.evaluate((node) => {
    const range = document.createRange();
    range.selectNodeContents(node);
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
  });
  await page.getByRole("button", { name: "Ask Mashy" }).click();
  const mashy = page.getByRole("region", { name: "Mashy" });
  await expect(mashy.locator(".mashy-passage")).toContainText("Which port?");
  await mashy.screenshot({ path: process.env.MASHY_SHOT_PICK ?? "test-results/mashy-pick.png" });
  await mashy.getByRole("button", { name: "Explain this in plain words" }).click();
  await expect(mashy.getByText("It asks which port to use.")).toBeVisible();
  expect(asked[0]?.passage).toBe("Which port?");
  // Letting the passage go stops sending it.
  await mashy.getByRole("button", { name: "Stop asking about this passage" }).click();
  await expect(mashy.locator(".mashy-passage")).toHaveCount(0);
});
