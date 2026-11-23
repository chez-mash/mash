import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("the dashboard's page", () => {
  // The server's Content Security Policy (default-src 'self') blocks inline
  // scripts, so one in index.html would silently never run once served; the
  // development server, which sends no policy, would not show it.
  it("loads every script from a file", () => {
    const page = readFileSync(new URL("../../index.html", import.meta.url), "utf8");
    const inline = [...page.matchAll(/<script\b([^>]*)>/g)].filter(
      ([, attributes]) => !/\bsrc=/.test(attributes ?? ""),
    );
    expect(inline).toEqual([]);
  });
});
