import { expect, type Page, test } from "@playwright/test";
import { fitsViewport, mockShell } from "./shell";

// The shared server panel: what any joined machine reads with its own token,
// managing machines with the admin key saved here, and joining with a string.
const NOW = 1_800_000_000;
const JOIN = `mash-join.${"x".repeat(120)}`;

const machines: Record<string, unknown>[] = [
  { group_id: "harbor", node_id: "build-box", state: "configured", delete_waiting: false },
  {
    group_id: "harbor",
    node_id: "laptop",
    state: "joined",
    fingerprint: "9f3a",
    credential_created_at: NOW - 86_400,
    published: {
      instance: "i",
      chats: 172,
      messages: 3410,
      last_synced_at: "2026-11-23T10:31:00Z",
    },
    delete_waiting: false,
  },
  {
    group_id: "harbor",
    node_id: "studio",
    state: "waiting",
    expires_at: NOW + 7_200,
    delete_waiting: false,
  },
];

async function mockServer(page: Page, adminKey: boolean, calls: unknown[] = []): Promise<void> {
  await mockShell(page);
  await page.route("**/api/shared/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    const body = route.request().postDataJSON() as Record<string, unknown> | null;
    if (body) calls.push({ path, ...body });
    if (path === "/api/shared/server")
      return route.fulfill({
        json: {
          server: "https://mash.harbor.example",
          group_id: "harbor",
          node_id: "laptop",
          admin_key: adminKey,
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
        json: {
          build: "4c1e09ab77d2",
          started_at: NOW - 600,
          now: NOW,
          machines,
          actions: [
            { at: NOW - 60, key: "ops", from: "laptop", action: "add", target: "harbor/studio" },
          ],
          keys: [{ name: "ops", created_at: NOW - 3_600 }],
        },
      });
    if (path === "/api/shared/admin/change")
      return route.fulfill({
        json:
          body?.change === "add"
            ? { join: { join: JOIN, expires_in_seconds: 86_400, restart_needed: false } }
            : {},
      });
    if (path === "/api/shared/admin/key") return route.fulfill({ json: { admin_key: true } });
    return route.fulfill({ status: 404, json: { error: { kind: "usage", message: "unknown" } } });
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

    test("any joined machine sees the server's machines, without an admin key", async ({
      page,
    }) => {
      await mockServer(page, false);
      await page.goto("/#/machines");
      const panel = page.getByRole("region", { name: "Shared server" });
      await expect(panel).toContainText("https://mash.harbor.example");
      const list = panel.getByRole("list", { name: "Machines in harbor" });
      await expect(list.getByRole("listitem")).toHaveCount(3);
      await expect(list).toContainText("waiting to join · its join string works for 2 more hours");
      await expect(list).toContainText("172 chats");
      await expect(panel.getByRole("button", { name: "Manage this server" })).toBeVisible();
      await shoot(page, "machines-readonly");
    });

    test("an admin adds a machine and sees its join string once", async ({ page }) => {
      const calls: unknown[] = [];
      await mockServer(page, true, calls);
      await page.goto("/#/machines");
      await page.getByLabel("Name the new machine").fill("desk");
      await page.getByRole("button", { name: "Create join string" }).click();
      const fresh = page.getByRole("status", { name: "Join string for desk" });
      await expect(fresh).toContainText(JOIN);
      await expect(fresh).toContainText("Send it privately");
      expect(calls).toContainEqual({
        path: "/api/shared/admin/change",
        change: "add",
        group_id: "harbor",
        node_id: "desk",
      });
      await expect(page.getByRole("list", { name: "Recent admin actions" })).toContainText(
        "add harbor/studio · key ops from laptop",
      );
      await shoot(page, "machines-admin");
      // The string lives only in the page that made it.
      await page.reload();
      await expect(page.getByText(JOIN)).toHaveCount(0);
    });

    test("deleting what a removed machine published needs its name typed", async ({ page }) => {
      const removed = { ...machines[1], state: "removed", delete_requested: false };
      machines.splice(1, 1, removed);
      const calls: unknown[] = [];
      await mockServer(page, true, calls);
      await page.goto("/#/machines");
      await page.getByRole("button", { name: "Delete what it published" }).first().click();
      const confirm = page.getByRole("form", { name: "Delete what laptop published" });
      const submit = confirm.getByRole("button", { name: "Delete what laptop published" });
      await expect(submit).toBeDisabled();
      await confirm.getByLabel("Type laptop to confirm").fill("Laptop");
      await expect(submit).toBeDisabled();
      await confirm.getByLabel("Type laptop to confirm").fill("laptop");
      await submit.click();
      expect(calls).toContainEqual({
        path: "/api/shared/admin/change",
        change: "delete",
        group_id: "harbor",
        node_id: "laptop",
        confirm: "laptop",
      });
      machines.splice(1, 1, { ...removed, state: "joined" });
    });

    test("a machine that has not joined checks a string before saving anything", async ({
      page,
    }) => {
      await mockShell(page);
      const calls: unknown[] = [];
      await page.route("**/api/shared/**", async (route) => {
        const path = new URL(route.request().url()).pathname;
        const body = route.request().postDataJSON() as Record<string, unknown> | null;
        if (body) calls.push(body);
        if (path === "/api/shared/server")
          return route.fulfill({ json: { admin_key: false, may_change: true } });
        return route.fulfill({
          json: {
            server: "https://mash.harbor.example",
            group_id: "harbor",
            node_id: "studio",
            check: { status: "ready", expires_in_seconds: 3600, machines: ["laptop", "build-box"] },
            joined: body?.apply === true,
          },
        });
      });
      await page.goto("/#/machines");
      const form = page.getByRole("form", { name: "Join a shared index" });
      await form.getByLabel("Join string").fill(JOIN);
      await form.getByRole("button", { name: "Check" }).click();
      await expect(form).toContainText("This computer joins as");
      await expect(form).toContainText("laptop, build-box");
      expect(calls).toEqual([{ join: JOIN, apply: false }]);
      await shoot(page, "join-preview");
      await form.getByRole("button", { name: "Save and continue" }).click();
      expect(calls).toEqual([
        { join: JOIN, apply: false },
        { join: JOIN, apply: true },
      ]);
    });
  });
}
