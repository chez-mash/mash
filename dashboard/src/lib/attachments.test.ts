import { describe, expect, it } from "vitest";
import { attachments, attachmentUrl, kind, size } from "./attachments";

const kept = {
  id: "1".repeat(64),
  name: "screenshot.png",
  media_type: "image/PNG",
  availability: "retained" as const,
  bytes: 2048,
  digest: "a".repeat(64),
  redacted: false,
};

describe("message attachments", () => {
  it("reads a kept picture from where its page was read", () => {
    const [local] = attachments("m1", [kept], { node: "laptop", shared: true });
    expect(local).toMatchObject({ image: true, mediaType: "image/png", bytes: 2048 });
    expect(local?.url).toBe(
      `/api/attachment?message=m1&attachment=${kept.id}&digest=${kept.digest}&node=laptop&shared=true`,
    );
    expect(attachmentUrl("m1", kept.id, kept.digest, { peer: "desk top" })).toBe(
      `/api/peer/desk%20top/api/attachment?message=m1&attachment=${kept.id}&digest=${kept.digest}`,
    );
  });

  it("offers other kept files as downloads and says why a file was not kept", () => {
    const [notes, svg, away, gone] = attachments(
      "m1",
      [
        { ...kept, name: "notes.txt", media_type: "text/plain" },
        { ...kept, name: "drawing.svg", media_type: "image/svg+xml" },
        { ...kept, availability: "not_uploaded", digest: null },
        { ...kept, availability: "retained", digest: null },
      ],
      {},
    );
    expect(notes).toMatchObject({ image: false });
    expect(notes?.url).toContain("/api/attachment?");
    // SVG can carry script, so it is a download like any other file.
    expect(svg).toMatchObject({ image: false });
    expect(away).toMatchObject({ image: false, missing: "Its machine has not shared the file" });
    expect(away?.url).toBeUndefined();
    expect(gone).toMatchObject({ image: false, missing: "The file could not be found" });
  });

  it("says a file too large to keep was not kept, with nothing to download", () => {
    const [scan] = attachments(
      "m1",
      [{ ...kept, media_type: "application/pdf", availability: "too_large", digest: null }],
      {},
    );
    expect(scan).toMatchObject({
      image: false,
      missing: "It was larger than the 4 MiB Mash keeps",
    });
    expect(scan?.url).toBeUndefined();
  });

  it("names a file's kind in a word or two", () => {
    expect(kind("application/pdf")).toBe("PDF");
    expect(kind("text/plain")).toBe("Text");
    expect(kind("text/x-python")).toBe("Text");
    expect(kind("image/svg+xml")).toBe("SVG image");
    expect(kind("image/heic")).toBe("HEIC image");
    expect(kind("application/octet-stream")).toBe("File");
  });

  it("names sizes the way people say them", () => {
    expect(size(900)).toBe("900 bytes");
    expect(size(282_983)).toBe("276 KB");
    expect(size(3 * 1024 * 1024)).toBe("3.0 MB");
  });
});
