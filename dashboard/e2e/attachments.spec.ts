import { expect, test } from "@playwright/test";

// A message's files show under its text: a kept picture as a thumbnail that
// opens full size, any other kept file as a card that downloads it, and a
// file Mash did not keep as a card saying so.
const PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAACgAAAAYCAIAAAAH5iiXAAAAKElEQVR4nGM4UaExIIhh1OJRi0ctHrV41OJRi0ctHrV41OJRi4ePxQAsO0ZM4fO3aAAAAABJRU5ErkJggg==",
  "base64",
);
const digest = "a".repeat(64);
const picture = {
  id: "1".repeat(64),
  name: "screenshot.png",
  media_type: "image/png",
  availability: "retained",
  bytes: PNG.length,
  digest,
  redacted: false,
};
// A one-page PDF, as a Claude Code `document` block would carry it.
const PDF = Buffer.from(
  "JVBERi0xLjQKMSAwIG9iago8PCAvVHlwZSAvQ2F0YWxvZyAvUGFnZXMgMiAwIFIgPj4KZW5kb2JqCjIgMCBvYmoKPDwgL1R5cGUgL1BhZ2VzIC9LaWRzIFszIDAgUl0gL0NvdW50IDEgPj4KZW5kb2JqCjMgMCBvYmoKPDwgL1R5cGUgL1BhZ2UgL1BhcmVudCAyIDAgUiAvTWVkaWFCb3ggWzAgMCA3MiA3Ml0gPj4KZW5kb2JqCnhyZWYKMCA0CjAwMDAwMDAwMDAgNjU1MzUgZiAKMDAwMDAwMDAwOSAwMDAwMCBuIAowMDAwMDAwMDU4IDAwMDAwIG4gCjAwMDAwMDAxMTUgMDAwMDAgbiAKdHJhaWxlcgo8PCAvU2l6ZSA0IC9Sb290IDEgMCBSID4+CnN0YXJ0eHJlZgoxODQKJSVFT0YK",
  "base64",
);
const report = {
  id: "3".repeat(64),
  name: "Quarterly report.pdf",
  media_type: "application/pdf",
  availability: "retained",
  bytes: PDF.length,
  digest: "b".repeat(64),
  redacted: false,
};
const elsewhere = {
  id: "2".repeat(64),
  name: "elsewhere.png",
  media_type: "image/png",
  availability: "not_allowed",
  bytes: null,
  digest: null,
  redacted: false,
};

test.beforeEach(async ({ page }) => {
  await page.route("**/api/records?**", (route) =>
    route.fulfill({ json: { records: [], next_offset: null } }),
  );
  await page.route("**/api/chats**", (route) =>
    route.fulfill({
      json: {
        local_node: "local",
        truncated: false,
        chats: [
          {
            chat_id: "files",
            node_id: "local",
            provider: "claude-code",
            title: "Chat with files",
            project: "/work/project",
            updated_at: Date.now(),
          },
        ],
      },
    }),
  );
  await page.route("**/api/chat?**", (route) =>
    route.fulfill({
      json: {
        message_count: 2,
        messages: [
          {
            id: "asked",
            ordinal: 0,
            role: "user",
            content: "What is wrong on this screen?",
            attachments: [picture, report, elsewhere],
          },
          { id: "answered", ordinal: 1, role: "assistant", content: "The header overlaps." },
        ],
      },
    }),
  );
  await page.route("**/api/attachment?**", (route) => {
    const url = new URL(route.request().url());
    const asked = (file: typeof picture) =>
      url.searchParams.get("message") === "asked" &&
      url.searchParams.get("attachment") === file.id &&
      url.searchParams.get("digest") === file.digest;
    if (asked(picture)) return route.fulfill({ body: PNG, contentType: "image/png" });
    if (asked(report))
      return route.fulfill({
        body: PDF,
        contentType: "application/pdf",
        headers: {
          "content-disposition":
            "attachment; filename=\"Quarterly report.pdf\"; filename*=UTF-8''Quarterly%20report.pdf",
        },
      });
    return route.fulfill({ status: 404, json: { error: { message: "not this attachment" } } });
  });
});

test("a kept picture shows as a thumbnail and opens full size", async ({ page }) => {
  await page.goto("/#/chat/files");
  const message = page.locator("#message-asked");
  const thumbnail = message.getByRole("button", { name: "Open screenshot.png full size" });
  await expect(thumbnail).toBeVisible();
  const image = thumbnail.locator("img");
  await expect(image).toHaveAttribute("loading", "lazy");
  await expect
    .poll(() => image.evaluate((element: HTMLImageElement) => element.naturalWidth))
    .toBe(40);

  await thumbnail.click();
  const full = page.getByRole("dialog", { name: "screenshot.png" });
  await expect(full).toBeVisible();
  await expect(full.getByRole("img", { name: "screenshot.png" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(full).toHaveCount(0);
});

test("a kept PDF is a card that downloads it under its own name", async ({ page }) => {
  await page.goto("/#/chat/files");
  const message = page.locator("#message-asked");
  await expect(message.getByText("Quarterly report.pdf")).toBeVisible();
  await expect(message.getByText(`PDF · ${PDF.length} bytes`)).toBeVisible();
  await expect(message.getByRole("img", { name: "Quarterly report.pdf" })).toHaveCount(0);
  const link = message.getByRole("link", { name: "Download Quarterly report.pdf" });
  await expect(link).toHaveAttribute("download", "Quarterly report.pdf");
  const [download] = await Promise.all([page.waitForEvent("download"), link.click()]);
  expect(download.suggestedFilename()).toBe("Quarterly report.pdf");
  // The link reads the kept bytes, served as a download.
  const served = await page.evaluate(
    async (href) => {
      const response = await fetch(href);
      return {
        type: response.headers.get("content-type"),
        disposition: response.headers.get("content-disposition"),
        bytes: Array.from(new Uint8Array(await response.arrayBuffer())),
      };
    },
    (await link.getAttribute("href")) ?? "",
  );
  expect(served.type).toBe("application/pdf");
  expect(served.disposition).toMatch(/^attachment; filename="Quarterly report.pdf"/);
  expect(Buffer.from(served.bytes).equals(PDF)).toBe(true);
});

test("a file Mash did not keep is a card, never a broken picture", async ({ page }) => {
  await page.goto("/#/chat/files");
  const message = page.locator("#message-asked");
  await expect(message.getByText("elsewhere.png")).toBeVisible();
  await expect(message.getByText(/Not kept\. It was outside the folders/)).toBeVisible();
  await expect(message.getByRole("img", { name: "elsewhere.png" })).toHaveCount(0);
  await expect(message.locator("img")).toHaveCount(1);
});

test("thumbnails fit a phone's width", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 760 });
  await page.goto("/#/chat/files");
  const thumbnail = page.getByRole("button", { name: "Open screenshot.png full size" });
  await expect(thumbnail).toBeVisible();
  const box = await thumbnail.boundingBox();
  expect(box?.height ?? 0).toBeLessThanOrEqual(182);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(375);
});
