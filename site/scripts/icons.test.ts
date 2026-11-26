import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { type IconName, icons, weightFor } from "../src/icons.ts";

const assets = join(import.meta.dirname, "..", "node_modules", "@phosphor-icons", "core", "assets");

function packaged(glyph: string, weight: "regular" | "bold"): string {
  const file = weight === "regular" ? `${glyph}.svg` : `${glyph}-${weight}.svg`;
  return readFileSync(join(assets, weight, file), "utf8").trim();
}

describe("the site's icons", () => {
  it("are exact copies of Phosphor's glyphs", () => {
    for (const name of Object.keys(icons) as IconName[]) {
      const { phosphor, ...weights } = icons[name];
      for (const weight of ["regular", "bold"] as const) {
        expect(packaged(phosphor, weight), `${name} ${weight}`).toBe(
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor">${weights[weight]}</svg>`,
        );
      }
    }
  });

  it("are bold when small and regular otherwise", () => {
    expect(["0.75rem", "0.875rem", "0.9375rem"].map(weightFor)).toEqual(["bold", "bold", "bold"]);
    expect(["1rem", "1.125rem", "1em"].map(weightFor)).toEqual(["regular", "regular", "regular"]);
  });
});
