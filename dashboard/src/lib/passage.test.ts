import { describe, expect, it } from "vitest";
import { passageAround, plainPassage, withoutMarkdown } from "./passage";

describe("search passages", () => {
  it("drops Markdown's marks and keeps the words", () => {
    expect(
      withoutMarkdown("use **access tokens** and `refresh_token` in [the docs](https://x.y)"),
    ).toBe("use access tokens and refresh_token in the docs");
    expect(withoutMarkdown("## Plan\n- keep *one* scheme\n> quoted _here_")).toBe(
      "Plan\nkeep one scheme\nquoted here",
    );
    // Underscores inside a name are not emphasis.
    expect(withoutMarkdown("set max_retry_count to 3")).toBe("set max_retry_count to 3");
  });

  it("starts and ends a cut passage on whole words", () => {
    expect(plainPassage("…o **access tokens that expire**, with a refresh tok…")).toBe(
      "…access tokens that expire, with a refresh…",
    );
    // A window that began on a space or a mark began between words.
    expect(plainPassage("… access tokens")).toBe("…access tokens");
    expect(plainPassage("…**access** tokens")).toBe("…access tokens");
    expect(plainPassage("whole passage, not cut")).toBe("whole passage, not cut");
  });

  it("reads a long message from a little before its first match", () => {
    const text = `${"filler words ".repeat(40)}the refresh token goes in the keychain ${"tail ".repeat(40)}`;
    const passage = passageAround(text, "keychain", 120);
    expect(passage.startsWith("…")).toBe(true);
    expect(passage.endsWith("…")).toBe(true);
    expect(passage).toContain("keychain");
    expect(passage.length).toBeLessThanOrEqual(122);
    expect(passageAround("short", "short", 120)).toBe("short");
  });
});
