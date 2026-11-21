import { expect, type Page, test } from "@playwright/test";
import type { Wire } from "../src/lib/routes";
import { fitsViewport } from "./shell";

const detected: Wire["FirstRun"] = {
  path: "/fixture/.mash/config.toml",
  revision: "",
  configured: false,
  editable: true,
  histories: {
    "claude-code": { roots: ["/fixture/.claude/projects"], indexed: true },
    codex: { roots: ["/fixture/.codex/sessions"], indexed: true },
  },
  since_days: 90,
  shared: false,
  integrations: [
    { agent: "claude-code", installed: true, connected: false },
    { agent: "codex", installed: false, connected: false },
  ],
  qdrant: { url: "http://127.0.0.1:6333", state: "reachable", reachable: true },
  models_installed: false,
  index: { state: "idle" },
};

async function mockSetup(page: Page, initial: Wire["FirstRun"] = detected) {
  let setup = initial;
  const posts: unknown[] = [];
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === "/api/services/access")
      return route.fulfill({ json: { authentication_required: false, local_node: "fixture" } });
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "fixture", truncated: false, chats: [] } });
    if (path !== "/api/services/setup") return route.fulfill({ json: [] });
    if (request.method() === "GET") return route.fulfill({ json: setup });
    const sent: unknown = request.postDataJSON();
    posts.push(sent);
    const indexing =
      typeof sent === "object" && sent !== null && "index" in sent && sent.index === true;
    setup = {
      ...setup,
      configured: true,
      revision: "saved",
      index: indexing ? { state: "running" } : { state: "idle" },
    };
    const done: Wire["FirstRunDone"] = {
      setup,
      report: {
        config: { path: setup.path, written: true, recorded_agents: [] },
        agents: [],
        next: [],
      },
    };
    return route.fulfill({ json: done });
  });
  return {
    posts,
    update: (change: Partial<Wire["FirstRun"]>) => {
      setup = { ...setup, ...change };
    },
  };
}

test("a first visit can confirm the detected local setup in one click", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const { posts, update } = await mockSetup(page);
  await page.goto("/");
  await expect(page).toHaveURL(/#\/setup$/);
  await expect(page.getByRole("heading", { name: "Set up Mash" })).toBeVisible();
  const review = page.getByRole("region", { name: "Set up Mash" });
  await expect(review).toContainText("Claude Code, Codex");
  await expect(review).toContainText("the last 90 days");
  await expect(review).toContainText("this machine only");
  await expect(page.getByRole("button", { name: "Next", exact: true })).toHaveCount(0);
  await page.getByText("Show history folders", { exact: true }).click();
  await expect(review.getByText("/fixture/.codex/sessions", { exact: true })).toBeVisible();
  expect(await fitsViewport(page)).toBe(true);
  expect(posts).toEqual([]);
  await page.getByRole("button", { name: "Set up and index" }).click();
  await expect(page.getByRole("heading", { name: "Setup is saved" })).toBeVisible();
  await expect(page.getByRole("status")).toContainText("Indexing your history");
  expect(posts).toEqual([
    {
      revision: "",
      agents: { "claude-code": true, codex: true },
      since_days: 90,
      connect: ["claude-code"],
      index: true,
    },
  ]);
  update({ index: { state: "finished", ok: true, chats: 2, messages: 8 } });
  await expect(page.getByRole("status")).toContainText("Indexed 2 chats and 8 messages.");
  await expect(page.getByRole("link", { name: "Open your chats" })).toBeVisible();
  await page.getByText("Search by meaning", { exact: true }).click();
  await expect(page.getByText("mash models install", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Configure and index for search by meaning" }),
  ).toHaveAttribute("href", "https://getmash.io/indexing/#enable-semantic-search");
});

test("the first review can start indexing with agent connections skipped", async ({ page }) => {
  const { posts, update } = await mockSetup(page);
  await page.goto("/");
  const review = page.getByRole("region", { name: "Set up Mash" });
  await page.getByRole("button", { name: "Skip agent connections" }).click();
  await expect(review.getByText("no agents", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Include installed agents" }).click();
  await expect(review.getByText("no agents", { exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "Skip agent connections" }).click();
  expect(posts).toEqual([]);
  await page.getByRole("button", { name: "Set up and index" }).click();
  await expect(page.getByRole("heading", { name: "Setup is saved" })).toBeVisible();
  expect(posts).toEqual([
    {
      revision: "",
      agents: { "claude-code": true, codex: true },
      since_days: 90,
      connect: [],
      index: true,
    },
  ]);
  update({ models_installed: true, index: { state: "finished", ok: true, chats: 2, messages: 8 } });
  await page.getByText("Search by meaning", { exact: true }).click();
  await expect(
    page.getByRole("link", { name: "Configure and index for search by meaning" }),
  ).toBeVisible();
  await expect(page.getByText("mash models install", { exact: true })).toHaveCount(0);
});

test("custom setup keeps the chosen paths, range and connections when checking Qdrant again", async ({
  page,
}) => {
  const { posts, update } = await mockSetup(page, {
    ...detected,
    qdrant: { ...detected.qdrant, state: "unreachable", reachable: false },
  });
  await page.goto("/#/setup");
  await expect(page.getByRole("button", { name: "Set up and index" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Set up without indexing" })).toBeEnabled();
  await page.getByRole("button", { name: "Customize setup" }).click();
  await expect(
    page.getByRole("heading", { name: "Which histories should Mash index?" }),
  ).toBeFocused();
  await page.getByRole("switch", { name: "Index Codex" }).click();
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await page.getByRole("button", { name: "another number of days" }).click();
  await page.getByRole("spinbutton", { name: "Days of history" }).fill("45");
  await page.getByRole("button", { name: "Next", exact: true }).click();
  await page.getByRole("switch", { name: "Connect Claude Code" }).click();
  await expect(page.getByRole("switch", { name: "Connect Codex" })).toBeDisabled();
  await page.getByRole("button", { name: "Review setup" }).click();
  await page.getByText("Show history folders", { exact: true }).click();
  await expect(page.getByText("/fixture/.claude/projects", { exact: true })).toBeVisible();
  await expect(page.getByText("/fixture/.codex/sessions", { exact: true })).toHaveCount(0);
  await expect(page.getByText("the last 45 days", { exact: true })).toBeVisible();
  await expect(page.getByText("no agents", { exact: true })).toBeVisible();
  await expect(page.getByText("mash qdrant install", { exact: true })).toBeVisible();
  expect(posts).toEqual([]);

  update({ qdrant: detected.qdrant });
  await page.getByRole("button", { name: "Check search service again" }).click();
  await expect(page.getByText("The search service is ready.")).toBeVisible();
  await page.getByRole("button", { name: "Set up and index" }).click();
  await expect(page.getByRole("heading", { name: "Setup is saved" })).toBeVisible();
  expect(posts).toEqual([
    {
      revision: "",
      agents: { "claude-code": true, codex: false },
      since_days: 45,
      connect: [],
      index: true,
    },
  ]);
});

test("setup rechecks a rejected search service after saving", async ({ page }) => {
  const reason = "Qdrant returned HTTP 401. Check that Mash's Qdrant API key matches.";
  const { posts, update } = await mockSetup(page, {
    ...detected,
    qdrant: { ...detected.qdrant, state: "rejected", reachable: false, message: reason },
  });
  await page.goto("/#/setup");
  await expect(page.getByText(reason, { exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Check the search service" })).toHaveAttribute(
    "href",
    "https://getmash.io/troubleshooting/#qdrant-connection-failed",
  );
  await expect(page.getByText("mash qdrant install", { exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Set up and index" })).toHaveCount(0);
  await page.getByRole("button", { name: "Set up without indexing" }).click();
  await expect(page.getByRole("heading", { name: "Setup is saved" })).toBeVisible();
  await expect(page.getByText(reason, { exact: true })).toBeVisible();
  await expect(page.getByText("mash index", { exact: true })).toHaveCount(0);
  update({ qdrant: detected.qdrant });
  await page.getByRole("button", { name: "Check search service again" }).click();
  await expect(page.getByText("mash index", { exact: true })).toBeVisible();
  expect(posts).toHaveLength(1);
  expect(posts[0]).toMatchObject({ index: false });
});
