// Federated search: one question asked of every source in the viewing scope —
// this machine's words, its search by meaning, and the other machines — with
// the merged ranking arriving as each source answers.
//
// The server merges and ranks; this file reads its stream and applies the one
// rule the reader sees: a list settles, then freezes. Until the reader moves
// to a result (or a short settling time passes) a newer ranking replaces the
// list; after that, rows already shown keep their places and later results
// are added below, so nothing jumps under the pointer or the Enter key.

import { parseSearch } from "./api";
import type { Answer, Wire } from "./routes";
import type { Scope } from "./scope.svelte";
import { send } from "./transport";
import type { SearchHit } from "./types";

export type SourceName = "words" | "meaning" | "machines";

export interface SourceState {
  readonly source: SourceName;
  /** Who the source is: this machine's name, "by meaning", a machine, or "your other machines". */
  readonly label: string;
  readonly state: "searching" | "done" | "failed" | "skipped";
  readonly count?: number;
  readonly message?: string;
  readonly timedOut?: boolean;
  readonly reason?: string;
}

export interface FederatedHit extends SearchHit {
  /** Found only by meaning: its words do not appear in the chat. */
  readonly byMeaning: boolean;
}

export interface FederatedUpdate {
  readonly hits: readonly FederatedHit[];
  readonly sources: readonly SourceState[];
  /** No source is still searching. */
  readonly done: boolean;
}

export interface FederatedParams {
  readonly query: string;
  readonly scope: Scope;
  /** While typing: fast word search and short time limits. */
  readonly live: boolean;
  readonly limit: number;
  /** Leave out search by meaning, as while the reader is still typing. */
  readonly wordsOnly?: boolean;
  readonly agents?: readonly string[];
  readonly projects?: readonly string[];
  readonly days?: number | null;
  /** Also search subagent chats, which are left out otherwise. */
  readonly includeSubagents?: boolean;
}

/** The scope as the stream takes it. */
export function scopeParam(scope: Scope): string {
  if (scope.kind === "node") return `node:${scope.node}`;
  return scope.kind;
}

function streamParams(params: FederatedParams): URLSearchParams {
  const query = new URLSearchParams({
    q: params.query,
    scope: scopeParam(params.scope),
    limit: String(params.limit),
  });
  if (params.live) query.set("live", "true");
  if (params.wordsOnly) query.set("words_only", "true");
  if (params.agents?.length) query.set("provider", params.agents.join(","));
  if (params.projects?.length) query.set("project", params.projects.join(","));
  if (params.days !== null && params.days !== undefined) query.set("days", String(params.days));
  if (params.includeSubagents) query.set("include_subagents", "true");
  return query;
}

type Line = Wire["Update"];
type Row = Answer<"/api/search">["results"][number];

/** One line of the stream as the page shows it. */
export function parseUpdate(line: Line, localNode: string): FederatedUpdate {
  const hits = line.results.flatMap((value) => {
    const row = value as Row & { matched_by?: unknown };
    const [hit] = parseSearch(
      { ...emptyAnswer, results: [row] as Answer<"/api/search">["results"] },
      localNode,
    ).hits;
    if (!hit) return [];
    const by = Array.isArray(row.matched_by) ? row.matched_by : [];
    return [{ ...hit, byMeaning: by.length === 1 && by[0] === "meaning" }];
  });
  const sources = line.sources.map(
    (wire): SourceState => ({
      source: wire.source,
      label: wire.label,
      state: wire.state,
      ...(wire.state === "done" ? { count: wire.count } : {}),
      ...(wire.state === "failed" ? { message: wire.message, timedOut: wire.timed_out } : {}),
      ...(wire.state === "skipped" ? { reason: wire.reason } : {}),
    }),
  );
  return { hits, sources, done: line.done };
}

const emptyAnswer = {
  query: "",
  realized_mode: "lexical",
  unit: "chats",
  results: [],
  shared: false,
} as unknown as Answer<"/api/search">;

/**
 * Search every source, calling `onUpdate` with the merged ranking after each
 * one answers. Resolves when the search is done; aborting `signal` stops it,
 * and the server stops the sources still running.
 */
export async function streamSearch(
  params: FederatedParams,
  localNode: string,
  signal: AbortSignal,
  onUpdate: (update: FederatedUpdate) => void,
): Promise<void> {
  const response = await send(`/api/search/stream?${streamParams(params)}`, {
    signal,
    cache: "no-store",
  }).catch((error: unknown) => {
    if (signal.aborted) throw error;
    throw new Error(
      "Could not reach Mash. Check your connection and that the dashboard server is running, then retry.",
      { cause: error },
    );
  });
  if (!response.ok) {
    let message = "Search failed. Try again in a moment.";
    try {
      message = ((await response.json()) as { error: { message: string } }).error.message;
    } catch {
      // Keep the general message.
    }
    throw new Error(message);
  }
  const deliver = (text: string) => {
    if (!text.trim()) return;
    const value = JSON.parse(text) as Line | { error: { message: string } };
    if ("error" in value) throw new Error(value.error.message);
    onUpdate(parseUpdate(value, localNode));
  };
  if (!response.body) {
    for (const line of (await response.text()).split("\n")) deliver(line);
    return;
  }
  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
  let pending = "";
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    pending += value;
    const lines = pending.split("\n");
    pending = lines.pop() ?? "";
    for (const line of lines) deliver(line);
  }
  deliver(pending);
}

/**
 * The rows to show after a newer ranking arrives. Before the list freezes the
 * newer ranking is shown as it is; after, rows already shown keep their places
 * (with their newest details) and rows not yet shown are added below, in rank
 * order.
 */
export function settle<T>(
  shown: readonly T[],
  ranked: readonly T[],
  frozen: boolean,
  key: (row: T) => string,
): T[] {
  if (!frozen) return [...ranked];
  const fresh = new Map(ranked.map((row) => [key(row), row]));
  const seen = new Set(shown.map(key));
  return [
    ...shown.map((row) => fresh.get(key(row)) ?? row),
    ...ranked.filter((row) => !seen.has(key(row))),
  ];
}

export const hitKey = (hit: SearchHit): string => `${hit.chat.machine}\u0000${hit.chat.id}`;

/** How long a list may still re-rank after its first results show. */
export const SETTLE_MS = 500;

/** One line saying what is still coming or what failed; empty when nothing is. */
export function statusLine(
  sources: readonly SourceState[],
  shown: number,
  done: boolean,
): { text: string; retry: boolean } {
  const searching = sources.filter((source) => source.state === "searching");
  const failed = sources.filter((source) => source.state === "failed");
  const parts: string[] = [];
  if (!done && shown) parts.push(`${shown} result${shown === 1 ? "" : "s"} so far`);
  if (searching.length)
    parts.push(`Searching ${searching.map((source) => source.label).join(", ")}…`);
  for (const source of failed)
    parts.push(
      source.timedOut
        ? `${source.label} didn't answer`
        : `${source.label}: ${source.message ?? "search failed"}`,
    );
  return { text: parts.join(" · "), retry: failed.length > 0 };
}
