// The services map's top band: the shared server and the other machines.
//
// A machine card comes from this dashboard's own service checks (its health,
// and a map to open), from the shared server's report (when it published), or
// from both. Health is said only where this dashboard checked it; when it
// published, only where the server reported it. Nothing is inferred.
import type { ServerMachine, ServerReport } from "./serverStatus";
import { publishWords, reported } from "./serverStatus";
import { type Machine, machineState, type ServiceState, stateLabel } from "./services.svelte";

/** A machine in the band. */
export interface Card {
  readonly id: string;
  /** This dashboard's check of it; without one there is no map to open. */
  readonly fleet: Machine | undefined;
  /** The shared server's report of it. */
  readonly reported: ServerMachine | undefined;
}

/** Every machine but the one whose map is open, from either source, by name. */
export function bandCards(others: readonly Machine[], report: ServerReport, open: string): Card[] {
  const ids = new Set(others.map((machine) => machine.id));
  if (report.kind === "reported")
    for (const machine of report.machines) if (machine.id !== open) ids.add(machine.id);
  return [...ids]
    .sort((a, b) => a.localeCompare(b))
    .map((name) => ({
      id: name,
      fleet: others.find((machine) => machine.id === name),
      reported: reported(report, name),
    }));
}

/** What the band says after "Shared server". */
export function bandWords(report: ServerReport): string {
  if (report.kind === "reported") return report.address;
  return report.kind === "unreachable" ? "didn't report its status" : "address not reported";
}

/** A box's colour class for a state; null is a place with no state of its own. */
export function tone(state: ServiceState | null): string {
  if (state === null) return "neutral";
  if (state === "healthy") return "ok";
  if (state === "failed") return "bad";
  if (state === "unknown") return "unknown";
  return "warn";
}

export function cardTone(card: Card): string {
  if (!card.fleet) return "neutral";
  return card.fleet.reachable ? tone(machineState(card.fleet)) : "unknown";
}

/** Its health where this dashboard checks it; empty otherwise. */
export function cardHealth(card: Card): string {
  if (!card.fleet) return "";
  return card.fleet.reachable ? stateLabel[machineState(card.fleet)] : "Did not answer";
}

/** A card publishes into the server when the server says it published, or,
 * without a report of it, when this dashboard sees a shared-server service on it. */
export function publishes(card: Card): boolean {
  if (card.reported) return card.reported.publishedAt !== null && !card.reported.removed;
  return !!card.fleet?.reachable && card.fleet.services.some((entry) => entry.kind === "server");
}

/** "publishes · 12m ago" for the open machine's own connection to the server. */
export function publishVerb(verb: string, own: ServerMachine | undefined, now: number): string {
  if (!own?.publishedAt) return verb;
  return `${verb} · ${publishWords(own, now).replace(/^published /, "")}`;
}
