/** The agent that produced a chat; matches mash's provider names. */
export type Agent = "claude-code" | "codex" | "opencode" | "copilot" | "cursor" | "antigravity";

export interface ChatSummary {
  readonly id: string;
  readonly title: string;
  readonly agent: Agent;
  readonly machine: string;
  readonly project: string;
  readonly updatedAt: string;
  readonly messageCount?: number;
  readonly updatedMs?: number;
  readonly createdMs?: number;
  /** Set when the agent marked this chat as a subagent it started. */
  readonly subagent?: Subagent;
  /** Subagent chats folded into this one in lists. */
  readonly subagents?: number;
  /** Read from the Claude desktop app's copy of a session it ran over SSH. */
  readonly sshCopy?: true;
}

/** What the agent recorded about a subagent chat. */
export interface Subagent {
  /** The chat that started it, from the same agent and machine. */
  readonly parentId?: string;
  /** The parent's title, when the parent is indexed with it. */
  readonly parentTitle?: string;
  readonly label?: string;
  readonly role?: string;
  readonly finished?: boolean;
  /** The parent's tool call that started it. */
  readonly spawnCallId?: string;
}

type ToolKind = "read" | "search" | "edit" | "run" | "web" | "agent";

/** One tool call and what the provider kept of its result. */
interface ToolCall {
  readonly kind: ToolKind;
  readonly verb: string;
  readonly target: string;
  readonly status: string;
  readonly tone: "ok" | "bad" | "plain";
  readonly cwd?: string;
  readonly output?: { readonly lines: readonly string[]; readonly omitted: number };
  readonly diff?: readonly string[];
  /** Why no output is shown, when the provider kept none. */
  readonly missing?: string;
}

type Block =
  | { readonly type: "text"; readonly markdown: string }
  | {
      readonly type: "code";
      readonly language: string;
      readonly file?: string;
      readonly code: string;
    }
  | { readonly type: "tools"; readonly calls: readonly ToolCall[] };

export interface Message {
  readonly id: string;
  readonly role: string;
  readonly ordinal?: number;
  readonly activity?: string;
  /** `context` is text the agent or its tooling put into the chat: nobody
   * said it and it answers nobody. */
  readonly presentation?: "dialogue" | "tool-call" | "tool-result" | "reasoning" | "context";
  /** What the person typed, when the message also carries context the agent
   * added around it. Absent when the whole message is what was said. */
  readonly dialogue?: string;
  readonly toolLabel?: string;
  /** The stored tool records of a step, as the agent wrote them. Inputs are
   * opaque provider payloads: shown as text, never interpreted. */
  readonly tools?: {
    readonly calls: readonly {
      readonly name: string | undefined;
      readonly callId: string | undefined;
      readonly input: unknown;
    }[];
    readonly results: readonly {
      readonly callId: string | undefined;
      readonly isError: boolean | undefined;
    }[];
  };
  readonly at: string;
  readonly blocks: readonly Block[];
  /** Files the message carried, in the order it carried them. */
  readonly attachments?: readonly Attachment[];
}

/** A file a message carried, and whether Mash kept a copy to show. */
export interface Attachment {
  readonly id: string;
  readonly name: string;
  readonly mediaType: string;
  readonly bytes?: number;
  /** Where its bytes are read; absent when Mash did not keep them. */
  readonly url?: string;
  /**
   * The same bytes as a download named for the file. The server says so in
   * its headers, which the desktop app's webview honours where a link's
   * `download` attribute alone is not.
   */
  readonly downloadUrl?: string;
  /** A kept picture the browser can draw, shown rather than downloaded. */
  readonly image: boolean;
  /** Why no copy is kept, when none is. */
  readonly missing?: string;
}

export interface SearchFilters {
  readonly agents: readonly Agent[];
  readonly machines: readonly string[];
  readonly projects: readonly string[];
  readonly days: number | null;
}

export interface SearchHit {
  readonly chat: ChatSummary;
  readonly messageId: string;
  readonly before: string;
  readonly match: string;
  readonly after: string;
}
