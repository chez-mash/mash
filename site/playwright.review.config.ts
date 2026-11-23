// Browser tests for the `make site-review` toolbar: a half-typed note survives
// dev server reloads. They run their own `astro dev` with a short reload hold
// and answer the review services' requests in the browser, so they neither
// need nor disturb a running review. Run with `pnpm test:review`.
import { defineConfig } from "@playwright/test";

export const reviewTestPort = 4339;
// The reload hold, shortened from two minutes so a forgotten draft lapses quickly.
export const holdMs = 6000;

export default defineConfig({
  testDir: "./tests/review",
  testMatch: "*.review.ts",
  workers: 1,
  timeout: 90_000,
  use: { baseURL: `http://127.0.0.1:${reviewTestPort}` },
  webServer: {
    command: `pnpm exec astro dev --ignore-lock --host 127.0.0.1 --port ${reviewTestPort}`,
    url: `http://127.0.0.1:${reviewTestPort}/backup/`,
    reuseExistingServer: false,
    timeout: 120_000,
    env: { AGENTATION_REVIEW_DRAFT_HOLD_MS: String(holdMs), ASTRO_DEV_BACKGROUND: "1" },
  },
});
