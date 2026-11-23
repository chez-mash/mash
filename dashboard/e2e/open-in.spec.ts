import { expect, test } from "@playwright/test";

// A chat of this machine opens in its agent's desktop app when the server
// names a link for it; without one, its session can be resumed in a terminal.
// Chats of other machines offer neither, since both act on this computer.
const codex = "01a1b649-827e-7692-a5b1-0de415a69df5";
const claude = "586f44fa-8aaa-4587-b69f-b763d3279ef1";
const elsewhere = "11111111-2222-3333-4444-555555555555";

const row = (chat_id: string, provider: string, node_id: string) => ({
  chat_id,
  node_id,
  provider,
  title: `${provider} fixture on ${node_id}`,
  project: "/work/project",
  updated_at: Date.now(),
});

let asked: string[] = [];

test.beforeEach(async ({ page }) => {
  asked = [];
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "laptop",
        truncated: false,
        chats: [
          row(codex, "codex", "laptop"),
          row(claude, "claude-code", "laptop"),
          row(elsewhere, "codex", "desk"),
        ],
      },
    }),
  );
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: {
        message_count: 1,
        messages: [{ id: "m0", ordinal: 0, role: "user", content: "Hello there." }],
      },
    }),
  );
  await page.route("**/api/open-in?**", (route) => {
    const url = new URL(route.request().url());
    const id = url.searchParams.get("id") ?? "";
    asked.push(id);
    route.fulfill({
      json: {
        link:
          id === codex && url.searchParams.get("agent") === "codex"
            ? { app: "Codex", url: `codex://threads/${id}` }
            : null,
      },
    });
  });
});

test("a Codex chat of this machine links to its thread in the Codex app", async ({ page }) => {
  await page.goto(`/#/chat/${codex}`);
  const open = page.getByRole("link", { name: "Open in Codex" });
  await expect(open).toHaveAttribute("href", `codex://threads/${codex}`);
  await expect(page.getByRole("button", { name: "Copy resume command" })).toHaveCount(0);
});

test("a chat no app opens offers the command that resumes it", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto(`/#/chat/${claude}`);
  const copy = page.getByRole("button", { name: "Copy resume command" });
  await copy.click();
  await expect(
    page.getByText("Resume command copied. Run it in the chat’s project folder."),
  ).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    `claude --resume ${claude}`,
  );
  await expect(page.getByRole("link", { name: /^Open in / })).toHaveCount(0);
});

test("a chat of another machine offers neither, and is not asked about", async ({ page }) => {
  await page.goto(`/#/chat/${elsewhere}?node=desk`);
  await expect(page.getByText("Hello there.")).toBeVisible();
  await expect(page.getByRole("link", { name: /^Open in / })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Copy resume command" })).toHaveCount(0);
  expect(asked).not.toContain(elsewhere);
});
