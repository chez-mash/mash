import type { ChatSummary, Subagent } from "./types";

/**
 * Subagent chats are chats an agent started for part of a task. Only the
 * agent's own marker makes a chat one (the server reads it from the agent's
 * files); nothing here guesses from titles.
 *
 * Lists fold a subagent into the chat that started it when that chat is in
 * the catalogue: same machine, same agent, the exact identity the agent
 * named. A subagent whose parent is not here stays a row of its own, so
 * nothing disappears and nothing is counted twice.
 */

function key(machine: string, agent: string, id: string): string {
  return `${machine}\u0000${agent}\u0000${id}`;
}

/** Whether `chat` was started by `parent`. */
function startedBy(chat: ChatSummary, parent: ChatSummary): boolean {
  return (
    chat.subagent?.parentId === parent.id &&
    chat.machine === parent.machine &&
    chat.agent === parent.agent
  );
}

export interface Folded {
  /** The chats a list shows: every chat except subagents folded into a parent. */
  readonly chats: readonly ChatSummary[];
  /** Every chat, parents with their counts, for lookups and the unfolded list. */
  readonly all: readonly ChatSummary[];
  /** How many subagent chats were folded into their parents. */
  readonly folded: number;
}

/** Folds subagents into their parents and gives each parent its count. */
export function fold(all: readonly ChatSummary[]): Folded {
  const present = new Set(all.map((chat) => key(chat.machine, chat.agent, chat.id)));
  const counts = new Map<string, number>();
  const folds = (chat: ChatSummary): boolean => {
    const parent = chat.subagent?.parentId;
    return parent !== undefined && present.has(key(chat.machine, chat.agent, parent));
  };
  for (const chat of all) {
    const parent = chat.subagent?.parentId;
    if (parent === undefined || !folds(chat)) continue;
    const at = key(chat.machine, chat.agent, parent);
    counts.set(at, (counts.get(at) ?? 0) + 1);
  }
  const withCounts = all.map((chat) => {
    const count = counts.get(key(chat.machine, chat.agent, chat.id)) ?? 0;
    return count ? { ...chat, subagents: count } : chat;
  });
  const chats = withCounts.filter((chat) => !folds(chat));
  return { chats, all: withCounts, folded: all.length - chats.length };
}

/** The subagents `parent` started, oldest first, as its page lists them. */
export function childrenOf(
  all: readonly ChatSummary[],
  parent: ChatSummary,
): readonly ChatSummary[] {
  return all
    .filter((chat) => startedBy(chat, parent))
    .sort((a, b) => (a.createdMs ?? a.updatedMs ?? 0) - (b.createdMs ?? b.updatedMs ?? 0));
}

/** The parent `chat` names, when it is in the catalogue. */
export function parentOf(all: readonly ChatSummary[], chat: ChatSummary): ChatSummary | undefined {
  if (!chat.subagent?.parentId) return undefined;
  return all.find((candidate) => startedBy(chat, candidate));
}

/** "done", "running" or nothing: only what the agent recorded. */
export function stateLabel(subagent: Subagent | undefined): string | undefined {
  if (subagent?.finished === true) return "done";
  if (subagent?.finished === false) return "running";
  return undefined;
}

/** `general-purpose · 58m · done`: the role, age and state the agent recorded. */
export function detailLabel(chat: ChatSummary, age: string): string {
  return [chat.subagent?.role, age, stateLabel(chat.subagent)].filter(Boolean).join(" · ");
}

/** What a list says about where a subagent came from. */
export function originLabel(chat: ChatSummary, parentTitle: string | undefined): string {
  if (!chat.subagent?.parentId) return "subagent";
  const title = parentTitle ?? chat.subagent.parentTitle;
  return title ? `subagent of ${title}` : "subagent of a chat not on this machine";
}

/** The subagent a spawning tool call started, when the link is exact. */
export function spawnedBy(
  children: readonly ChatSummary[],
  callId: string | undefined,
): ChatSummary | undefined {
  if (!callId) return undefined;
  return children.find((chat) => chat.subagent?.spawnCallId === callId);
}

/** ` · 11 subagent chats folded in`, after a count of listed chats. */
export function foldedNote(folded: number): string {
  if (!folded) return "";
  return ` · ${folded.toLocaleString()} subagent chat${folded === 1 ? "" : "s"} folded in`;
}
