/**
 * The quick search panel the desktop app shows from the menu bar (#/quick):
 * what it lists, where its rows lead, and how it closes. It reuses the
 * dashboard's live search, chat list and links; this file only decides what
 * the panel does with them.
 */

import { chatHref, searchHref } from "./route.svelte";
import { shareLink } from "./shareLink";
import type { Status } from "./status";
import type { Agent, ChatSummary, SearchHit } from "./types";

/** How many rows the panel shows. */
export const QUICK_ROWS = 5;

/** One row: a chat, at the message that matched when there was a search. */
export interface QuickRow {
  readonly chat: ChatSummary;
  readonly messageId: string;
  readonly passage: string;
}

/** What a query needs before the panel searches instead of listing recent chats. */
export function searching(query: string): boolean {
  return query.trim().length >= 2;
}

/** The rows for a query: recent chats until the reader has typed, then matches. */
export function quickRows(
  query: string,
  recent: readonly ChatSummary[],
  found: readonly SearchHit[],
): QuickRow[] {
  if (!searching(query))
    return recent.slice(0, QUICK_ROWS).map((chat) => ({ chat, messageId: "", passage: "" }));
  return found
    .slice(0, QUICK_ROWS)
    .map((hit) => ({ chat: hit.chat, messageId: hit.messageId, passage: hit.before }));
}

/** The selection after moving `step` rows, staying on the list. */
export function moveSelection(current: number, step: number, rows: number): number {
  if (rows === 0) return 0;
  return Math.min(rows - 1, Math.max(0, current + step));
}

/**
 * Where a row leads. In the desktop app that is a mash:// link, which the app
 * opens in its main window and then closes the panel; in a browser it is the
 * dashboard's own chat page.
 */
export function rowTarget(row: QuickRow, desktop: boolean): string {
  const route = chatHref(row.chat.id, row.chat.machine, row.messageId);
  return desktop ? shareLink(route, "", true) : route;
}

/** Mash's full search for the same words: other machines, identifiers, meaning. */
export function fullSearchTarget(query: string, desktop: boolean): string {
  const route = searchHref(query.trim() ? { q: query.trim() } : {});
  return desktop ? shareLink(route, "", true) : route;
}

/** Pages of the main window the panel's menu opens. */
export function pageTarget(route: "#/" | "#/settings", desktop: boolean): string {
  return desktop ? shareLink(route, "", true) : route;
}

/**
 * The footer's account of search on this machine: a dot and a few words.
 * `ready` is green, `busy` amber (search works partly, or will soon), and
 * `problem` red (the panel cannot search); `checking` has no colour yet.
 */
export interface QuickStatus {
  readonly tone: "ready" | "busy" | "problem" | "checking";
  readonly label: string;
}

/**
 * What the footer says, from `mash status`. It claims "Up to date" only when
 * search is ready and status names nothing left to do, and "Indexing…" only
 * when a watcher is running, since `index --watch` starts with a full pass.
 */
export function quickStatus(status: Status | undefined, failed: boolean): QuickStatus {
  if (failed) return { tone: "problem", label: "Search unavailable" };
  if (!status) return { tone: "checking", label: "Checking…" };
  if (!status.configuration.loaded) return { tone: "busy", label: "Not set up" };
  const chats = status.chats === 1 ? "1 chat" : `${status.chats.toLocaleString()} chats`;
  if (status.ready)
    return status.nextStep === null
      ? { tone: "ready", label: `Up to date · ${chats}` }
      : { tone: "busy", label: `Word search only · ${chats}` };
  if (status.nextStep === "index")
    return status.watching
      ? { tone: "busy", label: "Indexing…" }
      : { tone: "busy", label: "Not indexed yet" };
  return { tone: "problem", label: "Search unavailable" };
}

/**
 * Agents whose logo is drawn in ink. In the panel they take the text colour,
 * so they show in the system's dark mode as well as its light one; the rest
 * keep their own colour.
 */
const inkLogos: ReadonlySet<Agent> = new Set(["codex", "cursor", "opencode"]);

export function inkLogo(agent: Agent): boolean {
  return inkLogos.has(agent);
}

/**
 * Hide the panel. The app lets the dashboard ask for exactly this in the
 * panel's window (desktop/src-tauri/capabilities/quick.json); elsewhere there
 * is no panel to hide.
 */
export function hidePanel(scope: object = globalThis): void {
  const internals = (scope as { __TAURI_INTERNALS__?: { invoke?: Invoke } }).__TAURI_INTERNALS__;
  void internals?.invoke?.("plugin:window|hide", {}).catch(() => {});
}

type Invoke = (command: string, args: object) => Promise<unknown>;
