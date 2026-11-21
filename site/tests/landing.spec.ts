import { expect, type Locator, type Page, test } from "@playwright/test";

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
  const illustrations = page.locator(".mash-welcome .scene-image");
  await expect(illustrations).toHaveCount(3);
  for (const illustration of await illustrations.all()) {
    await expect(illustration).toHaveAttribute("src", /^\/welcome\/[a-z-]+\.svg$/);
    expect(
      await illustration.evaluate(async (img: HTMLImageElement) => {
        await img.decode();
        return (
          img.naturalWidth === Number(img.getAttribute("width")) &&
          img.naturalHeight === Number(img.getAttribute("height"))
        );
      }),
    ).toBe(true);
  }
  const more = page.getByRole("region", { name: "More about Mash" });
  await expect(more.getByRole("link")).toHaveText([
    "plugin for Claude Code, Codex and Copilot",
    "Hybrid search",
    "shared index",
  ]);
  // The seven feature scenes pin while their scroll-driven artwork is in view.
  await expect(page.locator(".mash-welcome .scene")).toHaveCount(7);
  await readyWelcome(page);
  await page.locator(".mash-agents").evaluate((element) => {
    element.scrollIntoView({ block: "center", behavior: "instant" });
  });
  await paintScroll(page);
  for (const mark of await marks.all()) await expect(mark).toHaveCSS("opacity", "1");
  // The conditions behind the figures are folded away, and open on request.
  const measured = page.locator(".mash-welcome .measured");
  await scrub(page.locator("#welcome-speed"), 0.55);
  await expect(measured.locator("p")).toBeHidden();
  await measured.locator("summary").click();
  await expect(measured.locator("dt")).toHaveText(["66 ms", "20 MB", "5× and 9×"]);
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

async function readyWelcome(page: Page): Promise<void> {
  await expect(page.locator(".mash-welcome")).toHaveClass(/\bmotion\b/);
  await page.evaluate(() => document.fonts.ready);
}

async function paintScroll(page: Page): Promise<void> {
  // Let the scroll event and the next animation frame render the position.
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
}

/** Scrolls to the given share of a scene's pinned stretch. */
async function scrub(scene: Locator, share: number): Promise<void> {
  await scene.evaluate((element, part) => {
    const stage = element.querySelector<HTMLElement>(".stage");
    if (!stage) throw new Error("Missing welcome stage");
    const { top, height } = element.getBoundingClientRect();
    const pinnedAt = Number.parseFloat(getComputedStyle(stage).top);
    const target = scrollY + top - pinnedAt + part * (height - stage.offsetHeight);
    scrollTo({ top: target, behavior: "instant" });
  }, share);
  await paintScroll(scene.page());
}

async function sceneState(scene: Locator) {
  return scene.evaluate((element) => {
    const picture = element.querySelector(".show");
    const stroke = element.querySelector(".art path");
    if (!picture || !stroke) throw new Error("Missing welcome artwork");
    const panelStyle = getComputedStyle(picture);
    const strokeStyle = getComputedStyle(stroke);
    return {
      scroll: scrollY,
      panel: [panelStyle.opacity, panelStyle.transform],
      stroke: [strokeStyle.opacity, strokeStyle.strokeDasharray, strokeStyle.strokeDashoffset],
      captions: [...element.querySelectorAll(".caps li")].map((caption) => {
        const style = getComputedStyle(caption);
        return [style.opacity, style.transform];
      }),
    };
  });
}

test("welcome illustrations and headings enter from opposite alternating sides", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 850 });
  await page.goto("/");
  await readyWelcome(page);
  for (const [id, direction] of [
    ["welcome-search", 1],
    ["welcome-mashy", -1],
  ] as const) {
    const scene = page.locator(`#${id}`);
    const offset = (selector: string) =>
      scene
        .locator(selector)
        .evaluate((node) => new DOMMatrix(getComputedStyle(node).transform).m41);
    expect(direction * (await offset(".snapshot-copy"))).toBeGreaterThan(0);
    expect(direction * (await offset(".show"))).toBeLessThan(0);

    await scrub(scene, 0.4);
    await expect(scene.locator(".show")).toHaveCSS("opacity", "1");
    expect(await offset(".snapshot-copy")).toBeCloseTo(0);
    expect(await offset(".show")).toBeCloseTo(0);
  }
});

test("the same scroll position shows the same search artwork and caption", async ({ page }) => {
  await page.goto("/");
  await readyWelcome(page);
  const scene = page.locator("#welcome-search");
  const image = scene.locator(".scene-image");
  await expect(scene.locator(".show")).toHaveCSS("opacity", "0");
  await scrub(scene, 0.3);
  const firstCaption = await sceneState(scene);
  expect(firstCaption.panel[0]).toBe("1");
  expect(firstCaption.captions.map(([opacity]) => opacity)).toEqual(["1", "0", "0"]);
  await expect(image).toHaveJSProperty("naturalWidth", Number(await image.getAttribute("width")));
  await scrub(scene, 0.75);
  const lastCaption = await sceneState(scene);
  expect(lastCaption.captions.map(([opacity]) => opacity)).toEqual(["0", "0", "1"]);
  await scrub(scene, 0.3);
  expect(await sceneState(scene)).toEqual(firstCaption);

  // Even a half-visible caption stays where scrolling left it.
  await scrub(scene, 0.17);
  const entering = await sceneState(scene);
  const opacity = Number(entering.captions[0]?.[0]);
  expect(opacity).toBeGreaterThan(0);
  expect(opacity).toBeLessThan(1);
  await page.waitForTimeout(500);
  expect(await sceneState(scene)).toEqual(entering);
});

for (const [name, viewport] of [
  ["desktop", { width: 1280, height: 850 }],
  ["phone", { width: 390, height: 700 }],
] as const) {
  test(`fast jumps show the destination without replaying skipped scenes on ${name}`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await page.goto("/");
    await readyWelcome(page);
    const memory = page.locator("#welcome-memory");
    const skipped = page.locator("#welcome-search .stage, #welcome-mashy .stage");
    const stages = page.locator(".mash-welcome .scene .stage");

    await scrub(memory, 0.3);
    expect((await sceneState(memory)).captions.map(([opacity]) => opacity)).toEqual([
      "1",
      "0",
      "0",
    ]);
    expect(
      await skipped.evaluateAll((items) =>
        items.every((item) => item.getBoundingClientRect().bottom <= 0),
      ),
    ).toBe(true);
    await expect(memory.locator(".scene-image")).toBeInViewport({ ratio: 0.98 });

    // Let the browser clamp an overshoot to its own scroll limit.
    await page.evaluate(() =>
      scrollTo({ top: document.documentElement.scrollHeight + innerHeight, behavior: "instant" }),
    );
    await paintScroll(page);
    const bottom = await page.evaluate(() => scrollY);
    await expect(page.locator(".mash-footer")).toBeInViewport();
    const offscreenStages = () =>
      stages.evaluateAll((items) =>
        items.every((item) => item.getBoundingClientRect().bottom <= 0),
      );
    expect(await offscreenStages()).toBe(true);
    await page.waitForTimeout(500);
    expect(await offscreenStages()).toBe(true);
    expect(await page.evaluate(() => scrollY)).toBe(bottom);
  });
}

test("a direct scene link skips replaying earlier scenes", async ({ page }) => {
  await page.goto("/#welcome-memory");
  await readyWelcome(page);
  await paintScroll(page);
  const heading = page.getByRole("heading", { name: "Save the decision. Keep the source." });
  await expect(heading).toBeInViewport();
  for (const id of ["welcome-search", "welcome-mashy"]) {
    await expect(page.locator(`#${id} .stage`)).toHaveCSS("opacity", "0");
  }
});

test("scrolling back draws a scene again and then undraws its entrance", async ({ page }) => {
  await page.goto("/");
  await readyWelcome(page);
  const scene = page.locator("#welcome-mashy");
  const strokes = scene.locator(".art path");
  const painted = () =>
    strokes.evaluateAll(
      (paths) => paths.filter((path) => Number(getComputedStyle(path).opacity) > 0).length,
    );
  expect(await painted()).toBe(0);
  await scrub(scene, -0.15);
  const entering = await painted();
  expect(entering).toBeGreaterThan(0);
  const all = await strokes.count();
  expect(entering).toBeLessThan(all);
  await scrub(scene, 0.05);
  expect(await painted()).toBe(all);

  await scrub(page.locator("#welcome-memory"), 0.3);
  expect(await painted()).toBe(0);
  await scrub(scene, 0.05);
  expect(await painted()).toBe(all);
  await scrub(scene, -0.15);
  expect(await painted()).toBe(entering);
  await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
  await paintScroll(page);
  expect(await painted()).toBe(0);
});

test("a scene's enlarged heading stays inside the screen", async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 700 });
  await page.goto("/");
  await readyWelcome(page);
  await scrub(page.locator("#welcome-speed"), 0.1);
  const heading = page.getByRole("heading", { name: "Fast, and light on your machine." });
  await expect(async () => {
    const box = await heading.boundingBox();
    expect(box?.x).toBeGreaterThanOrEqual(0);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(1000);
  }).toPass();
});

test("the Mashy scene finishes with the model service requirement", async ({ page }) => {
  await page.goto("/");
  await readyWelcome(page);
  const scene = page.locator("#welcome-mashy");
  await scrub(scene, 0.8);
  const last = scene.locator(".caps li").last();
  await expect(last).toHaveCSS("opacity", "1");
  await expect(scene.locator(".caps li").first()).toHaveCSS("opacity", "0");
  await expect(last).toHaveText("Optional. Requires your own model service and API key.");
});

test("the welcome page shows everything without scripting", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("/");
  const welcome = page.locator(".mash-welcome");
  const images = welcome.getByRole("img");
  await expect(images).toHaveCount(3);
  await expect(images.first()).toHaveAccessibleName(/why does checkout keep failing/);
  await expect(welcome.locator("[data-icon-tile] svg.ico").first()).toBeVisible();
  await expect
    .poll(() =>
      images.evaluateAll((items) =>
        items.every((item) => item instanceof HTMLImageElement && item.naturalHeight > 0),
      ),
    )
    .toBe(true);
  for (const image of await images.all()) await expect(image).toBeVisible();
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

for (const viewport of [
  { width: 320, height: 640 },
  { width: 390, height: 700 },
  { width: 740, height: 360 },
]) {
  test(`the search illustration fits an active ${viewport.width}×${viewport.height} scene`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await page.goto("/#welcome-search");
    await readyWelcome(page);
    const scene = page.locator("#welcome-search");
    await scrub(scene, 0.3);
    const image = scene.locator(".scene-image");
    await expect(scene.locator(".show")).toHaveCSS("opacity", "1");
    await expect(image).toBeInViewport({ ratio: 0.98 });
    const box = await image.boundingBox();
    expect(box).not.toBeNull();
    if (!box) throw new Error("Missing search illustration bounds");
    const ratio = await image.evaluate(
      (img: HTMLImageElement) => img.naturalWidth / img.naturalHeight,
    );
    expect(box.width / box.height).toBeCloseTo(ratio, 1);
  });
}
