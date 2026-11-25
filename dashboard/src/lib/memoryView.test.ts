import { describe, expect, it } from "vitest";
import { chatOf, groupByWeek, speaker, weekStart } from "./memoryView";
import type { NativeRecord } from "./records";
import type { ChatSummary } from "./types";

function record(id: string, recordedAt: number): NativeRecord {
  return {
    id,
    owner: "sam",
    node: "",
    scope: { kind: "personal" },
    currentRevision: 1,
    revisions: [
      {
        revision: 1,
        statement: id,
        rationale: "",
        review: "established",
        recordedBy: "Codex",
        recordedAt,
        effectiveAt: null,
        details: { kind: "note", state: "active" },
        evidence: [],
        evidenceCurrent: true,
        confirmations: 0,
      },
    ],
    conflicted: false,
    links: [],
    nextBeforeRevision: null,
    nextLinkOffset: null,
  };
}

describe("memory groups", () => {
  // Thursday 2026-11-26, noon local time.
  const now = new Date(2026, 10, 26, 12).getTime();

  it("starts the week on Monday at midnight", () => {
    expect(new Date(weekStart(now))).toEqual(new Date(2026, 10, 23));
    const monday = new Date(2026, 10, 23, 9).getTime();
    expect(weekStart(monday)).toBe(new Date(2026, 10, 23).getTime());
  });

  it("puts this week's records first and keeps the page's order in each group", () => {
    const records = [
      record("old", new Date(2026, 10, 20).getTime()),
      record("new", new Date(2026, 10, 25).getTime()),
      record("monday", new Date(2026, 10, 23, 0, 1).getTime()),
    ];
    const groups = groupByWeek(records, now);
    expect(groups.map((group) => group.label)).toEqual(["This week", "Earlier"]);
    expect(groups[0]?.records.map((item) => item.id)).toEqual(["new", "monday"]);
    expect(groups[1]?.records.map((item) => item.id)).toEqual(["old"]);
  });

  it("leaves out a group with nothing in it", () => {
    expect(groupByWeek([record("old", 0)], now).map((group) => group.label)).toEqual(["Earlier"]);
    expect(groupByWeek([], now)).toEqual([]);
  });
});

describe("quoted chats", () => {
  const chat = (id: string, machine: string) => ({ id, machine }) as ChatSummary;

  it("prefers the chat on the record's machine", () => {
    const chats = [chat("a", "laptop"), chat("a", "workstation")];
    expect(chatOf(chats, "a", "workstation")?.machine).toBe("workstation");
    expect(chatOf(chats, "a", "")?.machine).toBe("laptop");
    expect(chatOf(chats, "b", "")).toBeUndefined();
  });

  it("names who said it", () => {
    expect(speaker("user")).toBe("You");
    expect(speaker("assistant")).toBe("Assistant");
    expect(speaker(null)).toBe("Someone");
  });
});
