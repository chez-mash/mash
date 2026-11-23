// Getting from a chat in Mash to the same session in its agent. The server
// knows which desktop apps this machine has and which sessions they can open
// (`/api/open-in`); a terminal command to resume the session is built here.
import { read } from "./api";
import type { Agent } from "./types";

export interface AppLink {
  /** The app's name, as the button names it: "Codex", "Claude" or "Copilot". */
  readonly app: string;
  readonly url: string;
  /** The app does not have the session yet: following the link adds it. */
  readonly addsSession: boolean;
}

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
/** Mash's own ID for a chat, which is what a Copilot chat is known by. */
const hashed = /^[0-9a-f]{24}$/;

/**
 * Whether the chat's session belongs to the machine showing the dashboard,
 * where its agent and that agent's app are. A chat from another machine, or
 * read from the shared index, would open nothing here.
 */
export function onThisMachine(
  chat: { readonly machine: string },
  localNode: string | undefined,
  shared: boolean,
): boolean {
  return !shared && localNode !== undefined && chat.machine === localNode;
}

/**
 * Whether asking the server for an app link is worth it. Codex and Claude
 * Code chats are known by their session IDs; the server finds a Copilot
 * chat's session from Mash's ID.
 */
export function mayOpenInApp(agent: Agent, id: string): boolean {
  if (agent === "claude-code" || agent === "codex") return uuid.test(id);
  return agent === "copilot" && hashed.test(id);
}

/**
 * The link that opens the chat in its agent's desktop app, if this machine
 * has one. The chat's folder goes along, as Claude's link carries it.
 */
export async function appLink(
  chat: { readonly agent: Agent; readonly id: string; readonly project: string },
  signal: AbortSignal,
): Promise<AppLink | null> {
  const query = new URLSearchParams({ agent: chat.agent, id: chat.id });
  if (chat.project) query.set("cwd", chat.project);
  const answer = await read("/api/open-in", query, signal);
  return answer.link ?? null;
}

/**
 * The command that resumes the session in the agent's own command line, as
 * `claude --help` and `codex resume --help` document it. Claude Code finds a
 * session by the folder it ran in, so the command is run in the chat's project.
 */
export function resumeCommand(agent: Agent, id: string): string | undefined {
  if (!uuid.test(id)) return undefined;
  if (agent === "claude-code") return `claude --resume ${id}`;
  if (agent === "codex") return `codex resume ${id}`;
  return undefined;
}
