// Renders icons/tray.svg to the 44-pixel menu bar picture (22 points at 2x)
// with Playwright's Chromium, taken from the dashboard's installed packages:
//   (cd dashboard && pnpm install) && node desktop/icons/render-tray.mjs
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = (path) => fileURLToPath(new URL(path, import.meta.url));
const require = createRequire(here("../../dashboard/package.json"));
const { chromium } = require("@playwright/test");

const size = 44;
const svg = readFileSync(here("tray.svg"), "utf8");
const browser = await chromium.launch();
try {
  const page = await browser.newPage({ viewport: { width: size, height: size } });
  await page.setContent(
    `<style>html,body{margin:0;background:transparent}svg{display:block;width:${size}px;height:${size}px}</style>${svg}`,
  );
  await page.screenshot({
    path: here("../src-tauri/icons/tray.png"),
    omitBackground: true,
    clip: { x: 0, y: 0, width: size, height: size },
  });
} finally {
  await browser.close();
}
