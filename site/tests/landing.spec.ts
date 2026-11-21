import { expect, type Page, test } from "@playwright/test";

const agents = ["Claude Code", "Codex", "GitHub Copilot", "Cursor", "OpenCode", "Antigravity"];

test("the landing page says what Mash is and which agents it reads", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("Find the chat behind the decision.");
  await expect(page).toHaveTitle(/Find the chat behind the decision\./);
  const marks = page.locator(".mash-agents li");
  await expect(marks).toHaveCount(agents.length);
  await expect(marks).toHaveText(agents);
  // Every mark is drawn from a file the site actually serves.
  for (const mark of await page.locator(".mash-agents .mark").all()) {
    const file = await mark.evaluate((node) => getComputedStyle(node).maskImage.slice(5, -2));
    expect((await page.request.get(file)).ok()).toBe(true);
  }
  await expect(page.locator(".hero .tagline")).toHaveText(
    "Search your saved coding-agent chats in the browser dashboard or from an agent you connect. Your history stays on your machine by default.",
  );
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
  await expect(page.locator(".mash-welcome .tiles article")).toHaveCount(5);
  // Every section with a drawing but the last pins and plays the same way.
  await expect(page.locator(".mash-welcome .scene")).toHaveCount(7);
  // The marks rise in one after another when they come into view.
  await marks.first().scrollIntoViewIfNeeded();
  for (const mark of await marks.all()) await expect(mark).toHaveClass(/seen/);
  // The conditions behind the figures are folded away, and open on request.
  const measured = page.locator(".mash-welcome .measured");
  await expect(measured.locator("p")).toBeHidden();
  await measured.locator("summary").click();
  await expect(measured.locator("p")).toContainText("experimental and off until you");
  await expect(page.getByRole("navigation", { name: "Footer" }).getByRole("link")).toHaveText([
    "Docs",
    "Try it",
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

test("the hero leads with the full logo, in the one that suits the theme", async ({ page }) => {
  await page.goto("/");
  const shown = page.locator(".mash-logo img:visible");
  await expect(shown).toHaveCount(1);
  // It is drawn large without widening the page.
  const width = await shown.evaluate((node) => node.getBoundingClientRect().width);
  expect(width).toBeGreaterThan(280);
  expect(width).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
  expect(await shown.evaluate((node) => (node as HTMLImageElement).naturalWidth)).toBeGreaterThan(
    0,
  );
  const dark = await page.evaluate(() => document.documentElement.dataset.theme !== "light");
  await expect(shown).toHaveClass(dark ? /on-dark/ : /on-light/);
  await expect(page.locator("canvas")).toHaveCount(0);
});

test("the hero's install command copies to the clipboard", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  await page.getByRole("button", { name: "Install it yourself" }).click();
  const command = page.locator(".mash-install .picked code");
  // The browser under test runs on Linux, so the picker opens on the shell script.
  await expect(command).toHaveText(/^curl .*install\.sh \| sh -s -- --setup$/);
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
  // One box at a time: the agent's prompt first, the install command on request.
  await expect(install.locator(".command:visible")).toHaveCount(1);
  await expect(install.locator(".command:visible code")).toHaveText(/^Read https/);
  await install.getByRole("button", { name: "Install it yourself" }).click();
  await expect(install.locator(".command:visible")).toHaveCount(1);
  await expect(install.locator(".command:visible code")).toHaveText(/^curl /);
  await install.getByRole("button", { name: "Windows", exact: true }).click();
  await expect(command).toHaveText(/^& .*irm .*install\.ps1\)\)\) -Setup$/);
  await expect(install.locator(".picked .prompt")).toHaveText(">");
  await install.getByRole("button", { name: "Cargo", exact: true }).click();
  await expect(command).toHaveText(/^cargo \+stable install /);
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
  await expect(install.getByRole("button", { name: "Homebrew", exact: true })).toBeVisible();
  // Linux offers apt and not Homebrew.
  await install.getByRole("button", { name: "Linux", exact: true }).click();
  await expect(install.getByRole("button", { name: "Homebrew", exact: true })).toHaveCount(0);
  await install.getByRole("button", { name: "apt", exact: true }).click();
  await expect(command).toHaveText(/sudo apt install mash-cli$/);
  await expect(install.getByText("Includes the browser dashboard.")).toBeVisible();
  await expect(install.getByRole("link", { name: "Review the package sources" })).toBeVisible();
  await expect(install.getByRole("link", { name: "Inspect the installer" })).toBeHidden();
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
  await install.getByRole("button", { name: "Homebrew", exact: true }).click();
  await expect(install.locator(".picked code")).toHaveText("brew install mashium/tap/mash");
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
  expect(prompt).toBe("Read https://getmash.io/setup.md and set up Mash on this machine.");
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
});

test("visitors can inspect the installer and read about privacy before setup", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Inspect the installer", exact: true }).click();
  await expect(page).toHaveURL(/\/installation\/#check-the-release-download$/);
  await expect(page.getByRole("heading", { name: "Check the release download" })).toBeInViewport();
  await expect(page.getByRole("link", { name: "public release workflow" })).toBeVisible();

  await page.goto("/");
  await page.getByRole("link", { name: "See what stays local", exact: true }).click();
  await expect(page).toHaveURL(/\/privacy\/$/);
  await expect(page.getByRole("heading", { name: "Try a small local setup" })).toBeVisible();
  await expect(page.getByText("Custom number of days", { exact: true })).toBeVisible();
  await page.getByRole("link", { name: "See what uninstall removes", exact: true }).last().click();
  await expect(page).toHaveURL(/\/installation\/#uninstall$/);
  await expect(page.getByRole("heading", { name: "Uninstall", exact: true })).toBeInViewport();
});

/**
 * Scrolls so that a scene has played the given share of its pinned stretch.
 * A scene on the way that has not finished holds the page until it has, so
 * this keeps asking until the page is there.
 */
async function play(page: Page, heading: string, share: number): Promise<void> {
  const scene = page.locator(".mash-welcome .scene", { hasText: heading });
  await expect
    .poll(
      () =>
        scene.evaluate((element, part) => {
          const top = element.getBoundingClientRect().top + scrollY;
          const target = top + part * (element.getBoundingClientRect().height - innerHeight);
          if (Math.abs(scrollY - target) < 2) return true;
          scrollTo(0, target);
          return false;
        }, share),
      { timeout: 40_000 },
    )
    .toBe(true);
}

test("scrolling plays the search scene: the query is typed, then results arrive", async ({
  page,
}) => {
  await page.goto("/");
  const scene = page.locator(".mash-welcome .scene", { hasText: "Find it." });
  const query = scene.locator("[data-type]");
  const results = scene.locator(".hits li");
  // Before the scene is reached, nothing has been typed.
  await expect(query).toHaveText("");
  await play(page, "Find it.", 0.8);
  // However far the scroll jumped, every letter is typed before a result shows.
  await expect(query).toHaveText("why does checkout keep failing");
  await expect(results).toHaveCount(3);
  for (const result of await results.all()) await expect(result).toHaveCSS("opacity", "1");
  await expect(scene.locator(".caps li").last()).toHaveCSS("opacity", "1");
});

test("a scene scrolled past early stays on screen until it has played out", async ({ page }) => {
  await page.goto("/");
  const scene = page.locator(".mash-welcome .scene", { hasText: "Find it." });
  const stage = scene.locator(".stage");
  // Where the scene's stage stops being pinned.
  const pinEnds = () =>
    scene.evaluate((element) => {
      const pinned = element.querySelector<HTMLElement>(".stage");
      if (!pinned) throw new Error("no stage");
      const pinnedAt = Number.parseFloat(getComputedStyle(pinned).top);
      return scrollY + element.getBoundingClientRect().bottom - pinnedAt - pinned.offsetHeight;
    });
  const strokes = scene.locator(".art path");
  // Twice over: the scene does this on every pass, not only the first.
  for (const pass of [1, 2]) {
    await play(page, "Find it.", 0.05);
    const end = await pinEnds();
    const pinnedAt = await stage.evaluate((element) => element.getBoundingClientRect().top);
    // Scroll past it before it has played: a little the first time, several
    // screens the second. The page goes where it was sent.
    await page.evaluate((to) => scrollTo(0, to), end + (pass === 1 ? 500 : 2600));
    await expect(scene, `pass ${pass}`).toHaveClass(/kept/);
    expect(await page.evaluate(() => scrollY)).toBeGreaterThan(end + 400);
    // The stage has not moved, and it finishes there: the query is typed,
    // every result arrives, and the drawing is undrawn.
    expect(await stage.evaluate((element) => element.getBoundingClientRect().top)).toBeCloseTo(
      pinnedAt,
      0,
    );
    await expect(scene.locator("[data-type]")).toHaveText("why does checkout keep failing");
    await expect(scene.locator(".hits li").last()).toHaveCSS("opacity", "1", { timeout: 15_000 });
    await expect(scene).toHaveClass(/kept/);
    await expect(strokes.first()).toHaveCSS("opacity", "0", { timeout: 15_000 });
    // Then it lets go, and the page was never moved.
    await expect(scene).not.toHaveClass(/kept/);
    expect(await page.evaluate(() => scrollY)).toBeGreaterThan(end + 400);
  }
});

test("a scene's drawing draws itself, and undraws on the way back", async ({ page }) => {
  await page.goto("/");
  const strokes = page
    .locator(".mash-welcome .scene", { hasText: "Meet Mashy." })
    .locator(".art path");
  await expect(strokes.last()).toHaveCSS("opacity", "0");
  await play(page, "Meet Mashy.", 0.2);
  await expect(strokes.last()).toHaveCSS("opacity", "1");
  await page.evaluate(() => scrollTo(0, 0));
  await expect(strokes.last()).toHaveCSS("opacity", "0");
});

test("a scene's enlarged heading stays inside the screen", async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 700 });
  await page.goto("/");
  await play(page, "Fast, and light", 0.1);
  const heading = page.getByRole("heading", { name: "Fast, and light on your machine." });
  await expect(async () => {
    const box = await heading.boundingBox();
    expect(box?.x).toBeGreaterThanOrEqual(0);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(1000);
  }).toPass();
});

test("the Mashy conversation runs to its third answer", async ({ page }) => {
  await page.goto("/");
  const scene = page.locator(".mash-welcome .scene", { hasText: "Meet Mashy." });
  await play(page, "Meet Mashy.", 0.84);
  // The conversation plays at its own pace, so the last answer takes a few seconds.
  await expect(scene.locator(".reply").last()).toHaveCSS("opacity", "1", { timeout: 20_000 });
  // The suggestion that was picked has folded away with the others.
  await expect(scene.locator(".follows")).toHaveCSS("opacity", "0");
  await expect(scene.locator(".caps li").last()).toHaveText("Your model. It only reads.");
});

test("the welcome page shows everything without scripting", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("/");
  const welcome = page.locator(".mash-welcome");
  await expect(welcome.locator("[data-type]")).toHaveText("why does checkout keep failing");
  await expect(welcome.locator(".hits li")).toHaveCount(3);
  for (const result of await welcome.locator(".hits li").all()) await expect(result).toBeVisible();
  await expect(welcome.locator(".reply").last()).toBeVisible();
  await expect(welcome.locator(".ferris")).toBeVisible();
  await expect(welcome.locator(".stats dt").first()).toHaveText("66 ms");
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
