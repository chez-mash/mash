// What this machine shares with its shared server: per-agent counts for the
// review, and how far the server has received what it was sent. Counts come
// from this machine's own catalogue; progress only from what the server says.

import type { Machine } from "./shared";
import type { Agent, ChatSummary } from "./types";

export interface AgentShare {
  readonly agent: Agent;
  /** Chats of this agent indexed on this machine. */
  readonly chats: number;
  readonly shared: boolean;
}

/** One row per agent this machine indexes, with how many of its chats it holds. */
export function agentShares(
  chats: readonly ChatSummary[],
  localNode: string,
  indexed: readonly Agent[],
  unshared: readonly Agent[],
): AgentShare[] {
  const counts = new Map<Agent, number>(indexed.map((agent) => [agent, 0]));
  for (const chat of chats) {
    const count = counts.get(chat.agent);
    if (chat.machine === localNode && count !== undefined) counts.set(chat.agent, count + 1);
  }
  return [...counts].map(([agent, count]) => ({
    agent,
    chats: count,
    shared: !unshared.includes(agent),
  }));
}

export interface ShareSummary {
  readonly shared: number;
  readonly kept: number;
  readonly sharedAgents: readonly Agent[];
  readonly keptAgents: readonly Agent[];
}

export function shareSummary(rows: readonly AgentShare[]): ShareSummary {
  const shared = rows.filter((row) => row.shared);
  const kept = rows.filter((row) => !row.shared);
  const total = (list: readonly AgentShare[]) => list.reduce((sum, row) => sum + row.chats, 0);
  return {
    shared: total(shared),
    kept: total(kept),
    sharedAgents: shared.map((row) => row.agent),
    keptAgents: kept.map((row) => row.agent),
  };
}

/** The agents to keep off the server, in the order the settings name them. */
export function unsharedOf(rows: readonly AgentShare[]): Agent[] {
  return rows.filter((row) => !row.shared).map((row) => row.agent);
}

export type Progress =
  | { readonly state: "waiting" }
  | { readonly state: "receiving"; readonly sent: number; readonly total: number }
  | { readonly state: "published"; readonly chats: number; readonly at: string | null };

/**
 * How far the server has this machine's chats, by the server's own report:
 * a first publication still arriving, what it holds, or nothing yet. Nothing
 * here is estimated from this machine's side.
 */
export function publishProgress(machine: Machine | undefined): Progress {
  if (machine?.receiving)
    return {
      state: "receiving",
      sent: machine.receiving.covered,
      total: machine.receiving.total,
    };
  if (machine?.published)
    return {
      state: "published",
      chats: machine.published.chats,
      at: machine.published.last_synced_at ?? null,
    };
  return { state: "waiting" };
}
