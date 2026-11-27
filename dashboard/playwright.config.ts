import process from "node:process";
import { defineConfig } from "@playwright/test";

const liveUrl = process.env.MASH_DASHBOARD_URL;
const baseURL = liveUrl || "http://127.0.0.1:4179";

export default defineConfig({
  testDir: "./e2e",
  testMatch: liveUrl ? "**/*.live.ts" : "**/*.spec.ts",
  workers: 1,
  outputDir: "../../mash-artifacts/dashboard-browser",
  use: {
    baseURL,
    headless: true,
    ...(process.env.MASH_BROWSER_EXECUTABLE
      ? { launchOptions: { executablePath: process.env.MASH_BROWSER_EXECUTABLE } }
      : {}),
  },
  ...(liveUrl
    ? {}
    : {
        webServer: {
          command: "pnpm run dev --host 127.0.0.1 --port 4179 --strictPort",
          url: baseURL,
          reuseExistingServer: false,
        },
      }),
});
