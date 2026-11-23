import { expect, test } from "@playwright/test";

const agents = ["Claude Code", "Codex", "GitHub Copilot", "Cursor", "OpenCode", "Antigravity"];

test("the landing page says what Mash is and which agents it reads", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Your agents forget. Mash doesn't.");
  await expect(page).toHaveTitle(/Your agents forget\. Mash doesn't\./);
  const marks = page.locator(".mash-agents li");
  await expect(marks).toHaveCount(agents.length);
  await expect(marks).toHaveText(agents);
  // Every mark is drawn from a file the site actually serves.
  for (const mark of await page.locator(".mash-agents .mark").all()) {
    const file = await mark.evaluate((node) => getComputedStyle(node).maskImage.slice(5, -2));
    expect((await page.request.get(file)).ok()).toBe(true);
  }
  const blurb =
    "Mash supercharges your agents so that they can search and remember from each others' chats. Leave off with Claude and pick up with Codex, without having to re-explain yourself. 100% self-hosted and open-source.";
  await expect(page.locator(".hero .tagline")).toHaveText(blurb);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", blurb);
  await expect(page.locator(".hero .actions a")).toHaveText(["Get started", "Try the demo"]);
  await expect(page.locator(".mash-welcome h2")).toHaveText([
    "Find it. However you remember it.",
    "Meet Mashy. Ask your history anything.",
    "Save the decision. Keep the source.",
    "Fast, and light on your machine.",
    "Self-hosted, end to end. No cloud. No account.",
    "Written with <3 in Rust.",
    "Give your agents a memory.",
  ]);
  // The windows are drawn in HTML, each one image named for what it shows.
  const illustrations = page.locator(".mash-welcome [role=img]");
  await expect(illustrations).toHaveCount(3);
  for (const illustration of await illustrations.all()) {
    await expect(illustration).toHaveAccessibleName(/\w/);
  }
  const more = page.getByRole("region", { name: "More about Mash" });
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
  await expect(measured.locator("dt")).toHaveText(["66 ms", "20 MB", "5× and 9×"]);
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

test("the hero's two actions sit side by side", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
  const [first, second] = await page
    .locator(".hero .actions a")
    .evaluateAll((links) => links.map((link) => link.getBoundingClientRect().top));
  expect(second).toBe(first);
});

test("the hero leads with its stamp and headline, beside the handoff cards", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
  await expect(page.locator(".mash-hero .mash-stamp")).toHaveText("Open source · self-hosted");
  // The answer is ink under a butter highlighter stroke, not a coloured word.
  const answer = page.locator(".hero h1 .mash-turn");
  const [colour, heading] = await Promise.all([
    answer.evaluate((node) => getComputedStyle(node).color),
    page.locator(".hero h1").evaluate((node) => getComputedStyle(node).color),
  ]);
  expect(colour).toBe(heading);
  expect(await answer.evaluate((node) => getComputedStyle(node).backgroundImage)).toMatch(
    /linear-gradient/,
  );
  // The cards are an illustration beside the words, hidden from assistive technology.
  const cards = page.locator(".mash-hero .handoff");
  await expect(cards).toHaveAttribute("aria-hidden", "true");
  const [words, art] = await Promise.all([
    page.locator(".hero .copy").boundingBox(),
    cards.boundingBox(),
  ]);
  if (!words || !art) throw new Error("Missing hero bounds");
  expect(words.x + words.width).toBeLessThan(art.x);
  // On a phone the words stand alone and the two actions take the full width.
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(cards).toBeHidden();
  const widths = await page
    .locator(".hero .actions a")
    .evaluateAll((links) => links.map((link) => link.getBoundingClientRect().width));
  for (const width of widths) expect(width).toBeGreaterThan(340);
});

test("the hero's install command copies to the clipboard", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  await page.getByRole("button", { name: "Install it yourself" }).click();
  const command = page.locator(".mash-install .picked code");
  // The browser under test runs on Linux, so the picker opens on the shell script.
  await expect(command).toHaveText(/^curl .*install\.sh \| sh -s -- --dashboard$/);
  await expect(page.getByRole("link", { name: "Install separately" })).toBeVisible();
  const copy = page.getByRole("button", { name: "Copy the install command" });
  await copy.click();
  await expect(copy).toHaveText("Copied");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    await command.textContent(),
  );
});

test("the hero's install picker follows the chosen system and method", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  const install = page.locator(".mash-install");
  const command = install.locator(".picked code");
  const requirements = install.locator("[data-requirements]");
  const linuxSetup = install.getByRole("link", { name: "Linux setup without systemd" });
  // One box at a time: the agent's prompt first, the install command on request.
  await expect(install.locator(".command:visible")).toHaveCount(1);
  await expect(install.locator(".command:visible code")).toHaveText(/^Read https/);
  await expect(requirements).toBeHidden();
  await install.getByRole("button", { name: "Install it yourself" }).click();
  await expect(install.locator(".command:visible")).toHaveCount(1);
  await expect(install.locator(".command:visible code")).toHaveText(/^curl /);
  await expect(requirements).toContainText("Requires a systemd user session");
  await expect(linuxSetup).toBeVisible();
  await expect(linuxSetup).toHaveAttribute(
    "href",
    "/local-search/#linux-without-a-systemd-user-session",
  );
  await expect(install.getByRole("link", { name: "See browser setup" })).toHaveAttribute(
    "href",
    "/dashboard/#set-up-mash-from-the-dashboard",
  );
  await install.getByRole("button", { name: "Windows", exact: true }).click();
  await expect(command).toHaveText(/^& .*irm .*install\.ps1\)\)\) -Dashboard$/);
  await expect(install.locator(".picked .prompt")).toHaveText(">");
  await expect(requirements).toHaveText(
    "For Windows x64. Open PowerShell after signing in to the Windows desktop.",
  );
  await expect(linuxSetup).toBeHidden();
  await install.getByRole("button", { name: "Scoop", exact: true }).click();
  await expect(requirements).toHaveText("Requires Windows x64 and Scoop. Run in PowerShell.");
  await install.getByRole("button", { name: "Cargo", exact: true }).click();
  await expect(command).toHaveText(/^cargo \+stable install /);
  await expect(requirements).toContainText("a stable Rust toolchain");
  await expect(
    install.getByText("This command installs terminal search and agent tools."),
  ).toBeVisible();
  await expect(install.getByRole("link", { name: "Set up terminal search" })).toBeVisible();
  const buildDashboard = install.getByRole("link", { name: "Build with the dashboard" });
  await expect(buildDashboard).toBeVisible();
  await expect(
    install.getByRole("link", {
      name: "start the local search service and choose which histories to read",
    }),
  ).toBeHidden();
  await page.getByRole("button", { name: "Copy the install command" }).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    await command.textContent(),
  );
  // The method carries over to another system, and the choice is kept for the next visit.
  await install.getByRole("button", { name: "macOS", exact: true }).click();
  await expect(command).toHaveText(/^cargo \+stable install /);
  await page.reload();
  await expect(command).toBeVisible();
  await expect(command).toHaveText(/^cargo \+stable install /);
  await expect(requirements).toContainText("a stable Rust toolchain");
  await expect(install.getByRole("button", { name: "Homebrew", exact: true })).toBeVisible();
  // Linux offers apt and not Homebrew.
  await install.getByRole("button", { name: "Linux", exact: true }).click();
  await expect(install.getByRole("button", { name: "Homebrew", exact: true })).toHaveCount(0);
  await install.getByRole("button", { name: "apt", exact: true }).click();
  await expect(command).toHaveText(/sudo apt install mash-cli$/);
  await expect(requirements).toContainText("Debian 12+ or Ubuntu 22.04+");
  await expect(linuxSetup).toBeHidden();
  await expect(install.getByText("Includes the browser dashboard.")).toBeVisible();
  await expect(install.getByRole("link", { name: "Review the package sources" })).toBeVisible();
  await expect(buildDashboard).toBeHidden();

  await install.getByRole("button", { name: "Cargo", exact: true }).click();
  await expect(install.getByRole("link", { name: "Review the source build" })).toBeVisible();
  await expect(install.getByRole("link", { name: "Review the package sources" })).toBeHidden();
  await buildDashboard.click();
  await expect(page).toHaveURL(/\/install-methods\/#include-the-web-dashboard$/);
  await expect(page.getByRole("heading", { name: "Include the web dashboard" })).toBeVisible();
  await expect(
    page.locator("pre").filter({ hasText: "pnpm@11.28.5 --dir dashboard run build" }),
  ).toContainText("cargo +stable install --locked --path .");
});

test("a Homebrew visitor can review the package source and see what to do next", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto("/");
  const install = page.locator(".mash-install");
  await install.getByRole("button", { name: "Install it yourself" }).click();
  await install.getByRole("button", { name: "macOS", exact: true }).click();
  await expect(install.locator("[data-requirements]")).toContainText("Apple silicon Macs");
  await install.getByRole("button", { name: "Homebrew", exact: true }).click();
  await expect(install.locator(".picked code")).toHaveText("brew install mashium/tap/mash");
  await expect(install.locator("[data-requirements]")).toContainText(
    "an Apple silicon Mac and Homebrew",
  );
  await expect(install.getByText("Includes the browser dashboard.")).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await expect(
    install.getByRole("link", {
      name: "start the local search service and choose which histories to read",
    }),
  ).toHaveAttribute("href", "/installation/#start-the-local-search-service");
  await install.getByRole("link", { name: "Review the package sources" }).click();
  await expect(page).toHaveURL(/\/install-methods\/#install-with-a-package-manager$/);
  await expect(page.getByRole("link", { name: "Mashium’s Homebrew formula" })).toHaveAttribute(
    "href",
    "https://github.com/mashium/homebrew-tap/blob/main/Formula/mash.rb",
  );
  await expect(
    page.getByRole("link", { name: "see which installers passed for each release" }),
  ).toBeVisible();
});

test("the hero offers a prompt that hands setup to a coding agent", async ({
  page,
  context,
  request,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  await page.getByRole("button", { name: "Copy the prompt for your agent" }).click();
  const prompt = await page.evaluate(() => navigator.clipboard.readText());
  expect(prompt).toBe(
    "Read https://getmash.io/setup.md and set up Mash on this machine. Review the choices with me first.",
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
  await page.goto("/");
  await expect(
    page.locator(".mash-install").getByRole("link", {
      name: /Inspect the installer|Download and verify an archive|See what stays local/,
      includeHidden: true,
    }),
  ).toHaveCount(0);
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
  await expect(windows).toHaveCount(3);
  await expect(windows.first()).toHaveAccessibleName(/why does checkout keep failing/);
  await expect(welcome.locator(".ico svg").first()).toBeVisible();
  for (const picture of await windows.all()) await expect(picture).toBeVisible();
  const mashy = welcome.getByRole("img", { name: "Mashy, the Mash mascot" });
  await expect
    .poll(() => mashy.evaluate((img: HTMLImageElement) => img.naturalHeight))
    .toBeGreaterThan(0);
  for (const caption of await welcome.locator(".caps li").all())
    await expect(caption).toBeVisible();
  await welcome.locator(".measured summary").click();
  await expect(welcome.locator(".measured").getByText("66 ms", { exact: true })).toBeVisible();
  const install = page.locator(".mash-install");
  await expect(install.locator(".agent code")).toBeVisible();
  await expect(install.locator(".picked")).toBeHidden();
  await expect(install.getByRole("button")).toHaveCount(0);
  await install
    .getByRole("link", { name: "See the install steps for your operating system" })
    .click();
  await expect(page.getByRole("heading", { name: "Get Mash for your system" })).toBeVisible();
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
  for (const tile of await tiles.all()) await expect(tile.locator(".ico svg")).toBeVisible();
  const built = welcome.locator(".built li");
  await expect(built).toHaveText(["Tauri", "Qdrant", "Tokio"]);
  for (const chip of await built.all()) await expect(chip.locator(".tile svg")).toBeAttached();
  // The two closing actions are buttons of one size, each with the same arrow.
  const actions = welcome.locator(".closing .actions a");
  await expect(actions).toHaveText(["Get started", "Read the docs"]);
  await expect(actions.locator("svg")).toHaveCount(2);
  const heights = await actions.evaluateAll((links) =>
    links.map((link) => link.getBoundingClientRect().height),
  );
  expect(heights[1]).toBe(heights[0]);
});
