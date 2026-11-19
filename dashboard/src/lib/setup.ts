// First-run setup, as `mash init` asks it in a terminal: which histories to
// index, how far back, and which agents to connect. The machine finds the
// history locations itself; the page only chooses among what it found.
import { queryOptions } from "@tanstack/svelte-query";
import { isAgent } from "./api";
import { stableRead } from "./queries";
import type { Answer, Changed, Wire } from "./routes";
import { get, post } from "./services.svelte";
import type { Agent } from "./types";

/** An agent Mash connects itself to through that agent's own CLI. */
export type Integration = Wire["SetupAgent"];
export type IndexRun =
  | { readonly state: "idle" }
  | { readonly state: "running" }
  | { readonly state: "skipped"; readonly reason: string }
  | {
      readonly state: "finished";
      readonly ok: boolean;
      readonly chats: number | null;
      readonly messages: number | null;
      readonly message: string | null;
    };
export interface FirstRun {
  readonly path: string;
  /** Names the file's contents; empty while there is no file. */
  readonly revision: string;
  readonly configured: boolean;
  /** False when this dashboard shows setup but does not run it. */
  readonly editable: boolean;
  readonly histories: readonly {
    readonly agent: Agent;
    readonly indexed: boolean;
    readonly roots: readonly string[];
  }[];
  readonly sinceDays: number | null;
  /** Whether the file publishes to a shared server, which setup keeps. */
  readonly shared: boolean;
  readonly integrations: readonly {
    readonly agent: Integration;
    readonly installed: boolean;
    readonly connected: boolean;
  }[];
  readonly qdrant: {
    readonly url: string;
    readonly reachable: boolean;
    readonly message: string | null;
  };
  readonly modelsInstalled: boolean;
  /** The first index this dashboard started, if any. */
  readonly index: IndexRun;
}
/** What the reader has answered so far. */
export interface Answers {
  agents: Partial<Record<Agent, boolean>>;
  sinceDays: number | null;
  connect: Integration[];
}

function parseIndex(run: Wire["FirstRunIndex"]): IndexRun {
  switch (run.state) {
    case "skipped":
      return { state: "skipped", reason: run.reason };
    case "finished":
      return {
        state: "finished",
        ok: run.ok,
        chats: run.chats ?? null,
        messages: run.messages ?? null,
        message: run.message ?? null,
      };
    default:
      return { state: run.state };
  }
}

export function parseFirstRun(body: Answer<"/api/services/setup">): FirstRun {
  return {
    path: body.path,
    revision: body.revision,
    configured: body.configured,
    editable: body.editable,
    histories: Object.entries(body.histories).flatMap(([agent, row]) =>
      isAgent(agent) && row ? [{ agent, indexed: row.indexed, roots: row.roots }] : [],
    ),
    sinceDays: body.since_days ?? null,
    shared: body.shared,
    integrations: body.integrations,
    qdrant: {
      url: body.qdrant.url,
      reachable: body.qdrant.reachable,
      message: body.qdrant.message ?? null,
    },
    modelsInstalled: body.models_installed,
    index: parseIndex(body.index),
  };
}

export const firstRunKey = ["first-run"] as const;
/** Reread every two seconds while the first index runs, and not otherwise. */
export const firstRunOptions = queryOptions({
  ...stableRead,
  queryKey: firstRunKey,
  queryFn: async ({ signal }) => parseFirstRun(await get("/api/services/setup", signal)),
  refetchInterval: (query) => (query.state.data?.index.state === "running" ? 2000 : false),
});

/** The answers a reader starts from: everything found, and every installed agent. */
export function answersOf(setup: FirstRun): Answers {
  return {
    agents: Object.fromEntries(setup.histories.map((entry) => [entry.agent, entry.indexed])),
    sinceDays: setup.sinceDays,
    connect: setup.integrations.filter((entry) => entry.installed).map((entry) => entry.agent),
  };
}

export const days = { least: 1, most: 36_500 } as const;
export function validDays(value: number | null): boolean {
  return (
    value === null || (Number.isSafeInteger(value) && value >= days.least && value <= days.most)
  );
}

/** Why the answers cannot be used as they stand, or an empty string. */
export function problem(setup: FirstRun, answers: Answers): string {
  if (!setup.histories.length)
    return "No agent history was found on this machine in its usual locations.";
  if (!setup.histories.some((entry) => answers.agents[entry.agent] ?? entry.indexed))
    return "Choose at least one agent's history to index.";
  if (!validDays(answers.sinceDays)) return "History range must be between 1 and 36,500 days.";
  return "";
}

export interface Line {
  readonly label: string;
  readonly value: string;
}
/** The answers in the words the review step shows. */
export function summary(
  setup: FirstRun,
  answers: Answers,
  names: Readonly<Record<Agent, string>>,
): Line[] {
  const indexed = setup.histories
    .filter((entry) => answers.agents[entry.agent] ?? entry.indexed)
    .map((entry) => names[entry.agent]);
  const connected = setup.integrations
    .filter((entry) => answers.connect.includes(entry.agent))
    .map((entry) => names[entry.agent]);
  return [
    { label: "configuration file", value: setup.path },
    { label: "histories to index", value: indexed.join(", ") || "none" },
    {
      label: "history range",
      value: answers.sinceDays === null ? "all history" : `the last ${answers.sinceDays} days`,
    },
    { label: "connect Mash to", value: connected.join(", ") || "no agents" },
    {
      label: "search",
      value: setup.shared ? "this machine and the shared server" : "this machine only",
    },
  ];
}

export type Step = Wire["InitStep"];
export interface Outcome {
  readonly path: string;
  readonly written: boolean;
  /** Where the replaced configuration was kept, when one was. */
  readonly previous: string | null;
  readonly agents: readonly {
    readonly agent: Integration;
    readonly mcp: Step;
    readonly plugin: Step;
    readonly files: readonly string[];
    readonly undo: readonly string[];
  }[];
  readonly next: readonly string[];
}
export function parseOutcome(report: Changed<"/api/services/setup">["report"]): Outcome {
  return {
    path: report.config.path,
    written: report.config.written,
    previous: report.config.previous ?? null,
    agents: report.agents,
    next: report.next,
  };
}

/** A step of connecting an agent, in a few words. */
export function stepWord(step: Step): { readonly ok: boolean; readonly word: string } {
  switch (step.state) {
    case "done":
      return { ok: true, word: "added" };
    case "already-present":
      return { ok: true, word: "already there" };
    case "replaced":
      return { ok: true, word: "replaced an old one" };
    case "stale":
      return { ok: false, word: "old one could not be replaced" };
    case "failed":
      return { ok: false, word: "failed" };
  }
}

/** Saves the configuration, connects the agents and, when asked, starts the first index. */
export async function runSetup(
  setup: FirstRun,
  answers: Answers,
  index: boolean,
): Promise<{ outcome: Outcome; setup: FirstRun }> {
  const done = await post("/api/services/setup", {
    revision: setup.revision,
    agents: answers.agents,
    since_days: answers.sinceDays,
    connect: answers.connect,
    index,
  });
  return { outcome: parseOutcome(done.report), setup: parseFirstRun(done.setup) };
}

const skipped = "mash.setup.skipped";
/** Remember, for this browser tab, that the reader chose the dashboard over setup. */
export function skipSetup(): void {
  try {
    sessionStorage.setItem(skipped, "1");
  } catch {
    // Without storage the reader is offered setup again on the next visit.
  }
}
function hasSkipped(): boolean {
  try {
    return sessionStorage.getItem(skipped) === "1";
  } catch {
    return false;
  }
}
/**
 * Whether a visit should open on setup: the machine has no configuration,
 * this dashboard can run setup, the reader came to the front page and has
 * not already chosen to skip it.
 */
export function opensOnSetup(setup: FirstRun, page: string, skippedBefore = hasSkipped()): boolean {
  return !setup.configured && setup.editable && page === "overview" && !skippedBefore;
}
