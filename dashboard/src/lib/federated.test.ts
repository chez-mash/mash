import { describe, expect, it } from "vitest";
import { parseUpdate, type SourceState, scopeParam, settle, statusLine } from "./federated";

const key = (row: string) => row;

describe("settle then freeze", () => {
  it("shows a newer ranking as it is until the list freezes", () => {
    expect(settle(["a", "b"], ["c", "a", "b"], false, key)).toEqual(["c", "a", "b"]);
  });

  it("keeps rows already shown in place once frozen, adding new ones below in rank order", () => {
    // A late source ranked `c` first and `d` third; `a` and `b` stay put.
    expect(settle(["a", "b"], ["c", "a", "d", "b"], true, key)).toEqual(["a", "b", "c", "d"]);
  });

  it("never shows a chat twice", () => {
    const rows = settle(["a", "b"], ["b", "a", "b"], true, key);
    expect(new Set(rows).size).toBe(rows.length);
  });

  it("keeps a shown row a newer ranking dropped, so nothing vanishes under the pointer", () => {
    expect(settle(["a", "b"], ["b"], true, key)).toEqual(["a", "b"]);
  });
});

const source = (state: Partial<SourceState> & Pick<SourceState, "state">): SourceState => ({
  source: "machines",
  label: "xenia",
  ...state,
});

describe("status line", () => {
  it("says what is still coming", () => {
    const line = statusLine(
      [source({ state: "searching" }), source({ source: "words", label: "laptop", state: "done" })],
      3,
      false,
    );
    expect(line).toEqual({ text: "3 results so far · Searching xenia…", retry: false });
  });

  it("names a source that did not answer and offers to retry", () => {
    const line = statusLine([source({ state: "failed", timedOut: true })], 2, true);
    expect(line).toEqual({ text: "xenia didn't answer", retry: true });
    expect(statusLine([source({ state: "failed", message: "offline" })], 0, true).text).toBe(
      "xenia: offline",
    );
  });

  it("is empty when everything answered or was not asked", () => {
    expect(statusLine([source({ state: "skipped", reason: "x" })], 4, true).text).toBe("");
  });
});

describe("stream lines", () => {
  const row = (chat: string, matchedBy: string[], node?: string) => ({
    chat_id: chat,
    provider: "codex",
    score: 1,
    title: chat,
    ...(node ? { node_id: node } : {}),
    matches: [
      {
        message_id: `${chat}-m`,
        snippet: "the words",
        fusion_score: 1,
        role: "user",
        copies: [],
        projection_span: { start_char: 0, end_char: 3 },
      },
    ],
    matched_by: matchedBy,
  });

  it("tags only chats found by meaning alone, and keeps the machine", () => {
    const update = parseUpdate(
      {
        query: "q",
        local_node: "laptop",
        done: false,
        results: [
          row("a", ["words", "meaning"]),
          row("b", ["meaning"]),
          row("c", ["machines"], "xenia"),
        ],
        sources: [{ source: "machines", label: "xenia", state: "searching" }],
      },
      "laptop",
    );
    expect(update.hits.map((hit) => [hit.chat.id, hit.byMeaning, hit.chat.machine])).toEqual([
      ["a", false, "laptop"],
      ["b", true, "laptop"],
      ["c", false, "xenia"],
    ]);
    expect(update.sources[0]?.state).toBe("searching");
  });
});

it("asks the stream for the viewing scope", () => {
  expect(scopeParam({ kind: "all" })).toBe("all");
  expect(scopeParam({ kind: "local" })).toBe("local");
  expect(scopeParam({ kind: "node", node: "xenia" })).toBe("node:xenia");
});
