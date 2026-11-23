import { describe, expect, it } from "vitest";
import { browse, type Catalog } from "./api";
import { withoutCopies } from "./sshCopies";
import {
  childrenOf,
  detailLabel,
  fold,
  foldedNote,
  originLabel,
  parentOf,
  spawnedBy,
  stateLabel,
} from "./subagents";
import type { ChatSummary } from "./types";

function chat(id: string, extra: Partial<ChatSummary> = {}): ChatSummary {
  return {
    id,
    title: id,
    agent: "claude-code",
    machine: "laptop",
    project: "/repo",
    updatedAt: "",
    ...extra,
  };
}

const parent = chat("session");
const first = chat("a1", {
  createdMs: 1,
  subagent: {
    parentId: "session",
    label: "Fix tests",
    role: "general-purpose",
    finished: true,
    spawnCallId: "toolu_1",
  },
});
const second = chat("a2", { createdMs: 2, subagent: { parentId: "session", label: "Notes" } });
// The same parent ID on another machine, or from another agent, is another chat.
const elsewhere = chat("b1", { machine: "desktop", subagent: { parentId: "session" } });
const otherAgent = chat("c1", { agent: "codex", subagent: { parentId: "session" } });
const orphan = chat("d1", { subagent: { parentId: "gone", parentTitle: "Gone chat" } });
const unparented = chat("e1", { agent: "codex", subagent: { role: "review" } });
const all = [parent, first, second, elsewhere, otherAgent, orphan, unparented];

describe("subagent folding", () => {
  it("folds only subagents whose exact parent is listed, and counts them on it", () => {
    const folded = fold(all);
    expect(folded.chats.map((chat) => chat.id)).toEqual(["session", "b1", "c1", "d1", "e1"]);
    expect(folded.folded).toBe(2);
    expect(folded.chats[0]?.subagents).toBe(2);
    expect(folded.all).toHaveLength(all.length);
    expect(folded.all.find((chat) => chat.id === "session")?.subagents).toBe(2);
  });

  it("never folds a chat that is not marked, whatever its title says", () => {
    const brief = chat("brief", { title: "Work ONLY in /home/me" });
    expect(fold([parent, brief]).chats).toHaveLength(2);
    expect(fold([parent, brief]).folded).toBe(0);
  });

  it("finds children and parents by machine, agent and identity", () => {
    expect(childrenOf(all, parent).map((chat) => chat.id)).toEqual(["a1", "a2"]);
    expect(parentOf(all, first)).toBe(parent);
    expect(parentOf(all, elsewhere)).toBeUndefined();
    expect(parentOf(all, otherAgent)).toBeUndefined();
    expect(spawnedBy([first, second], "toolu_1")).toBe(first);
    expect(spawnedBy([first, second], undefined)).toBeUndefined();
    expect(spawnedBy([first, second], "toolu_2")).toBeUndefined();
  });

  it("labels state, origin and counts with what the agent recorded", () => {
    expect(stateLabel(first.subagent)).toBe("done");
    expect(stateLabel({ finished: false })).toBe("running");
    expect(stateLabel(second.subagent)).toBeUndefined();
    expect(detailLabel(first, "58m")).toBe("general-purpose · 58m · done");
    expect(detailLabel(second, "1h")).toBe("1h");
    expect(originLabel(first, "Ship it")).toBe("subagent of Ship it");
    expect(originLabel(orphan, undefined)).toBe("subagent of Gone chat");
    expect(originLabel(elsewhere, undefined)).toBe("subagent of a chat not on this machine");
    expect(originLabel(unparented, undefined)).toBe("subagent");
    expect(foldedNote(0)).toBe("");
    expect(foldedNote(11)).toBe(" · 11 subagent chats folded in");
    expect(foldedNote(1)).toBe(" · 1 subagent chat folded in");
  });

  it("lists folded or shown chats, counting what was folded under the filters", () => {
    const catalog: Catalog = { ...fold(all), localNode: "laptop", sharedOnly: false, warnings: [] };
    const filters = { agents: [], machines: ["laptop"], projects: [], days: null };
    const folded = browse(catalog, filters, 0);
    expect(folded.hits.map((hit) => hit.chat.id)).toEqual(["session", "c1", "d1", "e1"]);
    expect(folded.folded).toBe(2);
    const shown = browse(catalog, filters, 0, "shown");
    expect(shown.hits.map((hit) => hit.chat.id)).toEqual(["session", "a1", "a2", "c1", "d1", "e1"]);
    expect(shown.folded).toBe(0);
  });
});

describe("sessions copied over SSH", () => {
  const copy = chat("s1", { machine: "macbook", sshCopy: true });
  const original = chat("s1", { machine: "xenia" });
  it("lists the original once another machine shares it", () => {
    expect(withoutCopies([copy, original])).toEqual([original]);
    expect(withoutCopies([{ chat: copy }], [original])).toEqual([]);
  });
  it("keeps a copy whose original is not here, or is another agent's chat", () => {
    expect(withoutCopies([copy])).toEqual([copy]);
    const codex = chat("s1", { machine: "xenia", agent: "codex" });
    expect(withoutCopies([copy, codex])).toEqual([copy, codex]);
    // Two copies are not each other's original.
    const second = chat("s1", { machine: "laptop", sshCopy: true });
    expect(withoutCopies([copy, second])).toEqual([copy, second]);
  });
});
