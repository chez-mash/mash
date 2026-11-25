import type { Agent } from "./types";

export interface AgentInfo {
  readonly name: string;
  /** Its vendored provider logo (public/agents) as SVG markup filled with the
   * current colour, so a mark takes the colour of the element around it. */
  readonly logo: string;
  /** Its brand colour, the fill of its tile: a token from src/theme/tokens.css. */
  readonly tint: string;
  /** The colour of the logo on that tile: white, or dark where the tile is light. */
  readonly ink: string;
}

// The logos are inlined rather than drawn as <img> or a CSS mask: an <img>
// cannot take a colour, and a mask through an inline custom property drew
// nothing in Safari on iOS.
const files = import.meta.glob<string>("../../public/agents/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

/** The logo in `file` (under public/agents), filled with the current colour. */
function logo(file: string): string {
  const svg = Object.entries(files).find(([path]) => path.endsWith(`/agents/${file}`))?.[1];
  if (svg === undefined) throw new Error(`No agent logo named ${file}`);
  return svg
    .replace(/<title>.*?<\/title>/s, "")
    .replace(/\s(width|height|style|fill|role)="[^"]*"/g, "")
    .replace("<svg", '<svg fill="currentColor" aria-hidden="true" focusable="false"');
}

/** Turn an element into an agent's small mark, as AgentMark draws it. */
export function paintMark(mark: HTMLElement, info: AgentInfo): void {
  mark.className = "mark sm";
  mark.setAttribute("aria-hidden", "true");
  mark.style.background = info.tint;
  mark.style.color = info.ink;
  mark.innerHTML = info.logo;
}

export const agents: Readonly<Record<Agent, AgentInfo>> = {
  "claude-code": {
    name: "Claude Code",
    logo: logo("claude.svg"),
    tint: "var(--agent-claude-code)",
    ink: "var(--agent-on)",
  },
  codex: {
    name: "Codex",
    logo: logo("openai.svg"),
    tint: "var(--agent-codex)",
    ink: "var(--agent-on-neutral)",
  },
  opencode: {
    name: "OpenCode",
    logo: logo("opencode.svg"),
    tint: "var(--agent-opencode)",
    ink: "var(--agent-on)",
  },
  copilot: {
    name: "GitHub Copilot",
    logo: logo("githubcopilot.svg"),
    tint: "var(--agent-copilot)",
    ink: "var(--agent-on)",
  },
  cursor: {
    name: "Cursor",
    logo: logo("cursor.svg"),
    tint: "var(--agent-cursor)",
    ink: "var(--agent-on-neutral)",
  },
  antigravity: {
    name: "Antigravity",
    logo: logo("antigravity.svg"),
    tint: "var(--agent-antigravity)",
    ink: "var(--agent-on)",
  },
};

/** The agent a recorder's name stands for, by its id ("claude-code") or the
 * name shown for it ("Claude Code"), or undefined for anyone else. */
export function agentNamed(recorder: string): Agent | undefined {
  const wanted = recorder.trim().toLowerCase();
  return (Object.keys(agents) as Agent[]).find(
    (agent) => agent === wanted || agents[agent].name.toLowerCase() === wanted,
  );
}
