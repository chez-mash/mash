// What the memory pages show about a record beyond the record itself: its
// current revision, the week it falls in, and the chat its evidence quotes.
import type { NativeRecord, Revision } from "./records";
import type { ChatSummary } from "./types";

/** The revision a record calls current, or its first when that is missing. */
export function currentOf(record: NativeRecord): Revision | undefined {
  return (
    record.revisions.find((revision) => revision.revision === record.currentRevision) ??
    record.revisions[0]
  );
}

/** One record's key in a list: its id alone repeats across machines and owners. */
export function recordKey(record: NativeRecord): string {
  return JSON.stringify([record.node, record.owner, record.id]);
}

/** Midnight at the start of the reader's week (Monday), in milliseconds. */
export function weekStart(now: number): number {
  const day = new Date(now);
  day.setHours(0, 0, 0, 0);
  const sinceMonday = (day.getDay() + 6) % 7;
  day.setDate(day.getDate() - sinceMonday);
  return day.getTime();
}

export interface RecordGroup {
  readonly label: "This week" | "Earlier";
  readonly records: readonly NativeRecord[];
}

/**
 * The records saved this calendar week, then the rest, each in the order the
 * page gave them. A group with nothing in it is left out.
 */
export function groupByWeek(records: readonly NativeRecord[], now: number): RecordGroup[] {
  const start = weekStart(now);
  const recent = records.filter((record) => (currentOf(record)?.recordedAt ?? 0) >= start);
  const earlier = records.filter((record) => (currentOf(record)?.recordedAt ?? 0) < start);
  const groups: RecordGroup[] = [];
  if (recent.length) groups.push({ label: "This week", records: recent });
  if (earlier.length) groups.push({ label: "Earlier", records: earlier });
  return groups;
}

/** The chat a quote comes from, when the catalogue lists it. */
export function chatOf(
  chats: readonly ChatSummary[] | undefined,
  id: string,
  node: string,
): ChatSummary | undefined {
  const matches = (chats ?? []).filter((chat) => chat.id === id);
  return matches.find((chat) => !node || chat.machine === node) ?? matches[0];
}

/** Who a quote is from, in a caption: the reader, or the role that said it. */
export function speaker(role: string | null): string {
  if (role === "user") return "You";
  if (!role) return "Someone";
  return role.replace(/^./, (letter) => letter.toUpperCase());
}
