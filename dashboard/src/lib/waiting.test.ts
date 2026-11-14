import { describe, expect, it } from "vitest";
import { WAITING, waitingPhrase } from "./waiting";

describe("what Mashy says while it looks", () => {
  it("starts plainly and comes round again", () => {
    expect(waitingPhrase(0)).toBe("Searching your history…");
    expect(waitingPhrase(1)).toBe(WAITING[1]);
    expect(waitingPhrase(WAITING.length)).toBe("Searching your history…");
    expect(waitingPhrase(-1)).toBe(WAITING.at(-1));
  });
  it("never repeats a phrase within one round", () => {
    expect(new Set(WAITING).size).toBe(WAITING.length);
  });
});
