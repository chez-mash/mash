import { expect, type Page, test } from "@playwright/test";
import { recordFixture } from "./record-fixtures";

// Pages that do the same thing say it the same way: one refresh button,
// "any …" filter chips, a page title, and short labels with full details on hover.

const chats = [
  {
    chat_id: "chat-1",
    provider: "codex",
    title: "Tune the export queue",
    node_id: "laptop",
    project: "/home/sam/code/billing",
    updated_at: Date.now() - 3_600_000,
  },
  {
    chat_id: "chat-2",
    provider: "claude-code",
    title: "Pick a token lifetime",
    node_id: "workstation",
    project: "/home/sam/code/storefront",
    updated_at: Date.now() - 7_200_000,
  },
];

async function mockDashboard(page: Page, record = recordFixture()): Promise<URL[]> {
  const recordRequests: URL[] = [];
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/chats")
      return route.fulfill({ json: { local_node: "laptop", truncated: false, chats } });
    if (url.pathname === "/api/records") {
      recordRequests.push(url);
      return route.fulfill({ json: { records: [record], next_offset: null } });
    }
    if (url.pathname === "/api/record") return route.fulfill({ json: record });
    return route.fulfill({ status: 404, json: { error: { message: "Not in this fixture" } } });
  });
  return recordRequests;
}

test("every screen has exactly one Check for updates, at the foot of the rail", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  const recordRequests = await mockDashboard(page);
  const visible = () =>
    page
      .getByRole("button", { name: "Check for updates", exact: true })
      .evaluateAll((buttons) => buttons.filter((button) => button.checkVisibility()).length);
  for (const address of [
    "/#/",
    "/#/search",
    "/#/records?",
    "/#/record/preference-1",
    "/#/console",
  ]) {
    await page.goto(address);
    await expect(
      page.locator("#rail").getByRole("button", { name: "Check for updates" }),
    ).toBeVisible();
    expect(await visible(), address).toBe(1);
  }
  for (const old of [/reload chat list/i, /search again/i, /check jobs/i])
    await expect(page.getByRole("button", { name: old })).toHaveCount(0);
  // The rail's button reads the page's own data again, not only the chat list.
  await page.goto("/#/records?");
  await expect(page.getByRole("main").getByRole("article").first()).toBeVisible();
  const before = recordRequests.length;
  await page.locator("#rail").getByRole("button", { name: "Check for updates" }).click();
  await expect.poll(() => recordRequests.length).toBeGreaterThan(before);
});

test("memory has no rows of its own in the rail; its kinds are the page's tabs", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockDashboard(page);
  await page.goto("/#/records?kind=decision");
  const rail = page.locator("#rail");
  await expect(rail.getByRole("button", { name: "Expand Memory" })).toHaveCount(0);
  await expect(rail.getByRole("link", { name: "decisions", exact: true })).toHaveCount(0);
  await expect(rail.getByRole("link", { name: "Memory", exact: true })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    page
      .getByRole("navigation", { name: "Memory kind" })
      .getByRole("link", { name: "Decisions", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  // Chats keep their rows: they are pages of their own, not filters.
  await rail.getByRole("button", { name: "Expand Chats" }).click();
  await expect(rail.getByRole("link", { name: "Viewing history", exact: true })).toBeVisible();
});

test("the overview links to every chat in words, with the count in its title line", async ({
  page,
}) => {
  await mockDashboard(page);
  await page.goto("/");
  const all = page.getByRole("main").getByRole("link", { name: "All chats", exact: true });
  await expect(all).toHaveAttribute("href", "#/search");
  await expect(page.getByRole("main").getByText(/^\d+–\d+ of \d+$/)).toHaveCount(0);
  await expect(page.getByText(/^2 chats from \d+ agents? on 2 machines\./)).toBeVisible();
});

test("on a phone the search filters fold behind one Filters button that counts them", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockDashboard(page);
  await page.goto("/#/search?agent=codex&days=7");
  const main = page.getByRole("main");
  const toggle = main.getByRole("button", { name: /^Filters/ });
  await expect(toggle).toHaveText("Filters2");
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  const filters = main.getByRole("group", { name: "Filters" });
  await expect(filters).toBeHidden();
  const first = await main.getByRole("link", { name: /Tune the export queue/ }).boundingBox();
  expect(first?.y ?? 1000).toBeLessThan(400);
  await toggle.click();
  await expect(filters.getByRole("button", { name: "any project", exact: true })).toBeVisible();
  // Wide screens show the chips themselves and no button.
  await page.setViewportSize({ width: 1024, height: 768 });
  await expect(toggle).toBeHidden();
  await expect(filters).toBeVisible();
});

test("filter chips name what they leave open the same way on every page", async ({ page }) => {
  await mockDashboard(page);
  await page.goto("/#/search");
  const filters = page.getByRole("group", { name: "Filters" });
  for (const name of ["any agent", "any project", "any time", "sort: newest"])
    await expect(filters.getByRole("button", { name, exact: true })).toBeVisible();
  // Which machines is the viewing scope's to say, not a chip's.
  await expect(filters.getByRole("button", { name: "any machine", exact: true })).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Chats", exact: true })).toBeVisible();
  await page.goto("/#/projects");
  const projects = page.getByRole("group", { name: "Project filters" });
  for (const name of ["any agent", "any machine", "sort: newest"])
    await expect(projects.getByRole("button", { name, exact: true })).toBeVisible();
  // The search box is shaped like the chips beside it.
  const find = projects.locator(".find");
  const chip = projects.getByRole("button", { name: "any agent", exact: true });
  const radius = (locator: typeof find) =>
    locator.evaluate((element) => getComputedStyle(element).borderTopLeftRadius);
  expect(await radius(find)).toBe(await radius(chip));
});

test("the overview greets the reader and its chat rows name the agent, machine and project", async ({
  page,
}) => {
  await mockDashboard(page);
  await page.goto("/");
  await expect(
    page.getByRole("heading", { level: 1, name: /^Good (morning|afternoon|evening)$/ }),
  ).toBeVisible();
  const row = page
    .getByRole("main")
    .getByRole("region", { name: "Recent chats" })
    .getByRole("link", { name: /Tune the export queue/ });
  // The agent's mark leads the row; where the chat ran is a line under its
  // title, each name after the icon of its kind.
  await expect(row.locator(".mark svg")).toHaveCount(1);
  for (const kind of ["agent", "machine", "project"])
    await expect(row.locator(`.oc-m [data-place="${kind}"]`)).toHaveCount(1);
});

test("memory follows the viewing switcher, which names this machine once", async ({ page }) => {
  const requests = await mockDashboard(page);
  await page.goto("/#/records?");
  const switcher = page.locator("#top .vw-b");
  await switcher.click();
  const menu = page.getByRole("menu", { name: "Show chats and memory from" });
  await expect(menu.getByRole("menuitemradio")).toHaveText([
    /^\s*This machine · laptop/,
    /^\s*All machines/,
    /^\s*workstation/,
  ]);
  await menu.getByRole("menuitemradio", { name: /^\s*workstation/ }).click();
  await expect(page).toHaveURL(/scope=node%3Aworkstation/);
  await expect.poll(() => requests.at(-1)?.searchParams.get("node")).toBe("workstation");
  expect(requests.at(-1)?.searchParams.get("shared")).toBe("true");
  await expect(page.getByRole("main").getByText("workstation · published copy")).toBeVisible();
});

test("a memory shows its project and dates as the lists do and explains how to confirm it", async ({
  page,
}) => {
  const fixture = recordFixture();
  const revision = fixture.revisions[0];
  if (!revision) throw new Error("The fixture has no revision.");
  const recordedAt = Date.now() - 2 * 3_600_000;
  const unconfirmed = {
    ...fixture,
    scope: { kind: "repository", identity: "https://github.com/harbor/billing.git" },
    revisions: [
      { ...revision, recorded_at: recordedAt, body: { ...revision.body, review: "candidate" } },
    ],
  };
  await mockDashboard(page, unconfirmed);
  await page.goto("/#/record/preference-1");
  const header = page.locator(".rec-h");
  await expect(header.getByText("harbor/billing", { exact: true })).toHaveAttribute(
    "title",
    "https://github.com/harbor/billing.git",
  );
  const when = header.locator("time");
  await expect(when).toHaveText("2h ago");
  await expect(when).toHaveAttribute("title", new Date(recordedAt).toLocaleString());
  const note = page.getByRole("note");
  await expect(note).toContainText("To confirm this, ask your agent to review your memory");
  await expect(note.getByRole("link", { name: "How review works" })).toHaveAttribute(
    "href",
    "https://getmash.io/records/#correct-or-replace",
  );
});

test("an established memory has no confirmation note", async ({ page }) => {
  await mockDashboard(page);
  await page.goto("/#/record/preference-1");
  await expect(
    page.getByRole("heading", { name: "preference fixture on port 8080" }),
  ).toBeVisible();
  await expect(page.getByRole("note")).toHaveCount(0);
});

test("a link to an unknown project says the project was not found", async ({ page }) => {
  await mockDashboard(page);
  await page.goto("/#/project?name=nope");
  await expect(
    page.getByRole("heading", { level: 1, name: "Project not found", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Summary" })).toHaveCount(0);
  await page.getByRole("link", { name: "All projects", exact: true }).click();
  await expect(page).toHaveURL(/#\/projects$/);
  // A known project still opens as before.
  await page.goto("/#/project?name=%2Fhome%2Fsam%2Fcode%2Fbilling");
  await expect(page.getByRole("heading", { level: 1, name: "billing", exact: true })).toBeVisible();
});

test("on a phone the export choice and button come before the list of chats", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockDashboard(page);
  await page.goto("/#/export");
  const button = page.getByRole("button", { name: /^Export 2 chats/ });
  const list = page.getByRole("region", { name: "Chats to export" });
  await expect(button).toBeVisible();
  const [buttonBox, listBox] = await Promise.all([button.boundingBox(), list.boundingBox()]);
  expect(buttonBox?.y ?? Infinity).toBeLessThan(listBox?.y ?? 0);
});

test("viewing history describes the same lifetime in its subtitle and its empty state", async ({
  page,
}) => {
  await mockDashboard(page);
  await page.goto("/#/history");
  await expect(page.getByText(/Chats you opened since this page was loaded/)).toBeVisible();
  await expect(page.getByText(/until you reload the page/)).toBeVisible();
  await expect(page.getByRole("link", { name: "Browse chats", exact: true })).toHaveAttribute(
    "href",
    "#/search",
  );
});
