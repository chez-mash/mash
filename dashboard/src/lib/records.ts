import { read } from "./api";
import { stableRead } from "./queries";
import type { Answer, Wire } from "./routes";

export const kinds = ["decision", "preference", "fact"] as const;
export const states = [
  "proposed",
  "accepted",
  "active",
  "current",
  "superseded",
  "withdrawn",
  "retracted",
] as const;
export type Kind = (typeof kinds)[number];
export type Lifecycle = (typeof states)[number];
export type Scope =
  | { kind: "personal" }
  | { kind: "repository"; identity: string }
  | { kind: "local_project"; path: string }
  | { kind: "chat"; id: string };
export interface Evidence {
  chatId: string;
  messageId: string;
  quote: string;
  purpose: "context" | "endorsement" | "report" | null;
  sourceRole: string | null;
}
export type Details =
  | {
      kind: "decision";
      state: "proposed" | "accepted" | "superseded" | "withdrawn";
      /** Who made the choice: the person, or the agent of its own accord. */
      decidedBy: "user" | "agent";
      alternatives: readonly string[];
      consequences: readonly string[];
    }
  | {
      kind: "preference";
      state: "active" | "superseded" | "withdrawn";
      conditions: readonly string[];
    }
  | {
      kind: "fact";
      state: "current" | "superseded" | "retracted";
      reportedBy: string;
      validUntil: number | null;
    };
export interface Revision {
  revision: number;
  statement: string;
  rationale: string;
  review: "candidate" | "established";
  recordedBy: string;
  recordedAt: number;
  effectiveAt: number | null;
  details: Details;
  evidence: readonly Evidence[];
  evidenceCurrent: boolean;
  confirmations: number;
}
export interface NativeRecord {
  id: string;
  owner: string;
  node: string;
  scope: Scope;
  currentRevision: number;
  revisions: readonly Revision[];
  conflicted: boolean;
  links: readonly {
    sourceId: string;
    targetId: string;
    relation: "supersedes" | "supports" | "contradicts";
  }[];
  nextBeforeRevision: number | null;
  nextLinkOffset: number | null;
}
export function timingLabel(revision: Revision, now: number): string | null {
  if (revision.effectiveAt !== null && revision.effectiveAt > now) return "Not yet effective";
  if (
    revision.details.kind === "fact" &&
    revision.details.validUntil !== null &&
    revision.details.validUntil < now
  )
    return "Expired";
  return null;
}
export interface RecordPage {
  records: readonly NativeRecord[];
  nextOffset: number | null;
  nextCursor: { node: string; offset: number } | null;
}
function invalid(): never {
  throw new Error("Mash returned invalid record data. Reload after updating the dashboard.");
}
function details(row: Wire["Details"]): Details {
  switch (row.kind) {
    case "decision":
      return {
        kind: row.kind,
        state: row.state,
        decidedBy: row.decided_by ?? "user",
        alternatives: row.alternatives,
        consequences: row.consequences,
      };
    case "preference":
      return { kind: row.kind, state: row.state, conditions: row.conditions };
    case "fact":
      return {
        kind: row.kind,
        state: row.state,
        reportedBy: row.reported_by,
        validUntil: row.valid_until ?? null,
      };
  }
}
function revision(row: Wire["RecordRevision"]): Revision {
  return {
    revision: row.revision,
    statement: row.body.statement,
    rationale: row.body.rationale,
    review: row.body.review,
    recordedBy: row.body.recorded_by,
    recordedAt: row.recorded_at,
    effectiveAt: row.body.effective_at ?? null,
    details: details(row.body.details),
    evidenceCurrent: row.evidence_current,
    confirmations: row.confirmations,
    evidence: row.evidence.map((source) => ({
      chatId: source.chat_id,
      messageId: source.message_id,
      quote: source.quote,
      purpose: source.purpose ?? null,
      sourceRole: source.source_role ?? null,
    })),
  };
}
export function parseRecord(row: Wire["Record"]): NativeRecord {
  return {
    id: row.id,
    owner: row.owner,
    node: row.node_id ?? "",
    scope: row.scope,
    currentRevision: row.current_revision,
    revisions: row.revisions.map(revision),
    conflicted: row.conflicted,
    nextBeforeRevision: row.next_before_revision ?? null,
    nextLinkOffset: row.next_link_offset ?? null,
    links: row.links.map((link) => ({
      sourceId: link.source_id,
      targetId: link.target_id,
      relation: link.relation,
    })),
  };
}
/** A page whose record lacks the revision it calls current cannot be shown truthfully. */
export function parseRecordPage(row: Answer<"/api/records">): RecordPage {
  const records = row.records.map(parseRecord);
  if (
    records.some(
      (record) =>
        !record.revisions.some((revision) => revision.revision === record.currentRevision),
    )
  )
    return invalid();
  return {
    records,
    nextOffset: row.next_offset ?? null,
    nextCursor: row.next_cursor
      ? { node: row.next_cursor.node_id, offset: row.next_cursor.offset }
      : null,
  };
}
export async function recordPage(params: string, signal: AbortSignal): Promise<RecordPage> {
  return parseRecordPage(await read("/api/records", new URLSearchParams(params), signal));
}
export async function getRecord(
  id: string,
  node: string,
  before: number | null,
  linkOffset: number,
  signal: AbortSignal,
): Promise<NativeRecord> {
  const params = new URLSearchParams({ id, link_offset: String(linkOffset) });
  if (node) params.set("node", node);
  if (before !== null) params.set("before_revision", String(before));
  const record = parseRecord(await read("/api/record", params, signal));
  if (
    before === null &&
    !record.revisions.some((revision) => revision.revision === record.currentRevision)
  )
    return invalid();
  return record;
}
export function label(value: string): string {
  return value.replaceAll("_", " ").replace(/^./, (letter) => letter.toUpperCase());
}
/** A scope's short name for a chip: a repository as owner/name, a folder as
 * its last part. `scopeLabel` gives the full identity. */
export function scopeName(value: Scope): string {
  if (value.kind === "repository")
    return (
      value.identity.replace(/^[a-z+]+:\/\/[^/]+\//, "").replace(/\.git$/, "") || value.identity
    );
  if (value.kind === "local_project")
    return value.path.split(/[\\/]/).filter(Boolean).pop() ?? value.path;
  return scopeLabel(value);
}
export function scopeLabel(value: Scope): string {
  switch (value.kind) {
    case "personal":
      return "Personal";
    case "repository":
      return value.identity;
    case "local_project":
      return value.path;
    case "chat":
      return `Chat ${value.id}`;
  }
}

/** How settled a revision is, in the words lists and pages show. */
export function standing(revision: { review: string; details: Details }): string {
  if (revision.review === "candidate") return "unconfirmed";
  return revision.details.kind === "decision" && revision.details.decidedBy === "agent"
    ? "agent decided"
    : "established";
}

/** The fill of the pill that names a kind of memory (see .pill): a decision
 * is butter, a fact mint, a preference plain paper. */
export function kindPill(kind: Kind): "decision" | "ok" | "" {
  return kind === "decision" ? "decision" : kind === "fact" ? "ok" : "";
}
/** The fill of a pill for a lifecycle state: in force is mint, replaced or
 * taken back is red, proposed is plain. */
export function statePill(state: Lifecycle): "ok" | "bad" | "" {
  if (state === "accepted" || state === "active" || state === "current") return "ok";
  if (state === "proposed") return "";
  return "bad";
}
/** The fill of a pill for how settled a memory is: unconfirmed needs attention. */
export function standingPill(value: string): "warn" | "" {
  return value === "unconfirmed" ? "warn" : "";
}

/** One page of saved decisions, newest first; shared with the memory page's
 * own list of decisions, so the two are read once. */
export const decisionOptions = {
  ...stableRead,
  queryKey: ["records", "kind=decision"] as const,
  queryFn: ({ signal }: { signal: AbortSignal }) => recordPage("kind=decision", signal),
};
/**
 * How many decisions each chat produced, by chat id: a decision counts for
 * every chat its current evidence quotes, once per chat. Withdrawn decisions
 * no longer count. Only the records given are counted, so a count is a lower
 * bound when the page was not the last.
 */
export function decisionsByChat(records: readonly NativeRecord[]): ReadonlyMap<string, number> {
  const counts = new Map<string, number>();
  for (const record of records) {
    const current = record.revisions.find(
      (revision) => revision.revision === record.currentRevision,
    );
    if (current?.details.kind !== "decision" || current.details.state === "withdrawn") continue;
    for (const chat of new Set(current.evidence.map((source) => source.chatId)))
      counts.set(chat, (counts.get(chat) ?? 0) + 1);
  }
  return counts;
}
