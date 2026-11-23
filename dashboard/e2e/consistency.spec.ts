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

test("every page that can re-read its data offers the same Check for updates button", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mockDashboard(page);
  const main = page.getByRole("main");
  const check = (scope: ReturnType<Page["locator"]>) =>
    scope.getByRole("button", { name: "Check for updates", exact: true });
  await page.goto("/#/records?");
  await expect(check(page.locator("#rail"))).toBeVisible();
  await expect(check(main)).toBeVisible();
  await page.goto("/#/record/fact-1");
  await expect(check(main)).toBeVisible();
  for (const old of [/reload chat list/i, /search again/i, /check jobs/i])
    await expect(page.getByRole("button", { name: old })).toHaveCount(0);
});

test("filter chips name what they leave open the same way on every page", async ({ page }) => {
  await mockDashboard(page);
  await page.goto("/#/search");
  const filters = page.getByRole("group", { name: "Filters" });
  for (const name of ["any agent", "any machine", "any project", "any time", "sort: newest"])
    await expect(filters.getByRole("button", { name, exact: true })).toBeVisible();
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

test("the overview has a title and its chat rows carry the agent, machine and project icons", async ({
  page,
}) => {
  await mockDashboard(page);
  await page.goto("/");
  await expect(
    page.getByRole("heading", { level: 1, name: "Overview", exact: true }),
  ).toBeVisible();
  const row = page.getByRole("main").getByRole("link", { name: /Tune the export queue/ });
  await expect(row.locator(".mt svg")).toHaveCount(3);
});

test("memory sources name this machine once and only list other machines as shared", async ({
  page,
}) => {
  const requests = await mockDashboard(page);
  await page.goto("/#/records?");
  await page.getByRole("button", { name: "this machine", exact: true }).click();
  const menu = page.getByRole("group", { name: "this machine" });
  await expect(menu.getByRole("menuitemradio")).toHaveText([
    "this machine (laptop)",
    "all shared machines",
    "workstation (shared)",
  ]);
  await menu.getByRole("menuitemradio", { name: "workstation (shared)" }).click();
  await page.getByRole("button", { name: "Apply filters" }).click();
  await expect(page).toHaveURL(/shared=true&node=workstation/);
  await expect.poll(() => requests.at(-1)?.searchParams.get("node")).toBe("workstation");
  await expect(
    page.getByRole("button", { name: "workstation (shared)", exact: true }),
  ).toBeVisible();
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
  await page.goto("/#/record/fact-1");
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
  await page.goto("/#/record/fact-1");
  await expect(page.getByRole("heading", { name: "fact fixture on port 8080" })).toBeVisible();
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
