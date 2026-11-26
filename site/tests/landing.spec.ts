import { expect, test } from "@playwright/test";

const agents = ["Claude Code", "Codex", "GitHub Copilot", "Cursor", "OpenCode", "Antigravity"];

test("the landing page says what Mash is and which agents it reads", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Your agents forget. Mash doesn’t.");
  await expect(page).toHaveTitle("Mash: Supercharged context for your agents.");
  const marks = page.locator(".mash-agents li");
  await expect(marks).toHaveCount(agents.length);
  await expect(marks).toHaveText(agents);
  // Every mark is drawn: each tile holds its agent's logo.
  for (const mark of await page.locator(".mash-agents .tile").all()) {
    await expect(mark.locator("svg path").first()).toBeAttached();
  }
  const blurb =
    "Mash supercharges your agents so that they can search and remember from each others' chats. Leave off with Claude and pick up with Codex, without having to re-explain yourself. 100% self-hosted and open-source.";
  await expect(page.locator(".hero .tagline")).toHaveText(blurb);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", blurb);
  // Link previews carry the tagline.
  const tagline = "Supercharged context for your agents.";
  await expect(page.locator('meta[property="og:description"]')).toHaveAttribute("content", tagline);
  await expect(page.locator('meta[property="og:title"]')).toHaveAttribute(
    "content",
    `Mash: ${tagline}`,
  );
  await expect(page.locator(".hero .actions a")).toHaveText([/^\s*Download for \w+$/]);
  await expect(page.locator(".mash-welcome h2")).toHaveText([
    "Take shortcuts, not detours. Press ⌥ ⇧ Space and the chat you need is one line away.",
    "Find it. However you remember it.",
    "Meet Mashy. Ask anything about your chats.",
    "Save the decision. Keep the source.",
    "Fast, and light on your machine.",
    "Self-hosted, end to end. No cloud. No account.",
    "Fits the way you work.",
    "Written with <3 in Rust.",
    "Give your agents their history back.",
  ]);
  // The windows are drawn in HTML, each one image named for what it shows.
  const illustrations = page.locator(".mash-welcome [role=img]");
  await expect(illustrations).toHaveCount(4);
  for (const illustration of await illustrations.all()) {
    await expect(illustration).toHaveAccessibleName(/\w/);
  }
  const more = page.getByRole("region", { name: "Fits the way you work." });
  await expect(more.getByRole("link")).toHaveText([
    "plugin for Claude Code, Codex and Copilot",
    "Hybrid search",
    "shared index",
  ]);
  await page.locator(".mash-agents").scrollIntoViewIfNeeded();
  for (const mark of await marks.all()) await expect(mark).toBeVisible();
  // The conditions behind the figures are folded away, and open on request.
  const measured = page.locator(".mash-welcome .measured");
  await expect(measured.locator("p")).toBeHidden();
  await measured.locator("summary").click();
  await expect(measured.locator("dt")).toHaveText(["66 ms and 103k", "5× and 9×"]);
  await expect(measured.locator("p")).toContainText("experimental and off until you");
  await expect(page.getByRole("navigation", { name: "Footer" }).getByRole("link")).toHaveText([
    "Docs",
    "Try the demo",
    "Privacy",
    "Get help",
    "GitHub",
  ]);
});

test("the landing page fits a phone", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

test("the hero leads with the app download, then the other systems and ways in", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
  // A Linux x86-64 browser here, so the download is the Linux app.
  const download = page.locator(".hero .actions a").first();
  await expect(download).toHaveText(/Download for (Linux|Mac|Windows)/);
  await expect(download).toHaveAttribute(
    "href",
    /^https:\/\/github\.com\/mashhq\/mash\/releases\/latest\/download\/mash-desktop-/,
  );
  // "Also on" names the two other systems and leads to every download.
  const note = page.locator(".hero .download-note");
  const also = note.locator("a[data-also]");
  await expect(also).toHaveAttribute("href", "/downloads/#desktop-app");
  await expect(also).toHaveText(/^(macOS|Windows) and\s+(Windows|Linux)$/, {
    useInnerText: true,
  });
  await expect(note.locator(".ways a")).toHaveText([
    "Install with curl",
    "Raycast extension",
    "Try the demo",
  ]);
  // The header offers the Mac app beside the docs and the source.
  const header = page.locator("header.header");
  await expect(header.getByRole("link", { name: "Download for Mac" })).toBeVisible();
  await expect(header.getByRole("link", { name: "Docs", exact: true })).toBeVisible();
  await expect(header.getByRole("link", { name: "GitHub", exact: true })).toBeVisible();
});

test("the install tabs show each system's command, and copy it", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
  const install = page.locator("#install");
  const tabs = install.getByRole("tab");
  await expect(tabs).toHaveText(["Linux", "macOS", "Windows"]);
  await install.getByRole("tab", { name: "Windows" }).click();
  await expect(install.getByRole("tab", { name: "Windows" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  const panel = install.getByRole("tabpanel");
  await expect(panel).toHaveCount(1);
  await expect(panel).toContainText("install.ps1");
  // The arrow keys move between the tabs.
  await page.keyboard.press("ArrowLeft");
  await expect(install.getByRole("tab", { name: "macOS" })).toBeFocused();
  await expect(install.getByRole("tabpanel")).toContainText("install.sh | sh -s -- --dashboard");
  await install.getByRole("tabpanel").getByRole("button").click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    "curl -fsSL https://raw.githubusercontent.com/mashhq/mash/main/install.sh | sh -s -- --dashboard",
  );
  // On a phone the headline and the download still lead. Back at the top:
  // with the app icon over the headline, using the tabs scrolls the page.
  await page.setViewportSize({ width: 390, height: 844 });
  await page.evaluate(() => scrollTo(0, 0));
  await expect(page.locator(".hero h1")).toBeInViewport();
  await expect(page.locator(".hero .actions a")).toBeVisible();
});

test("a Homebrew visitor can review the package source and see what to do next", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto("/install-methods/#install-with-a-package-manager");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await expect(page.getByRole("link", { name: "MashHQ’s Homebrew formula" })).toHaveAttribute(
    "href",
    "https://github.com/mashhq/homebrew-tap/blob/main/Formula/mash.rb",
  );
  await expect(
    page.getByRole("link", { name: "see which installers passed for each release" }),
  ).toBeVisible();
});

test("the quick start offers a prompt that hands setup to a coding agent", async ({
  page,
  request,
}) => {
  await page.goto("/quick-start/");
  await expect(page.locator(".sl-markdown-content")).toContainText(
    "Read https://getmash.io/setup.md and set up Mash on this machine.",
  );
  // The file the prompt names is published as plain Markdown.
  const setup = await request.get("/setup.md");
  expect(setup.ok()).toBe(true);
  const instructions = await setup.text();
  expect(instructions).toContain(
    "To search on your own from the dashboard or terminal:\n\n```sh\nmash init --yes --no-agents\n```",
  );
  expect(instructions).toContain("https://getmash.io/install-methods/#include-the-web-dashboard");
  expect(instructions).toContain("With only the terminal and agent tools, show them");
  expect(instructions).toContain("unless the person asked you to run their");
  expect(instructions).toContain("`configuration.index.destination` set to `both`");
  expect(instructions).toMatch(
    /Proceed with\s+indexing only if they choose to use that destination/,
  );
  expect(instructions).toContain("If it is `rejected`, use `local_qdrant.message`");
});

test("visitors can inspect the installer and read about privacy before setup", async ({ page }) => {
  await page.goto("/quick-start/");
  await page.getByRole("link", { name: "how to verify the download", exact: true }).click();
  await expect(page).toHaveURL("/downloads/#verify-the-checksum");
  await page.getByRole("link", { name: "inspect the script", exact: true }).click();
  await expect(page).toHaveURL(/\/downloads\/#inspect-the-installer$/);
  await expect(page.getByRole("heading", { name: "Inspect the installer" })).toBeInViewport();
  await expect(page.getByRole("link", { name: "public release workflow" })).toBeVisible();

  await page.goto("/quick-start/");
  await page.getByRole("link", { name: "local trial guide", exact: true }).click();
  await expect(page).toHaveURL("/privacy/#try-a-small-local-setup");
  await expect(page.getByRole("heading", { name: "Try a small local setup" })).toBeVisible();
  const range = page.getByRole("listitem").filter({ hasText: "another number of days" });
  await expect(range.getByText("7", { exact: true })).toBeVisible();
  await expect(range).toContainText("can contain older messages");
  await expect(page.getByText("no agents", { exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "quick start installer" })).toHaveAttribute(
    "href",
    "/quick-start/#install-it-yourself",
  );
  await page.getByRole("link", { name: "See what uninstall removes", exact: true }).last().click();
  await expect(page).toHaveURL(/\/uninstall\/$/);
  await expect(page.getByRole("heading", { name: "Uninstall Mash", exact: true })).toBeInViewport();
  await expect(page.getByRole("link", { name: "Keep Mash’s copy in place" })).toBeVisible();
});

test("each welcome band comes in as it scrolls into view, and at once without motion", async ({
  browser,
}) => {
  for (const reducedMotion of ["no-preference", "reduce"] as const) {
    const context = await browser.newContext({ reducedMotion });
    const page = await context.newPage();
    await page.goto("/");
    const welcome = page.locator(".mash-welcome");
    await expect(welcome).toHaveClass(/\breveal\b/);
    const band = page.locator("#welcome-rust [data-reveal]");
    const opacity = () => band.evaluate((node) => getComputedStyle(node).opacity);
    expect(await opacity()).toBe(reducedMotion === "reduce" ? "1" : "0");
    await band.scrollIntoViewIfNeeded();
    await expect(band).toHaveClass(/\bshown\b/);
    await expect.poll(opacity).toBe("1");
    await context.close();
  }
});

test("the welcome page shows everything without scripting", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("/");
  const welcome = page.locator(".mash-welcome");
  const windows = welcome.locator("[role=img]");
  await expect(windows).toHaveCount(4);
  await expect(windows.nth(1)).toHaveAccessibleName(/Why does checkout keep failing/);
  await expect(welcome.locator(".ico svg").first()).toBeVisible();
  for (const picture of await windows.all()) await expect(picture).toBeVisible();
  const mashy = welcome.locator("img.mashy-face");
  await expect
    .poll(() => mashy.evaluate((img: HTMLImageElement) => img.naturalHeight))
    .toBeGreaterThan(0);
  // The app icon is drawn inline, over the headline and the closing call to action.
  await expect(page.locator(".hero .app-icon svg")).toBeVisible();
  await expect(welcome.locator(".closing .app-icon svg")).toBeVisible();
  for (const caption of await welcome.locator(".caps li").all())
    await expect(caption).toBeVisible();
  await welcome.locator(".measured summary").click();
  await expect(
    welcome.locator(".measured").getByText("66 ms and 103k", { exact: true }),
  ).toBeVisible();
  // Without scripting every system's install command shows.
  await expect(page.locator("#install [role=tabpanel]")).toHaveCount(3);
  for (const panel of await page.locator("#install [role=tabpanel]").all())
    await expect(panel).toBeVisible();
  // Without scripting the download button cannot tell the system, so it opens the downloads.
  await page.locator(".hero .actions").getByRole("link", { name: "Download for Mac" }).click();
  await expect(page).toHaveURL(/\/downloads\/#desktop-app$/);
  await expect(page.getByRole("heading", { name: "Desktop app" })).toBeVisible();
  await context.close();
});

test("the welcome page fits a phone in every section", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 700 });
  await page.goto("/");
  const height = await page.evaluate(() => document.documentElement.scrollHeight);
  for (let y = 0; y < height; y += 2100) {
    await page.evaluate((to) => scrollTo(0, to), y);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
      true,
    );
  }
});

test("every feature tile has an icon, the stack chips have logos, and the close offers two buttons", async ({
  page,
}) => {
  await page.goto("/");
  const welcome = page.locator(".mash-welcome");
  const tiles = welcome.locator("#welcome-features article");
  await expect(tiles).toHaveCount(5);
  for (const tile of await tiles.all())
    await expect(tile.locator(".ico svg, .systems svg").first()).toBeVisible();
  // The systems' tile shows their three marks.
  await expect(tiles.nth(2).locator(".systems svg")).toHaveCount(3);
  const built = welcome.locator(".built li");
  await expect(built).toHaveText(["Tauri", "Qdrant", "Tokio"]);
  for (const chip of await built.all()) await expect(chip.locator(".tile svg")).toBeAttached();
  // The two closing actions are buttons of one size, each led by its icon.
  const actions = welcome.locator(".closing .actions a");
  await expect(actions).toHaveText(["Get started", "Read the docs"]);
  await expect(actions.locator("svg")).toHaveCount(2);
  const heights = await actions.evaluateAll((links) =>
    links.map((link) => link.getBoundingClientRect().height),
  );
  expect(heights[1]).toBe(heights[0]);
});

test("Mashy shows one example at a time, and the pager picks another", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  const chat = page.locator("[data-mashy-chat]");
  const shown = chat.locator(".example.on .bubble");
  // With less motion the examples do not change by themselves.
  await expect(shown).toHaveText(["What caused the checkout outage last week?"]);
  await chat.getByRole("button", { name: "Show example 3 of 3" }).click();
  await expect(shown).toHaveText(["Did the payment retry change touch checkout?"]);
  await chat.getByRole("button", { name: "Next example" }).click();
  await expect(shown).toHaveText(["What caused the checkout outage last week?"]);
  await expect(chat.getByRole("button", { name: "Show example 1 of 3" })).toHaveAttribute(
    "aria-current",
    "true",
  );
  // Every example shares the window, so changing one never moves the page.
  const heights = new Set<number>();
  for (const example of [1, 2, 3]) {
    await chat.getByRole("button", { name: `Show example ${example} of 3` }).click();
    heights.add(
      await chat.locator(".mashy-window").evaluate((w) => w.getBoundingClientRect().height),
    );
  }
  expect(heights.size).toBe(1);
});
