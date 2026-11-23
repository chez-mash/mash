import { describe, expect, it } from "vitest";
import { bandCards, bandWords, cardHealth, cardTone, publishes, publishVerb } from "./mapBand";
import { serverReport } from "./serverStatus";
import type { Machine, Service } from "./services.svelte";
import type { Machine as SharedMachine } from "./shared";

const NOW = 1_800_000_000;
const sharedService = { id: "shared-server", kind: "server", state: "healthy" } as Service;
const fleetMachine = (
  id: string,
  reachable = true,
  services: Service[] = [sharedService],
): Machine => ({
  id,
  platform: "linux",
  checked_at: 1,
  reachable,
  services: reachable ? services : [],
  warnings: [],
});
const joined = (node: string, publishedAgo?: number): SharedMachine =>
  ({
    group_id: "harbor",
    node_id: node,
    state: "joined",
    delete_waiting: false,
    ...(publishedAgo === undefined
      ? {}
      : {
          published: {
            chats: 1,
            messages: 1,
            instance: "i",
            last_synced_at: new Date((NOW - publishedAgo) * 1000).toISOString(),
          },
        }),
  }) as SharedMachine;
const report = serverReport({
  admin_key: false,
  may_change: false,
  server: "https://mash.harbor.example",
  status: {
    build: "b",
    group_id: "harbor",
    now: NOW,
    restart_needed: false,
    started_at: NOW,
    machines: [joined("laptop", 60), joined("workstation", 600), joined("studio")],
  },
});

/** One card by name, failing the test when it is missing. */
function card(cards: ReturnType<typeof bandCards>, id: string) {
  const found = cards.find((entry) => entry.id === id);
  if (!found) throw new Error(`no card for ${id}`);
  return found;
}

describe("the services map's shared band", () => {
  it("joins this dashboard's machines with the server's, leaving out the open one", () => {
    const cards = bandCards(
      [fleetMachine("workstation"), fleetMachine("build-box", false)],
      report,
      "laptop",
    );
    expect(cards.map((card) => card.id)).toEqual(["build-box", "studio", "workstation"]);
    const buildBox = card(cards, "build-box");
    const studio = card(cards, "studio");
    const workstation = card(cards, "workstation");
    // Health only where this dashboard checked; publishing only where the server reported.
    expect(cardHealth(workstation)).toBe("Healthy");
    expect(cardHealth(buildBox)).toBe("Did not answer");
    expect(cardTone(buildBox)).toBe("unknown");
    expect(cardHealth(studio)).toBe("");
    expect(cardTone(studio)).toBe("neutral");
    expect(studio.fleet).toBeUndefined();
  });

  it("draws a machine's connection to the server only when it published", () => {
    const cards = bandCards([fleetMachine("workstation")], report, "laptop");
    expect(publishes(card(cards, "workstation"))).toBe(true);
    // The server knows studio, but it never published.
    expect(publishes(card(cards, "studio"))).toBe(false);
    // Without a report, a reachable machine with a shared-server service counts.
    const unreported = bandCards(
      [fleetMachine("a"), fleetMachine("b", true, [])],
      { kind: "none" },
      "laptop",
    );
    expect(unreported.map(publishes)).toEqual([true, false]);
  });

  it("says what it knows of the server and of this machine's last publish", () => {
    expect(bandWords(report)).toBe("https://mash.harbor.example");
    expect(bandWords({ kind: "none" })).toBe("address not reported");
    expect(bandWords({ kind: "unreachable", address: "x", message: "m" })).toBe(
      "didn't report its status",
    );
    const own =
      report.kind === "reported" ? report.machines.find((m) => m.id === "laptop") : undefined;
    expect(publishVerb("publishes", own, NOW)).toBe("publishes · 1m ago");
    expect(publishVerb("publishes", undefined, NOW)).toBe("publishes");
  });
});
