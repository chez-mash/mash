/**
 * Reading what `mash memory` answers: the decisions, preferences and facts
 * Mash remembers, each at its current revision, with the quotes that support
 * it. Plain data in and out, so it is tested without Raycast.
 */
import { MashError, rendered } from "./mash";

export type MemoryKind = "decision" | "preference" | "fact";

/** One remembered record at its current revision. */
export interface Memory {
  readonly id: string;
  readonly kind: MemoryKind;
  readonly statement: string;
  readonly rationale: string;
  /** When a preference applies, in the words it was recorded with. */
  readonly conditions: readonly string[];
  /** Where the record applies, short enough for a section title. */
  readonly scope: string;
  /** Not yet confirmed by the person: proposed by an agent from a chat. */
  readonly candidate: boolean;
  readonly recordedAt: Date | undefined;
  readonly evidence: readonly Evidence[];
}

/** A quote from a chat that supports a record. */
export interface Evidence {
  readonly chatId: string;
  readonly messageId: string;
  readonly quote: string;
  /** Who said it: `user` or `assistant`. */
  readonly role: string;
}

/** The most records one page holds; Mash lists at most 100 at a time. */
export const RECORD_PAGE = 100;

/** The arguments that list one page of current records, newest first. */
export function recordsArguments(offset: number): string[] {
  return ["memory", "--limit", String(RECORD_PAGE), "--offset", String(offset)];
}

const text = (value: unknown): string => (typeof value === "string" ? value : "");
const object = (value: unknown): Record<string, unknown> =>
  typeof value === "object" && value !== null ? (value as Record<string, unknown>) : {};

/** One page of `mash memory`: its records and where the next page starts. */
export function parseRecords(stdout: string): { memories: Memory[]; next: number | null } {
  let body: Record<string, unknown>;
  try {
    body = object(JSON.parse(stdout));
  } catch {
    throw new MashError("Mash gave an answer this extension could not read. Is Mash up to date?");
  }
  const error = object(body.error);
  if (body.error) throw new MashError(text(error.message) || "Mash couldn’t list what it remembers.");
  const records = Array.isArray(body.records) ? body.records : [];
  return {
    memories: records.flatMap((record) => memory(object(record))),
    next: typeof body.next_offset === "number" ? body.next_offset : null,
  };
}

function memory(record: Record<string, unknown>): Memory[] {
  const revisions = Array.isArray(record.revisions) ? record.revisions.map(object) : [];
  const current = revisions.find((revision) => revision.revision === record.current_revision) ?? revisions[0];
  if (!current) return [];
  const body = object(current.body);
  const details = object(body.details);
  const kind = text(details.kind);
  const statement = text(body.statement).trim();
  if (!statement || (kind !== "decision" && kind !== "preference" && kind !== "fact")) return [];
  return [
    {
      id: text(record.id),
      kind,
      statement,
      rationale: text(body.rationale).trim(),
      conditions: Array.isArray(details.conditions) ? details.conditions.map(text).filter(Boolean) : [],
      scope: scopeLabel(object(record.scope)),
      candidate: text(body.review) === "candidate",
      recordedAt: typeof current.recorded_at === "number" ? new Date(current.recorded_at) : undefined,
      evidence: (Array.isArray(current.evidence) ? current.evidence.map(object) : [])
        .map((entry) => ({
          chatId: text(entry.chat_id),
          messageId: text(entry.message_id),
          quote: text(entry.quote).trim(),
          role: text(entry.source_role),
        }))
        .filter((entry) => entry.chatId && entry.messageId),
    },
  ];
}

/**
 * Where a record applies, as a reader names it: a repository by its owner and
 * name, a project by its folder, and personal records as such.
 */
export function scopeLabel(scope: Record<string, unknown>): string {
  switch (scope.kind) {
    case "repository": {
      const identity = text(scope.identity).replace(/\.git$/, "");
      const parts = identity.split(/[/:]/).filter(Boolean);
      return parts.length >= 2 ? parts.slice(-2).join("/") : identity || "A repository";
    }
    case "local_project": {
      const path = text(scope.path).replace(/\/+$/, "");
      return path.split("/").pop() || "A project";
    }
    case "chat":
      return "One chat";
    default:
      return "Personal";
  }
}

/**
 * The record as Markdown for the detail pane: what, why, when it applies, and
 * the quotes behind it. Records are drawn from chats, which are untrusted, so
 * every part is rendered safely.
 */
export function memoryMarkdown(memory: Memory): string {
  const parts = [`## ${rendered(memory.statement).replace(/\n+/g, " ")}`];
  if (memory.rationale) parts.push(`**Why:** ${rendered(memory.rationale)}`);
  if (memory.conditions.length)
    parts.push(`**Applies when:**\n\n${memory.conditions.map((condition) => `- ${rendered(condition)}`).join("\n")}`);
  const quotes = memory.evidence
    .filter((entry) => entry.quote)
    .map((entry) => {
      const who = entry.role === "user" ? "you" : "the agent";
      return `${rendered(entry.quote)
        .split("\n")
        .map((line) => `> ${line}`)
        .join("\n")}\n>\n> — ${who}`;
    });
  if (quotes.length) parts.push(`**From the chat:**\n\n${quotes.join("\n\n")}`);
  return parts.join("\n\n");
}
