import { describe, expect, it } from "vitest";
import {
  ago,
  publishWords,
  reported,
  STALE_AFTER_SECONDS,
  serverReport,
  staleWords,
} from "./serverStatus";
import type { Machine, ServerView } from "./shared";

/** One machine of a report, failing the test when the server did not list it. */
function machineIn(report: ReturnType<typeof serverReport>, id: string) {
  const entry = reported(report, id);
  if (!entry) throw new Error(`${id} is not in the report`);
  return entry;
}

const NOW = 1_800_000_000;
const iso = (seconds: number) => new Date(seconds * 1000).toISOString();

function joined(node: string, publishedAt?: number, extra: Partial<Machine> = {}): Machine {
  return {
    group_id: "harbor",
    node_id: node,
    state: "joined",
    delete_waiting: false,
    ...(publishedAt === undefined
      ? {}
      : {
          published: { chats: 12, messages: 300, instance: "i", last_synced_at: iso(publishedAt) },
        }),
    ...extra,
  } as Machine;
}
function view(machines: Machine[], extra: Partial<ServerView> = {}): ServerView {
  return {
    admin_key: false,
    may_change: false,
    server: "https://mash.harbor.example",
    group_id: "harbor",
    node_id: "laptop",
    status: {
      build: "abcdef1234567890",
      group_id: "harbor",
      machines,
      now: NOW,
      restart_needed: false,
      started_at: NOW - 100,
    },
    ...extra,
  };
}

describe("the shared server's report", () => {
  it("says none when this machine has not joined a server", () => {
    expect(serverReport(undefined)).toEqual({ kind: "none" });
    expect(serverReport({ admin_key: false, may_change: false })).toEqual({ kind: "none" });
  });

  it("says why when the server is set but did not report its status", () => {
    const report = serverReport(
      view([], {
        status: null,
        problem: {
          kind: "shared",
          message: "the shared index at mash:443 did not answer in time",
          retryable: true,
        } as never,
      }),
    );
    expect(report).toEqual({
      kind: "unreachable",
      address: "https://mash.harbor.example",
      message: "the shared index at mash:443 did not answer in time",
    });
    // With no message at all it still says the status is missing, never "healthy".
    expect(serverReport(view([], { status: null }))).toMatchObject({ kind: "unreachable" });
  });

  it("lists machines with when each published, never published, or is still receiving", () => {
    const report = serverReport(
      view([
        joined("workstation", NOW - 28 * 60),
        joined("build-box", NOW - 5 * 86_400),
        joined("studio"),
        joined("tablet", undefined, { receiving: { covered: 40, total: 172 } }),
      ]),
    );
    expect(report.kind).toBe("reported");
    if (report.kind !== "reported") return;
    expect(report.address).toBe("https://mash.harbor.example");
    expect(report.group).toBe("harbor");
    expect(report.machines.map((machine) => machine.id)).toEqual([
      "build-box",
      "studio",
      "tablet",
      "workstation",
    ]);
    const at = (id: string) => machineIn(report, id);
    expect(publishWords(at("workstation"), NOW)).toBe("published 28m ago");
    expect(at("workstation").stale).toBe(false);
    expect(at("build-box").stale).toBe(true);
    expect(staleWords(at("build-box"), NOW)).toBe("5 days old");
    expect(publishWords(at("studio"), NOW)).toBe("nothing published yet");
    expect(at("studio").stale).toBe(false);
    expect(staleWords(at("studio"), NOW)).toBe("");
    expect(publishWords(at("tablet"), NOW)).toBe("first publish: 40 of 172 chats");
  });

  it("keeps a partial report honest: removed, waiting and unparsable times", () => {
    const report = serverReport(
      view([
        { ...joined("old"), state: "removed", delete_requested: false } as Machine,
        { ...joined("new"), state: "waiting", expires_at: NOW + 3600 } as Machine,
        {
          ...joined("odd"),
          published: { chats: 1, messages: 1, instance: "i", last_synced_at: "not a time" },
        } as Machine,
      ]),
    );
    if (report.kind !== "reported") throw new Error("expected a report");
    expect(publishWords(machineIn(report, "old"), NOW)).toBe("removed from the group");
    expect(publishWords(machineIn(report, "new"), NOW)).toBe("waiting to join");
    // A time it cannot read is not taken as recent or as stale.
    expect(reported(report, "odd")?.publishedAt).toBeNull();
    expect(reported(report, "odd")?.stale).toBe(false);
    expect(reported(report, "missing")).toBeUndefined();
  });

  it("calls a machine stale only past the threshold", () => {
    const edge = serverReport(
      view([joined("a", NOW - STALE_AFTER_SECONDS), joined("b", NOW - STALE_AFTER_SECONDS - 1)]),
    );
    expect(reported(edge, "a")?.stale).toBe(false);
    expect(reported(edge, "b")?.stale).toBe(true);
  });

  it("says how long ago in short words", () => {
    expect(ago(NOW - 10, NOW)).toBe("just now");
    expect(ago(NOW - 90, NOW)).toBe("1m ago");
    expect(ago(NOW - 3 * 3600, NOW)).toBe("3h ago");
    expect(ago(NOW - 86_400, NOW)).toBe("1 day ago");
    expect(ago(NOW + 50, NOW)).toBe("just now");
  });
});
