import process from "node:process";
import { expect, type Page, test } from "@playwright/test";
import { fitsViewport } from "./shell";
import { streamFromSearch } from "./stream";

// Subagent chats fold into the chat that started them, open on their own
// pages, and stay in search results labelled with their parent.
const now = Date.now();
const parent = {
  chat_id: "session",
  node_id: "laptop",
  provider: "claude-code",
  title: "Ship the 0.2.0 release",
  project: "/work/mash",
  created_at: now - 3_600_000,
  updated_at: now - 60_000,
  subagents: 2,
};
const windows = {
  chat_id: "agent-windows",
  node_id: "laptop",
  provider: "claude-code",
  title: "Work ONLY in /work/mash and fix the Windows tests",
  project: "/work/mash",
  created_at: now - 3_500_000,
  updated_at: now - 120_000,
  subagent: {
    parent_chat_id: "session",
    parent_title: parent.title,
    label: "Fix the failing Windows test suite",
    role: "general-purpose",
    finished: true,
    spawn_call_id: "toolu_windows",
  },
};
const notes = {
  ...windows,
  chat_id: "agent-notes",
  title: "Summarise the changes",
  created_at: now - 3_400_000,
  updated_at: now - 180_000,
  subagent: { parent_chat_id: "session", label: "Draft the release notes", role: "Explore" },
};
// A Codex subagent whose parent ran on another machine.
const orphan = {
  chat_id: "codex-child",
  node_id: "laptop",
  provider: "codex",
  title: "Check the links",
  project: "/work/docs",
  updated_at: now - 240_000,
  subagent: { parent_chat_id: "elsewhere", label: "Popper", role: "explorer" },
};
const plain = {
  chat_id: "plain",
  node_id: "laptop",
  provider: "codex",
  title: "Work ONLY in a folder: an ordinary chat",
  project: "/work/docs",
  updated_at: now - 300_000,
};
const chats = [parent, windows, notes, orphan, plain];

async function mock(page: Page): Promise<void> {
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/records")
      return route.fulfill({ json: { records: [], next_offset: null } });
    if (url.pathname === "/api/chats")
      return route.fulfill({ json: { local_node: "laptop", truncated: false, chats } });
    if (url.pathname === "/api/chat") {
      const id = url.searchParams.get("id");
      const messages =
        id === "session"
          ? [
              {
                id: "ask",
                ordinal: 0,
                role: "user",
                content: "get the windows tests green and cut 0.2.0",
              },
              {
                id: "spawn",
                ordinal: 1,
                role: "assistant",
                content: "",
                tools: {
                  tool_calls: [
                    {
                      call_id: "toolu_windows",
                      name: "Agent",
                      input: { description: "Fix the failing Windows test suite" },
                    },
                  ],
                },
              },
              {
                id: "out",
                ordinal: 2,
                role: "user",
                content: "done",
                tools: { tool_results: [{ call_id: "toolu_windows" }] },
              },
              { id: "tagged", ordinal: 3, role: "assistant", content: "Tagged v0.2.0." },
            ]
          : [
              {
                id: "brief",
                ordinal: 0,
                role: "user",
                content: "Find which commits broke each test.",
              },
              {
                id: "fixed",
                ordinal: 1,
                role: "assistant",
                content: "Three tests asserted old wording.",
              },
            ];
      return route.fulfill({ json: { message_count: messages.length, messages } });
    }
    if (url.pathname === "/api/search")
      // As Mash does, subagent chats are left out unless the search asks.
      return route.fulfill({
        json: {
          results: [
            ...(url.searchParams.get("include_subagents") === "true"
              ? [
                  {
                    ...windows,
                    score: 1,
                    matches: [
                      {
                        message_id: "fixed",
                        node_id: "laptop",
                        snippet: "three tests asserted old wording",
                      },
                    ],
                  },
                ]
              : []),
            {
              ...parent,
              score: 0.5,
              matches: [{ message_id: "ask", node_id: "laptop", snippet: "the windows tests" }],
            },
          ],
          next_offset: null,
        },
      });
    return route.fulfill({ json: [] });
  });
}

async function shoot(page: Page, name: string): Promise<void> {
  const width = page.viewportSize()?.width ?? 0;
  expect(await fitsViewport(page)).toBe(true);
  // Pictures for comparing with the design, only when asked for.
  const directory = process.env.MASH_SCREENSHOTS;
  if (!directory) return;
  await page.screenshot({
    path: `${directory}/${test.info().project.name || "chromium"}-${name}-${width}.png`,
    fullPage: true,
  });
}

for (const width of [1440, 390]) {
  test.describe(`at ${width}px`, () => {
    test.use({ viewport: { width, height: width > 500 ? 1000 : 844 } });
    test.beforeEach(({ page }) => mock(page));

    test("the chat list folds subagents into their parent until asked", async ({ page }) => {
      await page.goto("/#/search");
      const list = page.locator("#reslist");
      await expect(list.getByText(parent.title)).toBeVisible();
      await expect(list.locator(".chatrow").filter({ hasText: parent.title })).toContainText(
        "2 subagents",
      );
      await expect(list.getByText(windows.subagent.label)).toHaveCount(0);
      // A subagent whose parent is not here stays a row of its own.
      await expect(list.getByText("Popper")).toBeVisible();
      await expect(list.getByText("subagent of a chat not on this machine")).toBeVisible();
      // A title that reads like a brief is no evidence.
      await expect(list.getByText(plain.title)).toBeVisible();
      await expect(page.getByText("3 chats · 2 subagent chats folded in")).toBeVisible();
      await shoot(page, "list-folded");
      await page.goto("/#/search?subagents=shown");
      await expect(list.getByText(windows.subagent.label)).toBeVisible();
      await expect(
        list.locator(".chatrow").filter({ hasText: windows.subagent.label }),
      ).toContainText(`subagent of ${parent.title}`);
      await expect(page.getByText("5 chats", { exact: true })).toBeVisible();
    });

    test("a parent lists its subagents and links the call that started one", async ({ page }) => {
      await page.goto("/#/chat/session");
      await expect(page.getByRole("heading", { name: parent.title, level: 1 })).toBeVisible();
      const spawn = page.locator("a.fold.spawn");
      await expect(spawn).toContainText(`Started a subagent: ${windows.subagent.label}`);
      await expect(spawn).toContainText("done");
      await expect(spawn).toHaveAttribute("href", /agent-windows/);
      if (width > 700) {
        const panel = page.getByRole("region", { name: "Subagents" });
        await expect(panel.getByRole("heading")).toHaveText("subagents (2)");
        await expect(panel).toContainText("general-purpose");
        await expect(panel).toContainText(notes.subagent.label);
      }
      await shoot(page, "parent");
    });

    test("a subagent names its task and links back to its parent", async ({ page }) => {
      await page.goto("/#/chat/agent-windows");
      await expect(
        page.getByRole("heading", { name: windows.subagent.label, level: 1 }),
      ).toBeVisible();
      const back = page.getByRole("link", { name: new RegExp(`Subagent of ${parent.title}`) });
      await expect(back).toHaveAttribute("href", /#\/chat\/session/);
      await expect(page.locator(".chat-m")).toContainText("subagent · general-purpose");
      await expect(page.locator(".chat-m .pill.ok")).toHaveText("done");
      await shoot(page, "child");
      await page.goto("/#/chat/codex-child");
      await expect(page.getByRole("heading", { name: "Popper", level: 1 })).toBeVisible();
      await expect(page.getByText("Subagent of a chat not on this machine")).toBeVisible();
    });

    test("search leaves subagent chats out unless asked, then labels them", async ({ page }) => {
      await streamFromSearch(page);
      await page.goto("/#/search?q=windows+tests");
      const results = page.locator(".results");
      await expect(
        results.locator("a.card").filter({ hasText: parent.title }).first(),
      ).toBeVisible();
      await expect(
        results.locator("a.card").filter({ hasText: windows.subagent.label }),
      ).toHaveCount(0);
      await expect(page.locator(".chip", { hasText: "subagents: hidden" })).toBeAttached();
      await shoot(page, "search");
      // Asked to, a search includes them, each labelled with its parent.
      await page.goto("/#/search?q=windows+tests&subagent_hits=included");
      const hit = results.locator("a.card").filter({ hasText: windows.subagent.label });
      await expect(hit).toContainText(`subagent of ${parent.title}`);
      await expect(page.locator(".chip", { hasText: "subagents: included" })).toBeAttached();
      await shoot(page, "search-included");
    });
  });
}

test("a session copied over SSH is listed once, from the machine it ran on", async ({ page }) => {
  const original = {
    chat_id: "remote-session",
    node_id: "xenia",
    provider: "claude-code",
    title: "Deploy from the server",
    project: "/home/james/app",
    updated_at: now - 60_000,
  };
  const copy = { ...original, node_id: "laptop", ssh_copy: true };
  const lone = { ...copy, chat_id: "lone-copy", title: "Only the copy is here" };
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/records")
      return route.fulfill({ json: { records: [], next_offset: null } });
    if (url.pathname === "/api/chats")
      return route.fulfill({
        json: { local_node: "laptop", truncated: false, chats: [copy, original, lone] },
      });
    return route.fulfill({ json: [] });
  });
  await page.goto("/#/search");
  const list = page.locator("#reslist");
  const rows = list.locator(".chatrow").filter({ hasText: original.title });
  await expect(rows).toHaveCount(1);
  await expect(rows).toContainText("xenia");
  await expect(rows).not.toContainText("ran over SSH");
  await expect(list.locator(".chatrow").filter({ hasText: lone.title })).toContainText(
    "ran over SSH",
  );
  await expect(page.getByText("2 chats", { exact: true })).toBeVisible();
});
