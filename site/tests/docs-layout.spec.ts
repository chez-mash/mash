import { expect, test } from "@playwright/test";

test("a docs page shows where it sits, and its table of contents skips the title", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/tools/");
  await expect(
    page.getByRole("navigation", { name: "Breadcrumb" }).getByRole("listitem"),
  ).toHaveText(["Using Mash", "Reference", "MCP tool reference"]);
  const contents = page.locator("starlight-toc");
  await expect(contents.getByRole("link", { name: "Overview" })).toBeHidden();
  await expect(contents.getByRole("link", { name: "Search and read chats" })).toBeVisible();
});

test("the quick start draws its callout, Mashy's tip and a prompt that copies", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/quick-start/");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator(".mash-callout")).toContainText("Already using Mash?");
  await expect(guide.locator(".mash-mashy")).toContainText("mash dash");
  // Each install command is titled in its window and wraps rather than scrolls.
  await expect(guide.locator(".expressive-code .title")).toHaveText([
    "macOS (Apple silicon) or Linux (x86-64 or ARM64)",
    "Windows x64 (PowerShell)",
  ]);
  const copy = guide.getByRole("button", { name: "Copy the prompt for your agent" });
  await copy.click();
  await expect(copy).toHaveText("Copied");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    "Read https://getmash.io/setup.md and set up Mash on this machine. Review the choices with me first.",
  );
});

test("the French quick start keeps the same layout in French", async ({ page }) => {
  await page.goto("/fr/quick-start/");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator(".mash-mashy")).toContainText("mash dash");
  await expect(
    guide.getByRole("button", { name: "Copier la demande pour votre agent" }),
  ).toHaveText("Copier");
});

test("the tool reference marks reading and writing tools with pills", async ({ page }) => {
  await page.goto("/tools/");
  const guide = page.locator(".sl-markdown-content");
  await expect(guide.locator("p .mash-badge")).toHaveText(["READ", "WRITE"]);
  // The short maintenance tools sit two across; a row opens to the full width.
  const pair = guide.locator(".mash-tools.pair");
  await expect(pair.locator(".mash-tool")).not.toHaveCount(0);
  const tool = pair.locator(".mash-tool").first();
  await tool.locator("summary").click();
  await expect(tool.locator(".params, .none").first()).toBeVisible();
});

test("the comparison and the agent list use cards, tiles and pills", async ({ page }) => {
  await page.goto("/compare/");
  const fits = page.locator(".sl-markdown-content .mash-fits > li");
  await expect(fits).toHaveCount(3);
  await expect(fits.first()).toContainText("You already have chats worth keeping.");
  await page.goto("/supported-agents/");
  const first = page.locator(".sl-markdown-content table").first();
  await expect(first.locator(".mash-agent-tile")).toHaveCount(7);
  await expect(
    first.getByRole("row", { name: /^Claude Code/ }).locator(".mash-pill.ok"),
  ).toHaveText("Yes");
});
