import { expect, test } from "@playwright/test";

test("a docs page shows where it sits, and its table of contents skips the title", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/tools/");
  await expect(
    page.getByRole("navigation", { name: "Breadcrumb" }).getByRole("listitem"),
  ).toHaveText(["Reference", "MCP tool reference"]);
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
  await expect(guide.locator(".mash-mashy")).toContainText("filters");
  // Each step after installing shows the dashboard page it describes.
  await expect(guide.locator('img[src^="/screenshots/"]').first()).toBeVisible();
  // Each install command is titled in its window and wraps rather than scrolls.
  await expect(guide.locator(".expressive-code .title")).toHaveText([
    "macOS (Apple silicon) or Linux (x86-64 or ARM64)",
    "Windows x64 (PowerShell)",
  ]);
  // Both things to say to an agent copy: remembering a decision, and setup.
  const prompts = guide.getByRole("button", { name: "Copy the prompt for your agent" });
  await expect(prompts).toHaveCount(2);
  const copy = prompts.last();
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

test("why Mash shows each feature and the agent list uses tiles and pills", async ({ page }) => {
  await page.goto("/compare/");
  await expect(page.locator("h1")).toHaveText("Why Mash");
  await expect(page.locator('.sl-markdown-content img[src^="/screenshots/"]')).toHaveCount(5);
  await page.goto("/supported-agents/");
  const first = page.locator(".sl-markdown-content table").first();
  await expect(first.locator(".mash-agent-tile")).toHaveCount(7);
  await expect(
    first.getByRole("row", { name: /^Claude Code/ }).locator(".mash-pill.ok"),
  ).toHaveText("Yes");
});

test("the guides tip opens search and names the shortcut for the platform", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/guides/");
  const tip = page.getByRole("button", { name: "Search the docs" });
  await expect(tip.locator("kbd")).toHaveText(/^(Ctrl K|⌘ K)$/);
  await tip.click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("dialog").locator("input").first()).toBeFocused();
});

test("a missing page explains itself and offers search, home and the docs", async ({ page }) => {
  await page.goto("/no-such-guide/");
  await expect(page.locator("h1")).toHaveText("Page not found");
  await expect(page.locator(".mash-hero")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Search the docs" })).toBeVisible();
  const actions = page.locator(".mash-missing");
  await expect(actions.getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  await expect(actions.getByRole("link", { name: "Docs" })).toHaveAttribute("href", "/guides/");
});

test("every code window has the same copy button inside its title bar", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/quick-start/");
  const frame = page.locator(".expressive-code .frame.has-title").first();
  const bar = await frame.locator(".header").boundingBox();
  const copy = await frame.locator(".copy button").boundingBox();
  expect(bar && copy && copy.y + copy.height <= bar.y + bar.height).toBe(true);
  // A Windows shell's window has no Mac window dots.
  const windows = page.locator(".expressive-code .frame:has(pre[data-language='powershell'])");
  const dots = await windows
    .first()
    .locator(".header")
    .evaluate((header) => getComputedStyle(header, "::before").display);
  expect(dots).toBe("none");
  // The prompt's copy button matches the code windows': same height, corners
  // and lettering (a code window's word is drawn in its ::before).
  const prompt = page.getByRole("button", { name: "Copy the prompt for your agent" }).first();
  const code = frame.locator(".copy button");
  const style = (element: Element, part: string | null) => {
    const { height, borderRadius, fontSize, fontWeight } = getComputedStyle(element);
    const word = getComputedStyle(element, part);
    return [
      height,
      borderRadius,
      part ? word.fontSize : fontSize,
      part ? word.fontWeight : fontWeight,
    ];
  };
  expect(await prompt.evaluate(style, null)).toEqual(await code.evaluate(style, "::before"));
});

test("on a phone the docs menu starts with the links and the language picker", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/quick-start/");
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  const menu = page.locator("nav.sidebar");
  await expect(menu.getByRole("link", { name: "Try the demo" })).toBeInViewport();
  await expect(menu.locator("starlight-lang-select")).toBeInViewport();
});
