// What a machine's status dot means, decided in one place for every page.
//
// A dot is only as good as its evidence, so it says what was observed and how:
//
// 1. This dashboard's own service check, which asks the machine's dashboard
//    directly. An answer is a heartbeat: the dot shows the health the machine
//    reported. No answer leaves a hollow ring: the machine may be asleep or
//    away, which is not a fault this dashboard can see.
// 2. Otherwise, the shared server's report of when the machine last published.
//    A recent publish is green; a copy older than the stale limit is amber;
//    a machine that never published, or was removed, is a hollow ring.
// 3. Otherwise nothing is known, and the ring stays hollow. A dot is never
//    green by default.
import { agoLabel } from "./browse";
import { reported, type ServerReport } from "./serverStatus";
import { type Machine, machineState, stateLabel } from "./services.svelte";

/** The dot's colour class, as `.dot` in terminal.css draws it. */
export type PulseTone = "ok" | "warn" | "bad" | "none";

export interface Pulse {
  readonly tone: PulseTone;
  /** What the dot stands for, for its tooltip and screen readers. */
  readonly words: string;
}

export interface PulseEvidence {
  /** This dashboard's service checks, when it checks any machines. */
  readonly fleet: readonly Machine[] | undefined;
  readonly report: ServerReport;
  /** Milliseconds since 1970, for "checked 2m ago". */
  readonly now: number;
}

export function machinePulse(name: string, evidence: PulseEvidence): Pulse {
  const checked = evidence.fleet?.find((machine) => machine.id === name);
  if (checked) return checkedPulse(checked, evidence.now);
  const entry = reported(evidence.report, name);
  if (entry && evidence.report.kind === "reported") {
    if (entry.removed) return { tone: "none", words: "Removed from the shared group" };
    if (entry.publishedAt === null) return { tone: "none", words: "Has not published yet" };
    const when = agoLabel(entry.publishedAt * 1000, evidence.report.now * 1000);
    return entry.stale
      ? { tone: "warn", words: `Its shared copy is stale: last published ${when}` }
      : { tone: "ok", words: `Published ${when}` };
  }
  return { tone: "none", words: "No status reported" };
}

function checkedPulse(machine: Machine, now: number): Pulse {
  const checked = agoLabel(machine.checked_at * 1000, now);
  if (!machine.reachable) return { tone: "none", words: `Did not answer · checked ${checked}` };
  const state = machineState(machine);
  const tone: PulseTone =
    state === "healthy" ? "ok" : state === "failed" ? "bad" : state === "unknown" ? "none" : "warn";
  return { tone, words: `${stateLabel[state]} · checked ${checked}` };
}
