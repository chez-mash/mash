// The stories' visual baselines: one picture of every Storybook story, in
// light and in dark, taken from the static Storybook (`pnpm storybook:build`).
// The stories are the dashboard's design reference, so a token, a kit control
// or a building block that drifts in any of its states fails here.
//
// The list of stories is Storybook's own index, so a new story is pictured
// without being named here. A story's tags say how: "phone" takes it at the
// phone's size, and "screen" takes the screen rather than the whole page, for
// a story whose subject is placed on the screen (a dialog, the rail, Mashy).
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page, test } from "@playwright/test";

/** The same moment the pages are pictured at (pages.spec.ts). */
const now = new Date("2026-11-20T19:30:00Z");
const phone = { width: 390, height: 844 };

interface Entry {
  readonly id: string;
  readonly type: string;
  readonly tags?: readonly string[];
}

function stories(): Entry[] {
  const index = fileURLToPath(new URL("../storybook-static/index.json", import.meta.url));
  if (!existsSync(index))
    throw new Error(
      "No static Storybook to picture: run `pnpm storybook:build` (tools/visual.sh does).",
    );
  const { entries } = JSON.parse(readFileSync(index, "utf8")) as {
    entries: Record<string, Entry>;
  };
  return Object.values(entries).filter((entry) => entry.type === "story");
}

/**
 * Records how the story ended. Storybook announces it on its channel once the
 * story has drawn and its play function has finished, or when either failed;
 * the listener is attached the moment the channel is made, so nothing is missed.
 */
function watchStory(): void {
  const ended: Record<string, string> = {
    storyRendered: "rendered",
    storyErrored: "errored",
    storyThrewException: "threw",
    playFunctionThrewException: "play function threw",
    storyMissing: "missing",
  };
  const scope = window as unknown as Record<string, unknown>;
  let channel: unknown;
  Object.defineProperty(window, "__STORYBOOK_ADDONS_CHANNEL__", {
    configurable: true,
    get: () => channel,
    set: (made: { on(event: string, listener: () => void): void }) => {
      channel = made;
      for (const [event, outcome] of Object.entries(ended))
        made.on(event, () => {
          scope.mashStoryEnded ??= outcome;
        });
    },
  });
}

/** Waits for what arrives after the story: its fonts, its pictures and its late requests. */
async function settle(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
  await page.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all([...document.images].map((image) => image.decode().catch(() => undefined)));
  });
}

for (const story of stories()) {
  const tags = story.tags ?? [];
  test(story.id, async ({ page }, info) => {
    const theme = info.project.name.endsWith("dark") ? "dark" : "light";
    if (tags.includes("phone")) await page.setViewportSize(phone);
    await page.clock.setFixedTime(now);
    await page.addInitScript(watchStory);
    await page.goto(`iframe.html?id=${story.id}&viewMode=story&globals=theme:${theme}`);
    const ended = await page.waitForFunction(
      () => (window as unknown as { mashStoryEnded?: string }).mashStoryEnded,
    );
    expect(await ended.jsonValue(), "the story draws without an error").toBe("rendered");
    await settle(page);
    await expect(page).toHaveScreenshot(`${story.id}.png`, { fullPage: !tags.includes("screen") });
  });
}
