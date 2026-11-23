import { attachments, type Source } from "./attachments";
import type { Answer, ReadPath, Wire } from "./routes";
import { liveMachinesSeen } from "./scope.svelte";
import type { Applied } from "./search";
import { check } from "./session.svelte";
import { withoutCopies } from "./sshCopies";
import { fold } from "./subagents";
import { callsBefore, resultsAfter, unpaired } from "./toolcalls";
import { send } from "./transport";
import type { Agent, ChatSummary, Message, SearchFilters, SearchHit, Subagent } from "./types";

export interface Catalog {
  /** The chats lists show, with subagents folded into their parents. */
  readonly chats: readonly ChatSummary[];
  /** Every chat, subagents included, for lookups and the unfolded list. */
  readonly all: readonly ChatSummary[];
  /** How many subagent chats `chats` folds into their parents. */
  readonly folded: number;
  readonly localNode: string;
  /** The dashboard reads only the shared index: no machine's history is local. */
  readonly sharedOnly: boolean;
  /** This machine publishes to and reads a shared index. */
  readonly joined: boolean;
  readonly warnings: readonly string[];
}
export interface SearchPage {
  readonly hits: readonly SearchHit[];
  readonly intent?: Wire["Intent"];
  readonly nextOffset: number | null;
  readonly warnings: readonly string[];
}
export interface MessagePage {
  readonly messages: readonly Message[];
  /** Messages in the whole chat, when the server reports it. */
  readonly total?: number;
  /** Tool calls just before this page whose output opens it. They are shown
   * with that output and are not otherwise part of the page. */
  readonly before?: readonly Message[];
  /** Tool output just after this page for calls that end it. */
  readonly after?: readonly Message[];
  /** How many messages the server sent, including ones with nothing to show. */
  readonly rows?: number;
}

export function isAgent(value: string): value is Agent {
  return ["claude-code", "codex", "opencode", "copilot", "cursor", "antigravity"].includes(value);
}
function date(value: number | undefined): string {
  return value === undefined ? "Time unknown" : new Date(value).toLocaleString();
}
function subagent(row: Wire["Chat"] | Wire["ChatHit"]): Subagent | undefined {
  const marker = row.subagent;
  if (!marker) return undefined;
  return {
    ...(marker.parent_chat_id ? { parentId: marker.parent_chat_id } : {}),
    ...(marker.parent_title ? { parentTitle: marker.parent_title } : {}),
    ...(marker.label ? { label: marker.label } : {}),
    ...(marker.role ? { role: marker.role } : {}),
    ...(typeof marker.finished === "boolean" ? { finished: marker.finished } : {}),
    ...(marker.spawn_call_id ? { spawnCallId: marker.spawn_call_id } : {}),
  };
}
function chat(row: Wire["Chat"] | Wire["ChatHit"], localNode: string): ChatSummary {
  const updatedMs = row.updated_at ?? row.created_at ?? undefined;
  const createdMs = row.created_at ?? undefined;
  const marker = subagent(row);
  return {
    id: row.chat_id,
    // A subagent is named for its task rather than for the brief it was given.
    title: marker?.label || row.title || "Untitled chat",
    agent: row.provider,
    machine: row.node_id ?? localNode,
    project: row.project ?? "",
    updatedAt: date(updatedMs),
    ...(updatedMs === undefined ? {} : { updatedMs }),
    ...(createdMs === undefined ? {} : { createdMs }),
    ...(marker ? { subagent: marker } : {}),
    ...(row.ssh_copy ? { sshCopy: true as const } : {}),
  };
}
/**
 * Reads a route of this dashboard or, with `peer`, the same route of another
 * machine's own dashboard through this one (which adds that machine's token).
 */
export async function read<P extends ReadPath>(
  path: P,
  params: URLSearchParams,
  signal: AbortSignal,
  peer?: string,
): Promise<Answer<P>> {
  const target = peer ? `/api/peer/${encodeURIComponent(peer)}${path}` : path;
  const response = await send(`${target}?${params}`, {
    signal: AbortSignal.any([signal, AbortSignal.timeout(20_000)]),
    cache: "no-store",
  }).catch((error: unknown) => {
    if (signal.aborted) throw error;
    if (error instanceof Error && error.name === "TimeoutError")
      throw new Error("Mash took too long to respond. Try again in a moment.", { cause: error });
    throw new Error(
      "Could not reach Mash. Check your connection and that the dashboard server is running, then retry.",
      { cause: error },
    );
  });
  if (!response.ok) {
    // A session can end while the page is open; ask again so the sign-in shows.
    if (response.status === 401) void check();
    let detail = "Check that mash dash and the requested history source are available.";
    try {
      const failure = (await response.json()) as Wire["Failure"];
      detail = failure.error.message;
    } catch {
      /* Non-JSON HTTP failures keep the safe fallback. */
    }
    throw new Error(`Mash request failed (${response.status}). ${detail}`);
  }
  // The server generates the contract these types come from, and the dashboard
  // is built into the same binary, so the two cannot be out of step.
  return (await response.json()) as Answer<P>;
}
export function parseCatalog(body: Answer<"/api/chats">): Catalog {
  const all = withoutCopies(body.chats.map((row) => chat(row, body.local_node)));
  return {
    localNode: body.local_node,
    sharedOnly: body.shared_only,
    // Another machine's chats in the list also show a shared index is read.
    joined:
      (body.joined ?? false) ||
      body.chats.some((row) => !!row.node_id && row.node_id !== body.local_node),
    ...fold(all),
    warnings: [
      ...(body.shared_error ? [`Shared history is unavailable: ${body.shared_error}`] : []),
      ...(body.truncated
        ? ["Showing the newest 10,000 chats. Counts and filter choices cover this catalogue only."]
        : []),
    ],
  };
}
export async function catalog(signal: AbortSignal): Promise<Catalog> {
  return parseCatalog(await read("/api/chats", new URLSearchParams(), signal));
}
/** Another machine's own chats, read live from its dashboard. Its catalogue
 * also lists what it reads from the shared index; only its own are kept. */
export async function peerCatalog(peer: string, signal: AbortSignal): Promise<Catalog> {
  const all = parseCatalog(await read("/api/chats", new URLSearchParams(), signal, peer));
  const own = (chat: ChatSummary) => chat.machine === all.localNode;
  return { ...all, chats: all.chats.filter(own), all: all.all.filter(own) };
}
/** The other machines whose own dashboards this one can read live. */
export async function liveMachines(signal: AbortSignal): Promise<readonly string[]> {
  const machines = (await read("/api/viewing", new URLSearchParams(), signal)).machines;
  return Array.isArray(machines) ? machines.filter((id) => typeof id === "string") : [];
}
/** Empty submissions filter one catalogue snapshot; drafts never alter submitted results. */
export function browse(
  catalog: Catalog,
  filters: SearchFilters,
  now: number,
  subagents: "folded" | "shown" = "folded",
): SearchPage & { readonly folded: number } {
  const kept = (chat: ChatSummary) =>
    (!filters.agents.length || filters.agents.includes(chat.agent)) &&
    (!filters.machines.length || filters.machines.includes(chat.machine)) &&
    (!filters.projects.length || filters.projects.includes(chat.project)) &&
    (filters.days === null ||
      (chat.updatedMs !== undefined && chat.updatedMs >= now - filters.days * 86_400_000));
  const listed = catalog.chats.filter(kept);
  const shown = subagents === "shown" ? catalog.all.filter(kept) : listed;
  return {
    hits: shown.map((chat) => ({ chat, messageId: "", before: "", match: "", after: "" })),
    folded: subagents === "shown" ? 0 : catalog.all.filter(kept).length - listed.length,
    nextOffset: null,
    warnings: catalog.warnings,
  };
}
/** With `peer`, the search runs on that machine's own dashboard and keeps only
 * its own chats; `applied.machines` then does not apply. */
export async function search(
  applied: Applied,
  localNode: string,
  signal: AbortSignal,
  peer?: string,
): Promise<SearchPage> {
  const params = new URLSearchParams({
    q: applied.query,
    provider: applied.agents.join(","),
    machine: applied.machines.join(","),
    project: applied.projects.join(","),
    offset: String(applied.offset),
  });
  if (applied.days !== null) params.set("days", String(applied.days));
  if (applied.rank !== "relevance") params.set("sort", applied.rank);
  if (applied.intent !== "default") params.set("intent", applied.intent);
  if (applied.subagentHits === "included") params.set("include_subagents", "true");
  if (peer) {
    params.delete("machine");
    return ownHits(parseSearch(await read("/api/search", params, signal, peer), peer), peer);
  }
  return parseSearch(await read("/api/search", params, signal), localNode);
}
function ownHits(page: SearchPage, peer: string): SearchPage {
  return { ...page, hits: page.hits.filter((hit) => hit.chat.machine === peer) };
}

/**
 * Chats on this machine whose words match a query the reader is still typing:
 * a few, fast, by words only. Submitting the box runs the full `search`.
 */
export async function liveSearch(
  query: string,
  localNode: string,
  signal: AbortSignal,
  peer?: string,
): Promise<readonly SearchHit[]> {
  const limit = 5;
  const params = new URLSearchParams({ q: query, limit: String(limit) });
  const page = parseSearch(await read("/api/search/live", params, signal, peer), peer ?? localNode);
  return (peer ? ownHits(page, peer) : page).hits.slice(0, limit);
}

export function parseSearch(
  body: Answer<"/api/search"> | Answer<"/api/search/live">,
  localNode: string,
): SearchPage {
  return {
    ...(body.intent === "exact-mention" || body.intent === "related-discussion"
      ? { intent: body.intent }
      : {}),
    // The dashboard asks for chats; a result without matches is a message hit.
    hits: body.results.flatMap((row) => {
      if (!("matches" in row)) return [];
      const match = row.matches[0];
      const lookup = match?.lookup?.view;
      const metadata = chat(row, localNode);
      const machine = lookup ? (lookup.shared ?? localNode) : (match?.node_id ?? metadata.machine);
      return [
        {
          chat: { ...metadata, machine },
          messageId: lookup ? lookup.id : (match?.message_id ?? ""),
          before: match?.snippet ?? "",
          match: "",
          after: "",
        },
      ];
    }),
    nextOffset: body.next_offset ?? null,
    warnings: [
      body.shared_error,
      body.shared_context_error,
      ...(Array.isArray(body.freshness_warnings) ? body.freshness_warnings : []),
      body.notice,
    ].filter((text): text is string => typeof text === "string" && text.length > 0),
  };
}
function hasSomethingToShow(row: Answer<"/api/chat">["messages"][number]): boolean {
  return row.role !== "assistant" || row.content.trim() !== "[Thinking]";
}

/** `source` is where the page was read, so its attachments are read there too. */
export function parseMessages(body: Answer<"/api/chat">, source: Source = {}): MessagePage {
  return {
    ...(typeof body.message_count === "number" ? { total: body.message_count } : {}),
    rows: body.messages.length,
    // Thinking recorded without its text is kept in the index, where later
    // messages name it as their parent, and has nothing to show.
    messages: body.messages.filter(hasSomethingToShow).map((row) => {
      const tools = row.tools ?? {};
      const calls = tools.tool_calls ?? [];
      const results = tools.tool_results ?? [];
      const presentation =
        results.length || row.role === "tool"
          ? "tool-result"
          : calls.length
            ? "tool-call"
            : row.role === "assistant" && row.content.startsWith("[Thinking]")
              ? "reasoning"
              : (row.role !== "user" && row.role !== "assistant") || row.dialogue === ""
                ? "context"
                : "dialogue";
      // Only a person's message carries what they typed apart from its context.
      const said =
        presentation === "dialogue" && row.role === "user" && row.dialogue !== row.content
          ? row.dialogue
          : undefined;
      return {
        id: row.id,
        ordinal: row.ordinal,
        role: row.role,
        presentation,
        ...(said ? { dialogue: said } : {}),
        toolLabel: calls.map((call) => call.name ?? "Tool").join(", ") || "Tool output",
        at: date(row.created_at ?? undefined),
        blocks: [{ type: "text", markdown: row.content }],
        tools: {
          calls: calls.map((call) => ({
            name: call.name ?? undefined,
            callId: call.call_id ?? undefined,
            input: call.input ?? undefined,
          })),
          results: results.map((result) => ({
            callId: result.call_id ?? undefined,
            isError: result.is_error ?? undefined,
          })),
        },
        // Opaque provider inputs remain text, never interpreted as commands or markup.
        activity: Object.keys(tools).length ? JSON.stringify(tools, null, 2) : "",
        ...(row.attachments?.length
          ? { attachments: attachments(row.id, row.attachments, source) }
          : {}),
      };
    }),
  };
}
export const PAGE_SIZE = 100;
/** The most messages the server gives in one answer. */
export const WHOLE_PAGE = 500;
export async function transcript(
  id: string,
  node: string,
  offset: number,
  signal: AbortSignal,
  shared = false,
  limit = PAGE_SIZE,
): Promise<MessagePage> {
  const page = { offset: String(offset), limit: String(limit) };
  return fromOwner(node, signal, async (peer) => {
    const source: Source = peer ? { peer } : { node, shared };
    const params = peer
      ? { id, ...page }
      : { id, ...(node ? { node } : {}), ...(shared ? { shared: "true" } : {}), ...page };
    return parseMessages(
      await read("/api/chat", new URLSearchParams(params), signal, peer),
      source,
    );
  });
}
/**
 * A chat of a machine read live is read on that machine's own dashboard, as
 * its own; if that dashboard stops answering, its published copy is read.
 */
export async function fromOwner<T>(
  node: string,
  signal: AbortSignal,
  reading: (peer: string | undefined) => Promise<T>,
): Promise<T> {
  if (!node || !liveMachinesSeen.has(node)) return reading(undefined);
  try {
    return await reading(node);
  } catch (error) {
    if (signal.aborted) throw error;
    return reading(undefined);
  }
}
export async function context(
  id: string,
  node: string,
  signal: AbortSignal,
  shared = false,
): Promise<MessagePage> {
  return fromOwner(node, signal, async (peer) => {
    const params = new URLSearchParams({ id, context: "25" });
    if (node && !peer) params.set("node", node);
    if (shared && !peer) params.set("shared", "true");
    const source: Source = peer ? { peer } : { node, shared };
    return parseMessages(await read("/api/view", params, signal, peer), source);
  });
}

/** How far past a page's edge to look for the other half of a tool call. */
export const NEIGHBOURS = 16;
/**
 * A page whose first or last steps are half of a tool call, with the other
 * half read from just outside it. `start` is the position in the chat of the
 * page's first message. The page's own messages are returned unchanged.
 */
export async function withNeighbours(
  page: MessagePage,
  start: number,
  readRange: (offset: number, limit: number) => Promise<readonly Message[]>,
): Promise<MessagePage> {
  const { results, calls } = unpaired(page.messages);
  const reach = Math.min(NEIGHBOURS, start);
  const [preceding, following] = await Promise.all([
    results.length && reach ? readRange(start - reach, reach) : [],
    calls.length ? readRange(start + page.messages.length, NEIGHBOURS) : [],
  ]);
  // A call ID matching is what makes a neighbour part of this page; a chat
  // that changed between the two reads simply stays unpaired.
  const before = callsBefore(results, preceding);
  const after = resultsAfter(calls, following);
  return {
    ...page,
    ...(before.length ? { before } : {}),
    ...(after.length ? { after } : {}),
  };
}
/** Where reading a chat starts: its end, a message, or a position in it. */
export type ChatPlace =
  | { readonly at: "end" }
  | { readonly at: "message"; readonly message: string }
  | { readonly at: "offset"; readonly offset: number; readonly limit?: number };

/** A run of a chat's messages, from position `start` up to `end`. */
export interface ChatStretch {
  readonly start: number;
  readonly end: number;
  /** Messages in the whole chat, when the server reports it. */
  readonly total?: number;
  readonly page: MessagePage;
}

/**
 * The stretch of a chat at `place`. Its edges are positions in the chat, so
 * the stretches before and after it can be read without gaps or overlap. A
 * tool call cut off from its output by an edge arrives with it.
 */
export async function chatStretch(
  id: string,
  node: string,
  place: ChatPlace,
  signal: AbortSignal,
  shared = false,
): Promise<ChatStretch> {
  const read = (offset: number, limit: number) =>
    transcript(id, node, offset, signal, shared, limit);
  let start: number;
  let end: number;
  let page: MessagePage;
  if (place.at === "message") {
    page = await context(place.message, node, signal, shared);
    // Around a message the stretch says where it is by its messages' places.
    const places = page.messages.flatMap((message) =>
      message.ordinal === undefined ? [] : [message.ordinal],
    );
    if (!places.length) return { start: 0, end: 0, page };
    start = Math.min(...places);
    end = Math.max(...places) + 1;
  } else {
    const limit = place.at === "offset" ? (place.limit ?? PAGE_SIZE) : PAGE_SIZE;
    start = place.at === "offset" ? place.offset : 0;
    page = await read(start, limit);
    // The end of a chat longer than one read is found by its length.
    if (place.at === "end" && page.total !== undefined && page.total > limit) {
      start = page.total - limit;
      page = await read(start, limit);
    }
    end = start + (page.rows ?? page.messages.length);
  }
  const whole = await withNeighbours(
    page,
    start,
    async (from, limit) => (await read(from, limit)).messages,
  );
  return { start, end, ...(page.total !== undefined ? { total: page.total } : {}), page: whole };
}

/** The stretch just before `first`, or undefined at the start of the chat. */
export function earlierPlace(first: ChatStretch): ChatPlace | undefined {
  if (first.start <= 0) return undefined;
  const offset = Math.max(0, first.start - PAGE_SIZE);
  return { at: "offset", offset, limit: first.start - offset };
}

/** The stretch just after `last`, or undefined at the end of the chat as last read. */
export function laterPlace(last: ChatStretch): ChatPlace | undefined {
  // An answer that starts before where it was asked to is not the chat
  // reading on, so reading stops rather than repeat it.
  const first = last.page.messages[0]?.ordinal;
  if (first !== undefined && first < last.start) return undefined;
  const more = last.total === undefined ? last.end > last.start : last.end < last.total;
  return more ? { at: "offset", offset: last.end } : undefined;
}
