// What a state means at a glance: a tone, which gives it a colour (the status
// roles in theme/tokens.css) and a shape (one Lucide icon). Every page that
// shows a state draws it through these, beside the state's own word, so the
// colour never has to carry the meaning alone and the same meaning always
// looks the same.
import CircleCheck from "@lucide/svelte/icons/circle-check";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import CirclePause from "@lucide/svelte/icons/circle-pause";
import CircleQuestionMark from "@lucide/svelte/icons/circle-question-mark";
import CircleX from "@lucide/svelte/icons/circle-x";
import Info from "@lucide/svelte/icons/info";
import LoaderCircle from "@lucide/svelte/icons/loader-circle";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import type { ServiceState } from "./services.svelte";

/**
 * ok: healthy, done, in force. warn: needs a look. bad: failed, replaced,
 * contradicted. info: worth knowing, not a judgement. run: in progress.
 * pause: stopped on purpose. idle: quiet or switched off, not a fault.
 * unknown: nothing said either way.
 */
export type Tone = "ok" | "warn" | "bad" | "info" | "run" | "pause" | "idle" | "unknown";

export const toneIcons = {
  ok: CircleCheck,
  warn: TriangleAlert,
  bad: CircleX,
  info: Info,
  run: LoaderCircle,
  pause: CirclePause,
  idle: CircleDashed,
  unknown: CircleQuestionMark,
} as const;

/** A service's state as a tone. */
export function serviceTone(state: ServiceState): Tone {
  if (state === "healthy") return "ok";
  if (state === "failed") return "bad";
  if (state === "unknown") return "unknown";
  return "warn";
}

/** The older dot classes ("ok", "warn", "bad", "run", "none", "neutral") as tones. */
export function dotTone(dot: string): Tone {
  if (dot === "ok" || dot === "warn" || dot === "bad" || dot === "run") return dot;
  if (dot === "unknown") return "unknown";
  return "idle";
}

/** A machine's pulse (see machinePulse) as a tone: no news is idle, unless
 * nothing reports on the machine at all, which is unknown. */
export function pulseTone(pulse: {
  tone: "ok" | "warn" | "bad" | "none";
  unreported?: boolean;
}): Tone {
  if (pulse.tone !== "none") return pulse.tone;
  return pulse.unreported ? "unknown" : "idle";
}

/** A memory's lifecycle as a tone: in force is ok, proposed is information,
 * anything replaced or taken back is bad (the lists show those in red). */
export function lifecycleTone(state: string): Tone {
  if (state === "accepted" || state === "active") return "ok";
  if (state === "proposed") return "info";
  return "bad";
}

/** How settled a memory is (see `standing` in records.ts) as a tone:
 * unconfirmed needs a look, an agent's own decision is worth knowing. */
export function standingTone(standing: string): Tone {
  if (standing === "unconfirmed") return "warn";
  if (standing === "agent decided") return "info";
  return "ok";
}

const DAY = 24 * 3600 * 1000;
/** How a time should stand out in a list: within the last day it is fresh,
 * older than a month it is old, anything between is plain. */
export function recency(ms: number | undefined, now: number): "when-fresh" | "when-old" | "" {
  if (ms === undefined) return "";
  const age = now - ms;
  if (age < DAY) return "when-fresh";
  return age > 30 * DAY ? "when-old" : "";
}
