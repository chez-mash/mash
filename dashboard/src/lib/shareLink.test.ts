import { describe, expect, it } from "vitest";
import { inDesktopApp, shareLink } from "./shareLink";

describe("shared links", () => {
  const base = "http://127.0.0.1:8790/";

  it("is the page's own address in a browser", () => {
    expect(shareLink("#/chat/abc?node=laptop&message=m-2", base, false)).toBe(
      "http://127.0.0.1:8790/#/chat/abc?node=laptop&message=m-2",
    );
    expect(shareLink("#/chat/abc", "https://getmash.io/demo/", false)).toBe(
      "https://getmash.io/demo/#/chat/abc",
    );
  });

  it("is a mash:// link in the desktop app, whose port changes", () => {
    expect(shareLink("#/chat/abc?node=laptop&message=m-2", base, true)).toBe(
      "mash://chat/abc?node=laptop&message=m-2",
    );
    expect(shareLink("#/record/rec%2F1?node=laptop", base, true)).toBe(
      "mash://record/rec%2F1?node=laptop",
    );
    expect(shareLink("#/", base, true)).toBe("mash://");
  });

  it("knows the desktop app only by the flag the app sets", () => {
    expect(inDesktopApp({})).toBe(false);
    expect(inDesktopApp({ __MASH_DESKTOP__: "yes" })).toBe(false);
    expect(inDesktopApp({ __MASH_DESKTOP__: true })).toBe(true);
  });
});
