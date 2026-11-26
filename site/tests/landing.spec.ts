import { expect, type Page, test } from "@playwright/test";

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
  await expect(page.locator(".hero .tagline")).toHaveText(`${blurb} Read the docs →`);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", blurb);
  // Link previews carry the tagline.
  const tagline = "Supercharged context for your agents.";
  await expect(page.locator('meta[property="og:description"]')).toHaveAttribute("content", tagline);
  await expect(page.locator('meta[property="og:title"]')).toHaveAttribute(
    "content",
    `Mash: ${tagline}`,
  );
  await expect(page.locator(".hero .actions a")).toHaveText([
    /^\s*Download for \w+$/,
    "Try the demo",
  ]);
  await expect(page.locator(".mash-welcome h2")).toHaveText([
    "One history. Everywhere you work.",
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
  // Five of them are the showcase's surfaces, of which one shows at a time.
  await expect(illustrations).toHaveCount(9);
  for (const illustration of await illustrations.all()) {
    await expect(illustration).toHaveAttribute("aria-label", /\w/);
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
  // The demo is a button of its own beside the download, the same height.
  const demo = page.locator(".hero .actions a").nth(1);
  await expect(demo).toHaveText("Try the demo");
  await expect(demo).toHaveAttribute("href", "/demo/");
  await expect(demo.locator("svg")).toBeVisible();
  const [downloadBox, demoBox] = [await download.boundingBox(), await demo.boundingBox()];
  expect(demoBox?.height).toBe(downloadBox?.height);
  expect(demoBox?.y).toBe(downloadBox?.y);
  expect(demoBox?.x).toBeGreaterThan((downloadBox?.x ?? 0) + (downloadBox?.width ?? 0));
  // The docs follow the opening paragraph as its last words, in bold.
  const docs = page.locator(".hero .tagline a.docs");
  await expect(docs).toHaveText("Read the docs →");
  await expect(docs).toHaveAttribute("href", "/guides/");
  expect(
    Number(await docs.evaluate((link) => getComputedStyle(link).fontWeight)),
  ).toBeGreaterThanOrEqual(600);
  // The row of quieter ways in has a bar only between its links.
  await expect(note.locator(".ways a")).toHaveText(["Install with curl", "Raycast extension"]);
  await expect(note.locator(".ways > :last-child")).toHaveText("Raycast extension");
  // The header offers the Mac app beside the docs and the source.
  const header = page.locator("header.header");
  await expect(
    header.getByRole("link", { name: /^Download for (Mac|Windows|Linux)$/ }),
  ).toBeVisible();
  await expect(header.getByRole("link", { name: "Docs", exact: true })).toBeVisible();
  await expect(header.getByRole("link", { name: "GitHub", exact: true })).toBeVisible();
});

test("on a phone the landing header is the logo alone, and the hero's buttons stack", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  const header = page.locator("header.header");
  await expect(header.locator("a.brand")).toBeVisible();
  // No menu button and no links: the hero and the footer lead to the docs.
  await expect(header.locator(".mash-menu, .sl-menu-button")).toHaveCount(0);
  await expect(header.getByRole("link")).toHaveCount(1);
  await expect(header.getByRole("button")).toHaveCount(0);
  const buttons = page.locator(".hero .actions a");
  await expect(buttons).toHaveText([/Download for (Linux|Mac|Windows)|Download/, "Try the demo"]);
  const [first, second] = [await buttons.nth(0).boundingBox(), await buttons.nth(1).boundingBox()];
  expect(second?.x).toBe(first?.x);
  expect(second?.width).toBe(first?.width);
  expect(second?.height).toBe(first?.height);
  expect(second?.y).toBeGreaterThan((first?.y ?? 0) + (first?.height ?? 0));
  expect(first?.width).toBeGreaterThan(300);
  // The ways in wrap on a phone, so no bar is left hanging at a line's end.
  const ways = page.locator(".hero .ways");
  await expect(page.locator(".hero .tagline a.docs")).toBeVisible();
  await expect(ways.locator("span")).toHaveCount(1);
  for (const bar of await ways.locator("span").all()) await expect(bar).toBeHidden();
  await expect(
    page.getByRole("navigation", { name: "Footer" }).getByRole("link", { name: "Docs" }),
  ).toHaveAttribute("href", "/guides/");
});

test("every feature tile draws its icon in the same chip, the systems one each", async ({
  page,
}) => {
  await page.goto("/");
  const more = page.getByRole("region", { name: "Fits the way you work." });
  await expect(more.getByRole("heading", { level: 3 })).toHaveText([
    "Context, before you ask.",
    "RAG over your own history.",
    "macOS. Linux. Windows.",
    "Every machine.",
    "Your chats, exported.",
  ]);
  await expect(more.locator("article").last()).toContainText("Markdown, HTML or PDF");
  const chips = more.locator(".tiles .ico");
  await expect(chips).toHaveCount(7);
  const systems = more.locator(".tiles .os-chips .ico");
  await expect(systems).toHaveCount(3);
  await expect(systems.locator("svg.os-mark")).toHaveCount(3);
  const looks = await chips.evaluateAll((all) =>
    all.map((chip) => {
      const { width, height, borderRadius, backgroundColor, color } = getComputedStyle(chip);
      const icon = chip.querySelector("svg")?.getBoundingClientRect();
      return [width, height, borderRadius, backgroundColor, color, icon?.width, icon?.height];
    }),
  );
  for (const each of looks) expect(each).toEqual(looks[0]);
  // The three marks sit in a row.
  const boxes = await systems.evaluateAll((all) =>
    all.map((chip) => chip.getBoundingClientRect().toJSON()),
  );
  expect(new Set(boxes.map((box) => box.y)).size).toBe(1);
  expect(boxes[1].x).toBeGreaterThan(boxes[0].x + boxes[0].width);
  expect(boxes[2].x).toBeGreaterThan(boxes[1].x + boxes[1].width);
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
  await expect(page.locator(".hero .actions a").first()).toBeVisible();
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

test("each welcome band comes in as it scrolls into view, whatever the motion preference", async ({
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
    expect(await opacity()).toBe("0");
    await band.scrollIntoViewIfNeeded();
    await expect(band).toHaveClass(/\bshown\b/);
    await expect.poll(opacity).toBe("1");
    // Once in, the band is the stylesheet's again.
    await expect.poll(() => band.getAttribute("style")).toBeFalsy();
    await context.close();
  }
});

for (const [name, viewport] of [
  ["a wide screen", { width: 1440, height: 900 }],
  ["a phone", { width: 390, height: 844 }],
] as const) {
  test(`on ${name} a stroke of butter is drawn under the headline's last line alone, as wide as its words`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await page.goto("/");
    const hero = page.locator(".mash-hero");
    const lines = hero.locator("h1 > span");
    await expect(hero.locator("h1")).toHaveText("Your agents forget. Mash doesn’t.");
    await expect(lines).toHaveCount(2);
    await expect(lines.nth(0).locator("svg")).toHaveCount(0);
    await expect(hero.locator("h1 svg")).toHaveCount(1);
    const rule = lines.nth(1).locator(".accent svg.rule");
    // Once the entrance has played the stroke is whole, and the stylesheet's again.
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    await expect(hero).not.toHaveAttribute("data-motion");
    const draw = rule.locator(".draw");
    await expect.poll(() => draw.getAttribute("style")).toBeFalsy();
    await expect(draw).toHaveCSS("stroke-dashoffset", "0px");
    await expect(rule).toBeVisible();
    // The pat of butter in the wordmark, and the words stay ink.
    await expect(rule).toHaveCSS("fill", "rgb(245, 200, 75)");
    const ink = await lines.nth(0).evaluate((line) => getComputedStyle(line).color);
    await expect(lines.nth(1).locator(".accent")).toHaveCSS("color", ink);
    const at = await lines
      .nth(1)
      .locator(".accent")
      .evaluate((accent) => {
        const words = document.createRange();
        words.selectNodeContents(accent);
        const box = (rect: DOMRect) => ({
          left: rect.left,
          right: rect.right,
          top: rect.top,
          bottom: rect.bottom,
        });
        const stroke = accent.querySelector("svg > path");
        const tagline = document.querySelector(".hero .tagline");
        if (!stroke || !tagline) throw new Error("the stroke or the tagline is missing");
        return {
          words: box(words.getBoundingClientRect()),
          stroke: box(stroke.getBoundingClientRect()),
          size: Number.parseFloat(getComputedStyle(accent).fontSize),
          tagline: tagline.getBoundingClientRect().top,
        };
      });
    // The words are on one line, and the stroke runs from their first edge to their last.
    expect(at.words.bottom - at.words.top).toBeLessThan(1.5 * at.size);
    expect(Math.abs(at.stroke.left - at.words.left)).toBeLessThan(4);
    expect(Math.abs(at.stroke.right - at.words.right)).toBeLessThan(4);
    // It lies under the letters, clear of the paragraph below.
    expect(at.stroke.top).toBeGreaterThan(at.words.top + 0.7 * at.size);
    expect(at.stroke.bottom).toBeLessThan(at.tagline);
  });
}

/** Scrolls to the bottom in small steps, as a reader would, so every scene plays. */
async function scrollThrough(page: Page): Promise<void> {
  for (;;) {
    const left = await page.evaluate(
      () => document.documentElement.scrollHeight - innerHeight - scrollY,
    );
    if (left <= 1) break;
    await page.mouse.wheel(0, 240);
    await page.waitForTimeout(60);
  }
}

for (const [name, viewport] of [
  ["a wide screen", { width: 1440, height: 900 }],
  ["a phone", { width: 390, height: 844 }],
] as const) {
  test(`on ${name} every scene ends as the page is served, and nothing changes place`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await page.goto("/");
    const hero = page.locator(".mash-hero");
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    await expect(hero).not.toHaveAttribute("data-motion");
    for (const line of await hero.locator("h1 span").all())
      await expect(line).toHaveCSS("opacity", "1");
    await expect(hero.locator(".actions")).toHaveCSS("opacity", "1");
    // Count layout shifts from here on: the scenes themselves must add none.
    await page.evaluate(() => {
      const seen = { shift: 0 };
      Object.assign(window, { mashShift: seen });
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries() as (PerformanceEntry & {
          value: number;
          hadRecentInput: boolean;
        })[])
          if (!entry.hadRecentInput) seen.shift += entry.value;
      }).observe({ type: "layout-shift" });
    });
    const height = () => page.evaluate(() => document.documentElement.scrollHeight);
    const before = await height();
    await scrollThrough(page);

    const welcome = page.locator(".mash-welcome");
    // Quick search: the whole query, every result, and the selection two rows down.
    await expect(welcome.locator("#welcome-shortcuts .quick-q b")).toHaveText("refresh token");
    await expect(welcome.locator("#welcome-shortcuts .quick li")).toHaveCount(5);
    for (const row of await welcome.locator("#welcome-shortcuts .quick li").all())
      await expect(row).toHaveCSS("opacity", "1");
    const rows = await welcome
      .locator("#welcome-shortcuts .quick li")
      .evaluateAll((items) => items.map((item) => item.getBoundingClientRect().top));
    await expect
      .poll(() =>
        welcome
          .locator("#welcome-shortcuts .quick-sel")
          .evaluate((node) => node.getBoundingClientRect().top),
      )
      .toBeCloseTo(rows[2] ?? 0, 0);
    // Search: three results, and each match highlighted by the stylesheet again.
    for (const result of await welcome.locator(".search-window .result").all())
      await expect(result).toHaveCSS("opacity", "1");
    for (const match of await welcome.locator(".search-window mark").all())
      await expect.poll(() => match.getAttribute("style")).toBeFalsy();
    // Memory: the decision, the two steps and the later session.
    for (const part of await welcome.locator(".memory-thread > *").all()) {
      await expect(part).toHaveCSS("opacity", "1");
      await expect.poll(() => part.getAttribute("style")).toBeFalsy();
    }
    // The agents, the figures, Mashy's first answer and the closing card.
    for (const agent of await welcome.locator(".mash-agents li").all())
      await expect(agent).toHaveCSS("opacity", "1");
    await expect(welcome.locator(".stats dt b")).toHaveText(["66", "103", "5", "9"]);
    await expect(welcome.locator('.example[data-example="1"] .answer p').first()).toHaveText(
      "A database migration locked the orders table for about four minutes. Claude Code described the lock on the build box:",
    );
    for (const part of await welcome.locator(".closing > *").all())
      await expect(part).toHaveCSS("opacity", "1");
    // Nothing is left part of the way in.
    await expect
      .poll(() =>
        page.evaluate(() =>
          [...document.querySelectorAll<HTMLElement>(".mash-hero *, .mash-welcome [data-reveal]")]
            .filter((node) => node.style.opacity !== "" && node.style.opacity !== "1")
            .map((node) => node.className),
        ),
      )
      .toEqual([]);
    expect(
      await page.evaluate(
        () => (window as unknown as { mashShift: { shift: number } }).mashShift.shift,
      ),
    ).toBeLessThan(0.001);
    // Only the pinned panel adds to the page, and only on a wide screen.
    expect(await height()).toBe(before);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= document.documentElement.clientWidth,
      ),
    ).toBe(true);
  });
}

test("the pinned search panel holds for a screen and a half, then lets the page go on", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  const desk = page.locator("#welcome-shortcuts .desk");
  const field = page.locator("#welcome-shortcuts .quick-q b");
  const top = () => desk.evaluate((node) => Math.round(node.getBoundingClientRect().top));
  // Before the pin the field waits empty; in it the page stays put while the search types.
  await expect(field).toHaveText("Search every chat");
  const start = await desk.evaluate(
    (node) => node.getBoundingClientRect().top + scrollY - innerHeight * 0.12,
  );
  await page.evaluate((y) => scrollTo(0, y + 300), start);
  await expect.poll(top).toBe(108);
  await expect(field).toHaveText(/^refresh t?o?$/);
  await page.evaluate((y) => scrollTo(0, y + 1000), start);
  await expect.poll(top).toBe(108);
  await expect(field).toHaveText("refresh token");
  // Past a screen and a half the panel scrolls away and the next band arrives.
  await page.evaluate((y) => scrollTo(0, y + 1350 + 400), start);
  await expect.poll(top).toBe(108 - 400);
  await page.locator("#welcome-search").scrollIntoViewIfNeeded();
  await expect(page.locator("#welcome-search .search-window")).toBeInViewport();
  // Scrolling back up runs it backwards.
  await page.evaluate((y) => scrollTo(0, y - 200), start);
  await expect(field).toHaveText("Search every chat");
});

test("links into the page land on their target with the pinned panel above it", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  for (const id of ["install", "welcome-search", "welcome-memory"]) {
    await page.goto(`/#${id}`);
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    await expect(page.locator(`#${id}`)).toBeInViewport();
    const top = await page.locator(`#${id}`).evaluate((node) => node.getBoundingClientRect().top);
    expect(top).toBeGreaterThanOrEqual(0);
    expect(top).toBeLessThan(200);
  }
  // A link followed on the page itself gets there too.
  await page.goto("/");
  await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
  await page.locator(".hero .ways").getByRole("link", { name: "Install with curl" }).click();
  await expect(page.locator("#install")).toBeInViewport();
});

test("the welcome page shows everything without scripting", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto("/");
  const welcome = page.locator(".mash-welcome");
  // The showcase's surfaces aside (surfaces.spec.ts): one of those shows at a time.
  const windows = welcome.locator("[role=img]:not(.picture)");
  await expect(windows).toHaveCount(4);
  await expect(windows.nth(1)).toHaveAccessibleName(/Why does checkout keep failing/);
  await expect(welcome.locator(".ico svg").first()).toBeVisible();
  for (const picture of await windows.all()) await expect(picture).toBeVisible();
  const mashy = welcome.locator("img.mashy-face");
  await expect
    .poll(() => mashy.evaluate((img: HTMLImageElement) => img.naturalHeight))
    .toBeGreaterThan(0);
  // Nothing waits for the motion: the hero is unmarked and every scene is in its final state.
  await expect(page.locator(".mash-hero")).not.toHaveAttribute("data-motion");
  for (const part of await page
    .locator(".hero :is(h1 span, .tagline, .actions, .download-note)")
    .all()) {
    await expect(part).toBeVisible();
    await expect(part).toHaveCSS("opacity", "1");
  }
  // The stroke under the headline's last line is served whole.
  const stroke = page.locator(".hero h1 > span").nth(1).locator("svg.rule");
  await expect(stroke).toBeVisible();
  await expect(stroke.locator(".draw")).toHaveCSS("stroke-dashoffset", "0px");
  await expect(welcome.locator("#welcome-shortcuts .quick-q b")).toHaveText("refresh token");
  await expect(welcome.locator("#welcome-shortcuts .quick li.on")).toHaveCount(1);
  for (const part of await welcome
    .locator(
      "#welcome-shortcuts .quick li, .search-window .result, .memory-thread > *, .mash-agents li, .stats > div, .closing > *",
    )
    .all()) {
    await expect(part).toBeVisible();
    await expect(part).toHaveCSS("opacity", "1");
  }
  await expect(welcome.locator(".stats dt b")).toHaveText(["66", "103", "5", "9"]);
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
    await expect(tile.locator(".ico svg").first()).toBeVisible();
  // The systems' tile shows their three marks, each in a chip.
  await expect(tiles.nth(2).locator(".os-chips .ico svg")).toHaveCount(3);
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
