import { describe, expect, it } from "vitest";
import { wrapPoints } from "./wrap";

describe("wrapPoints", () => {
  it("breaks a path after each slash and before a file's extension", () => {
    expect(wrapPoints("/home/sam/.mash/mash.sqlite")).toEqual([
      "/",
      "home/",
      "sam/",
      ".mash/",
      "mash",
      ".sqlite",
    ]);
  });

  it("keeps a whole host name piece together and breaks before its dots", () => {
    expect(wrapPoints("https://mash.harbor.example")).toEqual([
      "https://",
      "mash",
      ".harbor",
      ".example",
    ]);
  });

  it("breaks Windows paths at backslashes", () => {
    expect(wrapPoints("C:\\Users\\sam\\mash.toml")).toEqual([
      "C:\\",
      "Users\\",
      "sam\\",
      "mash",
      ".toml",
    ]);
  });

  it("leaves words and numbers whole", () => {
    expect(wrapPoints("contextual-rerank")).toEqual(["contextual-rerank"]);
    expect(wrapPoints("the last 90 days")).toEqual(["the last 90 days"]);
    expect(wrapPoints("127.0.0.1")).toEqual(["127.0.0.1"]);
  });
});
