/**
 * Talking to Mash: finding the `mash` program, reading what `mash search
 * --live` and `mash view` answer, and the mash:// links that open a chat in the
 * Mash desktop app. Everything here is plain data in and out, so it is tested
 * without Raycast.
 */
import { join } from "node:path";

/** One chat that mentions what was typed, at its best matching message. */
export interface Hit {
  readonly chatId: string;
  readonly messageId: string;
  readonly title: string;
  readonly agent: string;
  readonly project: string;
  readonly updatedAt: Date | undefined;
  readonly passage: string;
}

/** One message around a hit, as `mash view` returns it. */
export interface Message {
  readonly id: string;
  readonly role: string;
  readonly content: string;
}

/**
 * Where Mash's install methods put `mash`, in the order to try. Raycast starts
 * commands with a short PATH, so the usual places are named outright.
 */
export function installLocations(home: string): string[] {
  return [
    join(home, ".local/bin/mash"), // the install script
    "/opt/homebrew/bin/mash", // Homebrew on Apple silicon
    "/usr/local/bin/mash", // Homebrew on Intel, and manual installs
    join(home, ".cargo/bin/mash"), // cargo-binstall
    join(home, ".local/share/mise/shims/mash"), // mise
    "/Applications/Mash.app/Contents/MacOS/mash", // the engine inside the desktop app
  ];
}

/** The `mash` to run: the one the reader configured, else the first installed. */
export function findMash(
  configured: string | undefined,
  home: string,
  exists: (path: string) => boolean,
): string | undefined {
  const chosen = configured?.trim();
  if (chosen) return exists(chosen) ? chosen : undefined;
  return installLocations(home).find(exists);
}

/** The arguments for a live search: this machine, by words, a short list. */
export function liveSearchArguments(query: string): string[] {
  return ["search", query, "--live", "--limit", "20"];
}

/** The arguments that read a message with a few on either side of it. */
export function viewArguments(messageId: string): string[] {
  return ["view", messageId, "--context", "3"];
}

/** Mash answers with JSON even when it fails; its message says what to do. */
export class MashError extends Error {}

function answer(stdout: string): Record<string, unknown> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    throw new MashError("Mash gave an answer this extension could not read. Is Mash up to date?");
  }
  if (typeof parsed !== "object" || parsed === null) {
    throw new MashError("Mash gave an answer this extension could not read.");
  }
  const body = parsed as Record<string, unknown>;
  const error = body.error as { message?: unknown } | undefined;
  if (error) {
    throw new MashError(typeof error.message === "string" ? error.message : "Mash couldn’t search.");
  }
  return body;
}

const text = (value: unknown): string => (typeof value === "string" ? value : "");

/** The chats in a `mash search` answer, each at its best matching message. */
export function parseSearch(stdout: string): Hit[] {
  const results = answer(stdout).results;
  if (!Array.isArray(results)) return [];
  return results.flatMap((row: Record<string, unknown>) => {
    const match = Array.isArray(row.matches) ? (row.matches[0] as Record<string, unknown>) : undefined;
    const chatId = text(row.chat_id);
    const messageId = text(match?.message_id);
    if (!chatId || !messageId) return [];
    return [
      {
        chatId,
        messageId,
        title: text(row.title).trim() || "Untitled chat",
        agent: text(row.provider),
        project: text(row.project),
        updatedAt: typeof row.updated_at === "number" ? new Date(row.updated_at) : undefined,
        passage: text(match?.snippet).trim(),
      },
    ];
  });
}

/** The messages in a `mash view` answer, oldest first. */
export function parseView(stdout: string): Message[] {
  const messages = answer(stdout).messages;
  if (!Array.isArray(messages)) return [];
  return messages.map((row: Record<string, unknown>) => ({
    id: text(row.id),
    role: text(row.role) || "message",
    content: text(row.content),
  }));
}

/** The mash:// link that opens a hit's chat at its message in the desktop app. */
export function chatLink(hit: Pick<Hit, "chatId" | "messageId">): string {
  const query = new URLSearchParams({ message: hit.messageId });
  return `mash://chat/${encodeURIComponent(hit.chatId)}?${query}`;
}

/** The mash:// link for Mash's full search of the same words. */
export function searchLink(query: string): string {
  return `mash://search?${new URLSearchParams({ q: query.trim() })}`;
}

/**
 * Chat history is untrusted text, and Raycast renders Markdown, where an image
 * or link would load a remote address. Text is shown in a code block fenced
 * longer than any run of backticks inside it, so nothing in it is interpreted.
 */
export function literal(content: string, limit = 4000): string {
  const clipped = content.length > limit ? `${content.slice(0, limit)}…` : content;
  const longest = Math.max(2, ...Array.from(clipped.matchAll(/`+/g), (run) => run[0].length));
  const fence = "`".repeat(longest + 1);
  return `${fence}text\n${clipped}\n${fence}`;
}

/**
 * Chat history rendered as Markdown, so code, lists and emphasis read as they
 * did in the chat. History is untrusted, and Raycast loads images, so every
 * image is turned into a link (links load nothing until clicked) and raw HTML
 * tags are shown as text. Code blocks and code spans are left alone: Raycast
 * shows them literally.
 */
export function rendered(content: string, limit = 4000): string {
  const clipped = content.length > limit ? `${content.slice(0, limit)}…` : content;
  let fence = "";
  return clipped
    .split("\n")
    .map((line) => {
      const opener = /^\s{0,3}(`{3,}|~{3,})/.exec(line)?.[1];
      if (fence) {
        if (opener && opener[0] === fence[0] && opener.length >= fence.length) fence = "";
        return line;
      }
      if (opener) {
        fence = opener;
        return line;
      }
      return outsideCode(line);
    })
    .join("\n");
}

/** Defuses images and HTML in one line, outside its code spans. */
function outsideCode(line: string): string {
  const parts = line.split(/(`+)/);
  let open = "";
  return parts
    .map((part) => {
      if (/^`+$/.test(part)) {
        if (!open) open = part;
        else if (part === open) open = "";
        return part;
      }
      return open ? part : part.replace(/!\[/g, "\\![").replace(/<(?=[a-zA-Z!/?])/g, "&lt;");
    })
    .join("");
}

/** Names agents the way Mash does. */
export function agentName(agent: string): string {
  const names: Record<string, string> = {
    "claude-code": "Claude Code",
    codex: "Codex",
    copilot: "GitHub Copilot",
    cursor: "Cursor",
    opencode: "OpenCode",
    antigravity: "Antigravity",
  };
  return names[agent] ?? agent;
}

/** The logo file under assets/agents for each agent Mash reads, and its brand colour.
 * Logos in ink follow Raycast's theme instead. */
export function agentLogo(agent: string): { file: string; tint?: string } | undefined {
  const logos: Record<string, { file: string; tint?: string }> = {
    "claude-code": { file: "claude.svg", tint: "#d97757" },
    codex: { file: "openai.svg" },
    copilot: { file: "githubcopilot.svg", tint: "#8957e5" },
    cursor: { file: "cursor.svg" },
    opencode: { file: "opencode.svg" },
    antigravity: { file: "antigravity.svg", tint: "#3d7fd6" },
  };
  return logos[agent];
}
