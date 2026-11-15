// A page of a chat as the reader sees it: what was said, and between it the
// folded runs of steps (tool calls with their output, thinking, context).
import { isStep, messageText } from "./message";
import { spaced, type ToolCall, toolCalls } from "./toolcalls";
import type { Message } from "./types";

export type Item =
  | { readonly kind: "message"; readonly message: Message }
  | {
      readonly kind: "steps";
      readonly messages: readonly Message[];
      /** Tool calls from just before the page whose output is in this run. */
      readonly before?: readonly Message[];
      /** Tool output from just after the page for calls in this run. */
      readonly after?: readonly Message[];
    };

export type StepPart =
  | { readonly kind: "call"; readonly call: ToolCall; readonly messages: readonly Message[] }
  | { readonly kind: "note"; readonly message: Message };

interface Page {
  readonly messages: readonly Message[];
  readonly before?: readonly Message[];
  readonly after?: readonly Message[];
}

/** Consecutive steps read as one run, so the conversation stays the main thing. */
export function transcriptItems(page: Page): Item[] {
  const runs: (Message | Message[])[] = [];
  for (const message of page.messages) {
    const last = runs.at(-1);
    if (!isStep(message)) runs.push(message);
    else if (Array.isArray(last)) last.push(message);
    else runs.push([message]);
  }
  return runs.map((run, index): Item => {
    if (!Array.isArray(run)) return { kind: "message", message: run };
    const before = index === 0 ? page.before : undefined;
    const after = index === runs.length - 1 ? page.after : undefined;
    return {
      kind: "steps",
      messages: run,
      ...(before?.length ? { before } : {}),
      ...(after?.length ? { after } : {}),
    };
  });
}

type Run = Extract<Item, { kind: "steps" }>;

/** The run's tool calls, each with the message it belongs under. A call from
 * the page before is shown only for the output this run holds. */
function runCalls(run: Run): { call: ToolCall; message: Message }[] {
  const all = [...(run.before ?? []), ...run.messages, ...(run.after ?? [])];
  const calls = toolCalls(all);
  const own = new Set(run.messages.map((message) => message.id));
  return all.flatMap((message) =>
    calls
      .filter((call) => call.id === message.id || call.id.startsWith(`${message.id}:`))
      .filter((call) => own.has(message.id) || (call.resultId && own.has(call.resultId)))
      .map((call) => ({ call, message })),
  );
}

/** A run as what the reader sees: each tool call once, with its output, and
 * thinking and context in between, in the order they happened. */
export function stepParts(run: Run): StepPart[] {
  const calls = runCalls(run);
  const parts: StepPart[] = [];
  for (const message of [...(run.before ?? []), ...run.messages]) {
    if (message.presentation === "reasoning" || message.presentation === "context")
      parts.push({ kind: "note", message });
    for (const { call, message: holder } of calls) {
      if (holder !== message) continue;
      parts.push({
        kind: "call",
        call,
        messages: run.messages.filter(
          (other) =>
            (other.id === message.id && call.id === message.id) || other.id === call.resultId,
        ),
      });
    }
  }
  return parts;
}

/** What a context message is, by who it is attributed to. */
export function contextLabel(message: Message): string {
  if (message.role === "user") return "Context added by the agent";
  if (message.role === "system") return "System instructions";
  if (message.role === "developer") return "Developer instructions";
  if (message.role !== "assistant") return message.role || "Context";
  const type = spaced(recordType(messageText({ ...message, activity: "" })));
  return type ? `Activity: ${type}` : "Activity";
}
function recordType(text: string): string | undefined {
  if (!text.trimStart().startsWith("{")) return undefined;
  try {
    const record: unknown = JSON.parse(text);
    return typeof record === "object" &&
      record !== null &&
      "type" in record &&
      typeof record.type === "string"
      ? record.type
      : undefined;
  } catch {
    return undefined;
  }
}

export function stepLabel(message: Message): string {
  if (message.presentation === "context") return contextLabel(message);
  if (message.presentation === "reasoning") return "Thinking";
  return message.presentation === "tool-result" ? "Tool output" : message.toolLabel || "Tool call";
}

/** The one line a folded run shows. */
export function stepsSummary(run: Run): string {
  const calls = runCalls(run).map(({ call }) => call);
  if (!calls.length && run.messages.every((message) => message.presentation === "context")) {
    const [only] = run.messages;
    return only && run.messages.length === 1
      ? contextLabel(only)
      : `${run.messages.length.toLocaleString()} context items`;
  }
  const names = [...new Set(calls.map((call) => call.name))];
  const count = calls.length || run.messages.length;
  const noun = calls.length ? "tool call" : "step";
  const failed = calls.filter((call) => call.status?.failed).length;
  return `${count} ${noun}${count === 1 ? "" : "s"}${names.length ? `: ${names.slice(0, 4).join(", ")}${names.length > 4 ? ", …" : ""}` : ""}${failed ? ` · ${failed} failed` : ""}`;
}
