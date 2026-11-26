// Captures the dashboard pictures the guides show, from the demo build and
// its sample history, into assets/screenshots/ as WebP. Build the demo first
// with `pnpm build:demo` in ../dashboard, then run `pnpm screenshots`.
//
// The pictures are committed so the guides render on GitHub too. Run this
// again after a visible dashboard change and review the images before
// committing them.
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { extname } from "node:path";
import { chromium, type Page } from "@playwright/test";

const demo = new URL("../../dashboard/dist-demo/", import.meta.url);
const output = new URL("../../assets/screenshots/", import.meta.url);

/** Each picture: its file name, the demo page, and what to do before taking it. */
interface Shot {
  readonly name: string;
  readonly route: string;
  readonly height?: number;
  readonly width?: number;
  /** Leave out the rail and the top bar, so the page itself reads at a readable size. */
  readonly withoutRail?: boolean;
  readonly prepare?: (page: Page) => Promise<void>;
}

const shots: readonly Shot[] = [
  // The demo cannot run setup and says so; the picture shows what setup reviews.
  {
    name: "setup",
    route: "#/setup",
    width: 900,
    height: 520,
    prepare: (page) => removeAll(page, "main .card.warn"),
  },
  { name: "overview", route: "#/" },
  // The README's opening picture: the whole dashboard, rail and logo included,
  // open at the message where the sign-in decision was made.
  {
    name: "dashboard",
    route: "#/chat/sign-in-tokens?node=laptop&message=sign-in-tokens-2",
    height: 720,
  },
  {
    name: "search",
    route: "#/search?q=checkout",
    height: 520,
    withoutRail: true,
    prepare: (page) => removeText(page, "Demo search matches"),
  },
  {
    name: "chat",
    route: "#/chat/sign-in-tokens?node=laptop&message=sign-in-tokens-2",
    height: 640,
    withoutRail: true,
  },
  { name: "memory", route: "#/records", height: 600, withoutRail: true },
  {
    name: "record",
    route: "#/record/record-sign-in-tokens?node=laptop",
    height: 600,
    withoutRail: true,
  },
  { name: "agents", route: "#/agents", height: 480, withoutRail: true },
  { name: "machines", route: "#/machines", height: 400, withoutRail: true },
  { name: "export", route: "#/export", height: 640, withoutRail: true },
  { name: "settings", route: "#/settings", height: 640, withoutRail: true },
  { name: "services", route: "#/services", height: 820, withoutRail: true },
  { name: "tools", route: "#/console", height: 600, withoutRail: true },
];

/** The rail's usual width and the top bar's height, added to a picture that
 * leaves them out so the page keeps its width and height. */
const railWidth = 220;
const topBarHeight = 52;

const types: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".woff2": "font/woff2",
};

/** Serve the demo build at /demo/, as the site does. */
const server = createServer(async (request, response) => {
  const path = new URL(request.url ?? "/", "http://demo").pathname.replace(/^\/demo\//, "");
  const file = new URL(path === "" ? "index.html" : path, demo);
  try {
    const body = await readFile(file);
    response.writeHead(200, {
      "content-type": types[extname(file.pathname)] ?? "application/octet-stream",
    });
    response.end(body);
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const { port } = server.address() as AddressInfo;
await readFile(new URL("index.html", demo)).catch(() => {
  throw new Error("No demo build. Run `pnpm build:demo` in dashboard/ first.");
});

await mkdir(output, { recursive: true });
const browser = await chromium.launch();
try {
  for (const shot of shots) {
    const page = await browser.newPage({
      viewport: {
        width: (shot.width ?? 1280) + (shot.withoutRail ? railWidth : 0),
        height: (shot.height ?? 760) + (shot.withoutRail ? topBarHeight : 0),
      },
      deviceScaleFactor: 2,
      colorScheme: "light",
    });
    // A fixed clock keeps ages such as "2h" the same from run to run.
    await page.clock.setFixedTime(new Date("2026-11-22T12:00:00Z"));
    await page.goto(`http://127.0.0.1:${port}/demo/${shot.route}`);
    await page.waitForLoadState("networkidle");
    // The demo's banner and motion are not part of the dashboard people install.
    await page.addStyleTag({
      content:
        "aside.demo{display:none!important} *{animation:none!important;transition:none!important;caret-color:transparent!important}",
    });
    // Mashy's button floats over the bottom right corner of the page; a picture
    // cropped to the page alone is taken without it.
    if (shot.withoutRail)
      await page.addStyleTag({ content: "#mashy-fab{visibility:hidden!important}" });
    await shot.prepare?.(page);
    await page.waitForTimeout(300);
    // Without the rail, the top bar's left end (the logo) is cut off, so the
    // picture starts under the bar and shows only the page.
    const rail = shot.withoutRail ? await page.locator("#rail").boundingBox() : null;
    const bar = shot.withoutRail ? await page.locator("#top").boundingBox() : null;
    const viewport = page.viewportSize() ?? { width: 0, height: 0 };
    const left = rail ? rail.x + rail.width : 0;
    const top = bar ? bar.y + bar.height : 0;
    const png = await page.screenshot({
      clip: { x: left, y: top, width: viewport.width - left, height: viewport.height - top },
    });
    await writeFile(new URL(`${shot.name}.webp`, output), await webp(page, png));
    await page.close();
    console.log(`Captured ${shot.name}`);
  }
} finally {
  await browser.close();
  server.close();
}

/** Remove what the demo adds for itself, so the picture shows installed Mash. */
async function removeAll(page: Page, selector: string): Promise<void> {
  await page.locator(selector).evaluateAll((found) => {
    for (const element of found) element.remove();
  });
}

async function removeText(page: Page, text: string): Promise<void> {
  await page.getByText(text).evaluateAll((found) => {
    for (const element of found) element.remove();
  });
}

/** Re-encode a PNG as WebP with the browser’s own encoder. */
async function webp(page: Page, png: Buffer): Promise<Buffer> {
  const encoded = await page.evaluate(
    async (data) => {
      const bitmap = await createImageBitmap(await (await fetch(data)).blob());
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      canvas.getContext("2d")?.drawImage(bitmap, 0, 0);
      const blob = await canvas.convertToBlob({ type: "image/webp", quality: 0.86 });
      const bytes = new Uint8Array(await blob.arrayBuffer());
      let text = "";
      for (const byte of bytes) text += String.fromCharCode(byte);
      return btoa(text);
    },
    `data:image/png;base64,${png.toString("base64")}`,
  );
  return Buffer.from(encoded, "base64");
}
