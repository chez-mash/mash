import { describe, expect, it } from "vitest";
import { recordFixture } from "../../e2e/record-fixtures";
import * as wire from "./records";
import { kinds, standing, timingLabel } from "./records";
import { trimmed } from "./trimmed";

const parseRecord = (row: unknown) => wire.parseRecord(trimmed(row));
const parseRecordPage = (page: unknown) => wire.parseRecordPage(trimmed(page));

describe("record API boundary", () => {
  it("marks future and expired conclusions at their time boundaries", () => {
    const revision = parseRecord(recordFixture()).revisions[0];
    if (revision?.details.kind !== "fact") throw new Error("Missing fixture fact");
    expect(timingLabel({ ...revision, effectiveAt: 101 }, 100)).toBe("Not yet effective");
    expect(
      timingLabel(
        { ...revision, effectiveAt: 100, details: { ...revision.details, validUntil: 100 } },
        100,
      ),
    ).toBeNull();
    expect(
      timingLabel(
        { ...revision, effectiveAt: null, details: { ...revision.details, validUntil: 99 } },
        100,
      ),
    ).toBe("Expired");
  });
  it("tells a choice the agent made from the person's and from a candidate", () => {
    const fixture = recordFixture("id", "decision");
    const theirs = parseRecord(fixture).revisions[0];
    if (!theirs) throw new Error("Missing fixture decision");
    expect(standing(theirs)).toBe("established");
    const revision = fixture.revisions[0];
    if (!revision) throw new Error("Missing fixture revision");
    const by = (decided_by: string) => ({
      ...revision,
      body: { ...revision.body, details: { ...revision.body.details, decided_by } },
    });
    const made = by("agent");
    const agents = parseRecord({ ...fixture, revisions: [made] }).revisions[0];
    if (!agents) throw new Error("Missing agent decision");
    expect(standing(agents)).toBe("agent decided");
    expect(standing({ ...agents, review: "candidate" })).toBe("unconfirmed");
  });
  it("decodes all four kinds without interpreting archived markup", () => {
    for (const kind of kinds) {
      const fixture = recordFixture("id", kind);
      const record = parseRecord(fixture);
      expect(record.revisions[0]?.details.kind).toBe(kind);
      expect(record.revisions[0]?.evidence[0]?.messageId).toBe("message-1");
      expect(record.node).toBe("");
    }
    const fixture = recordFixture();
    const revision = fixture.revisions[0];
    const parsed = parseRecord({
      ...fixture,
      node_id: "remote/one",
      revisions: [
        { ...revision, body: { ...revision?.body, statement: "<script>untrusted</script>" } },
      ],
    });
    expect(parsed.node).toBe("remote/one");
    expect(parsed.revisions[0]?.statement).toBe("<script>untrusted</script>");
  });
  it("preserves empty shared pages with a continuation cursor", () => {
    expect(
      parseRecordPage({
        records: [],
        next_offset: null,
        next_cursor: { node_id: "remote", offset: 20 },
      }),
    ).toEqual({ records: [], nextOffset: null, nextCursor: { node: "remote", offset: 20 } });
  });
});

describe("decisions by chat", () => {
  const decision = (id: string, chats: string[], state = "accepted") => {
    const fixture = recordFixture(id, "decision");
    const [revision] = fixture.revisions;
    if (!revision) throw new Error("fixture without a revision");
    return parseRecord({
      ...fixture,
      revisions: [
        {
          ...revision,
          body: { ...revision.body, details: { ...revision.body.details, state } },
          evidence: chats.map((chat, index) => ({
            ...revision.evidence[0],
            chat_id: chat,
            message_id: `message-${index}`,
          })),
        },
      ],
    });
  };

  it("counts each decision once for every chat its evidence quotes", () => {
    const counts = wire.decisionsByChat([
      decision("a", ["chat-1", "chat-1"]),
      decision("b", ["chat-1", "chat-2"]),
      parseRecord(recordFixture("c", "fact")),
    ]);
    expect(Object.fromEntries(counts)).toEqual({ "chat-1": 2, "chat-2": 1 });
  });

  it("leaves out withdrawn decisions", () => {
    expect(wire.decisionsByChat([decision("a", ["chat-1"], "withdrawn")]).size).toBe(0);
  });
});

describe("memory source menu", () => {
  it("names this machine only when it has an index of its own", () => {
    expect(wire.memorySources("laptop", false, ["laptop", "desk"], "")).toEqual([
      { value: "", label: "this machine (laptop)" },
      { value: "shared", label: "all shared machines" },
      { value: "node:desk", label: "desk (shared)" },
    ]);
    expect(wire.memorySourceLabel("", false)).toBe("this machine");
  });

  it("in shared-only mode offers the shared machines, its own published copy included", () => {
    const sources = wire.memorySources("laptop", true, ["laptop", "desk"], "");
    expect(sources).toEqual([
      { value: "", label: "all shared machines" },
      { value: "node:laptop", label: "laptop (shared)" },
      { value: "node:desk", label: "desk (shared)" },
    ]);
    expect(sources.some((source) => source.label.includes("this machine"))).toBe(false);
    expect(wire.memorySourceLabel("", true)).toBe("all shared");
    expect(wire.memorySourceLabel("node:desk", true)).toBe("desk (shared)");
  });
});
