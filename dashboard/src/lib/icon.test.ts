import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  defaultIconWeights,
  type IconWeights,
  iconWeight,
  readIconWeights,
  weightTokens,
} from "./icon.svelte";

const tokens = readFileSync(join(import.meta.dirname, "..", "theme", "tokens.css"), "utf8");

/** One block of tokens.css as a computed style would answer for it. */
function block(selector: string) {
  const start = tokens.indexOf(`${selector} {`);
  const body = tokens.slice(start, tokens.indexOf("\n}", start));
  return {
    getPropertyValue: (name: string) =>
      new RegExp(`${name}:\\s*([^;]+);`).exec(body)?.[1]?.trim() ?? "",
  };
}

describe("icon weight", () => {
  it("is bold when small and in text, and regular when large", () => {
    expect([11, 12, 14, 15].map((size) => iconWeight(size))).toEqual(Array(4).fill("bold"));
    expect([16, 18, 20, 24].map((size) => iconWeight(size))).toEqual(Array(4).fill("bold"));
    expect([28, 32, 48].map((size) => iconWeight(size))).toEqual(["regular", "regular", "regular"]);
  });

  it("fills the current item and bolds a warning at any size", () => {
    for (const size of [12, 16, 32]) {
      expect(iconWeight(size, "on")).toBe("fill");
      expect(iconWeight(size, "strong")).toBe("bold");
    }
  });

  it("takes its weights from the theme it is given", () => {
    const light: IconWeights = { ...defaultIconWeights, base: "regular", large: "light" };
    expect(iconWeight(16, "rest", light)).toBe("regular");
    expect(iconWeight(32, "rest", light)).toBe("light");
  });
});

describe("the theme's icon weights", () => {
  it("are the defaults", () => {
    const root = block(":root");
    for (const name of Object.values(weightTokens))
      expect(root.getPropertyValue(name), name).not.toBe("");
    expect(readIconWeights(root)).toEqual(defaultIconWeights);
  });

  it("keep the default for a role that is missing or misspelt", () => {
    const style = { getPropertyValue: (name: string) => (name === "--icon-weight" ? "heavy" : "") };
    expect(readIconWeights(style)).toEqual(defaultIconWeights);
  });
});
