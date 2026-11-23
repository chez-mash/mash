import type { Answer } from "../lib/routes";
import { chats, localMachine } from "./history";
import { records } from "./records";
import { terms } from "./search";

// The demo's memories as the dashboard's record routes answer them.

const DAY = 86_400_000;
type Entry = (typeof records)[number];

/** Where a memory was saved: the machine of the chat it quotes. */
function savedOn(entry: Entry): string {
  return chats.find((chat) => chat.id === entry.source.chat)?.machine ?? localMachine;
}
/** A memory as the dashboard reads it. Like Mash, a record read from this
 * machine's own index has no node; one read from the shared server names it. */
export function record(entry: Entry, now: number, shared: boolean): Answer<"/api/record"> {
  const details =
    entry.kind === "decision"
      ? {
          kind: entry.kind,
          state: "accepted" as const,
          alternatives: [...(entry.alternatives ?? [])],
          consequences: [...(entry.consequences ?? [])],
        }
      : entry.kind === "preference"
        ? { kind: entry.kind, state: "active" as const, conditions: [] }
        : { kind: entry.kind, state: "current" as const, reported_by: "Sam", valid_until: null };
  return {
    id: entry.id,
    owner: "sam",
    ...(shared ? { node_id: savedOn(entry) } : {}),
    scope: entry.project ? { kind: "repository", identity: entry.project } : { kind: "personal" },
    current_revision: 1,
    conflicted: false,
    revisions: [
      {
        revision: 1,
        recorded_at: Math.round(now - entry.daysAgo * DAY),
        evidence_current: true,
        confirmations: 0,
        confirmation_history: [],
        body: {
          statement: entry.statement,
          rationale: entry.rationale,
          recorded_by: entry.recordedBy,
          effective_at: null,
          review: entry.review,
          details,
        },
        evidence: [
          {
            chat_id: entry.source.chat,
            message_id: `${entry.source.chat}-${entry.source.message}`,
            fingerprint: `demo-${entry.id}`,
            quote: entry.source.quote,
            purpose: "endorsement",
            source_role: "user",
          },
        ],
      },
    ],
    links: [],
    next_before_revision: null,
    next_link_offset: null,
  };
}
export function recordList(params: URLSearchParams, now: number): Answer<"/api/records"> {
  const wanted = terms(params.get("q") ?? "");
  const kind = params.get("kind");
  const review = params.get("review");
  const chat = params.get("chat");
  const state = params.get("state");
  const shared = params.get("shared") === "true";
  const node = params.get("node");
  const current = ["accepted", "active", "current"];
  // This machine's own index holds only the memories saved on it; the shared
  // server holds every machine's, or one machine's when a node is named.
  const from = (entry: Entry) =>
    shared ? !node || savedOn(entry) === node : savedOn(entry) === localMachine;
  return {
    records: records
      .filter(
        (entry) =>
          from(entry) &&
          (!kind || entry.kind === kind) &&
          (!review || entry.review === review) &&
          (!chat || entry.source.chat === chat) &&
          (!state || current.includes(state)) &&
          wanted.every((term) =>
            `${entry.statement} ${entry.rationale}`.toLowerCase().includes(term),
          ),
      )
      .map((entry) => record(entry, now, shared)),
    next_offset: null,
  };
}
