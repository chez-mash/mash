import { expect, type Page, test } from "@playwright/test";
import { fitsViewport } from "./shell";

// The plugin-mismatch warning offers one button that deploys Mash through
// Fleet manager, reinstalls the plugin on the machine, and then leaves only
// the agent restart to the person. Every route here is mocked.

const present = (component: string) => ({
  component,
  state: "present",
  message: `${component} found`,
  remediation: null,
});
function installation(fixed: boolean) {
  const plugin = fixed
    ? present("plugin")
    : {
        component: "plugin",
        state: "outdated",
        message:
          "Installed Mash plugin 0.4.0 is older than 0.5.0, the version bundled with this CLI.",
        remediation: "Update the Mash plugin through your agent, then restart it.",
      };
  return {
    build: "0123456789abcdef",
    agents: [
      {
        agent: "claude-code",
        expected: true,
        connected: fixed,
        steps: [present("mcp"), plugin, present("skill"), present("hooks")],
      },
    ],
  };
}

interface Mock {
  fleet: boolean;
  reinstall: "succeeded" | "refused";
}
async function mock(page: Page, { fleet, reinstall }: Mock) {
  const seen = { deploys: [] as unknown[], reinstalls: [] as unknown[] };
  let fixed = false;
  let deployed = false;
  let reinstalled = false;
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    switch (url.pathname) {
      case "/api/services/access":
        return route.fulfill({ json: { authentication_required: false, local_node: "xenia" } });
      case "/api/services":
        return route.fulfill({
          json: {
            machines: [
              {
                id: "macbook",
                platform: "darwin",
                checked_at: 1,
                reachable: true,
                warnings: [],
                services: [],
              },
            ],
          },
        });
      case "/api/services/installation":
        expect(url.searchParams.get("machine")).toBe("macbook");
        return route.fulfill({ json: installation(fixed) });
      case "/api/services/fleet":
        return route.fulfill({
          json: fleet
            ? {
                enabled: true,
                revision: "rev-1",
                actions: [{ action: "deploy", targets: ["macbook"] }],
                services: [],
              }
            : { enabled: false, revision: null, actions: [], services: [] },
        });
      case "/api/services/fleet/jobs": {
        if (request.method() === "POST") {
          const body = request.postDataJSON();
          seen.deploys.push(body);
          deployed = true;
          return route.fulfill({
            status: 202,
            json: {
              request: body,
              requested_at: 1,
              completed_at: null,
              outcome: "running",
              targets: [
                { id: "macbook", state: "running", progress: null, detail: "Building Mash" },
              ],
            },
          });
        }
        const submitted = seen.deploys.at(-1);
        return route.fulfill({
          json: submitted
            ? [
                {
                  request: submitted,
                  requested_at: 1,
                  completed_at: 2,
                  outcome: deployed ? "succeeded" : "running",
                  targets: [{ id: "macbook", state: "succeeded", progress: null, detail: null }],
                },
              ]
            : [],
        });
      }
      case "/api/services/action": {
        const body = request.postDataJSON();
        seen.reinstalls.push(body);
        if (reinstall === "refused")
          return route.fulfill({
            status: 502,
            json: { error: { message: "Remote action outcome could not be verified." } },
          });
        reinstalled = true;
        return route.fulfill({
          json: {
            request: body,
            requested_at: 2,
            completed_at: null,
            outcome: "pending",
            message: "Reinstalling the plugin with `mash init --yes --no-index`.",
          },
        });
      }
      case "/api/services/operations": {
        const body = seen.reinstalls.at(-1);
        if (reinstalled) fixed = true;
        return route.fulfill({
          json: body
            ? [
                {
                  request: body,
                  requested_at: 2,
                  completed_at: 3,
                  outcome: "succeeded",
                  message: "The plugin is current for Claude Code.",
                },
              ]
            : [],
        });
      }
    }
    return route.fulfill({ json: { local_node: "xenia", chats: [], truncated: false } });
  });
  return seen;
}

for (const width of [390, 1280]) {
  test(`one button updates Mash and the plugin, then leaves the restart at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    const seen = await mock(page, { fleet: true, reinstall: "succeeded" });
    await page.goto("/#/services?machine=macbook");
    const fix = page.getByRole("group", { name: "Fix the plugin on macbook" });
    await expect(fix).toBeVisible();
    await expect(page.getByText("does not match the Mash installed on this machine")).toBeVisible();
    // The manual steps stay as a fallback, folded away.
    await expect(page.getByText("Do it by hand")).toBeVisible();
    await expect(page.getByRole("button", { name: "Copy the command" })).toBeHidden();
    await page.screenshot({ path: `test-results/plugin-repair-${width}.png`, fullPage: true });
    await fix.getByRole("button", { name: "Update Mash and the plugin" }).click();
    // Deploying is heavy, so it is confirmed first.
    const confirm = fix.getByRole("group", { name: "Confirm plugin fix" });
    await expect(confirm).toContainText("Deploy Mash to macbook, then reinstall the plugin there?");
    expect(seen.deploys).toHaveLength(0);
    await confirm.getByRole("button", { name: "Update and reinstall" }).click();
    await expect(fix.getByText("Plugin reinstalled on macbook.")).toBeVisible({ timeout: 15_000 });
    await expect(fix.getByText("This one is yours:")).toBeVisible();
    expect(seen.deploys).toEqual([
      expect.objectContaining({ action: "deploy", targets: ["macbook"], revision: "rev-1" }),
    ]);
    expect(seen.reinstalls).toEqual([
      expect.objectContaining({
        machine: "macbook",
        service: "agent-plugins",
        action: "reinstall_plugin",
        control_revision: "0123456789abcdef",
      }),
    ]);
    await page.screenshot({ path: `test-results/plugin-repair-done-${width}.png`, fullPage: true });
    // A fresh check finds the plugin current; the restart reminder stays.
    await fix.getByRole("button", { name: "Check again" }).click();
    await expect(page.getByText("does not match the Mash installed on this machine")).toHaveCount(
      0,
    );
    await expect(page.getByText("This one is yours:")).toBeVisible();
    expect(await fitsViewport(page)).toBe(true);
  });
}

test("without fleet access the button only reinstalls, and a refusal opens the manual steps", async ({
  page,
}) => {
  const seen = await mock(page, { fleet: false, reinstall: "refused" });
  await page.goto("/#/services?machine=macbook");
  const fix = page.getByRole("group", { name: "Fix the plugin on macbook" });
  await expect(fix.getByText("This dashboard cannot update Mash on macbook.")).toBeVisible();
  await fix.getByRole("button", { name: "Reinstall the plugin" }).click();
  await expect(fix.getByRole("alert")).toContainText(
    "Remote action outcome could not be verified.",
  );
  expect(seen.deploys).toHaveLength(0);
  expect(seen.reinstalls).toHaveLength(1);
  await expect(page.getByRole("button", { name: "Copy the command" })).toBeVisible();
});
