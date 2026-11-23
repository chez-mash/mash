import { expect, type Page, test } from "@playwright/test";
import { fitsViewport } from "./shell";

// The Services page draws every machine's services as a map: the chosen
// machine's pipeline in full, the shared server it publishes to, and the
// other machines as cards. These fixtures cover a healthy group, a stopped
// search service with a machine that does not answer, and a phone.

const base = {
  checked_at: 1794140000,
  pid: null,
  diagnostics: [],
  logs_available: false,
  compatibility: null,
  control_revision: "fixture",
  control_note: "Managed in this dashboard's services configuration.",
  actions: [],
};
function service(id: string, name: string, kind: string, state = "healthy", extra = {}) {
  const summary = state === "healthy" ? "Running · ready" : state === "stopped" ? "Stopped" : state;
  return { ...base, id, name, kind, state, summary, ...extra };
}
function laptop(searchStopped: boolean) {
  return {
    id: "laptop",
    platform: "macos",
    checked_at: 1794140000,
    reachable: true,
    warnings: [],
    services: [
      service("watcher-leadership", "Index watcher", "watcher", "healthy", {
        summary: "Running · watching this index",
      }),
      service("sqlite", "SQLite index", "sqlite", "healthy", { summary: "Readable · compatible" }),
      searchStopped
        ? service("managed-qdrant", "Qdrant", "qdrant", "stopped", {
            actions: ["start"],
            logs_available: true,
          })
        : service("managed-qdrant", "Qdrant", "qdrant", "healthy", {
            actions: ["stop", "restart"],
          }),
      service("runtime-41", "Mash MCP · PID 41", "mcp", "healthy", { pid: 41 }),
      service("runtime-42", "Mash MCP · PID 42", "mcp", "healthy", { pid: 42 }),
      service("shared-server", "Shared server endpoint", "server", "healthy", {
        summary: "Readiness endpoint responded successfully",
      }),
    ],
  };
}
function publisher(id: string, reachable = true) {
  return {
    id,
    platform: "linux",
    checked_at: 1794140000,
    reachable,
    warnings: reachable ? [] : ["Machine could not be reached"],
    services: reachable
      ? [
          service("sqlite", "SQLite index", "sqlite"),
          service("shared-server", "Shared server endpoint", "server"),
        ]
      : [],
  };
}

async function mock(page: Page, problem: boolean): Promise<{ actions: () => number }> {
  let actions = 0;
  await page.route("**/api/**", (route) => {
    const url = new URL(route.request().url());
    if (url.pathname === "/api/services/access")
      return route.fulfill({ json: { authentication_required: false, local_node: "laptop" } });
    if (url.pathname === "/api/services")
      return route.fulfill({
        json: {
          machines: [laptop(problem), publisher("workstation", !problem), publisher("build-box")],
        },
      });
    if (url.pathname === "/api/services/operations") return route.fulfill({ json: [] });
    if (url.pathname === "/api/services/installation")
      return route.fulfill({ json: { build: "0123456789abcdef", agents: [] } });
    if (url.pathname === "/api/services/action") {
      actions++;
      return route.fulfill({ status: 500, json: { error: { message: "not in this test" } } });
    }
    return route.fulfill({ json: { local_node: "laptop", chats: [], truncated: false } });
  });
  return { actions: () => actions };
}

async function shoot(page: Page, name: string): Promise<void> {
  const width = page.viewportSize()?.width ?? 0;
  expect(await fitsViewport(page)).toBe(true);
  const directory = process.env.MASH_SCREENSHOTS;
  if (!directory) return;
  await page.evaluate(() => scrollTo(0, 0));
  await page.screenshot({
    path: `${directory}/${test.info().project.name || "chromium"}-${name}-${width}.png`,
    fullPage: true,
  });
}

test.describe("at 1440px", () => {
  test.use({ viewport: { width: 1440, height: 1000 } });

  test("the map shows each machine's services and how they connect", async ({ page }) => {
    await mock(page, false);
    await page.goto("/#/services");
    const map = page.getByRole("region", { name: "How your services connect" });
    await expect(map).toBeVisible();
    for (const name of [
      "Agent histories",
      "Automatic indexing",
      "SQLite index",
      "Local search",
      "Mash MCP",
      "Shared server",
    ])
      await expect(map.getByRole("link", { name: new RegExp(name) })).toBeVisible();
    await expect(map.getByRole("link", { name: /Mash MCP.*2 sessions/ })).toBeVisible();
    // Other machines are cards that open their own map.
    await expect(map.getByRole("link", { name: /workstation.*Healthy/ })).toHaveAttribute(
      "href",
      "#/services?machine=workstation",
    );
    // The connections are there in words too.
    await expect(map.getByRole("list", { name: "Connections" })).toContainText(
      "Automatic indexing publishes Shared server: working",
    );
    // There is no list view to switch to.
    await expect(page.getByRole("heading", { name: "services", exact: true })).toHaveCount(0);
    await map.getByRole("link", { name: /Local search/ }).click();
    await expect(page).toHaveURL(/place=search/);
    const panel = page.getByRole("complementary", { name: "Local search details" });
    await expect(panel).toContainText("built from the SQLite copy of your chats");
    await expect(panel.getByRole("link", { name: "Full details" })).toHaveAttribute(
      "href",
      "#/services?machine=laptop&service=managed-qdrant",
    );
    await expect(map.getByRole("link", { name: /Local search/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
    await shoot(page, "healthy");
    await page.keyboard.press("Tab");
    await panel.getByRole("link", { name: "Close details" }).click();
    await expect(panel).toHaveCount(0);
  });

  test("a stopped service shows what it breaks and how to start it", async ({ page }) => {
    const fixture = await mock(page, true);
    await page.goto("/#/services?machine=laptop&place=search");
    const map = page.getByRole("region", { name: "How your services connect" });
    await expect(map.getByRole("link", { name: /Local search.*Stopped/ })).toBeVisible();
    // The readers are not at fault, but they cannot search.
    await expect(map.getByRole("link", { name: /Mash MCP.*local search is down/ })).toBeVisible();
    await expect(map.getByRole("list", { name: "Connections" })).toContainText(
      "Mash MCP searches Local search: broken",
    );
    await expect(map.getByRole("link", { name: /workstation.*Did not answer/ })).toBeVisible();
    await expect(
      page.getByRole("region", { name: "Needs attention on workstation" }),
    ).toBeVisible();
    const panel = page.getByRole("complementary", { name: "Local search details" });
    await expect(panel).toContainText("What this stops");
    await panel.getByRole("button", { name: "Start", exact: true }).click();
    const confirm = panel.getByRole("group", { name: "Confirm service action" });
    await expect(confirm).toContainText("Start Qdrant on laptop?");
    // Asked from the panel, the question appears once.
    await expect(page.getByRole("group", { name: "Confirm service action" })).toHaveCount(1);
    await shoot(page, "problem");
    await confirm.getByRole("button", { name: "Cancel" }).click();
    expect(fixture.actions()).toBe(0);
  });
});

test.describe("at 390px", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test("on a phone the map reads top to bottom and stays tappable", async ({ page }) => {
    await mock(page, true);
    await page.goto("/#/services");
    const map = page.getByRole("region", { name: "How your services connect" });
    for (const name of [
      "Shared server",
      "Agent histories",
      "Automatic indexing",
      "SQLite index",
      "Local search",
    ])
      await expect(map.getByRole("link", { name: new RegExp(name) })).toBeVisible();
    await expect(map.getByText("searched by")).toBeVisible();
    await shoot(page, "phone");
    await map.getByRole("link", { name: /Local search/ }).click();
    await expect(page.getByRole("complementary", { name: "Local search details" })).toBeVisible();
    expect(await fitsViewport(page)).toBe(true);
  });
});
