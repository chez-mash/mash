// Getting from a chat in Mash to the same session in its agent. The server
// knows which desktop apps this machine has and which sessions they can open
// (`/api/open-in`); a terminal command to resume the session is built here.
import { read } from "./api";
import type { Agent } from "./types";

export interface AppLink {
  /** The app's name, as the button names it: "Codex" or "Claude". */
  readonly app: string;
  readonly url: string;
}

/** The agents whose desktop apps can open one of their sessions. */
const withApps: readonly Agent[] = ["claude-code", "codex"];

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

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

/** Whether asking the server for an app link is worth it. */
export function mayOpenInApp(agent: Agent, id: string): boolean {
  return withApps.includes(agent) && uuid.test(id);
}

/** The link that opens the chat in its agent's desktop app, if this machine has one. */
export async function appLink(
  agent: Agent,
  id: string,
  signal: AbortSignal,
): Promise<AppLink | null> {
  const answer = await read("/api/open-in", new URLSearchParams({ agent, id }), signal);
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
