import { describe, expect, it } from "vitest";
import * as api from "./api";
import { browse } from "./api";
import { trimmed } from "./trimmed";

const parseCatalog = (body: unknown) => api.parseCatalog(trimmed(body));
const parseMessages = (body: unknown) => api.parseMessages(trimmed(body));
const parseSearch = (body: unknown, localNode: string) => api.parseSearch(trimmed(body), localNode);

const row = {
  chat_id: "same-id",
  provider: "codex",
  title: "<script>untrusted</script>",
  project: "/project",
  updated_at: 1_000_000_000,
};
describe("dashboard API boundary", () => {
  it("opens the evidence source from lookup even when a chat's representative differs", () => {
    const page = parseSearch(
      {
        results: [
          {
            ...row,
            node_id: "representative",
            matches: [
              {
                message_id: "raw",
                snippet: "match",
                lookup: { view: { id: "anchor", shared: "evidence-node" } },
              },
            ],
          },
        ],
      },
      "local",
    );
    expect(page.hits[0]?.chat.machine).toBe("evidence-node");
    expect(page.hits[0]?.messageId).toBe("anchor");
  });
  it("preserves machine identities, timestamps and partial-result warnings", () => {
    const catalog = parseCatalog({
      local_node: "local",
      truncated: true,
      shared_error: "unreachable",
      chats: [
        { ...row, node_id: "local" },
        { ...row, node_id: "remote" },
      ],
    });
    expect(catalog.chats.map((chat) => chat.machine)).toEqual(["local", "remote"]);
    expect(catalog.chats[0]?.title).toBe(row.title);
    expect(catalog.chats[0]?.updatedMs).toBe(row.updated_at);
    expect(catalog.chats[0]?.messageCount).toBeUndefined();
    expect(catalog.warnings).toHaveLength(2);
  });
  it("applies project, machine, agent and date filters to empty searches", () => {
    const catalog = parseCatalog({
      local_node: "local",
      truncated: false,
      chats: [
        row,
        { ...row, chat_id: "old", updated_at: 0 },
        { ...row, chat_id: "other", project: "/other" },
      ],
    });
    const filters = {
      agents: ["codex" as const],
      machines: ["local"],
      projects: ["/project"],
      days: 1,
    };
    expect(browse(catalog, filters, row.updated_at).hits.map((hit) => hit.chat.id)).toEqual([
      "same-id",
    ]);
    expect(browse(catalog, { ...filters, machines: ["remote"] }, row.updated_at).hits).toEqual([]);
  });
  it("retains provider roles, tool input and raw message text for safe rendering", () => {
    const page = parseMessages({
      messages: [
        {
          id: "a",
          ordinal: 37,
          role: "tool",
          content: "<img src=x onerror=alert(1)>",
          tools: { tool_calls: [{ name: "run", input: { command: "echo hello" } }] },
        },
      ],
    });
    expect(page.messages[0]?.role).toBe("tool");
    expect(page.messages[0]?.ordinal).toBe(37);
    expect(page.messages[0]?.activity).toContain("echo hello");
    expect(page.messages[0]?.blocks).toEqual([
      { type: "text", markdown: "<img src=x onerror=alert(1)>" },
    ]);
  });
  it("keeps the chat's length so a reader can jump to its last page", () => {
    expect(parseMessages({ message_count: 342, messages: [] }).total).toBe(342);
    expect(parseMessages({ messages: [] }).total).toBeUndefined();
  });
  it("only an assistant message can be reasoning", () => {
    const page = parseMessages({
      messages: [
        { id: "a", ordinal: 0, role: "assistant", content: "[Thinking] about it" },
        { id: "u", ordinal: 1, role: "user", content: "[Thinking] is a tag I typed" },
        { id: "s", ordinal: 2, role: "system", content: "[Thinking] in a system prompt" },
      ],
    });
    expect(page.messages.map((message) => message.presentation)).toEqual([
      "reasoning",
      "dialogue",
      "context",
    ]);
  });
  it("treats system, developer and unknown roles as context, never as replies", () => {
    const page = parseMessages({
      messages: ["system", "developer", "critic"].map((role, ordinal) => ({
        id: role,
        ordinal,
        role,
        content: "You are a coding agent.",
      })),
    });
    expect(page.messages.map((message) => message.presentation)).toEqual([
      "context",
      "context",
      "context",
    ]);
  });
  it("reads the dialogue field: absent is dialogue, empty is context, different is what was typed", () => {
    const page = parseMessages({
      messages: [
        { id: "whole", ordinal: 0, role: "user", content: "fix it" },
        { id: "same", ordinal: 1, role: "user", content: "fix it", dialogue: "fix it" },
        {
          id: "injected",
          ordinal: 2,
          role: "user",
          content: "<reminder>x</reminder>",
          dialogue: "",
        },
        {
          id: "bookkeeping",
          ordinal: 3,
          role: "assistant",
          content: '{"type":"compaction"}',
          dialogue: "",
        },
        {
          id: "wrapped",
          ordinal: 4,
          role: "user",
          content: "<ide>open file</ide>\nfix it",
          dialogue: "fix it",
        },
        { id: "reply", ordinal: 5, role: "assistant", content: "Done.", dialogue: "Done" },
      ],
    });
    expect(page.messages.map((message) => [message.presentation, message.dialogue])).toEqual([
      ["dialogue", undefined],
      ["dialogue", undefined],
      ["context", undefined],
      ["context", undefined],
      ["dialogue", "fix it"],
      ["dialogue", undefined],
    ]);
    // The whole message is kept for the fold and for exports.
    expect(page.messages[4]?.blocks).toEqual([
      { type: "text", markdown: "<ide>open file</ide>\nfix it" },
    ]);
  });
  it("tool steps and reasoning come before context when deciding what a message is", () => {
    const page = parseMessages({
      messages: [
        {
          id: "r",
          ordinal: 0,
          role: "user",
          content: "out",
          dialogue: "",
          tools: { tool_results: [{ call_id: "t1" }] },
        },
        {
          id: "c",
          ordinal: 1,
          role: "assistant",
          content: "",
          dialogue: "",
          tools: { tool_calls: [{ name: "Bash", call_id: "t1" }] },
        },
        { id: "t", ordinal: 2, role: "assistant", content: "[Thinking] hm", dialogue: "" },
        { id: "o", ordinal: 3, role: "tool", content: "out" },
      ],
    });
    expect(page.messages.map((message) => message.presentation)).toEqual([
      "tool-result",
      "tool-call",
      "reasoning",
      "tool-result",
    ]);
  });
});

describe("tool calls split from their output by the edge of a page", () => {
  const said = (id: string, ordinal: number) => ({ id, ordinal, role: "user", content: id });
  const call = (id: string, ordinal: number, callId: string) => ({
    id,
    ordinal,
    role: "assistant",
    content: "",
    tools: { tool_calls: [{ name: "Bash", call_id: callId, input: { command: "ls" } }] },
  });
  const result = (id: string, ordinal: number, callId: string) => ({
    id,
    ordinal,
    role: "user",
    content: `output of ${callId}`,
    tools: { tool_results: [{ call_id: callId, is_error: false }] },
  });
  const chat = parseMessages({
    messages: [
      said("q0", 0),
      call("c1", 1, "t1"),
      call("c2", 2, "t2"),
      // A page of three ends here, between two calls and their outputs.
      result("r1", 3, "t1"),
      result("r2", 4, "t2"),
      said("q5", 5),
      call("c6", 6, "t6"),
      result("r6", 7, "t6"),
    ],
  }).messages;
  function reader() {
    const asked: [number, number][] = [];
    return {
      asked,
      read: async (offset: number, limit: number) => {
        asked.push([offset, limit]);
        return chat.slice(offset, offset + limit);
      },
    };
  }

  it("reads the outputs that follow a page ending in unanswered calls", async () => {
    const { asked, read } = reader();
    const messages = chat.slice(0, 3);
    const page = await api.withNeighbours({ messages, total: 8 }, 0, read);
    expect(asked).toEqual([[3, api.NEIGHBOURS]]);
    expect(page.after?.map((message) => message.id)).toEqual(["r1", "r2"]);
    expect(page.before).toBeUndefined();
    // What the page itself holds is untouched.
    expect(page.messages).toBe(messages);
    expect(page.total).toBe(8);
  });

  it("reads the calls before a page that opens with their outputs", async () => {
    const { asked, read } = reader();
    const page = await api.withNeighbours({ messages: chat.slice(3, 6) }, 3, read);
    expect(asked).toEqual([[0, 3]]);
    expect(page.before?.map((message) => message.id)).toEqual(["c1", "c2"]);
    expect(page.after).toBeUndefined();
  });

  it("asks for nothing when every call on the page has its output", async () => {
    const { asked, read } = reader();
    const page = await api.withNeighbours({ messages: chat.slice(5, 8) }, 5, read);
    expect(asked).toEqual([]);
    expect(page).toEqual({ messages: chat.slice(5, 8) });
  });

  it("pairs nothing when the neighbours are not the other half", async () => {
    const page = await api.withNeighbours({ messages: chat.slice(0, 3) }, 0, async () =>
      chat.slice(5, 8),
    );
    expect(page.after).toBeUndefined();
  });
});
