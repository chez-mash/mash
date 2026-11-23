import { describe, expect, it } from "vitest";
import { MashError } from "./mash";
import { memoryMarkdown, parseRecords, recordsArguments, scopeLabel } from "./records";

const record = (overrides: Record<string, unknown> = {}) => ({
  id: "record/o/1",
  owner: "o",
  scope: { kind: "repository", identity: "https://github.com/mashium/mash.git" },
  current_revision: 2,
  revisions: [
    {
      revision: 1,
      body: { statement: "Old wording", rationale: "", review: "candidate", details: { kind: "decision" } },
    },
    {
      revision: 2,
      body: {
        statement: "Ship signed Mac builds only",
        rationale: "Gatekeeper blocks unsigned apps",
        review: "established",
        details: { kind: "decision", state: "accepted" },
      },
      recorded_at: 1_795_402_046_418,
      evidence: [
        { chat_id: "c1", message_id: "m1", quote: "only ship signed builds", source_role: "user" },
        { chat_id: "", message_id: "m2", quote: "no chat", source_role: "user" },
      ],
    },
  ],
  ...overrides,
});

describe("reading what Mash remembers", () => {
  it("asks for one page of records", () => {
    expect(recordsArguments(100)).toEqual(["records", "--limit", "100", "--offset", "100"]);
  });

  it("reads each record at its current revision, with its supporting quotes", () => {
    const { memories, next } = parseRecords(JSON.stringify({ records: [record()], next_offset: 100 }));
    expect(next).toBe(100);
    expect(memories).toEqual([
      {
        id: "record/o/1",
        kind: "decision",
        statement: "Ship signed Mac builds only",
        rationale: "Gatekeeper blocks unsigned apps",
        conditions: [],
        scope: "mashium/mash",
        candidate: false,
        recordedAt: new Date(1_795_402_046_418),
        evidence: [{ chatId: "c1", messageId: "m1", quote: "only ship signed builds", role: "user" }],
      },
    ]);
  });

  it("skips records it cannot show, and says what Mash said when it fails", () => {
    const unknown = record({ revisions: [{ revision: 2, body: { statement: "x", details: { kind: "rumour" } } }] });
    expect(parseRecords(JSON.stringify({ records: [unknown] })).memories).toEqual([]);
    expect(() => parseRecords(JSON.stringify({ error: { message: "run mash init" } }))).toThrow("run mash init");
    expect(() => parseRecords("not json")).toThrow(MashError);
  });

  it("names where a record applies the way a reader would", () => {
    expect(scopeLabel({ kind: "repository", identity: "git@github.com:mashium/mash.git" })).toBe("mashium/mash");
    expect(scopeLabel({ kind: "local_project", path: "/home/me/work/site/" })).toBe("site");
    expect(scopeLabel({ kind: "personal" })).toBe("Personal");
    expect(scopeLabel({ kind: "chat", id: "c1" })).toBe("One chat");
  });

  it("shows a record's text safely: an image in a quote is escaped, so nothing loads", () => {
    const [memory] = parseRecords(JSON.stringify({ records: [record()] })).memories;
    if (!memory) throw new Error("no record");
    const markdown = memoryMarkdown({
      ...memory,
      conditions: ["on release builds"],
      evidence: [{ chatId: "c1", messageId: "m1", quote: "see ![x](https://example.com/p.png)", role: "assistant" }],
    });
    expect(markdown).toContain("## Ship signed Mac builds only");
    expect(markdown).toContain("**Why:** Gatekeeper blocks unsigned apps");
    expect(markdown).toContain("- on release builds");
    expect(markdown).toContain("\\![x]");
    expect(markdown).not.toMatch(/(^|[^\\])!\[x\]/);
    expect(markdown).toContain("— the agent");
  });
});
