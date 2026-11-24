import { describe, expect, it } from "vitest";
import { machinePulse } from "./machinePulse";
import { serverReport } from "./serverStatus";
import type { Machine } from "./services.svelte";
import type { ServerView, Machine as SharedMachine } from "./shared";

const NOW = 1_800_000_000;
const iso = (seconds: number) => new Date(seconds * 1000).toISOString();

function checked(id: string, reachable: boolean, state = "healthy"): Machine {
  return {
    id,
    platform: "linux",
    checked_at: NOW - 120,
    reachable,
    services: reachable
      ? [{ id: "sqlite", kind: "sqlite", state, summary: "", checked_at: NOW } as never]
      : [],
    warnings: [],
  } as Machine;
}
function published(node: string, at?: number, seen?: number): SharedMachine {
  return {
    group_id: "harbor",
    node_id: node,
    state: "joined",
    delete_waiting: false,
    ...(seen === undefined ? {} : { last_seen_at: seen }),
    ...(at === undefined
      ? {}
      : { published: { chats: 1, messages: 1, instance: "i", last_synced_at: iso(at) } }),
  } as SharedMachine;
}
const report = serverReport({
  admin_key: false,
  may_change: false,
  server: "https://mash.harbor.example",
  group_id: "harbor",
  node_id: "laptop",
  status: {
    build: "b",
    group_id: "harbor",
    machines: [
      published("fresh", NOW - 3600),
      published("old", NOW - 5 * 86_400),
      published("new"),
      published("beating", NOW - 5 * 86_400, NOW - 50),
      published("quiet", NOW - 3600, NOW - 2 * 3600),
    ],
    now: NOW,
    restart_needed: false,
    started_at: NOW,
  },
} as ServerView);
const evidence = (fleet?: Machine[]) => ({ fleet, report, now: NOW * 1000 });

describe("a machine's status dot", () => {
  it("shows the health a machine reported when this dashboard checked it", () => {
    expect(machinePulse("box", evidence([checked("box", true)]))).toEqual({
      tone: "ok",
      words: "Healthy · checked 2m ago",
    });
    expect(machinePulse("box", evidence([checked("box", true, "failed")])).tone).toBe("bad");
    expect(machinePulse("box", evidence([checked("box", true, "stopped")])).tone).toBe("warn");
  });

  it("is a hollow ring, not a fault, for a machine that did not answer", () => {
    expect(machinePulse("box", evidence([checked("box", false)]))).toEqual({
      tone: "none",
      words: "Did not answer · checked 2m ago",
    });
  });

  it("prefers this dashboard's check over the server's report", () => {
    expect(machinePulse("old", evidence([checked("old", true)])).tone).toBe("ok");
    expect(machinePulse("quiet", evidence([checked("quiet", true)])).tone).toBe("ok");
  });

  it("is green while a machine's watcher keeps checking in, however old its copy", () => {
    expect(machinePulse("beating", evidence())).toEqual({
      tone: "ok",
      words: "Running · checked in just now",
    });
  });

  it("is a hollow ring once check-ins stop, even after a recent publish", () => {
    expect(machinePulse("quiet", evidence())).toEqual({
      tone: "none",
      words: "Quiet · last checked in 2h ago",
    });
  });

  it("takes a heartbeat over a check that did not answer", () => {
    expect(machinePulse("beating", evidence([checked("beating", false)])).tone).toBe("ok");
  });

  it("is never green on a publish alone, and amber for a stale copy", () => {
    expect(machinePulse("fresh", evidence())).toEqual({
      tone: "none",
      words: "Not checking in · last published 1h ago",
    });
    expect(machinePulse("old", evidence()).tone).toBe("warn");
    expect(machinePulse("new", evidence()).tone).toBe("none");
  });

  it("is never green without evidence", () => {
    expect(machinePulse("stranger", evidence())).toEqual({
      tone: "none",
      words: "No status reported",
    });
    expect(
      machinePulse("stranger", { fleet: undefined, report: { kind: "none" }, now: NOW * 1000 })
        .tone,
    ).toBe("none");
  });
});
