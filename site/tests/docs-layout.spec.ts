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
    "Windows x64 or ARM64 (PowerShell)",
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

test("a guide shows its screenshots and the agent list uses tiles, answers and marks", async ({
  page,
}) => {
  await page.goto("/dashboard/");
  await expect(page.locator("h1")).toHaveText("Use the dashboard");
  await expect(page.locator('.sl-markdown-content img[src^="/screenshots/"]')).toHaveCount(3);
  await page.goto("/supported-agents/");
  const first = page.locator(".sl-markdown-content table").first();
  await expect(first.locator(".mash-agent-tile")).toHaveCount(7);
  await expect(
    first.getByRole("row", { name: /^Claude Code/ }).locator(".mash-answer.yes"),
  ).toHaveText("Yes");
  // A dash in the connect column is a minus that reads aloud as no.
  await expect(first.getByRole("row", { name: /^OpenCode/ }).locator(".mash-answer.no")).toHaveText(
    "No",
  );
  // The tested-with table heads its system columns with their marks.
  const tested = page.locator(".sl-markdown-content table").nth(1);
  await expect(tested.locator("thead .mash-os")).toHaveCount(3);
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

test("on a phone every page's menu button is the same square at the left with three lines", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  // What a button looks like and where it and its neighbours sit.
  const measure = async (button: string) => {
    const box = async (selector: string) => {
      const found = await page.locator(selector).filter({ visible: true }).first().boundingBox();
      return found && [found.x, found.y, found.width, found.height].map(Math.round);
    };
    const look = await page.locator(button).evaluate((element) => {
      const { borderRadius, backgroundColor, boxShadow, color } = getComputedStyle(element);
      const lines = element.querySelector("svg:not(.close-menu)")?.innerHTML;
      return { borderRadius, backgroundColor, boxShadow, color, lines };
    });
    return {
      button: await box(button),
      glyph: await box(`${button} svg`),
      brand: (await box("header a.brand"))?.slice(0, 2),
      search: await box("site-search button[data-open-modal]"),
      ...look,
    };
  };
  // A page with a sidebar has Starlight's button, which the others match.
  await page.goto("/quick-start/");
  const sidebar = await measure(".sl-menu-button");
  expect(sidebar.button).toEqual([16, 10, 36, 36]);
  // Search is the same square, ending at the gutter on the other side.
  expect(sidebar.search).toEqual([338, 10, 36, 36]);
  expect(sidebar.lines).toContain("<path");
  // The guides directory is a docs page: the same button opens the sidebar.
  for (const path of ["/guides/", "/es/guides/"]) {
    await page.goto(path);
    await expect(page.locator(".mash-menu")).toHaveCount(0);
    expect(await measure(".sl-menu-button"), path).toEqual(sidebar);
  }
  // The missing page has no sidebar, and its own menu matches the button.
  await page.goto("/no-such-page/");
  await expect(page.locator(".sl-menu-button")).toHaveCount(0);
  const own = await measure(".mash-menu summary");
  expect(own).toEqual(sidebar);
  // The logo follows the button and search ends the row.
  const [button, brand, search] = [own.button?.[0], own.brand?.[0], own.search?.[0]];
  expect(brand).toBeGreaterThan((button ?? 0) + 36);
  expect(search).toBeGreaterThan(brand ?? 0);
  // It still opens the links and the language menu; the site has no theme menu.
  await page.locator(".mash-menu summary").click();
  const panel = page.locator(".mash-menu .panel");
  await expect(panel.locator("starlight-lang-select select")).toBeVisible();
  await expect(page.locator("starlight-theme-select")).toHaveCount(0);
});

// What a sidebar lists: its section labels, then every link's words and address.
const sidebarIndex = (sidebar: import("@playwright/test").Locator) =>
  sidebar.evaluate((nav) => ({
    sections: [...nav.querySelectorAll(".top-level > li > details > summary")].map((label) =>
      label.textContent?.trim(),
    ),
    links: [...nav.querySelectorAll<HTMLAnchorElement>(".mash-sidebar a")].map((link) => [
      link.textContent?.trim(),
      link.getAttribute("href"),
    ]),
    controls: [...nav.querySelectorAll<HTMLAnchorElement>(".mash-menu-top a")].map((link) => [
      link.textContent?.trim(),
      link.getAttribute("href"),
    ]),
  }));

for (const [directory, guide, quickStart] of [
  ["/guides/", "/quick-start/", "Quick start"],
  ["/es/guides/", "/es/quick-start/", "Guía de inicio rápido"],
] as const) {
  test(`on a phone the menu on ${directory} is the docs sidebar every guide shows`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(guide);
    const expected = await sidebarIndex(page.locator("nav.sidebar"));
    expect(expected.sections.length).toBe(6);

    await page.goto(directory);
    const menu = page.locator("nav.sidebar");
    const pane = menu.locator("#starlight__sidebar");
    await expect(pane).toBeHidden();
    await expect(page.locator(".mash-menu")).toHaveCount(0);
    // The page still stands alone: no breadcrumb over its title.
    await expect(page.locator(".mash-crumbs")).toHaveCount(0);
    const button = page.locator(".sl-menu-button");
    await button.click();
    await expect(pane).toBeVisible();
    // The same sections, links and order as a guide's sidebar.
    expect(await sidebarIndex(menu)).toEqual(expected);
    await expect(menu.getByRole("link", { name: quickStart, exact: true })).toHaveAttribute(
      "href",
      guide,
    );
    // The directory is the current page there.
    await expect(menu.locator('.mash-sidebar a[aria-current="page"]')).toHaveAttribute(
      "href",
      directory,
    );
    // The links and both menus lead the panel, as on a guide, and it fills
    // the screen under the header.
    await expect(menu.locator(".mash-menu-top a.mash-nav")).toHaveCount(3);
    await expect(menu.locator('.mash-menu-top a[href="/demo/"]')).toBeInViewport();
    await expect(
      menu.locator('.mash-menu-top a[href="https://github.com/mashhq/mash"]'),
    ).toBeInViewport();
    await expect(menu.locator("starlight-lang-select select")).toBeInViewport();
    await expect(menu.locator("starlight-theme-select")).toHaveCount(0);
    const box = await pane.boundingBox();
    expect(box && [box.x, box.width, Math.round(box.y + box.height)]).toEqual([0, 390, 844]);
    // The same button closes it.
    await button.click();
    await expect(pane).toBeHidden();
    // A link in it leads to its guide, whose sidebar marks that page.
    await button.click();
    await menu.getByRole("link", { name: quickStart, exact: true }).click();
    await expect(page).toHaveURL(guide);
    await expect(page.locator('nav.sidebar .mash-sidebar a[aria-current="page"]')).toHaveAttribute(
      "href",
      guide,
    );
  });
}

test("on a phone the guides directory's tip is as wide as the cards under it", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ["/guides/", "/es/guides/"]) {
    await page.goto(path);
    const tip = await page.locator(".mash-guides-intro .mash-mashy").boundingBox();
    const card = await page.locator(".mash-guides .mash-guide").first().boundingBox();
    expect([tip?.x, tip?.width], path).toEqual([card?.x, card?.width]);
    expect(card?.width).toBe(358);
  }
});

test("on a wide screen the guides directory keeps the full width, with no sidebar column", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/guides/");
  await expect(page.locator("nav.sidebar")).toBeHidden();
  await expect(page.locator(".sl-menu-button")).toBeHidden();
  // The header holds what the phone's menu does.
  const header = page.locator("header.header");
  await expect(header.getByRole("link", { name: "GitHub" })).toBeVisible();
  await expect(header.locator("starlight-lang-select select")).toBeVisible();
  // The cards sit in the middle of the page, three to a row, with the tip
  // beside the lead at its own width.
  const cards = page.locator(".mash-guides");
  const box = await cards.boundingBox();
  expect(Math.round((box?.x ?? 0) * 2 + (box?.width ?? 0))).toBe(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
  expect(box?.width).toBeGreaterThan(1000);
  const tip = await page.locator(".mash-guides-intro .mash-mashy").boundingBox();
  expect(Math.round(tip?.width ?? 0)).toBe(288);
});
