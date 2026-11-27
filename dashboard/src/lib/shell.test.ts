import { describe, expect, it } from "vitest";
import { shellOf } from "./shell";

describe("shell", () => {
  it("is the system the desktop app names", () => {
    expect(shellOf({ __MASH_SHELL__: "macos" })).toBe("macos");
    expect(shellOf({ __MASH_SHELL__: "windows" })).toBe("windows");
    expect(shellOf({ __MASH_SHELL__: "linux" })).toBe("linux");
  });

  it("is a browser when nothing, or something unknown, is named", () => {
    expect(shellOf({})).toBe("browser");
    expect(shellOf({ __MASH_SHELL__: "beos" })).toBe("browser");
    expect(shellOf({ __MASH_SHELL__: 7 })).toBe("browser");
  });
});
