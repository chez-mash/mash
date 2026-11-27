import { expect, type Locator, type Page, test } from "@playwright/test";
import { fitsViewport } from "./shell";

const status = {
  ready: true,
  chats: 12,
  messages: 340,
  searchable_messages: 300,
  models_installed: true,
  query_models: "available",
  local_qdrant: { state: "reachable", url: "http://127.0.0.1:6334" },
  realized_mode: "hybrid",
  database_path: "/fixture/mash.db",
  recommended_action: null,
  configuration: {
    path: "/fixture/config.toml",
    loaded: true,
    providers: { codex: { roots: ["/fixture/codex"] } },
    index: { since_days: null, destination: "both" },
    embeddings: { backend: "local" },
    qdrant: { url: "http://127.0.0.1:6334", vector_search: { algorithm: "hnsw" } },
    views: { query: { ranking: "rrf" } },
    records: { enabled: true },
    redaction: { output: true, publication: true },
    shared: { group_id: "personal", node_id: "fixture", api_url: "http://shared.invalid" },
  },
};
const settings = {
  path: "/fixture/config.toml",
  revision: "r1",
  editable: true,
  agents: {
    codex: { indexed: true, roots: ["/fixture/codex"] },
    cursor: { indexed: false, roots: ["/fixture/cursor"] },
  },
  since_days: null,
  destination: "both",
  shared: true,
  memory_capture: true,
  hide_credentials: { output: true, publication: true },
};

async function mock(
  page: Page,
  served: unknown,
  posts: unknown[],
  report: unknown = status,
): Promise<void> {
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    if (path === "/api/status") return route.fulfill({ json: report });
    if (path === "/api/services/settings") {
      if (request.method() === "POST") {
        const body = request.postDataJSON();
        posts.push(body);
        if (posts.length === 1)
          return route.fulfill({
            status: 400,
            json: {
              error: {
                message:
                  "the configuration file changed since these settings were loaded; reload them and try again",
              },
            },
          });
        return route.fulfill({
          json: {
            ...settings,
            revision: "r2",
            agents: { ...settings.agents, cursor: { indexed: true, roots: ["/fixture/cursor"] } },
            since_days: 30,
            hide_credentials: { output: true, publication: false },
          },
        });
      }
      return route.fulfill({ json: served });
    }
    if (path === "/api/assistant") return route.fulfill({ json: { enabled: false, model: null } });
    if (path === "/api/assistant/models")
      return route.fulfill({ json: { models: ["another/model", "test/model"] } });
    if (path === "/api/services/access")
      return route.fulfill({ json: { authentication_required: false, local_node: "fixture" } });
    if (path === "/api/chats")
      return route.fulfill({ json: { local_node: "fixture", truncated: false, chats: [] } });
    return route.fulfill({ json: [] });
  });
}

test("word search is ready when optional models are absent", async ({ page }) => {
  await mock(page, settings, [], {
    ...status,
    query_models: "not-required",
    models_installed: false,
    realized_mode: "lexical",
  });
  await page.goto("/#/settings");
  await expect(page.getByText("Local search ready", { exact: true })).toBeVisible();
  await expect(page.getByText("not needed for word search", { exact: true })).toBeVisible();
  await expect(page.getByText("mash models install", { exact: true })).toHaveCount(0);
  const guide = page.getByRole("link", { name: "Set up Mashy", exact: true });
  await expect(guide).toHaveAttribute(
    "href",
    "https://getmash.io/configuration/#the-dashboard-assistant",
  );
  // Guides open beside the dashboard, like every documentation link here.
  await expect(guide).toHaveAttribute("target", "_blank");
  await expect(page.getByText("Mashy answers questions about your history.")).toContainText(
    "paste its API key",
  );
});

for (const [state, action, link] of [
  ["unreachable", "start-qdrant", "Start or repair the search service"],
  ["rejected", "index", "Check the search service"],
] as const) {
  test(`settings show how to recover from a ${state} search service`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await mock(page, settings, [], {
      ...status,
      ready: false,
      local_qdrant: { ...status.local_qdrant, state },
      recommended_action: action,
      configuration: { ...status.configuration, loaded: state === "rejected" },
    });
    await page.goto("/#/settings");
    await expect(page.getByText("Local search needs attention", { exact: true })).toBeVisible();
    await expect(page.getByRole("link", { name: link, exact: true })).toHaveAttribute(
      "href",
      "https://getmash.io/troubleshooting/#qdrant-connection-failed",
    );
    await expect(page.getByText("mash index", { exact: true })).toHaveCount(0);
    await expect(page.getByText("mash init", { exact: true })).toHaveCount(0);
    if (state === "unreachable") {
      await expect(page.getByRole("link", { name: "Set up Mash here" })).toBeVisible();
    }
    await expect(page.getByText("Messages eligible for search", { exact: true })).toBeVisible();
    expect(await fitsViewport(page)).toBe(true);
  });
}

for (const width of [390, 1280]) {
  test(`settings are changed, reviewed and saved at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    const posts: unknown[] = [];
    await mock(page, settings, posts);
    await page.goto("/#/settings");
    await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
    await expect(page.getByRole("link", { name: "Settings guide", exact: true })).toHaveAttribute(
      "href",
      "https://getmash.io/dashboard-settings/",
    );
    await expect(page.getByRole("link", { name: "Connect an agent" })).toHaveAttribute(
      "href",
      "#/setup",
    );
    // Nothing is offered to save until something differs from the file.
    await expect(page.getByRole("region", { name: "Unsaved settings" })).toHaveCount(0);
    // Search internals are shown, never offered as a choice.
    await expect(page.getByText("HNSW", { exact: true })).toBeVisible();
    await expect(page.getByRole("main").getByRole("switch")).toHaveCount(5);

    const cursor = page.getByRole("switch", { name: "Index Cursor" });
    await expect(cursor).toHaveAccessibleDescription(/still look up chats already indexed/);
    await expect(page.getByRole("link", { name: "Remove an agent connection" })).toHaveAttribute(
      "href",
      "https://getmash.io/agents/#disconnect-an-agent",
    );
    await cursor.click();
    await page.getByRole("radio", { name: "recent updates" }).click();
    const range = page.getByRole("spinbutton", { name: "Days of history" });
    await expect(range).toHaveAccessibleDescription(/Chats already indexed stay searchable/);
    await expect(page.getByRole("link", { name: "How to remove an indexed chat" })).toHaveAttribute(
      "href",
      "https://getmash.io/privacy/#remove-a-chat",
    );
    await range.fill("30");
    await page.getByRole("switch", { name: "Hide credentials when publishing" }).click();
    await expect(
      page.getByText(/The next publication replaces this machine's shared copy/),
    ).toContainText("That copy can include credentials found in chats.");
    const bar = page.getByRole("region", { name: "Unsaved settings" });
    await expect(bar).toContainText("3 unsaved changes");
    await bar.getByRole("button", { name: "Review changes" }).click();
    await expect(bar.getByRole("listitem")).toHaveText([
      /index Cursor\s*off\s*→\s*on/,
      /history range\s*all history\s*→\s*the last 30 days/,
      /hide credentials when publishing\s*on\s*→\s*off/,
    ]);
    expect(await fitsViewport(page)).toBe(true);

    // A file edited elsewhere in the meantime is not overwritten; the reason is shown.
    await bar.getByRole("button", { name: "Save to file" }).click();
    await expect(bar.getByRole("alert")).toContainText("changed since these settings were loaded");
    await expect(bar).toContainText("3 unsaved changes");
    await bar.getByRole("button", { name: "Save to file" }).click();
    await expect(bar.getByRole("status")).toContainText("Saved 3 changes to the file");
    await expect(bar.getByRole("status")).toContainText("New requests use the saved settings");
    await expect(bar.getByRole("status")).toContainText("run mash index on this machine");
    expect(posts).toHaveLength(2);
    expect(posts[1]).toEqual({
      revision: "r1",
      agents: { codex: true, cursor: true },
      since_days: 30,
      destination: "both",
      memory_capture: true,
      hide_credentials: { output: true, publication: false },
      // The settings page never changes which agents stay unshared.
      unshared: null,
      assistant: null,
    });
    await expect(page.getByRole("switch", { name: "Index Cursor" })).toHaveAttribute(
      "aria-checked",
      "true",
    );

    // Turning every agent off cannot be saved.
    await page.getByRole("switch", { name: "Index Cursor" }).click();
    await page.getByRole("switch", { name: "Index Codex" }).click();
    await expect(bar.getByRole("alert")).toContainText("at least one agent");
    await expect(bar.getByRole("button", { name: "Save to file" })).toBeDisabled();
    await bar.getByRole("button", { name: "Discard" }).click();
    await expect(page.getByRole("switch", { name: "Index Codex" })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    expect(posts).toHaveLength(2);
  });
}

test("a dashboard that does not accept changes shows settings without controls", async ({
  page,
}) => {
  const posts: unknown[] = [];
  await mock(page, { ...settings, editable: false }, posts);
  await page.goto("/#/settings");
  await expect(page.getByText(/does not change settings/)).toBeVisible();
  await expect(page.getByRole("link", { name: "Settings guide", exact: true })).toBeVisible();
  await expect(page.getByRole("main").getByRole("switch")).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Connect an agent" })).toHaveCount(0);
  await expect(page.getByText("not indexed", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Save to file" })).toHaveCount(0);
});

test("Mashy is switched off and given another model where an assistant is configured", async ({
  page,
}) => {
  const posts: unknown[] = [{}];
  await mock(page, { ...settings, assistant: { enabled: true, model: "test/model" } }, posts);
  await page.goto("/#/settings");
  await expect(page.getByText("Mashy has no API key on this computer yet.")).toBeVisible();
  const model = page.getByRole("combobox", { name: "Mashy's model" });
  await expect(model).toHaveValue("test/model");
  // The service's own list of models is offered as choices.
  await expect(page.locator("#assistant-models option")).toHaveCount(2);
  await page.getByRole("switch", { name: "Mashy", exact: true }).click();
  await model.fill("another/model");
  const bar = page.getByRole("region", { name: "Unsaved settings" });
  await bar.getByRole("button", { name: "Review changes" }).click();
  await expect(bar.getByRole("listitem")).toHaveText([
    /Mashy\s*on\s*→\s*off/,
    /Mashy's model\s*test\/model\s*→\s*another\/model/,
  ]);
  await bar.getByRole("button", { name: "Save to file" }).click();
  await expect.poll(() => posts.length).toBe(2);
  expect(posts[1]).toMatchObject({ assistant: { enabled: false, model: "another/model" } });
});

test("Mashy is set up from settings by choosing a service and pasting a key", async ({ page }) => {
  const sent: Record<string, unknown>[] = [];
  let enabled = false;
  await mock(page, settings, []);
  // Later routes win: the setup itself, and Mashy's state before and after.
  await page.route("**/api/services/assistant", async (route) => {
    sent.push(route.request().postDataJSON());
    enabled = true;
    await route.fulfill({
      json: {
        ...settings,
        revision: "r2",
        assistant: { enabled: true, model: "anthropic/claude-haiku-5.5" },
      },
    });
  });
  await page.route("**/api/assistant", (route) =>
    route.fulfill({ json: { enabled, model: enabled ? "anthropic/claude-haiku-5.5" : null } }),
  );
  await page.goto("/#/settings");
  const turnOn = page.getByRole("button", { name: "Turn on Mashy" });
  await expect(page.getByRole("radio", { name: "OpenRouter" })).toBeChecked();
  await expect(turnOn).toBeDisabled();
  await page.getByLabel("API key").fill("  sk-or-test  ");
  await turnOn.click();
  await expect
    .poll(() => sent)
    .toEqual([
      {
        revision: "r1",
        key: "sk-or-test",
        base_url: "https://openrouter.ai/api/v1",
        model: "anthropic/claude-haiku-5.5",
      },
    ]);
  // Once set up, the switch and model take the form's place, and the key is not shown.
  await expect(page.getByRole("switch", { name: "Mashy" })).toBeVisible();
  await expect(page.getByLabel("Mashy's model")).toHaveValue("anthropic/claude-haiku-5.5");
  await expect(page.getByText("sk-or-test")).toHaveCount(0);
});

test("Mashy's link to settings opens at the assistant section", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 700 });
  await mock(page, settings, []);
  await page.goto("/#/settings?section=assistant");
  await expect(page.getByRole("button", { name: "Turn on Mashy" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Models and Mashy" })).toBeInViewport();
});

test("another service needs its address and model before Mashy can be turned on", async ({
  page,
}) => {
  await mock(page, settings, []);
  await page.goto("/#/settings");
  await page.getByText("Another service", { exact: true }).click();
  await page.getByLabel("API key").fill("k");
  const turnOn = page.getByRole("button", { name: "Turn on Mashy" });
  await expect(turnOn).toBeDisabled();
  await page.getByLabel("API address").fill("https://api.example.com/v1");
  await page.locator("#settings-mashy-model").fill("example-model");
  await expect(turnOn).toBeEnabled();
});

test("a read-only dashboard names the file to edit, with a copy button and guides", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await mock(page, { ...settings, editable: false }, []);
  await page.goto("/#/settings");
  const note = page.getByText(/does not change settings/);
  await expect(note.getByRole("link", { name: "Why settings are read-only" })).toHaveAttribute(
    "href",
    "https://getmash.io/dashboard-settings/#if-settings-are-read-only",
  );
  const footer = page.getByText(/To change the search engine or storage settings/);
  await expect(footer.getByRole("link", { name: "Configuration guide" })).toBeVisible();
  // One Copy path, beside the path it copies.
  await expect(page.getByRole("main").getByRole("button", { name: "Copy path" })).toHaveCount(1);
  const header = page.locator(".set-path").first();
  await expect(header).toContainText("/fixture/config.toml");
  await header.getByRole("button", { name: "Copy path" }).click();
  await expect(header.getByRole("button", { name: "Copied" })).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe("/fixture/config.toml");
  // A path wraps between its parts, never inside a name.
  await expect(page.getByRole("definition").filter({ hasText: "/fixture/mash.db" })).toContainText(
    "/fixture/mash.db",
  );
  expect(
    await page
      .getByRole("definition")
      .filter({ hasText: "/fixture/mash.db" })
      .evaluate((node) => node.querySelectorAll("wbr").length),
  ).toBeGreaterThan(0);
  // Settings use the same width as the other list pages: the page's whole
  // content width, inside the room kept for the floating controls, with the
  // sections' names listed beside them.
  const main = await page.locator("#main").evaluate((element) => {
    const style = getComputedStyle(element);
    return (
      element.clientWidth -
      Number.parseFloat(style.paddingLeft) -
      Number.parseFloat(style.paddingRight)
    );
  });
  const section = await page.locator("#main .set-layout").first().boundingBox();
  expect(Math.round(section?.width ?? 0)).toBeGreaterThan(Math.round(main * 0.9));
});

test("read-only settings line up: paths break only after slashes and cards fill their rows", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await mock(page, { ...settings, editable: false }, []);
  await page.goto("/#/settings");
  const database = page.getByRole("definition").filter({ hasText: "/fixture/mash.db" });
  // Every break opportunity follows a slash, so "mash.db" is never split at its dot.
  const before = await database.evaluate((node) =>
    node.innerHTML
      .replace(/<!--.*?-->/g, "")
      .split("<wbr>")
      .slice(0, -1)
      .map((piece) => piece.at(-1)),
  );
  expect(before.length).toBeGreaterThan(0);
  expect(before.every((last) => last === "/" || last === "\\")).toBe(true);
  // Sections stack in one column, each card as wide as the others.
  const card = (name: string) =>
    page
      .locator("section.sec")
      .filter({ has: page.getByRole("heading", { level: 2, name, exact: true }) })
      .locator(".card")
      .first()
      .boundingBox();
  const [indexing, privacy] = await Promise.all([card("Indexing"), card("Privacy")]);
  expect(Math.round(indexing?.x ?? 0)).toBe(Math.round(privacy?.x ?? 1));
  expect(Math.round(indexing?.width ?? 0)).toBe(Math.round(privacy?.width ?? 1));
  expect(indexing?.y ?? 0).toBeLessThan(privacy?.y ?? 0);
  // Mashy's state sits at the end of its row, like the privacy settings.
  const mashy = page.locator(".set-mashy .set-row").first();
  const row = await mashy.boundingBox();
  const state = await mashy.getByRole("definition").first().boundingBox();
  expect((row?.x ?? 0) + (row?.width ?? 0) - ((state?.x ?? 0) + (state?.width ?? 0))).toBeLessThan(
    40,
  );
});

test("OpenAI is offered as a service and asks which model to use", async ({ page }) => {
  await mock(page, settings, []);
  await page.goto("/#/settings");
  const services = page.getByRole("group", { name: /model service/ });
  await expect(services.getByRole("radio")).toHaveCount(4);
  await services.getByText("OpenAI", { exact: true }).click();
  await expect(page.getByRole("radio", { name: "OpenAI" })).toBeChecked();
  await expect(page.getByRole("link", { name: "get an API key from OpenAI" })).toHaveAttribute(
    "href",
    "https://platform.openai.com/api-keys",
  );
  await page.getByLabel("API key").fill("sk-test");
  const turnOn = page.getByRole("button", { name: "Turn on Mashy" });
  await expect(turnOn).toBeDisabled();
  await page.locator("#settings-mashy-model").fill("a-model");
  await expect(turnOn).toBeEnabled();
});

/** A box as the browser draws it inside its parent, rounded to a hundredth of a pixel. */
const drawn = (locator: Locator) =>
  locator.evaluate((element) => {
    const { left, top, width, height } = element.getBoundingClientRect();
    const parent = element.parentElement?.getBoundingClientRect();
    return [left - (parent?.left ?? 0), top - (parent?.top ?? 0), width, height].map(
      (value) => Math.round(value * 100) / 100,
    );
  });

test("a segmented control slides its highlight to the choice made", async ({ page }) => {
  await mock(page, settings, []);
  await page.goto("/#/settings");
  const range = page.getByRole("group", { name: "History range" });
  const highlight = range.locator('[data-slot="toggle-group-indicator"]');
  const choice = (name: string) => range.getByRole("radio", { name, exact: true });
  const running = () => highlight.evaluate((element) => element.getAnimations().length);
  const [first, second] = ["all history", "recent updates"];

  // It starts in place, on the choice already made, without moving there.
  await expect(range).toHaveAttribute("data-sliding", "");
  expect(await running()).toBe(0);
  await expect(choice(first)).toBeChecked();
  const from = await drawn(choice(first));
  expect(await drawn(highlight)).toEqual(from);
  const heightBefore = (await range.boundingBox())?.height;

  // Half-way through the slide it is between the two choices. The slide is
  // short, so the test stretches it to stop it half-way without a race.
  expect(await highlight.evaluate((element) => getComputedStyle(element).transitionDuration)).toBe(
    "0.2s",
  );
  await page.addStyleTag({
    content: '[data-slot="toggle-group-indicator"] { transition-duration: 20s !important; }',
  });
  await choice(second).click();
  await expect.poll(running).toBeGreaterThan(0);
  await highlight.evaluate((element) => {
    for (const animation of element.getAnimations()) {
      animation.pause();
      animation.currentTime = 10_000;
    }
  });
  const to = await drawn(choice(second));
  const [left, , width] = await drawn(highlight);
  expect(left).toBeGreaterThan(from[0] ?? 0);
  expect(left).toBeLessThan(to[0] ?? 0);
  expect(width).toBeGreaterThan(Math.min(from[2] ?? 0, to[2] ?? 0));
  expect(width).toBeLessThan(Math.max(from[2] ?? 0, to[2] ?? 0));
  // Only the highlight moves: the choices stay where they were.
  expect(await drawn(choice(first))).toEqual(from);

  await highlight.evaluate((element) => {
    for (const animation of element.getAnimations()) animation.finish();
  });
  expect(await drawn(highlight)).toEqual(to);
  expect((await range.boundingBox())?.height).toBe(heightBefore);

  // The arrow keys move between the choices, and the highlight follows the one picked.
  await page.keyboard.press("ArrowLeft");
  await expect(choice(first)).toBeFocused();
  await page.keyboard.press("Space");
  await expect(choice(first)).toBeChecked();
  await highlight.evaluate((element) => {
    for (const animation of element.getAnimations()) animation.finish();
  });
  expect(await drawn(highlight)).toEqual(from);

  // With less motion it switches at once.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await choice(second).click();
  await expect(choice(second)).toBeChecked();
  expect(await running()).toBe(0);
  expect(await drawn(highlight)).toEqual(to);
});
