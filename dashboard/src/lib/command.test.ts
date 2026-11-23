import { describe, expect, it } from "vitest";
import { mashCommand } from "./command";

describe("terminal commands", () => {
  it("say mash in a browser", () => {
    expect(mashCommand("index", {})).toBe("mash index");
    // Only the app's own flag makes a page trust the program it names.
    expect(mashCommand("index", { __MASH_DESKTOP_COMMAND__: "/tmp/other" })).toBe("mash index");
  });

  it("name the app's own mash inside the desktop app", () => {
    const app = {
      __MASH_DESKTOP__: true,
      __MASH_DESKTOP_COMMAND__: "/Applications/Mash.app/Contents/MacOS/mash",
    };
    expect(mashCommand("qdrant install", app)).toBe(
      "/Applications/Mash.app/Contents/MacOS/mash qdrant install",
    );
    expect(mashCommand("index", { __MASH_DESKTOP__: true })).toBe("mash index");
    expect(mashCommand("index", { __MASH_DESKTOP__: true, __MASH_DESKTOP_COMMAND__: null })).toBe(
      "mash index",
    );
  });
});
