import { describe, expect, it } from "vitest";
import {
  fullSearchTarget,
  hidePanel,
  inkLogo,
  moveSelection,
  pageTarget,
  quickRows,
  quickStatus,
  rowTarget,
  searching,
} from "./quick";
import type { Status } from "./status";
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

  it("says search is up to date only when status names nothing left to do", () => {
    const base = {
      ready: true,
      chats: 1073,
      nextStep: null,
      watching: false,
      configuration: { loaded: true },
    } as unknown as Status;
    const at = (change: object) => quickStatus({ ...base, ...change } as Status, false);
    expect(at({})).toEqual({ tone: "ready", label: "Up to date · 1,073 chats" });
    expect(at({ chats: 1 }).label).toBe("Up to date · 1 chat");
    expect(at({ nextStep: "models install" })).toEqual({
      tone: "busy",
      label: "Word search only · 1,073 chats",
    });
    expect(at({ ready: false, nextStep: "index", watching: true })).toEqual({
      tone: "busy",
      label: "Indexing…",
    });
    expect(at({ ready: false, nextStep: "index" }).label).toBe("Not indexed yet");
    expect(at({ ready: false, nextStep: "start-qdrant" })).toEqual({
      tone: "problem",
      label: "Search unavailable",
    });
    expect(at({ ready: false, nextStep: "init", configuration: { loaded: false } })).toEqual({
      tone: "busy",
      label: "Not set up",
    });
    expect(quickStatus(undefined, true).tone).toBe("problem");
    expect(quickStatus(undefined, false)).toEqual({ tone: "checking", label: "Checking…" });
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
