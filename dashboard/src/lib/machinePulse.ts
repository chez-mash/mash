// What a machine's status dot means, decided in one place for every page.
//
// A dot is only as good as its evidence, so it says what was observed and how:
//
// 1. This dashboard's own service check, which asks the machine's dashboard
//    directly. An answer shows the health the machine reported.
// 2. Otherwise, the machine's heartbeat: its watcher checks in with the shared
//    server every minute while it runs. A recent check-in is green; an old
//    one is a hollow ring, since a machine asleep or away is not a fault.
// 3. Otherwise, how the shared server says it published. A machine that does
//    not check in, an older Mash say, is never green on that alone: a stale
//    copy is amber and anything else a hollow ring.
// 4. Otherwise nothing is known, and the ring stays hollow. A dot is never
//    green by default.
import { agoLabel } from "./browse";
import { reported, type ServerMachine, type ServerReport } from "./serverStatus";
import { type Machine, machineState, stateLabel } from "./services.svelte";

/** The dot's colour class, as `.dot` in terminal.css draws it. */
export type PulseTone = "ok" | "warn" | "bad" | "none";

export interface Pulse {
  readonly tone: PulseTone;
  /** What the dot stands for, for its tooltip. */
  readonly words: string;
}

export interface PulseEvidence {
  /** This dashboard's service checks, when it checks any machines. */
  readonly fleet: readonly Machine[] | undefined;
  readonly report: ServerReport;
  /** Milliseconds since 1970, for "checked 2m ago". */
  readonly now: number;
}

/** A watcher checks in every minute; three missed check-ins is quiet. */
export const QUIET_AFTER_SECONDS = 3 * 60;

export function machinePulse(name: string, evidence: PulseEvidence): Pulse {
  const checked = evidence.fleet?.find((machine) => machine.id === name);
  if (checked?.reachable) return checkedPulse(checked, evidence.now);
  const entry = reported(evidence.report, name);
  const serverNow = evidence.report.kind === "reported" ? evidence.report.now : 0;
  if (entry?.lastSeenAt != null) return heartbeatPulse(entry.lastSeenAt, serverNow);
  if (checked) {
    const when = agoLabel(checked.checked_at * 1000, evidence.now);
    return { tone: "none", words: `Did not answer · checked ${when}` };
  }
  if (entry) return publishedPulse(entry, serverNow);
  return { tone: "none", words: "No status reported" };
}

function checkedPulse(machine: Machine, now: number): Pulse {
  const checked = agoLabel(machine.checked_at * 1000, now);
  const state = machineState(machine);
  const tone: PulseTone =
    state === "healthy" ? "ok" : state === "failed" ? "bad" : state === "unknown" ? "none" : "warn";
  return { tone, words: `${stateLabel[state]} · checked ${checked}` };
}

function heartbeatPulse(lastSeenAt: number, serverNow: number): Pulse {
  const when = agoLabel(lastSeenAt * 1000, serverNow * 1000);
  return serverNow - lastSeenAt <= QUIET_AFTER_SECONDS
    ? { tone: "ok", words: `Running · checked in ${when}` }
    : { tone: "none", words: `Quiet · last checked in ${when}` };
}

function publishedPulse(entry: ServerMachine, serverNow: number): Pulse {
  if (entry.removed) return { tone: "none", words: "Removed from the shared group" };
  if (entry.publishedAt === null)
    return { tone: "none", words: "Not checking in · nothing published yet" };
  const when = agoLabel(entry.publishedAt * 1000, serverNow * 1000);
  return entry.stale
    ? { tone: "warn", words: `Not checking in · its shared copy is stale, last published ${when}` }
    : { tone: "none", words: `Not checking in · last published ${when}` };
}
