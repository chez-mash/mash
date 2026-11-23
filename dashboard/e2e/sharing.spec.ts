import { expect, type Page, test } from "@playwright/test";
import type { Wire } from "../src/lib/routes";
import { fitsViewport } from "./shell";

// Choosing what a joined machine shares, publishing it, and watching it arrive.
const NOW = 1_800_000_000;
const SERVER = "https://mash.harbor.example";

function chat(index: number, provider: string, node: string | null): Record<string, unknown> {
  return {
    chat_id: `chat-${index}`,
    ...(node ? { node_id: node } : {}),
    provider,
    title: `Chat ${index}`,
    updated_at: "2026-11-23T10:00:00Z",
    message_count: 3,
  };
}
// This machine (laptop) holds three Codex chats and one Cursor chat; another
// machine's chat is never counted as laptop's.
const chats = [
  chat(1, "codex", null),
  chat(2, "codex", null),
  chat(3, "codex", null),
  chat(4, "cursor", null),
  chat(5, "codex", "workstation"),
];

function settings(destination: "local" | "both", unshared: string[] = []) {
  return {
    path: "/home/sam/.mash/config.toml",
    revision: "r1",
    editable: true,
    agents: {
      codex: { indexed: true, roots: ["/home/sam/.codex/sessions"] },
      cursor: { indexed: true, roots: ["/home/sam/.cursor/chats"] },
    },
    since_days: 90,
    destination,
    shared: true,
    unshared,
    memory_capture: true,
    hide_credentials: { output: false, publication: true },
  };
}

interface Mock {
  destination: "local" | "both";
  machine?: Record<string, unknown>;
  admin?: boolean;
}

async function mock(page: Page, options: Mock, saved: unknown[] = []): Promise<void> {
  const own = {
    group_id: "harbor",
    node_id: "laptop",
    state: "joined",
    delete_waiting: false,
    ...options.machine,
  };
  const machines = [
    own,
    { group_id: "harbor", node_id: "workstation", state: "joined", delete_waiting: false },
  ];
  let current = settings(options.destination);
  await page.route("**/api/**", (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "laptop", truncated: false, chats } });
    if (path === "/api/services/settings") {
      if (request.method() === "POST") {
        const body = request.postDataJSON() as Record<string, unknown>;
        saved.push(body);
        current = {
          ...current,
          destination: body.destination as "local" | "both",
          unshared: (body.unshared as string[] | null) ?? current.unshared,
        };
      }
      return route.fulfill({ json: current });
    }
    if (path === "/api/shared/server")
      return route.fulfill({
        json: {
          server: SERVER,
          group_id: "harbor",
          node_id: "laptop",
          admin_key: options.admin ?? false,
          may_change: true,
          status: {
            build: "4c1e09ab77d2",
            started_at: NOW - 600,
            now: NOW,
            group_id: "harbor",
            restart_needed: false,
            machines,
          },
        },
      });
    if (path === "/api/shared/admin")
      return route.fulfill({
        json: { build: "b", started_at: NOW, now: NOW, machines, actions: [], keys: [] },
      });
    return route.fulfill({ json: [] });
  });
}

async function shoot(page: Page, name: string): Promise<void> {
  expect(await fitsViewport(page)).toBe(true);
  const directory = process.env.MASH_SCREENSHOTS;
  if (directory) {
    const width = page.viewportSize()?.width ?? 0;
    await page.screenshot({ path: `${directory}/${name}-${width}.png`, fullPage: true });
  }
}

for (const width of [1440, 390]) {
  test.describe(`at ${width}px`, () => {
    test.use({ viewport: { width, height: width > 500 ? 1000 : 844 } });

    test("a joined machine chooses what it shares before anything is published", async ({
      page,
    }) => {
      const saved: unknown[] = [];
      await mock(page, { destination: "local" }, saved);
      await page.goto("/#/machines");
      const review = page.getByRole("region", { name: "Choose what laptop shares" });
      await expect(review).toContainText("Nothing has been sent yet");
      const agents = review.getByRole("list", { name: "Agent histories" });
      await expect(agents).toContainText("3 chats");
      await expect(agents).toContainText("1 chat");
      await expect(review.getByRole("button", { name: "Publish 4 chats" })).toBeVisible();
      // Keep Cursor on this machine.
      await review.getByRole("switch", { name: "Share Cursor" }).click();
      await expect(agents).toContainText("stays on laptop");
      await expect(review.getByRole("button", { name: "Publish 3 chats" })).toBeVisible();
      await expect(review.getByLabel("Summary")).toContainText("laptop, workstation");
      await shoot(page, "share-review");
      await review.getByRole("button", { name: "Publish 3 chats" }).click();
      expect(saved).toEqual([
        expect.objectContaining({ destination: "both", unshared: ["cursor"], since_days: 90 }),
      ]);
      await expect(page.getByRole("region", { name: "Publishing to harbor" })).toBeVisible();
    });

    test("leaving every agent out publishes nothing", async ({ page }) => {
      await mock(page, { destination: "local" });
      await page.goto("/#/machines");
      const review = page.getByRole("region", { name: "Choose what laptop shares" });
      await review.getByRole("switch", { name: "Share Codex" }).click();
      await review.getByRole("switch", { name: "Share Cursor" }).click();
      await expect(review.getByText("Nothing will be shared.")).toBeVisible();
      await expect(review.getByRole("button", { name: /^Publish/ })).toBeDisabled();
    });

    test("saving for later keeps the index local", async ({ page }) => {
      const saved: unknown[] = [];
      await mock(page, { destination: "local" }, saved);
      await page.goto("/#/machines");
      await page.getByRole("button", { name: "Save and publish later" }).click();
      await expect
        .poll(() => saved)
        .toEqual([expect.objectContaining({ destination: "local", unshared: [] })]);
    });

    test("a first publication shows what the server has received", async ({ page }) => {
      await mock(page, {
        destination: "both",
        machine: { receiving: { covered: 40, total: 172 } },
      });
      await page.goto("/#/machines");
      const done = page.getByRole("region", { name: "Publishing to harbor" });
      await expect(done).toContainText("40 of 172 chats received by the server");
      await shoot(page, "share-receiving");
    });

    test("a finished publication says when, and what is shared can change", async ({ page }) => {
      await mock(page, {
        destination: "both",
        machine: {
          published: {
            instance: "i",
            chats: 172,
            messages: 900,
            last_synced_at: "2026-11-23T10:31:00Z",
          },
        },
      });
      await page.goto("/#/machines");
      const done = page.getByRole("region", { name: "Publishing to harbor" });
      await expect(done).toContainText("172 chats from laptop are on the server");
      await shoot(page, "share-done");
      await done.getByRole("button", { name: "Change what’s shared" }).click();
      await expect(page.getByRole("region", { name: "Choose what laptop shares" })).toContainText(
        "Turning an agent off later removes its chats from the server",
      );
    });

    test("nothing published yet points at automatic indexing", async ({ page }) => {
      await mock(page, { destination: "both" });
      await page.goto("/#/machines");
      const done = page.getByRole("region", { name: "Publishing to harbor" });
      await expect(done).toContainText("The server has nothing from laptop yet");
      await expect(done.getByRole("link", { name: "Services" })).toBeVisible();
    });

    test("adding a machine under a name the group already has says so", async ({ page }) => {
      await mock(page, { destination: "both", admin: true });
      await page.goto("/#/machines");
      await page.getByLabel("Name the new machine").fill("workstation");
      await expect(page.getByRole("alert")).toContainText(
        "harbor already has a machine called workstation",
      );
      await expect(page.getByRole("button", { name: "Create join string" })).toBeDisabled();
      await expect(
        page.getByRole("button", { name: "replace workstation’s credential" }),
      ).toBeVisible();
      await shoot(page, "share-name-taken");
    });

    test("first-run setup offers joining a shared index", async ({ page }) => {
      const first: Wire["FirstRun"] = {
        path: "/fixture/.mash/config.toml",
        revision: "",
        configured: false,
        editable: true,
        histories: { codex: { roots: ["/fixture/.codex/sessions"], indexed: true } },
        since_days: 90,
        shared: false,
        integrations: [],
        qdrant: { url: "http://127.0.0.1:6333", state: "reachable", reachable: true },
        models_installed: false,
        index: { state: "idle" },
      };
      await page.route("**/api/**", (route) => {
        const path = new URL(route.request().url()).pathname;
        if (path === "/api/services/setup") return route.fulfill({ json: first });
        if (path === "/api/shared/server")
          return route.fulfill({ json: { admin_key: false, may_change: true } });
        if (path === "/api/chats")
          return route.fulfill({ json: { local_node: "fixture", truncated: false, chats: [] } });
        return route.fulfill({ json: [] });
      });
      await page.goto("/#/setup");
      const choice = page.getByRole("group", {
        name: "Where should this computer’s chats be searched?",
      });
      await expect(choice.getByLabel(/On this computer only/)).toBeChecked();
      await expect(page.getByRole("form", { name: "Join a shared index" })).toHaveCount(0);
      await choice.getByLabel(/Join a shared index/).check();
      await expect(page.getByRole("form", { name: "Join a shared index" })).toBeVisible();
      await shoot(page, "setup-join");
    });
  });
}
