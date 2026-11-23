import { expect, test } from "@playwright/test";
import { fitsViewport } from "./shell";

const tools = {
  build_revision: "0123456789abcdef",
  contract_fingerprint: "fedcba9876543210",
  tools: [
    {
      name: "forget",
      description: "Remove one chat from the index.",
      annotations: { readOnlyHint: false, destructiveHint: true },
      inputSchema: {
        type: "object",
        required: ["chat_id"],
        properties: { chat_id: { type: "string" } },
      },
    },
    {
      name: "search",
      description: "Find chats by what was said in them. It never indexes.",
      annotations: { readOnlyHint: true, destructiveHint: false },
      inputSchema: {
        type: "object",
        required: ["query"],
        properties: {
          query: { type: "string", description: "Words to look for." },
          limit: { type: ["integer", "null"], default: null, description: "How many." },
          unit: { type: "string", enum: ["chats", "messages"] },
        },
      },
    },
  ],
};

for (const width of [390, 1280]) {
  test(`the console lists every tool, runs readers and locks writers at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    const calls: URLSearchParams[] = [];
    let busy = true;
    await page.route("**/api/**", (route) => {
      const url = new URL(route.request().url());
      expect(route.request().method()).toBe("GET");
      if (url.pathname === "/api/console/tools") return route.fulfill({ json: tools });
      if (url.pathname === "/api/console/call") {
        calls.push(url.searchParams);
        if (busy) {
          busy = false;
          return route.fulfill({
            json: {
              failed: true,
              reply: {
                error: {
                  kind: "index-busy",
                  message: "the index is being updated",
                  retryable: true,
                },
              },
              text: "",
            },
          });
        }
        return route.fulfill({
          json: {
            failed: false,
            reply: {
              results: [1, 2, 3, 4, 5].map((n) => ({ title: `<b>chat ${n}</b>` })),
              next_offset: null,
            },
            text: "",
          },
        });
      }
      return route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } });
    });
    await page.goto("/#/console");
    await expect(page.getByRole("heading", { name: "Tool explorer", exact: true })).toBeVisible();
    await expect(
      page.getByText("2 tools · 1 can run here · 1 change data and are locked"),
    ).toBeVisible();

    // A tool that changes data shows its parameters but cannot be run.
    await page.getByRole("button", { name: /forget/ }).click();
    const locked = page.locator("#tool-forget");
    await expect(locked.getByText("chat_id")).toBeVisible();
    await expect(locked.getByRole("button", { name: "Run" })).toHaveCount(0);
    await expect(locked.getByRole("textbox")).toHaveCount(0);

    await page.getByRole("button", { name: /search/ }).click();
    const search = page.locator("#tool-search");
    const run = search.getByRole("button", { name: "Run", exact: true });
    // A required parameter left empty names itself and holds Run back.
    await expect(run).toBeDisabled();
    await expect(search.getByText("query is required.")).toBeVisible();
    await search.getByRole("textbox").first().fill("watcher loop");
    await search.getByRole("textbox").nth(1).fill("5");
    await search.getByRole("combobox").selectOption("messages");
    await expect(search.getByLabel("Response")).toHaveCount(0);
    await run.click();
    // A reply that says it is worth retrying reads as busy, not as broken.
    const response = search.getByRole("region", { name: "Response" });
    await expect(response).toContainText("Busy");
    await expect(response.getByRole("alert")).toHaveText("the index is being updated");
    await run.click();
    await expect(response).toContainText("OK");
    await expect(response).toContainText("5 items");
    expect(JSON.parse(calls[1]?.get("arguments") ?? "")).toEqual({
      query: "watcher loop",
      limit: 5,
      unit: "messages",
    });
    expect(calls[1]?.get("name")).toBe("search");
    // Archived text in a reply is shown as text, and long lists fold.
    await expect(response.getByText('"<b>chat 1</b>"')).toBeVisible();
    await expect(response.locator("b")).toHaveCount(0);
    await expect(response.getByText("chat 2")).toHaveCount(0);
    await response.getByRole("button", { name: "show 4 more of 4" }).click();
    await expect(response.getByText('"<b>chat 5</b>"')).toBeVisible();

    await page.getByRole("searchbox", { name: "Filter tools" }).fill("forg");
    await expect(page.locator("#tool-search")).toHaveCount(0);
    await expect(page.locator("#tool-forget")).toBeVisible();
    expect(await fitsViewport(page)).toBe(true);
  });
}

test("tools start folded, fold long option lists and style every field alike", async ({ page }) => {
  const optional = Object.fromEntries(
    ["after", "before", "days", "local", "unit"].map((name) => [
      name,
      name === "local"
        ? { type: "boolean", description: "Search only the local index." }
        : name === "unit"
          ? { type: "string", enum: ["chats", "messages"], description: "Chats or messages." }
          : { type: "integer", description: "A bound." },
    ]),
  );
  const listing = {
    ...tools,
    tools: [
      {
        name: "search",
        description:
          "Find past chats. Drill down with unit=messages and copy lookup.view into `view`.",
        annotations: { readOnlyHint: true },
        inputSchema: {
          type: "object",
          required: ["query"],
          properties: { query: { type: "string", description: "Words." }, ...optional },
        },
      },
    ],
  };
  await page.route("**/api/**", (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/console/tools") return route.fulfill({ json: listing });
    return route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } });
  });
  await page.goto("/#/console");
  const button = page.getByRole("button", { name: /search/ });
  await expect(button).toHaveAttribute("aria-expanded", "false");
  await button.click();
  const search = page.locator("#tool-search");
  // The notes for agents are kept but folded, with names in code type.
  const notes = search.getByText("How agents use it (1 note)");
  await expect(search.getByText("unit=messages")).toBeHidden();
  await notes.click();
  await expect(search.locator(".op-d code")).toHaveText(["unit=messages", "lookup.view", "view"]);
  // The required field shows; the five optional ones wait behind a link.
  await expect(search.getByRole("textbox")).toHaveCount(1);
  await search.getByText("5 more options").click();
  await expect(search.getByRole("textbox")).toHaveCount(4);
  const choice = search.getByRole("combobox").first();
  await expect(choice).toHaveClass(/unset/);
  const look = (selector: string) =>
    search
      .locator(selector)
      .first()
      .evaluate((node) => {
        const style = getComputedStyle(node);
        return { weight: style.fontWeight, appearance: style.appearance, color: style.color };
      });
  const field = await look("input.prm-i[placeholder='not set']");
  const select = await look("select.prm-i");
  expect(select.weight).toBe(field.weight);
  expect(select.appearance).toBe("none");
  await choice.selectOption("true");
  await expect(choice).not.toHaveClass(/unset/);
});
