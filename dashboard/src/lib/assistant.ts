// Mashy, the dashboard's assistant: the conversation with `/api/assistant`.
// The dashboard server talks to the model; this side sends the conversation
// and reads the answer as a stream of events. Nothing here holds a key.
import { read } from "./api";
import {
  agentHref,
  chatHref,
  machineHref,
  projectHref,
  recordHref,
  searchHref,
} from "./route.svelte";
import { send } from "./transport";

export interface Turn {
  readonly role: "user" | "assistant";
  readonly content: string;
}
/** The chat on screen, so "this chat" means something to the assistant. */
export interface OpenChat {
  readonly id: string;
  readonly node?: string;
  readonly shared?: boolean;
  readonly title?: string;
}
type Reason = "unreachable" | "refused" | "limited" | "failed";
export type AssistantEvent =
  | { readonly type: "searching" }
  | { readonly type: "text"; readonly delta: string }
  | { readonly type: "done"; readonly partial: boolean }
  | { readonly type: "failed"; readonly reason: Reason; readonly message: string };

const reasons: readonly string[] = ["unreachable", "refused", "limited", "failed"];

/** One event of an answer, or undefined when it is not what the server sends. */
export function parseEvent(data: string): AssistantEvent | undefined {
  let body: unknown;
  try {
    body = JSON.parse(data);
  } catch {
    return undefined;
  }
  if (typeof body !== "object" || body === null) return undefined;
  const event = body as Record<string, unknown>;
  switch (event.type) {
    case "searching":
      return { type: "searching" };
    case "text":
      return typeof event.delta === "string" ? { type: "text", delta: event.delta } : undefined;
    case "done":
      return typeof event.partial === "boolean"
        ? { type: "done", partial: event.partial }
        : undefined;
    case "failed":
      return typeof event.message === "string" &&
        typeof event.reason === "string" &&
        reasons.includes(event.reason)
        ? { type: "failed", reason: event.reason as Reason, message: event.message }
        : undefined;
    default:
      return undefined;
  }
}

/** Whether this dashboard has an assistant, and which model answers. */
export const assistantOptions = {
  queryKey: ["assistant"],
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    read("/api/assistant", new URLSearchParams(), signal),
  staleTime: Infinity,
  retry: false,
} as const;

/** The models the configured service offers, for choosing one in settings. */
export const assistantModelsOptions = {
  queryKey: ["assistant", "models"],
  queryFn: async ({ signal }: { signal: AbortSignal }) =>
    (await read("/api/assistant/models", new URLSearchParams(), signal)).models,
  staleTime: Infinity,
  retry: false,
} as const;

/**
 * Asks a question and calls `heard` for each event of the answer. Rejects
 * when the dashboard turns the question down or the stream breaks; an
 * aborted question rejects with the abort.
 */
export async function ask(
  messages: readonly Turn[],
  openChat: OpenChat | undefined,
  signal: AbortSignal,
  heard: (event: AssistantEvent) => void,
  passage = "",
): Promise<void> {
  const response = await send("/api/assistant", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      messages,
      ...(openChat ? { open_chat: openChat } : {}),
      // Text the reader selected and is asking about.
      ...(passage ? { passage: passage.slice(0, PASSAGE) } : {}),
      // So "yesterday" and every time in an answer are on this reader's clock.
      utc_offset: -new Date().getTimezoneOffset(),
    }),
    cache: "no-store",
    signal,
  });
  if (!response.ok || !response.body) {
    let detail = `Mashy couldn’t answer (${response.status}).`;
    try {
      const failure = (await response.json()) as { error?: { message?: unknown } };
      if (typeof failure.error?.message === "string") detail = failure.error.message;
    } catch {
      /* A failure without a JSON body keeps the fallback. */
    }
    throw new Error(detail);
  }
  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
  let pending = "";
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return;
    pending += value;
    let end = pending.indexOf("\n");
    while (end >= 0) {
      const line = pending.slice(0, end);
      pending = pending.slice(end + 1);
      if (line.startsWith("data:")) {
        const event = parseEvent(line.slice(5).trim());
        if (event) heard(event);
      }
      end = pending.indexOf("\n");
    }
  }
}

/** The most of a selected passage that is sent with a question. */
export const PASSAGE = 2000;

const LINK = /\]\((chat|memory|project|agent|machine|search|docs):([^)\s]+)\)/g;

function target(kind: string, rest: string): string {
  const [id = "", query = ""] = rest.split("?", 2);
  let name = id;
  try {
    name = decodeURIComponent(id);
  } catch {
    /* An ID that is not percent-encoded is used as written. */
  }
  const params = new URLSearchParams(query);
  const node = params.get("node") ?? "";
  switch (kind) {
    case "chat":
      // A chat link may name the message to open at.
      return chatHref(name, node, params.get("message") ?? "", node !== "");
    case "project":
      return projectHref(name);
    case "agent":
      return agentHref(name);
    case "machine":
      return machineHref(name);
    case "search":
      return searchHref({ q: name });
    case "docs":
      // One of Mash's guides, on the documentation site.
      return `https://getmash.io/${encodeURIComponent(name)}/`;
    default:
      return recordHref(name, node);
  }
}

/**
 * The assistant names chats and memories as `chat:ID` and `memory:ID` links.
 * This turns them into the dashboard's own addresses; every other link is
 * left for the sanitizer.
 */
export function linkify(text: string): string {
  return text.replace(LINK, (_, kind: string, rest: string) => `](${target(kind, rest)})`);
}

export interface Answer {
  /** What Mashy said. */
  readonly body: string;
  /** Questions it suggests asking next. */
  readonly next: readonly string[];
}

/**
 * Mashy ends an answer with the questions it suggests next, one per line
 * after a `§`. This separates them from the answer, also while it is still
 * arriving.
 */
export function answer(text: string): Answer {
  const [body = "", ...lines] = text.split(/^[ \t]*§/m);
  const next = lines
    .map((line) => (line.split("\n", 1)[0] ?? "").trim())
    .filter((line) => line.length > 0 && line.length <= 120)
    .slice(0, 3);
  return { body: body.trimEnd(), next };
}

export interface Source {
  readonly kind: "chat" | "memory";
  readonly title: string;
  readonly href: string;
}

const SOURCE = /\[([^\]\n]+)\]\((chat|memory):([^)\s]+)\)/g;

/**
 * The chats and memories an answer links to, each once, in order. A chat
 * linked at several messages is one source, opening at the first.
 */
export function sources(text: string): Source[] {
  const found = new Map<string, Source>();
  for (const [, title = "", kind = "", rest = ""] of text.matchAll(SOURCE)) {
    if (kind !== "chat" && kind !== "memory") continue;
    const href = target(kind, rest);
    const [id = "", query = ""] = rest.split("?", 2);
    const key = `${kind}:${id}?${new URLSearchParams(query).get("node") ?? ""}`;
    if (!found.has(key)) found.set(key, { kind: kind === "chat" ? "chat" : "memory", title, href });
  }
  return [...found.values()];
}
