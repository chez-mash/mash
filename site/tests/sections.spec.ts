import { expect, test } from "@playwright/test";

// The bands that go deeper than the showcase, in the order the page shows
// them: each has a claim, a few plain sentences or points, and a way onward.
const order = [
  "welcome-surfaces",
  "welcome-shortcuts",
  "welcome-search",
  "welcome-agents",
  "welcome-memory",
  "welcome-reading",
  "welcome-machines",
  "welcome-privacy",
  "welcome-mashy",
  "welcome-speed",
  "welcome-platforms",
  "welcome-features",
  "welcome-rust",
];

test("the landing page goes from the overview to one band per capability, in order", async ({
  page,
}) => {
  await page.goto("/");
  const ids = await page
    .locator(".mash-welcome > section[id]")
    .evaluateAll((bands) => bands.map((band) => band.id));
  expect(ids).toEqual(order);
  // Neighbouring bands never share the quieter surface.
  const tinted = await page
    .locator(".mash-welcome > section[id]")
    .evaluateAll((bands) =>
      bands.map(
        (band) => band.classList.contains("tint") && !band.classList.contains("mashy-band"),
      ),
    );
  for (let at = 1; at < tinted.length; at += 1)
    expect(tinted[at] && tinted[at - 1], order[at]).toBe(false);
});

for (const band of [
  {
    id: "welcome-search",
    links: ["/search/"],
    says: ["forgiving a typo", "BM25 and embeddings"],
  },
  {
    id: "welcome-memory",
    links: ["/records/"],
    says: ["Decisions, preferences and notes"],
  },
  {
    id: "welcome-reading",
    links: ["/dashboard/#find-and-read-a-chat"],
    says: ["Subagents stay with the chat that started them", "Markdown, HTML or PDF"],
    picture: /four tool calls folded into one line/,
  },
  {
    id: "welcome-machines",
    links: ["/search-across-machines/", "/shared-index/"],
    says: ["a Mash server you run", "Sharing is off until you set it up"],
    picture: /asks three machines at once/,
  },
  {
    id: "welcome-platforms",
    links: [
      "/downloads/#desktop-app",
      "/installation/",
      "https://github.com/mashhq/mash/tree/main/raycast",
    ],
    says: ["It keeps itself up to date", "mash init"],
  },
  {
    id: "welcome-features",
    links: ["/mcp/", "/search-languages/", "/dashboard-access/", "/backup/", "/troubleshooting/"],
    says: ["names what is wrong"],
  },
]) {
  test(`the ${band.id} band says what is true and leads to its guides`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");
    await expect(page.locator(".mash-welcome")).toHaveClass(/\breveal\b/);
    const section = page.locator(`#${band.id}`);
    await section.locator("h2").scrollIntoViewIfNeeded();
    await expect(section.locator("[data-reveal]")).toHaveClass(/\bshown\b/);
    await expect(section.getByRole("heading", { level: 2 })).toBeVisible();
    for (const words of band.says) await expect(section).toContainText(words);
    for (const href of band.links) {
      const link = section.locator(`a[href="${href}"]`);
      await expect(link, href).toHaveCount(1);
      await link.scrollIntoViewIfNeeded();
      await expect(link, href).toBeVisible();
    }
    if (band.picture) {
      const picture = section.getByRole("img");
      await expect(picture).toHaveAccessibleName(band.picture);
      const box = await picture.boundingBox();
      expect(box?.x).toBeGreaterThanOrEqual(0);
      expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(390);
    }
    // Nothing in the band runs past the screen, and the page has no sideways scroll.
    const widest = await section.evaluate((node) =>
      Math.max(
        ...[...node.querySelectorAll("*")].map((part) => part.getBoundingClientRect().right),
      ),
    );
    expect(widest).toBeLessThanOrEqual(390);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
  });
}

test("the guides the new bands lead to exist, with the sections they name", async ({ page }) => {
  for (const target of [
    "/search/",
    "/records/",
    "/dashboard/#find-and-read-a-chat",
    "/search-across-machines/",
    "/shared-index/",
    "/installation/",
    "/mcp/",
    "/search-languages/",
    "/dashboard-access/",
    "/backup/",
    "/troubleshooting/",
  ]) {
    const response = await page.goto(target);
    expect(response?.status(), target).toBe(200);
    const [, anchor] = target.split("#");
    if (anchor) await expect(page.locator(`[id="${anchor}"]`), target).toHaveCount(1);
  }
});
