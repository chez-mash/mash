import process from "node:process";
import { defineConfig } from "@playwright/test";

// The visual baselines: pictures of the demo build's main pages and of every
// Storybook story, compared pixel by pixel. Fonts and antialiasing differ between systems, so the
// pictures are only ever taken inside the Playwright container image that CI
// uses: run `pnpm visual` or `pnpm visual:update` (tools/visual.sh), not this
// configuration directly. See docs/internal/verification.md, "Visual baselines".
const scratch = process.env.MASH_VISUAL_SCRATCH;
if (!process.env.MASH_VISUAL_CONTAINER && !scratch)
  throw new Error(
    "The visual baselines are compared inside the Playwright container: run `pnpm visual`.",
  );

// The container gives the same pixels on every run, so the tolerance is
// there only to absorb rounding, and it is small enough that a slightly
// retinted surface or one changed icon still fails: a pixel counts as changed
// past a colour distance a reader could not see (a few levels of one channel),
// and a picture fails once forty of its pixels changed. Do not loosen it to
// get a picture through. `tools/visual.sh --exact` allows no difference at
// all, which is how a new page is shown to give the same picture every time.
const tolerance = process.env.MASH_VISUAL_EXACT
  ? { threshold: 0, maxDiffPixels: 0 }
  : { threshold: 0.02, maxDiffPixels: 40 };

const port = 4181;
const address = `http://127.0.0.1:${port}/demo/`;
// The static Storybook (`pnpm storybook:build`), served beside the demo.
const storiesPort = 4182;
const storiesAddress = `http://127.0.0.1:${storiesPort}/`;
const desktop = { width: 1280, height: 860 };
const phone = { width: 390, height: 844 };
const pages = "pages.spec.ts";
const stories = "stories.spec.ts";

export default defineConfig({
  testDir: "./visual",
  // Outside the container the pictures cannot match the committed ones;
  // MASH_VISUAL_SCRATCH names a throwaway folder to keep them in while a new
  // page's steps are being written.
  snapshotPathTemplate: `${scratch ?? "{testDir}/baselines"}/{projectName}/{arg}{ext}`,
  outputDir: "./visual/results",
  reporter: [["list"], ["html", { outputFolder: "./visual/report", open: "never" }]],
  fullyParallel: true,
  workers: 4,
  retries: 0,
  forbidOnly: true,
  expect: {
    toHaveScreenshot: {
      animations: "disabled",
      caret: "hide",
      scale: "css",
      stylePath: "./visual/still.css",
      ...tolerance,
    },
  },
  use: {
    baseURL: address,
    headless: true,
    deviceScaleFactor: 1,
    locale: "en-US",
    timezoneId: "UTC",
    launchOptions: { args: ["--hide-scrollbars"] },
  },
  projects: [
    { name: "desktop-light", testMatch: pages, use: { viewport: desktop, colorScheme: "light" } },
    { name: "desktop-dark", testMatch: pages, use: { viewport: desktop, colorScheme: "dark" } },
    { name: "phone-light", testMatch: pages, use: { viewport: phone, colorScheme: "light" } },
    // The phone's dark pictures cover three pages only, to keep the set small.
    {
      name: "phone-dark",
      testMatch: pages,
      grep: /@phone-dark/,
      use: { viewport: phone, colorScheme: "dark" },
    },
    // Every story, in each theme; a story tagged "phone" is taken at the
    // phone's size (visual/stories.spec.ts).
    {
      name: "stories-light",
      testMatch: stories,
      use: { baseURL: storiesAddress, viewport: desktop, colorScheme: "light" },
    },
    {
      name: "stories-dark",
      testMatch: stories,
      use: { baseURL: storiesAddress, viewport: desktop, colorScheme: "dark" },
    },
  ],
  webServer: [
    {
      // Through Node rather than pnpm, which the container does not have.
      command: `node node_modules/vite/bin/vite.js preview --mode demo --host 127.0.0.1 --port ${port} --strictPort`,
      url: address,
      reuseExistingServer: false,
    },
    {
      command: `node node_modules/vite/bin/vite.js preview --outDir storybook-static --host 127.0.0.1 --port ${storiesPort} --strictPort`,
      url: `${storiesAddress}index.json`,
      reuseExistingServer: false,
    },
  ],
});
