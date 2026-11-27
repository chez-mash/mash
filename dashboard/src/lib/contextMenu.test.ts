import { describe, expect, it, vi } from "vitest";
import { chatMenu } from "./chatMenu";
import { contextMenu, linesFor, type MenuEntry } from "./contextMenu.svelte";

const event = () =>
  ({ clientX: 40, clientY: 60, preventDefault: vi.fn() }) as unknown as MouseEvent;

describe("context menu", () => {
  it("sends the app titles and rules, never what a line does", () => {
    const entries: MenuEntry[] = [
      { title: "Open", run: () => {} },
      "separator",
      { title: "Copy Link", run: () => {}, enabled: false },
    ];
    expect(linesFor(entries)).toEqual([
      { title: "Open", enabled: true },
      {},
      { title: "Copy Link", enabled: false },
    ]);
  });

  it("asks the desktop app for its menu and runs the line it says was chosen", async () => {
    const invoke = vi.fn(async () => undefined);
    const run = vi.fn();
    const right = event();
    contextMenu.show(
      right,
      [{ title: "Open", run }, "separator", { title: "Other", run: vi.fn() }],
      {
        __MASH_DESKTOP__: true,
        __TAURI_INTERNALS__: { invoke },
      },
    );
    expect(right.preventDefault).toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith("context_menu", {
      lines: [{ title: "Open", enabled: true }, {}, { title: "Other", enabled: true }],
    });
    expect(contextMenu.open).toBeNull();
    contextMenu.chosen(0);
    expect(run).toHaveBeenCalledOnce();
    // An answer is used once; a stray one runs nothing.
    contextMenu.chosen(0);
    contextMenu.chosen(1);
    expect(run).toHaveBeenCalledOnce();
  });

  it("draws the menu itself in the Linux app, where GTK cannot open one at the pointer", () => {
    const invoke = vi.fn(async () => undefined);
    contextMenu.show(event(), [{ title: "Open", run: () => {} }], {
      __MASH_DESKTOP__: true,
      __MASH_SHELL__: "linux",
      __TAURI_INTERNALS__: { invoke },
    });
    expect(invoke).not.toHaveBeenCalled();
    expect(contextMenu.open).toMatchObject({ x: 40, y: 60 });
    contextMenu.close();
  });

  it("draws the menu itself in a browser, at the pointer", () => {
    contextMenu.show(event(), [{ title: "Open", run: () => {} }], {});
    expect(contextMenu.open).toMatchObject({ x: 40, y: 60 });
    contextMenu.close();
    expect(contextMenu.open).toBeNull();
  });

  it("opens nothing for a menu of rules alone", () => {
    const right = event();
    contextMenu.show(right, ["separator"], {});
    expect(right.preventDefault).not.toHaveBeenCalled();
    expect(contextMenu.open).toBeNull();
  });
});

describe("a chat's menu", () => {
  const titles = (entries: MenuEntry[]) =>
    entries.map((entry) => (entry === "separator" ? "—" : entry.title));
  const codex = { agent: "codex" as const, id: "0b8f6a4e-5c1d-4a2b-9e3f-7d6c5b4a3e21" };

  it("opens the chat and copies its link", () => {
    expect(titles(chatMenu(codex, "#/chat/x", false))).toEqual(["Open", "—", "Copy Link"]);
  });

  it("copies the resume command only for a chat of this machine its agent can resume", () => {
    expect(titles(chatMenu(codex, "#/chat/x", true))).toEqual([
      "Open",
      "—",
      "Copy Link",
      "Copy Resume Command",
    ]);
    expect(titles(chatMenu({ agent: "cursor", id: "abc" }, "#/chat/x", true))).toEqual([
      "Open",
      "—",
      "Copy Link",
    ]);
  });
});
