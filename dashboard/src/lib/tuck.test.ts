import { describe, expect, it } from "vitest";
import { goingDown } from "./tuck";

const page = (top: number) => ({ top, viewport: 800, height: 4000 });

describe("whether the reader is scrolling down", () => {
  it("follows the direction the reader moves in", () => {
    expect(goingDown(false, 100, page(140))).toBe(true);
    expect(goingDown(true, 140, page(120))).toBe(false);
    expect(goingDown(true, 140, page(140))).toBe(true);
  });
  it("is never so at either end of the page", () => {
    expect(goingDown(true, 10, page(0))).toBe(false);
    expect(goingDown(false, 3100, page(3200))).toBe(false);
    // Rubber-banding past either end.
    expect(goingDown(true, 10, page(-30))).toBe(false);
  });
  it("ignores the page jumping by itself", () => {
    expect(goingDown(false, 100, page(1500))).toBe(false);
    expect(goingDown(true, 1500, page(400))).toBe(true);
  });
});
