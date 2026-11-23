import { exportDocumentStyle } from "../theme/exportDocument";
import { type MessagePage, transcript, WHOLE_PAGE } from "./api";
import { markdown } from "./markdown";
import { isStep, messageText } from "./message";
import { inDesktopApp } from "./shareLink";
import { contextLabel } from "./steps";
import type { Message } from "./types";

export type ExportFormat = "markdown" | "html" | "pdf";

export type ReadPage = (
  id: string,
  node: string,
  offset: number,
  signal: AbortSignal,
  limit: number,
) => Promise<MessagePage>;

/**
 * Reads every page, failing visibly rather than downloading a truncated
 * export. The first answer says how long the chat is, so the rest are asked
 * for together instead of one after another.
 */
export async function collectChat(
  id: string,
  node: string,
  signal: AbortSignal,
  progress: (count: number) => void,
  readPage: ReadPage = (id, node, offset, signal, limit) =>
    transcript(id, node, offset, signal, false, limit),
): Promise<readonly Message[]> {
  const messages: Message[] = [];
  const seen = new Set<string>();
  let characters = 0;
  // False once a page comes back empty: the chat ends there.
  const keep = (page: MessagePage): boolean => {
    for (const message of page.messages) {
      if (seen.has(message.id))
        throw new Error("Chat changed during export. Try again after indexing finishes.");
      seen.add(message.id);
      characters += messageText(message).length;
      if (characters > 50_000_000)
        throw new Error(
          "This chat exceeds the browser export size limit. No partial file was saved.",
        );
      messages.push(message);
    }
    progress(messages.length);
    return page.messages.length > 0;
  };
  const first = await readPage(id, node, 0, signal, WHOLE_PAGE);
  if (!keep(first)) return messages;
  let offset = WHOLE_PAGE;
  // One page past the stated length confirms the end without another trip.
  const known = Math.min(first.total ?? 0, LIMIT);
  if (known > WHOLE_PAGE) {
    signal.throwIfAborted();
    const offsets: number[] = [];
    for (; offset < known + WHOLE_PAGE; offset += WHOLE_PAGE) offsets.push(offset);
    // A few at a time, so a very long chat does not ask for everything at once.
    for (let at = 0; at < offsets.length; at += TOGETHER) {
      signal.throwIfAborted();
      const pages = await Promise.all(
        offsets
          .slice(at, at + TOGETHER)
          .map((start) => readPage(id, node, start, signal, WHOLE_PAGE)),
      );
      for (const page of pages) if (!keep(page)) return messages;
    }
  }
  for (; offset < LIMIT; offset += WHOLE_PAGE) {
    signal.throwIfAborted();
    if (!keep(await readPage(id, node, offset, signal, WHOLE_PAGE))) return messages;
  }
  throw new Error(
    "This chat exceeds the 100,000-position browser export limit. No partial file was saved.",
  );
}
const LIMIT = 100_000;
const TOGETHER = 6;

function escapeHtml(text: string): string {
  return text.replace(
    /[&<>"']/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character] ??
      character,
  );
}

function label(message: Message): string {
  const kind = message.presentation;
  return kind === "tool-result"
    ? "Tool output"
    : kind === "tool-call"
      ? message.toolLabel || "Tool call"
      : kind === "reasoning"
        ? "Reasoning"
        : kind === "context"
          ? contextLabel(message)
          : message.role;
}
/** The heading over the whole of a message whose own words are shown first. */
const ADDED = "Context added by the agent";
/** A fence longer than any run of backticks in `text`, so it is kept verbatim. */
function fenced(text: string): string {
  let length = 3;
  for (const match of text.matchAll(/`+/g)) length = Math.max(length, match[0].length + 1);
  const fence = "`".repeat(length);
  return `${fence}\n${text}\n${fence}`;
}

export function markdownExport(title: string, node: string, messages: readonly Message[]): string {
  return `# ${title.replace(/\s+/g, " ")}\n\nMachine: ${node}\n\n${messages
    .map((message) => {
      const text = messageText(message);
      // Steps are fenced, which also preserves tool output verbatim.
      const body = isStep(message)
        ? fenced(text)
        : message.dialogue === undefined
          ? text
          : `${message.dialogue}\n\n### ${ADDED}\n\n${fenced(text)}`;
      return `## ${label(message)} · ${message.at}\n\n${body}`;
    })
    .join("\n\n---\n\n")}\n`;
}

function htmlBody(message: Message): string {
  const whole = `<pre>${escapeHtml(messageText(message))}</pre>`;
  if (isStep(message)) return whole;
  if (message.dialogue === undefined) return markdown(messageText(message));
  return `${markdown(message.dialogue)}<h2>${ADDED}</h2>${whole}`;
}

export function htmlExport(title: string, node: string, messages: readonly Message[]): string {
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>${escapeHtml(title)}</title><style>${exportDocumentStyle}</style></head><body><header><h1>${escapeHtml(title)}</h1><p>Machine: ${escapeHtml(node)} · ${messages.length} messages</p></header>${messages.map((message) => `<article><h2>${escapeHtml(label(message))} · ${escapeHtml(message.at)}</h2>${htmlBody(message)}</article>`).join("\n")}</body></html>`;
}

export function downloadChat(content: string, format: "markdown" | "html"): void {
  const url = URL.createObjectURL(
    new Blob([content], {
      type: format === "html" ? "text/html;charset=utf-8" : "text/markdown;charset=utf-8",
    }),
  );
  const link = document.createElement("a");
  link.href = url;
  link.download = `mash-chat.${format === "html" ? "html" : "md"}`;
  link.click();
  // Let the browser consume the download before releasing its backing data.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/**
 * Prints or closes the PDF tab. A webview does neither for a page, so inside
 * the desktop app the tab asks the app instead, through an address the app
 * catches (desktop/src-tauri/src/print.rs).
 */
function finish(popup: Window, action: "print" | "close"): void {
  if (inDesktopApp()) popup.location.href = `mash://${action}`;
  else if (action === "print") popup.print();
  else popup.close();
}

/** The chat an export reads, and how to read its pages. */
export interface ExportSource {
  readonly id: string;
  readonly node: string;
  readonly title: string;
  /** The machine named in the exported file. */
  readonly machine: string;
  readonly readPage: ReadPage;
}

/**
 * Exports the entire chat, never only what is loaded, telling the reader how
 * it goes through `notify`. Call it straight from the click: a PDF opens its
 * print tab before anything is read, while the browser still counts the
 * click as the reader's, and a blocked tab is said rather than worked around.
 */
export async function exportChat(
  format: ExportFormat,
  source: ExportSource,
  signal: AbortSignal,
  notify: (notice: string) => void,
): Promise<void> {
  const popup = format === "pdf" ? window.open("", "_blank") : null;
  if (format === "pdf" && !popup) {
    notify("Allow a new tab to prepare the PDF export.");
    return;
  }
  if (popup) {
    popup.opener = null;
    popup.document.body.textContent = "Preparing the entire chat for printing…";
  }
  const { title, machine } = source;
  notify("Preparing export…");
  try {
    const messages = await collectChat(
      source.id,
      source.node,
      signal,
      (count) => notify(`Preparing export: ${count.toLocaleString()} messages…`),
      source.readPage,
    );
    signal.throwIfAborted();
    if (format === "pdf" && popup) {
      if (popup.closed) throw new Error("The PDF preview tab was closed.");
      popup.document.open();
      popup.document.write(htmlExport(title, machine, messages));
      popup.document.close();
      popup.focus();
      finish(popup, "print");
      notify("Choose Save as PDF in the print dialog. The preview includes the entire chat.");
    } else {
      downloadChat(
        format === "markdown"
          ? markdownExport(title, machine, messages)
          : htmlExport(title, machine, messages),
        format === "markdown" ? "markdown" : "html",
      );
      notify(`Exported ${messages.length.toLocaleString()} messages.`);
    }
  } catch (error) {
    if (popup) finish(popup, "close");
    notify(
      signal.aborted
        ? "Export cancelled."
        : error instanceof Error
          ? error.message
          : "Export failed.",
    );
  }
}
