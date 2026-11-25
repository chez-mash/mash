import { describe, expect, it } from "vitest";
import { applyAppearance, parseAppearance } from "./appearance.svelte";

describe("appearance", () => {
  it("accepts the three choices and falls back to the system", () => {
    expect(parseAppearance("light")).toBe("light");
    expect(parseAppearance("dark")).toBe("dark");
    expect(parseAppearance("system")).toBe("system");
    expect(parseAppearance("sepia")).toBe("system");
    expect(parseAppearance(null)).toBe("system");
  });

  it("pins a theme with data-theme and clears it for the system", () => {
    const root = { dataset: {} as Record<string, string> } as unknown as HTMLElement;
    applyAppearance("dark", root);
    expect(root.dataset.theme).toBe("dark");
    applyAppearance("light", root);
    expect(root.dataset.theme).toBe("light");
    applyAppearance("system", root);
    expect(root.dataset.theme).toBeUndefined();
  });
});
