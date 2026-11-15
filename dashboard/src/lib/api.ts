import type { Answer, ReadPath, Wire } from "./routes";
import { check } from "./session.svelte";
import { callsBefore, resultsAfter, unpaired } from "./toolcalls";
import { send } from "./transport";
import type { Agent, ChatSummary, Message, SearchFilters, SearchHit } from "./types";

export interface Catalog {
  readonly chats: readonly ChatSummary[];
  readonly localNode: string;
  readonly warnings: readonly string[];
}
export interface SearchPage {
  readonly hits: readonly SearchHit[];
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
}

export function isAgent(value: string): value is Agent {
  return ["claude-code", "codex", "opencode", "copilot", "cursor", "antigravity"].includes(value);
}
function date(value: number | undefined): string {
  return value === undefined ? "Time unknown" : new Date(value).toLocaleString();
}
function chat(row: Wire["Chat"] | Wire["ChatHit"], localNode: string): ChatSummary {
  const updatedMs = row.updated_at ?? row.created_at ?? undefined;
  return {
    id: row.chat_id,
    title: row.title || "Untitled chat",
    agent: row.provider,
    machine: row.node_id ?? localNode,
    project: row.project ?? "",
    updatedAt: date(updatedMs),
    ...(updatedMs === undefined ? {} : { updatedMs }),
  };
}
export async function read<P extends ReadPath>(
  path: P,
  params: URLSearchParams,
  signal: AbortSignal,
): Promise<Answer<P>> {
  const response = await send(`${path}?${params}`, {
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
  return {
    localNode: body.local_node,
    chats: body.chats.map((row) => chat(row, body.local_node)),
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
/** Empty submissions filter one catalogue snapshot; drafts never alter submitted results. */
export function browse(catalog: Catalog, filters: SearchFilters, now: number): SearchPage {
  return {
    hits: catalog.chats
      .filter(
        (chat) =>
          (!filters.agents.length || filters.agents.includes(chat.agent)) &&
          (!filters.machines.length || filters.machines.includes(chat.machine)) &&
          (!filters.projects.length || filters.projects.includes(chat.project)) &&
          (filters.days === null ||
            (chat.updatedMs !== undefined && chat.updatedMs >= now - filters.days * 86_400_000)),
      )
      .map((chat) => ({ chat, messageId: "", before: "", match: "", after: "" })),
    nextOffset: null,
    warnings: catalog.warnings,
  };
}
export async function search(
  query: string,
  filters: SearchFilters,
  offset: number,
  localNode: string,
  signal: AbortSignal,
  sort: "relevance" | "recent" = "relevance",
): Promise<SearchPage> {
  const params = new URLSearchParams({
    q: query,
    provider: filters.agents.join(","),
    machine: filters.machines.join(","),
    project: filters.projects.join(","),
    offset: String(offset),
  });
  if (filters.days !== null) params.set("days", String(filters.days));
  if (sort !== "relevance") params.set("sort", sort);
  return parseSearch(await read("/api/search", params, signal), localNode);
}

export function parseSearch(body: Answer<"/api/search">, localNode: string): SearchPage {
  return {
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
    warnings: [body.shared_error, body.shared_context_error, body.notice].flatMap((text) =>
      text ? [text] : [],
    ),
  };
}
export function parseMessages(body: Answer<"/api/chat">): MessagePage {
  return {
    ...(typeof body.message_count === "number" ? { total: body.message_count } : {}),
    messages: body.messages.map((row) => {
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
  return parseMessages(
    await read(
      "/api/chat",
      new URLSearchParams({
        id,
        ...(node ? { node } : {}),
        ...(shared ? { shared: "true" } : {}),
        offset: String(offset),
        limit: String(limit),
      }),
      signal,
    ),
  );
}
export async function context(
  id: string,
  node: string,
  signal: AbortSignal,
  shared = false,
): Promise<MessagePage> {
  const params = new URLSearchParams({ id, context: "25" });
  if (node) params.set("node", node);
  if (shared) params.set("shared", "true");
  return parseMessages(await read("/api/view", params, signal));
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
/** One page of a chat as the reader sees it, or the messages around `anchor`. */
export async function transcriptPage(
  id: string,
  node: string,
  offset: number,
  anchor: string,
  signal: AbortSignal,
  shared = false,
): Promise<MessagePage> {
  const page = anchor
    ? await context(anchor, node, signal, shared)
    : await transcript(id, node, offset, signal, shared);
  // Around a match the page says where it is by its first message's place in the chat.
  const start = anchor ? page.messages[0]?.ordinal : offset;
  if (start === undefined) return page;
  return withNeighbours(
    page,
    start,
    async (from, limit) => (await transcript(id, node, from, signal, shared, limit)).messages,
  );
}
