import { describe, expect, it } from "vitest";
import type { Catalog } from "./api";
import { recordSource } from "./records";
import {
  effectiveScope,
  inScope,
  parseScope,
  type Reach,
  scopeCatalog,
  scopeName,
  scopeSentence,
  scopeText,
  withoutScope,
} from "./scope.svelte";
import type { ChatSummary } from "./types";

const joined: Reach = { localNode: "laptop", joined: true, sharedOnly: false, live: ["xenia"] };
const alone: Reach = { localNode: "laptop", joined: false, sharedOnly: false, live: [] };
const server: Reach = { localNode: "server", joined: true, sharedOnly: true, live: [] };
const chat = (id: string, machine: string): ChatSummary => ({
  id,
  title: id,
  agent: "codex",
  machine,
  project: "",
  updatedAt: "",
});

describe("viewing scope", () => {
  it("reads and writes its address form, and ignores anything else", () => {
    for (const text of ["local", "all", "node:xenia"])
      expect(scopeText(parseScope(text) ?? { kind: "all" })).toBe(text);
    for (const text of ["", "node:", "everything", null]) expect(parseScope(text)).toBeNull();
  });

  it("defaults to all machines once joined, and to this machine before", () => {
    expect(effectiveScope(null, joined)).toEqual({ kind: "all" });
    expect(effectiveScope(null, alone)).toEqual({ kind: "local" });
    expect(effectiveScope(null, server)).toEqual({ kind: "all" });
  });

  it("never shows a scope this dashboard cannot", () => {
    // An unjoined machine has no other machines and no shared index.
    expect(effectiveScope({ kind: "all" }, alone)).toEqual({ kind: "local" });
    expect(effectiveScope({ kind: "node", node: "desk" }, alone)).toEqual({ kind: "local" });
    // A shared-only dashboard has no index of its own.
    expect(effectiveScope({ kind: "local" }, server)).toEqual({ kind: "all" });
    // This machine by name is this machine.
    expect(effectiveScope({ kind: "node", node: "laptop" }, joined)).toEqual({ kind: "local" });
    // A machine read live needs no shared index.
    expect(effectiveScope({ kind: "node", node: "xenia" }, { ...alone, live: ["xenia"] })).toEqual({
      kind: "node",
      node: "xenia",
    });
  });

  it("narrows the catalogue to what the scope covers", () => {
    const chats = [chat("a", "laptop"), chat("b", "xenia"), chat("c", "desk")];
    const catalog: Catalog = {
      chats,
      all: chats,
      folded: 0,
      localNode: "laptop",
      sharedOnly: false,
      joined: true,
      warnings: [],
    };
    const ids = (scope: Parameters<typeof scopeCatalog>[1]) =>
      scopeCatalog(catalog, scope).chats.map((c) => c.id);
    expect(ids({ kind: "all" })).toEqual(["a", "b", "c"]);
    expect(ids({ kind: "local" })).toEqual(["a"]);
    expect(ids({ kind: "node", node: "desk" })).toEqual(["c"]);
    expect(inScope(chat("x", "xenia"), { kind: "local" }, "laptop")).toBe(false);
  });

  it("always says which copy a machine's view is", () => {
    expect(scopeName({ kind: "node", node: "xenia" }, joined)).toBe("xenia · live");
    expect(scopeName({ kind: "node", node: "desk" }, joined)).toBe("desk · published copy");
    expect(scopeName({ kind: "all" }, joined)).toBe("All machines");
    expect(scopeSentence({ kind: "node", node: "desk" }, joined, false)).toMatch(
      /hasn't published isn't here/,
    );
    expect(scopeSentence({ kind: "local" }, joined, false)).toMatch(
      /including chats it hasn't published/,
    );
  });

  it("reads memories from the index the scope names", () => {
    expect(recordSource({ kind: "local" }, joined)).toEqual({ params: {} });
    expect(recordSource({ kind: "all" }, joined)).toEqual({ params: { shared: "true" } });
    expect(recordSource({ kind: "all" }, alone)).toEqual({ params: {} });
    expect(recordSource({ kind: "node", node: "desk" }, joined)).toEqual({
      params: { shared: "true", node: "desk" },
    });
    expect(recordSource({ kind: "node", node: "xenia" }, joined)).toEqual({
      params: {},
      peer: "xenia",
    });
  });

  it("is taken out of the address before a page reads its own parameters", () => {
    expect(withoutScope("#/records?scope=all")).toBe("#/records");
    expect(withoutScope("#/search?q=x&scope=node%3Adesk")).toBe("#/search?q=x");
    expect(withoutScope("#/?scope=local")).toBe("#/");
    expect(withoutScope("#/search?q=x")).toBe("#/search?q=x");
  });
});
