// Browser tests for the `make site-review` toolbar: a half-typed note survives
// dev server reloads. They serve a one-page fixture site from a fresh copy in
// tests/review/.fixture/ on its own port, edit only that copy, and answer the
// review services' requests in the browser, so a running review is never
// reloaded or sent notes. Run with `pnpm test:review`.
import { defineConfig } from "@playwright/test";

export const reviewTestPort = 4339;
// The reload hold, shortened from two minutes so a forgotten draft lapses quickly.
export const holdMs = 6000;

const fixture = "tests/review/.fixture";

export default defineConfig({
  testDir: "./tests/review",
  testMatch: "*.review.ts",
  workers: 1,
  timeout: 90_000,
  use: { baseURL: `http://127.0.0.1:${reviewTestPort}` },
  webServer: {
    command: [
      `rm -rf ${fixture}`,
      `cp -r tests/review/fixture ${fixture}`,
      `pnpm exec astro dev --root ${fixture} --ignore-lock --host 127.0.0.1 --port ${reviewTestPort}`,
    ].join(" && "),
    url: `http://127.0.0.1:${reviewTestPort}/`,
    reuseExistingServer: false,
    timeout: 120_000,
    env: { AGENTATION_REVIEW_DRAFT_HOLD_MS: String(holdMs), ASTRO_DEV_BACKGROUND: "1" },
  },
});
