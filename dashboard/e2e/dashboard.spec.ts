import { expect, type Locator, test } from "@playwright/test";
import { searchBox } from "./shell";

test.beforeEach(async ({ page }) => {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
});

const chat = {
  chat_id: "same-id",
  provider: "codex",
  title: "Real fixture chat",
  project: "/work/project",
  updated_at: Date.now(),
};
test("direct chats load before the catalogue and tool output stays literal", async ({ page }) => {
  let release: (() => void) | undefined;
  const catalogReady = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/api/chats**", async (route) => {
    await catalogReady;
    await route.fulfill({
      json: { local_node: "local", truncated: false, chats: [{ ...chat, node_id: "local" }] },
    });
  });
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: {
        messages: [
          {
            id: "reply",
            ordinal: 0,
            role: "assistant",
            content:
              '## A readable heading\n\n- First\n- Second\n\n```rust\nfn main() {\n    println!("hello");\n}\n```',
          },
          {
            id: "tool",
            ordinal: 1,
            role: "user",
            content: "# README\n\n<div>literal HTML</div>",
            tools: { tool_results: [{ call_id: "call" }] },
          },
        ],
      },
    }),
  );
  await page.goto("/#/chat/same-id");
  try {
    await expect(page.getByRole("heading", { name: "A readable heading" })).toBeVisible();
    await expect(page.locator("main pre code")).toContainText('    println!("hello");');
    await expect(page.getByRole("heading", { name: "README" })).toHaveCount(0);
    // Tool output is folded into a run of steps; both levels open on request.
    const steps = page.getByRole("group").filter({ hasText: "1 tool call" }).first();
    await steps.locator("summary").first().click();
    await steps.locator("details.tc > summary").click();
    await expect(steps.locator("pre")).toContainText("# README");
    await expect(steps.locator("pre")).toContainText("<div>literal HTML</div>");
    await expect(steps.locator("pre *")).toHaveCount(0);
  } finally {
    release?.();
  }
  await expect(page.getByRole("heading", { name: chat.title })).toBeVisible();
});
test("direct search matches load without waiting for catalogue identity", async ({ page }) => {
  let release: (() => void) | undefined;
  const ready = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/api/chats**", async (route) => {
    await ready;
    await route.fulfill({ json: { local_node: "workstation", truncated: false, chats: [] } });
  });
  await page.route("**/api/view?**", (route) => {
    const params = new URL(route.request().url()).searchParams;
    expect(params.get("node")).toBe("workstation");
    expect(params.get("id")).toBe("anchor");
    return route.fulfill({
      json: {
        messages: [{ id: "anchor", ordinal: 42, role: "assistant", content: "Selected message" }],
      },
    });
  });
  await page.goto("/#/chat/same-id?node=workstation&message=anchor");
  try {
    await expect(page.getByText("Selected message", { exact: true })).toBeVisible();
  } finally {
    release?.();
  }
});

for (const shared of [false, true]) {
  const source = shared ? "shared" : "local";
  test.describe(`${source} message links`, () => {
    test.use({ hasTouch: shared, viewport: { width: shared ? 390 : 1280, height: 850 } });

    test(`copy a ${source} message link and reopen its exact source`, async ({ context, page }) => {
      const node = shared ? "workstation & build" : "laptop";
      const messages = [
        { id: "prompt", ordinal: 0, role: "user", content: "Keep the trim test." },
        { id: "reply/2", ordinal: 1, role: "assistant", content: "I will keep that test." },
      ] as const;
      const views: URLSearchParams[] = [];
      await context.grantPermissions(["clipboard-read", "clipboard-write"]);
      await context.route("**/api/**", (route) => route.fulfill({ status: 404 }));
      await context.route("**/api/chats**", (route) =>
        route.fulfill({
          json: {
            local_node: "laptop",
            truncated: false,
            chats: [{ ...chat, node_id: node }],
          },
        }),
      );
      await context.route("**/api/chat?**", (route) => route.fulfill({ json: { messages } }));
      await context.route("**/api/records?**", (route) =>
        route.fulfill({ json: { records: [], next_offset: null } }),
      );
      await context.route("**/api/view?**", (route) => {
        views.push(new URL(route.request().url()).searchParams);
        return route.fulfill({ json: { messages } });
      });
      const params = new URLSearchParams({ node, ...(shared ? { shared: "true" } : {}) });
      await page.goto(`/?transient=keep-out-of-link#/chat/${chat.chat_id}?${params}`);
      const message = messages[shared ? 1 : 0];
      const row = page.locator("[data-message]").filter({ hasText: message.content });
      await row.getByRole("button", { name: "Copy message", exact: true }).press("Enter");
      await expect(page.getByText("Copied message.", { exact: true })).toBeVisible();
      expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(message.content);
      const copy = row.getByRole("button", { name: "Copy link to message" });
      if (shared) await copy.tap();
      else await copy.press("Enter");
      await expect(page.getByText("Copied message link.", { exact: true })).toBeVisible();
      const url = new URL(await page.evaluate(() => navigator.clipboard.readText()));
      expect(url.origin).toBe(new URL(page.url()).origin);
      expect(url.pathname).toBe("/");
      expect(url.search).toBe("");

      const reopened = await context.newPage();
      try {
        await reopened.goto(url.href);
        await expect(reopened.locator(".hitbox.hit")).toContainText(message.content);
        expect(views.at(-1)?.get("id")).toBe(message.id);
        expect(views.at(-1)?.get("node")).toBe(node);
        expect(views.at(-1)?.get("shared")).toBe(shared ? "true" : null);
      } finally {
        await reopened.close();
      }
    });
  });
}

test("real API shapes, explicit search, source identity, safe content and stable refresh", async ({
  page,
}) => {
  // Network HTTP previews do not expose this secure-context-only API.
  await page.addInitScript(() => {
    Object.defineProperty(crypto, "randomUUID", { value: undefined });
  });
  let title = chat.title;
  let content =
    "Initial **reply**\n\n<script>window.pwned = true</script><img src=https://evil.example/pixel onerror=alert(1)>[bad](javascript:alert(1))";
  let searchRequests = 0;
  let catalogueRequests = 0;
  const chatRequests: URL[] = [];
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/chats") {
      catalogueRequests++;
      await route.fulfill({
        json: {
          local_node: "local",
          truncated: false,
          shared_error: null,
          chats: [
            { ...chat, title, node_id: "local" },
            { ...chat, node_id: "remote" },
          ],
        },
      });
    } else if (url.pathname === "/api/search") {
      searchRequests++;
      await route.fulfill({
        json: {
          results: [
            {
              ...chat,
              node_id: "remote",
              matches: [{ message_id: "match-id", snippet: "matching snippet" }],
            },
          ],
          next_offset: null,
        },
      });
    } else if (url.pathname === "/api/records") {
      await route.fulfill({ json: { records: [], next_offset: null } });
    } else {
      chatRequests.push(url);
      // A chat of a hundred messages that does not state its length, so the
      // reader finds its end by reading past it.
      const messages = [
        { id: "match-id", ordinal: 0, role: "assistant", created_at: Date.now(), content },
        ...Array.from({ length: 99 }, (_, index) => ({
          id: `later-${index}`,
          ordinal: index + 1,
          role: "assistant",
          content: `Later reply ${index}`,
        })),
      ];
      const offset = Number(url.searchParams.get("offset") ?? 0);
      const limit = Number(url.searchParams.get("limit") ?? 100);
      await route.fulfill({
        json: {
          id: "same-id",
          messages:
            url.pathname === "/api/chat"
              ? messages.slice(offset, offset + limit)
              : messages.slice(0, 1),
        },
      });
    }
  });
  await page.clock.install();
  await page.goto("/");
  const main = page.getByRole("main");
  await expect(main.getByRole("link", { name: /Real fixture chat/ })).toHaveCount(2);
  title = "Refreshed catalogue";
  await page.clock.runFor(31_000);
  await expect(page.getByRole("link", { name: /Refreshed catalogue/ })).toHaveCount(0);
  expect(catalogueRequests).toBe(1);
  await searchBox(page).fill("needle");
  expect(searchRequests).toBe(0);
  await searchBox(page).press("Enter");
  await expect(page.getByText("matching snippet")).toBeVisible();
  expect(searchRequests).toBe(1);
  await searchBox(page).fill("unsubmitted draft");
  await page.clock.runFor(31_000);
  expect(searchRequests).toBe(1);
  await expect(page.getByText("matching snippet")).toBeVisible();
  await main.getByRole("link", { name: /Real fixture chat/ }).click();
  const reply = page.locator("#message-match-id");
  await expect(reply.getByText("Initial", { exact: false })).toBeVisible();
  expect(chatRequests.at(-1)?.searchParams.get("node")).toBe("remote");
  expect(
    await page.locator(".prose script, .prose img, .prose a[href^='javascript:']").count(),
  ).toBe(0);
  expect(await page.evaluate(() => "pwned" in window)).toBe(false);
  await expect(reply.locator("strong")).toHaveText("reply");
  content = "Updated reply";
  await page.locator("#rail").getByRole("button", { name: "Check for updates" }).click();
  await expect(page.getByRole("button", { name: /This chat has changed/ })).toBeVisible();
  await expect(page.getByText("Updated reply", { exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: /This chat has changed/ }).click();
  await expect(page.getByText("Updated reply", { exact: true })).toBeVisible();
  // Reading on past the linked message asks the same machine for the rest.
  await expect(page.locator("#message-later-98")).toBeAttached();
  const reads = chatRequests.filter((url) => url.pathname === "/api/chat");
  expect(reads.map((url) => url.searchParams.get("offset"))).toContain("1");
  expect(reads.every((url) => url.searchParams.get("node") === "remote")).toBe(true);
  expect(errors).toEqual([]);
});

// Chat lists follow the event stream. Where it is unavailable, the rail still
// offers a way to ask.
test("the reader can ask for new chats without reloading the document", async ({ page }) => {
  let title = chat.title;
  let documents = 0;
  page.on("request", (request) => {
    if (request.resourceType() === "document") documents++;
  });
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: [{ ...chat, title, node_id: "local" }],
      },
    }),
  );
  await page.goto("/");
  const main = page.getByRole("main");
  await expect(main.getByRole("link", { name: /Real fixture chat/ })).toBeVisible();
  title = "Refreshed catalogue";
  await page.locator("#rail").getByRole("button", { name: "Check for updates" }).click();
  await expect(main.getByRole("link", { name: /Refreshed catalogue/ })).toBeVisible();
  expect(documents).toBe(1);
});

test("chat navigation retains the sidebar and refresh requires an explicit request", async ({
  page,
}) => {
  let requests = 0;
  let documents = 0;
  page.on("request", (request) => {
    // No dashboard answers here, so the event stream keeps trying to connect;
    // one that is connected and quiet asks for nothing.
    if (request.url().includes("/api/") && !request.url().includes("/api/events")) requests++;
    if (request.resourceType() === "document") documents++;
  });
  let content = "Original answer";
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: Array.from({ length: 80 }, (_, index) => ({
          ...chat,
          chat_id: `chat-${index}`,
          title: `Chat ${index}`,
          node_id: "local",
        })),
      },
    }),
  );
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: { messages: [{ id: "answer", ordinal: 0, role: "assistant", content }] },
    }),
  );
  await page.clock.install();
  await page.goto("/#/chat/chat-0?node=local");
  await expect(page.getByText("Original answer", { exact: true })).toBeVisible();
  const sidebar = page.getByRole("navigation", { name: "Prompts and sessions" });
  await expect(sidebar.getByRole("link", { name: /Chat 0/ })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await sidebar.evaluate((element) => element.setAttribute("data-retained", "yes"));
  await page.evaluate(() => {
    location.hash = "#/chat/chat-1?node=local";
  });
  await expect(page.getByRole("heading", { name: "Chat 1", exact: true })).toBeVisible();
  await expect(sidebar).toHaveAttribute("data-retained", "yes");
  await expect(sidebar.getByRole("link", { name: /Chat 1/ })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByText("Original answer", { exact: true })).toBeVisible();
  content = "Background replacement";
  await page.waitForLoadState("networkidle");
  const beforeIdle = requests;
  await page.clock.runFor(91_000);
  expect(requests).toBe(beforeIdle);
  expect(documents).toBe(1);
  await expect(page.getByRole("button", { name: /This chat has changed/ })).toHaveCount(0);
  await page.locator("#rail").getByRole("button", { name: "Check for updates" }).click();
  await expect(page.getByRole("button", { name: /This chat has changed/ })).toBeVisible();
  await expect(page.getByText("Original answer", { exact: true })).toBeVisible();
  await expect(page.getByText("Background replacement", { exact: true })).toHaveCount(0);
  await expect(sidebar).toHaveAttribute("data-retained", "yes");
});

test("switching a scrolled chat keeps the top bar and sidebar stationary", async ({ page }) => {
  let documents = 0;
  page.on("request", (request) => {
    if (request.isNavigationRequest() && request.frame() === page.mainFrame()) documents++;
  });
  let release: (() => void) | undefined;
  const ready = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: Array.from({ length: 50 }, (_, i) => ({
          ...chat,
          chat_id: `chat-${i}`,
          title: `Chat ${i}`,
          node_id: "local",
        })),
      },
    }),
  );
  await page.route("**/api/chat?**", async (route) => {
    const id = new URL(route.request().url()).searchParams.get("id");
    if (id === "chat-4") await ready;
    await route.fulfill({
      json: {
        messages: [
          {
            id: `message-${id}`,
            ordinal: 0,
            role: "assistant",
            content: "Long paragraph.\n\n".repeat(120),
          },
        ],
      },
    });
  });
  await page.goto("/#/chat/chat-0?node=local");
  await page.locator("#message-message-chat-0").waitFor();
  // Chats open at their end, which can be part of a pixel down, so places
  // are compared to the whole pixel.
  const place = async (locator: Locator) => {
    const box = await locator.boundingBox();
    return (
      box && Object.fromEntries(Object.entries(box).map(([key, value]) => [key, Math.round(value)]))
    );
  };
  const header = searchBox(page);
  const mainNav = page.getByRole("navigation", { name: "Main" });
  const sidebar = page.getByRole("navigation", { name: "Prompts and sessions" });
  // The sidebar lists the project's six newest sessions.
  await expect(sidebar.getByRole("link", { name: /^Chat \d/ })).toHaveCount(6);
  // The chat opened at its end; measure from the top of the page.
  await page.evaluate(() => window.scrollTo(0, 0));
  const initialHeader = await place(header);
  const initialNav = await place(mainNav);
  const initialSidebar = await place(sidebar);
  await sidebar.evaluate((element) => element.setAttribute("data-retained", "yes"));
  await page.evaluate(() => window.scrollTo(0, 600));
  expect(await page.evaluate(() => window.scrollY)).toBe(600);
  // Sticky chrome stays where it was while the transcript scrolls under it.
  expect(await place(header)).toEqual(initialHeader);
  expect(await place(mainNav)).toEqual(initialNav);
  try {
    await sidebar.getByRole("link", { name: /Chat 4/ }).click();
    await expect(page.getByText("Loading messages…", { exact: true })).toBeVisible();
    expect(await place(header)).toEqual(initialHeader);
    expect(await place(mainNav)).toEqual(initialNav);
    await expect(sidebar).toHaveAttribute("data-retained", "yes");
    // A newly opened chat starts at its top, with the sidebar where it began.
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
    expect(await place(sidebar)).toEqual(initialSidebar);
    expect(documents).toBe(1);
  } finally {
    release?.();
  }
  await page.locator("#message-message-chat-4").waitFor();
  expect(await place(header)).toEqual(initialHeader);
  expect(await place(mainNav)).toEqual(initialNav);
  expect(documents).toBe(1);
});

test("Export downloads the entire chat as safe HTML and Markdown", async ({ page }) => {
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: { local_node: "local", truncated: false, chats: [{ ...chat, node_id: "local" }] },
    }),
  );
  await page.route("**/api/chat?**", (route) => {
    const offset = Number(new URL(route.request().url()).searchParams.get("offset"));
    return route.fulfill({
      json: {
        messages:
          offset >= 1000
            ? []
            : [
                {
                  id: `message-${offset}`,
                  ordinal: offset,
                  role: "assistant",
                  content:
                    offset === 0
                      ? "First **message** <script>window.pwned=true</script><img src=https://evil.example/pixel>"
                      : "Last message on another page",
                },
              ],
      },
    });
  });
  await page.goto("/#/chat/same-id?node=local");
  await expect(page.getByRole("heading", { name: chat.title })).toBeVisible();
  const openExport = page.getByRole("button", { name: "Export", exact: true });
  for (const format of ["Markdown", "HTML"]) {
    await openExport.click();
    const downloaded = page.waitForEvent("download");
    await page.getByRole("button", { name: `Export to ${format}`, exact: true }).click();
    const download = await downloaded;
    const stream = await download.createReadStream();
    if (!stream) throw new Error("Missing export download");
    let content = "";
    for await (const chunk of stream) content += chunk.toString();
    expect(content).toContain("Last message on another page");
    if (format === "HTML") {
      expect(content).toContain("<strong>message</strong>");
      expect(content).not.toContain("<script>");
      expect(content).not.toContain("<img");
    }
    await expect(page.getByRole("status").filter({ hasText: "Exported" })).toBeVisible();
  }
  await openExport.click();
  await expect(page.getByRole("button", { name: "Export to PDF", exact: true })).toBeEnabled();
  await page.context().addInitScript(() => {
    window.print = () => {};
  });
  const opened = page.waitForEvent("popup");
  await page.getByRole("button", { name: "Export to PDF", exact: true }).click();
  const preview = await opened;
  await expect(preview.getByText("Last message on another page", { exact: true })).toBeVisible();
  await expect(preview.locator("script, img")).toHaveCount(0);
  const pdf = await preview.pdf();
  expect(pdf.subarray(0, 5).toString()).toBe("%PDF-");
  await preview.close();
});

test("network errors are visible and recoverable", async ({ page }) => {
  await page.route("**/api/chats**", (route) =>
    route.fulfill({ status: 503, body: "unavailable" }),
  );
  await page.goto("/");
  await expect(page.getByRole("alert")).toContainText("503");
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: { local_node: "local", truncated: false, shared_error: "remote offline", chats: [] },
    }),
  );
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await expect(page.getByText("No indexed chats yet.", { exact: false })).toBeVisible();
  await expect(page.getByRole("status").filter({ hasText: "remote offline" })).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});
