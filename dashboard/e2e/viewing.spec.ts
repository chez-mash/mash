import { expect, type Page, test } from "@playwright/test";
import { recordFixture } from "./record-fixtures";
import { streamFromSearch } from "./stream";

// The viewing switcher: whose history the overview, chats, search, memory,
// projects and agents show. See the Viewing* artboards on the design canvas.

const now = Date.now();
const chat = (id: string, node: string, provider = "codex", ago = 1) => ({
  chat_id: id,
  node_id: node,
  provider,
  title: `${id} on ${node}`,
  project: "/work/storefront",
  updated_at: now - ago * 3_600_000,
});
const shared = [
  chat("sign-in", "laptop", "claude-code", 1),
  chat("checkout", "workstation", "codex", 2),
  chat("queue", "workstation", "codex", 3),
  chat("staging", "build-box", "antigravity", 5),
];
const xeniaOwn = [chat("live-one", "xenia", "codex", 1), chat("live-two", "xenia", "cursor", 2)];

const serverNow = Math.floor(now / 1000);
const published = (node: string, secondsAgo: number) => ({
  group_id: "harbor",
  node_id: node,
  state: "joined",
  delete_waiting: false,
  published: {
    chats: 2,
    messages: 20,
    instance: "i",
    last_synced_at: new Date((serverNow - secondsAgo) * 1000).toISOString(),
  },
});
/** The shared server's report: workstation published recently, build-box days ago. */
const serverView = {
  admin_key: false,
  may_change: false,
  server: "https://mash.harbor.example",
  group_id: "harbor",
  node_id: "laptop",
  status: {
    build: "fedcba98",
    group_id: "harbor",
    now: serverNow,
    restart_needed: false,
    started_at: serverNow - 100,
    machines: [
      published("laptop", 60),
      published("workstation", 28 * 60),
      published("build-box", 5 * 86_400),
    ],
  },
};

interface Setup {
  readonly joined?: boolean;
  readonly sharedOnly?: boolean;
  /** Machines with a configured dashboard this one can read live. */
  readonly live?: readonly string[];
  /** Whether xenia's dashboard answers. */
  readonly xeniaAnswers?: boolean;
  /** Whether the shared server reports when each machine published. */
  readonly report?: boolean;
}

/** A dashboard on laptop, reading a shared index unless told otherwise. */
async function mock(page: Page, setup: Setup = {}): Promise<URL[]> {
  const asked: URL[] = [];
  const joined = setup.joined ?? true;
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    asked.push(url);
    const path = url.pathname;
    if (path === "/api/viewing") return route.fulfill({ json: { machines: setup.live ?? [] } });
    if (path === "/api/shared/server")
      return route.fulfill({
        json: setup.report ? serverView : { admin_key: false, may_change: false },
      });
    if (path.startsWith("/api/peer/xenia/")) {
      if (!setup.xeniaAnswers)
        return route.fulfill({
          status: 504,
          json: { error: { message: "xenia's dashboard didn't answer in time" } },
        });
      if (path.endsWith("/api/chats"))
        return route.fulfill({
          json: {
            local_node: "xenia",
            truncated: false,
            shared_only: false,
            joined: true,
            // Its catalogue also lists what it reads from the shared index.
            chats: [...xeniaOwn, chat("elsewhere", "workstation")],
          },
        });
      if (path.endsWith("/api/records"))
        return route.fulfill({ json: { records: [recordFixture()], next_offset: null } });
      return route.fulfill({ json: { results: [], next_offset: null } });
    }
    if (path === "/api/chats")
      return route.fulfill({
        json: {
          local_node: "laptop",
          truncated: false,
          shared_only: setup.sharedOnly ?? false,
          joined,
          chats: joined ? shared : shared.filter((row) => row.node_id === "laptop"),
        },
      });
    if (path === "/api/records")
      return route.fulfill({
        json: { records: [{ ...recordFixture(), node_id: "workstation" }], next_offset: null },
      });
    if (path === "/api/search")
      return route.fulfill({
        json: {
          results: [
            { ...chat("checkout", "workstation"), matches: [{ message_id: "m1" }] },
            { ...chat("sign-in", "laptop"), matches: [{ message_id: "m2" }] },
          ],
          next_offset: null,
        },
      });
    return route.fulfill({ json: {} });
  });
  return asked;
}

async function shoot(page: Page, name: string): Promise<void> {
  const width = page.viewportSize()?.width ?? 0;
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  const directory = process.env.MASH_SCREENSHOTS;
  if (!directory) return;
  const browser = page.context().browser()?.browserType().name() ?? "browser";
  await page.screenshot({ path: `${directory}/${browser}-${name}-${width}.png`, fullPage: true });
}

const menu = (page: Page) => page.getByRole("group", { name: "Show chats and memory from" });
const option = (page: Page, name: RegExp) => menu(page).getByRole("menuitemradio", { name });
async function openSwitcher(page: Page): Promise<void> {
  // On a phone the sidebar is a drawer behind the menu button.
  if ((page.viewportSize()?.width ?? 0) <= 800)
    await page.getByRole("button", { name: "Open navigation" }).click();
  await page.locator("#rail .vw-b").click();
  await expect(menu(page)).toBeVisible();
}

for (const width of [1440, 390]) {
  test.describe(`at ${width}px`, () => {
    test.use({ viewport: { width, height: width > 500 ? 1000 : 844 } });

    test("the switcher lists this machine, all machines and each other machine", async ({
      page,
    }) => {
      await mock(page);
      await page.goto("/#/");
      await expect(
        page.getByRole("main").getByText("All machines", { exact: true }),
      ).toBeVisible();
      await openSwitcher(page);
      await expect(menu(page).getByRole("menuitemradio")).toHaveText([
        /^\s*This machine · laptop.*1$/,
        /^\s*All machines.*4$/,
        /^\s*build-box.*1$/,
        /^\s*workstation.*2$/,
      ]);
      await expect(option(page, /^\s*All machines/)).toHaveAttribute("aria-checked", "true");
      await shoot(page, "menu");
    });

    test("all machines lists every machine's chats, each naming its machine", async ({ page }) => {
      await mock(page);
      await page.goto("/#/search");
      await expect(
        page.getByRole("main").getByText("All machines", { exact: true }),
      ).toBeVisible();
      const list = page.locator("#reslist");
      for (const id of ["sign-in", "checkout", "queue", "staging"])
        await expect(list.getByText(new RegExp(`^${id} on`))).toBeVisible();
      await expect(list.locator(".chatrow").filter({ hasText: "checkout" })).toContainText(
        "workstation",
      );
      // The machine is the scope's to choose, so there is no chip for it.
      await expect(page.getByRole("button", { name: "any machine", exact: true })).toHaveCount(0);
      await shoot(page, "all");
    });

    test("one machine shows its published copy and says unpublished work isn't there", async ({
      page,
    }) => {
      const asked = await mock(page);
      await page.goto("/#/records?");
      await openSwitcher(page);
      await option(page, /^\s*workstation/).click();
      await expect(page).toHaveURL(/scope=node%3Aworkstation/);
      const main = page.getByRole("main");
      await expect(main.getByText("workstation · published copy", { exact: true })).toBeVisible();
      if (width > 800)
        await expect(main.getByText(/Work it hasn't published isn't here/)).toBeVisible();
      await expect
        .poll(() => asked.filter((url) => url.pathname === "/api/records").at(-1)?.search)
        .toContain("node=workstation");
      await shoot(page, "one-machine");
      // The choice is remembered, and the address carries it to another page.
      await page.goto("/#/search");
      await expect(main.getByText("workstation · published copy", { exact: true })).toBeVisible();
      await expect(page.locator("#reslist").getByText(/on laptop/)).toHaveCount(0);
      await expect(page.locator("#reslist").getByText(/^checkout on/)).toBeVisible();
    });

    test("an unjoined machine offers only itself and a way to join", async ({ page }) => {
      await mock(page, { joined: false });
      await page.goto("/#/");
      await expect(
        page.getByRole("main").getByText("This machine · laptop", { exact: true }),
      ).toBeVisible();
      await openSwitcher(page);
      await expect(menu(page).getByRole("menuitemradio")).toHaveText([/^\s*This machine/]);
      await expect(menu(page).getByRole("link", { name: "Join a shared index" })).toHaveAttribute(
        "href",
        "#/machines",
      );
      await shoot(page, "not-joined");
    });
  });
}

test("this machine shows only what it indexed, searched there alone", async ({ page }) => {
  await mock(page);
  await streamFromSearch(page);
  // The search asks every source in the scope at once; this machine's scope
  // keeps it to this machine's own index.
  const streamed: URL[] = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.pathname === "/api/search/stream") streamed.push(url);
  });
  await page.goto("/#/search?scope=local");
  const main = page.getByRole("main");
  await expect(main.getByText("This machine · laptop", { exact: true })).toBeVisible();
  await expect(page.locator("#reslist").getByText(/on workstation/)).toHaveCount(0);
  await expect(page.locator("#reslist").getByText(/^sign-in on laptop/)).toBeVisible();
  await page.getByLabel("Search your history").fill("tokens");
  await page.getByLabel("Search your history").press("Enter");
  await expect
    .poll(() =>
      streamed
        .filter((url) => !url.searchParams.has("live"))
        .at(-1)
        ?.searchParams.get("scope"),
    )
    .toBe("local");
});

test("a machine read live shows its own chats and says it is live", async ({ page }) => {
  const asked = await mock(page, { live: ["xenia"], xeniaAnswers: true });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/#/search");
  await openSwitcher(page);
  await expect(option(page, /^\s*xenia/).getByText("live", { exact: true })).toBeVisible();
  await option(page, /^\s*xenia/).click();
  const main = page.getByRole("main");
  await expect(main.getByText("xenia · live", { exact: true })).toBeVisible();
  await expect(main.getByText(/read live from its dashboard/)).toBeVisible();
  const list = page.locator("#reslist");
  await expect(list.getByText(/^live-one on xenia/)).toBeVisible();
  // What xenia reads from the shared index is not xenia's own.
  await expect(list.getByText(/^elsewhere on/)).toHaveCount(0);
  // Everything about xenia was asked of xenia's dashboard through this one.
  expect(asked.some((url) => url.pathname === "/api/peer/xenia/api/chats")).toBe(true);
  await shoot(page, "live");
  // Its memories come from its own dashboard too.
  await page.goto("/#/records?");
  // The chats page stays in the document, hidden, for going back to it.
  const shown = page.locator("main .scope-n:visible");
  await expect(shown.getByText("xenia · live", { exact: true })).toBeVisible();
  await expect
    .poll(() => asked.some((url) => url.pathname === "/api/peer/xenia/api/records"))
    .toBe(true);
});

test("a machine whose dashboard doesn't answer is shown from its published copy, saying so", async ({
  page,
}) => {
  await mock(page, { live: ["xenia"], xeniaAnswers: false });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/#/search?scope=node%3Axenia");
  const main = page.getByRole("main");
  await expect(
    main.getByText("xenia's dashboard didn't answer, so this is its published copy."),
  ).toBeVisible();
  await expect(main.getByText("xenia · published copy", { exact: true })).toBeVisible();
  await shoot(page, "live-fallback");
});

test("a shared-only dashboard has no this-machine choice", async ({ page }) => {
  await mock(page, { sharedOnly: true });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/#/search?scope=local");
  await expect(page.getByRole("main").getByText("All machines", { exact: true })).toBeVisible();
  await openSwitcher(page);
  await expect(option(page, /^\s*This machine/)).toHaveCount(0);
});

for (const width of [1440, 390]) {
  test(`the switcher warns of a stale copy and nothing more at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: width > 500 ? 1000 : 844 });
    await mock(page, { report: true });
    await page.goto("/#/");
    await openSwitcher(page);
    await expect(option(page, /build-box/).getByText("5 days old")).toHaveAttribute(
      "title",
      "Published 5 days ago",
    );
    await expect(option(page, /workstation/)).not.toContainText("ago");
    await expect(option(page, /workstation/)).not.toContainText("days old");
    await shoot(page, "menu-published");
    await option(page, /build-box/).click();
    if (width > 500) {
      // The button names the machine in full, with what it shows in a pill.
      const button = page.locator("#rail .vw-b");
      await expect(button).toContainText("build-box");
      await expect(button.locator(".vw-p")).toHaveText("published");
    }
  });
}
