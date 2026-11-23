import { describe, expect, it } from "vitest";
import { desktopDownload } from "../src/download.ts";

describe("the desktop download for a visitor's computer", () => {
  it("offers each system its installer", () => {
    expect(desktopDownload({ platform: "MacIntel", userAgent: "Mozilla/5.0 (Macintosh)" })).toEqual(
      {
        system: "macOS",
        href: "https://github.com/mashium/mash/releases/latest/download/mash-desktop-macos-arm64.dmg",
      },
    );
    expect(
      desktopDownload({ platform: "Win32", userAgent: "Mozilla/5.0 (Windows NT 10.0)" })?.system,
    ).toBe("Windows");
    expect(
      desktopDownload({ platform: "Linux x86_64", userAgent: "Mozilla/5.0 (X11; Linux x86_64)" })
        ?.href,
    ).toMatch(/linux-x86_64\.AppImage$/);
  });

  it("offers nothing where there is no installer", () => {
    expect(
      desktopDownload({ platform: "Linux aarch64", userAgent: "X11; Linux aarch64" }),
    ).toBeUndefined();
    expect(
      desktopDownload({ platform: "iPhone", userAgent: "Mozilla/5.0 (iPhone)" }),
    ).toBeUndefined();
    expect(
      desktopDownload({ platform: "Linux armv8l", userAgent: "Linux; Android 15; Mobile" }),
    ).toBeUndefined();
    expect(desktopDownload({})).toBeUndefined();
  });
});
