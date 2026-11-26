// Made-up data for stories: the demo's history (src/demo), read through the
// same parsers the dashboard reads `mash dash` with. A story never shows real
// chats; what the demo lacks is made up here, in the demo's own world.
import { respond } from "../demo/backend";
import { type Catalog, parseCatalog } from "../lib/api";
import { type NativeRecord, parseRecordPage, type Revision } from "../lib/records";
import type { Answer, ReadPath } from "../lib/routes";
import { type Machine, parseMachines } from "../lib/services.svelte";
import type { ChatSummary } from "../lib/types";

/** The clock every fixture is dated from; the visual baselines stop it. */
export const now = Date.now();

/** The demo's answer to one read, as `mash dash` would send it. */
export function demoAnswer<P extends ReadPath>(path: P, query = ""): Answer<P> {
  const reply = respond(`${path}${query}`, "GET", now);
  if (reply.status !== 200) throw new Error(`the demo does not answer ${path}${query}`);
  return reply.body as Answer<P>;
}

export const catalog: Catalog = parseCatalog(demoAnswer("/api/chats"));
const records: readonly NativeRecord[] = parseRecordPage(demoAnswer("/api/records")).records;

/** A chat by its id in the demo's history. */
export function chat(id: string): ChatSummary {
  const found = catalog.all.find((entry) => entry.id === id);
  if (!found) throw new Error(`no chat ${id} in the demo`);
  return found;
}

/** The first of the demo's memories of one kind. */
export function recordOf(kind: string): NativeRecord {
  const found = records.find((entry) =>
    entry.revisions.some(
      (revision) => revision.revision === entry.currentRevision && revision.details.kind === kind,
    ),
  );
  if (!found) throw new Error(`no ${kind} in the demo`);
  return found;
}

/** A memory with its current revision changed, and what else is true of the record. */
export function revised(
  record: NativeRecord,
  change: Partial<Revision>,
  top: Partial<NativeRecord> = {},
): NativeRecord {
  return {
    ...record,
    ...top,
    revisions: record.revisions.map((revision) =>
      revision.revision === record.currentRevision ? { ...revision, ...change } : revision,
    ),
  };
}

/** The demo has no note, so one is made from its first preference. */
export function note(): NativeRecord {
  const from = recordOf("preference");
  return revised(
    from,
    {
      statement: "The staging database is refreshed from production every Sunday night.",
      details: { kind: "note", state: "active" },
    },
    { id: "record-staging-refresh" },
  );
}

/** A chat an agent started as a subagent, and the chat that started some. */
export const subagentChat: ChatSummary = required(
  catalog.all.find((entry) => entry.subagent),
  "a subagent chat",
);
export const parentChat: ChatSummary = required(
  catalog.all.find((entry) => entry.subagents),
  "a chat with subagents",
);

/** A value the demo is known to hold; a story fails loudly when it stops holding it. */
export function required<T>(value: T | undefined, what: string): T {
  if (value === undefined) throw new Error(`the demo has no ${what}`);
  return value;
}

type WireMachine = Answer<"/api/services">["machines"][number];
type WireService = WireMachine["services"][number];

/** The demo's laptop as `/api/services` reports it, with some of it changed. */
export function wireMachine(
  change: Partial<WireMachine> = {},
  services: (service: WireService) => WireService = (service) => service,
): WireMachine {
  const laptop = demoAnswer("/api/services/local");
  return { ...laptop, ...change, services: laptop.services.map(services) };
}

/** One machine's services as the services page lists them, with their controls. */
export function controlled(states: Readonly<Record<string, WireService["state"]>>): Machine {
  const summary = { healthy: "Running", stopped: "Stopped", failed: "Exited with status 1" };
  const wire = wireMachine({}, (service) => {
    const state = states[service.kind];
    if (state !== "healthy" && state !== "stopped" && state !== "failed") return service;
    return {
      ...service,
      state,
      summary: summary[state],
      pid: state === "healthy" ? 4821 : null,
      actions: state === "healthy" ? ["restart", "stop"] : ["start"],
      control_revision: "story",
    };
  });
  return required(parseMachines({ machines: [wire] })[0], "machine");
}
