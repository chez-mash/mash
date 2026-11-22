import { describe, expect, it } from "vitest";
import {
  fullSearchTarget,
  hidePanel,
  inkLogo,
  moveSelection,
  pageTarget,
  quickRows,
  rowTarget,
  searching,
} from "./quick";
import type { ChatSummary } from "./types";

const chat = (id: string): ChatSummary => ({
  id,
  title: `Chat ${id}`,
  agent: "codex",
  machine: "laptop",
  project: "/work",
  updatedAt: "",
});

describe("the quick search panel", () => {
  it("lists recent chats until two letters are typed, then matches", () => {
    const recent = ["a", "b", "c", "d", "e", "f"].map(chat);
    const found = [
      { chat: chat("m"), messageId: "m-3", before: "the passage", match: "", after: "" },
    ];
    expect(searching("d")).toBe(false);
    expect(quickRows("d", recent, found).map((row) => row.chat.id)).toEqual([
      "a",
      "b",
      "c",
      "d",
      "e",
    ]);
    expect(quickRows(" de ", recent, found)).toEqual([
      { chat: chat("m"), messageId: "m-3", passage: "the passage" },
    ]);
    expect(quickRows("zz", recent, [])).toEqual([]);
  });

  it("keeps the selection on the list", () => {
    expect(moveSelection(0, -1, 3)).toBe(0);
    expect(moveSelection(1, 1, 3)).toBe(2);
    expect(moveSelection(2, 1, 3)).toBe(2);
    expect(moveSelection(4, 1, 0)).toBe(0);
  });

  it("opens rows and pages as mash:// links in the desktop app, as routes in a browser", () => {
    const row = { chat: chat("c/1"), messageId: "m 2", passage: "" };
    expect(rowTarget(row, true)).toBe("mash://chat/c%2F1?node=laptop&message=m+2");
    expect(rowTarget(row, false)).toBe("#/chat/c%2F1?node=laptop&message=m+2");
    expect(fullSearchTarget(" checkout ", true)).toBe("mash://search?q=checkout");
    expect(fullSearchTarget("", false)).toBe("#/search");
    expect(pageTarget("#/settings", true)).toBe("mash://settings");
    expect(pageTarget("#/", true)).toBe("mash://");
  });

  it("draws ink logos in the text colour so they show in dark mode", () => {
    expect(inkLogo("codex")).toBe(true);
    expect(inkLogo("claude-code")).toBe(false);
  });

  it("asks the app to hide its window, and does nothing in a browser", async () => {
    const calls: string[] = [];
    hidePanel({ __TAURI_INTERNALS__: { invoke: async (command: string) => calls.push(command) } });
    hidePanel({});
    await Promise.resolve();
    expect(calls).toEqual(["plugin:window|hide"]);
  });
});
