import { describe, expect, test } from "vitest";
import type { Machine } from "./shared";
import { agentShares, publishProgress, shareSummary, unsharedOf } from "./sharing";
import type { ChatSummary } from "./types";

const chat = (agent: ChatSummary["agent"], machine: string): ChatSummary => ({
  id: `${agent}-${machine}-${Math.random()}`,
  title: "t",
  agent,
  machine,
  project: "",
  updatedAt: "",
});

describe("what a machine shares", () => {
  const chats = [
    chat("codex", "studio"),
    chat("codex", "studio"),
    chat("cursor", "studio"),
    // Another machine's chats are never counted as this one's.
    chat("codex", "laptop"),
    // An agent this machine does not index is not offered.
    chat("opencode", "studio"),
  ];

  test("counts only this machine's chats of the agents it indexes", () => {
    const rows = agentShares(chats, "studio", ["codex", "cursor", "claude-code"], ["cursor"]);
    expect(rows).toEqual([
      { agent: "codex", chats: 2, shared: true },
      { agent: "cursor", chats: 1, shared: false },
      { agent: "claude-code", chats: 0, shared: true },
    ]);
    expect(shareSummary(rows)).toEqual({
      shared: 2,
      kept: 1,
      sharedAgents: ["codex", "claude-code"],
      keptAgents: ["cursor"],
    });
    expect(unsharedOf(rows)).toEqual(["cursor"]);
  });
});

describe("publish progress comes only from the server", () => {
  const machine = (extra: Partial<Machine>): Machine =>
    ({
      group_id: "g",
      node_id: "studio",
      state: "active",
      delete_waiting: false,
      ...extra,
    }) as Machine;

  test("nothing reported yet is waiting", () => {
    expect(publishProgress(undefined)).toEqual({ state: "waiting" });
    expect(publishProgress(machine({}))).toEqual({ state: "waiting" });
  });
  test("a first publication still arriving reports sent of total", () => {
    expect(publishProgress(machine({ receiving: { covered: 40, total: 172 } }))).toEqual({
      state: "receiving",
      sent: 40,
      total: 172,
    });
  });
  test("a finished publication reports what the server holds", () => {
    expect(
      publishProgress(
        machine({
          published: {
            instance: "i",
            chats: 172,
            messages: 900,
            last_synced_at: "2026-11-23T10:00:00Z",
          },
        }),
      ),
    ).toEqual({ state: "published", chats: 172, at: "2026-11-23T10:00:00Z" });
  });
});
