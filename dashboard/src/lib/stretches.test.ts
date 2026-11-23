import { describe, expect, it } from "vitest";
import type { ChatStretch } from "./api";
import { joinStretches, rereadPlace, rereadUpdate, withLastReplaced } from "./stretches";
import type { Message } from "./types";

function message(id: string, text = id): Message {
  return { id, role: "user", at: "", blocks: [{ type: "text", markdown: text }] };
}

function stretch(start: number, ids: readonly string[], total?: number): ChatStretch {
  return {
    start,
    end: start + ids.length,
    ...(total !== undefined ? { total } : {}),
    page: { messages: ids.map((id) => message(id)) },
  };
}

describe("joining loaded stretches", () => {
  it("has nothing to show before a stretch loads", () => {
    expect(joinStretches([])).toBeUndefined();
  });

  it("reads the stretches as one page, keeping a shared message once", () => {
    const first = stretch(0, ["a", "b"]);
    const last: ChatStretch = {
      ...stretch(2, ["b", "c"], 9),
      page: { messages: [message("b"), message("c")], after: [message("d")] },
    };
    const joined = joinStretches([
      { ...first, page: { ...first.page, before: [message("z")] } },
      last,
    ]);
    expect(joined?.messages.map((message) => message.id)).toEqual(["a", "b", "c"]);
    expect(joined?.total).toBe(9);
    expect(joined?.before?.map((message) => message.id)).toEqual(["z"]);
    expect(joined?.after?.map((message) => message.id)).toEqual(["d"]);
  });

  it("takes the chat's length from the first stretch when the last does not say", () => {
    expect(joinStretches([stretch(0, ["a"], 4), stretch(1, ["b"])])?.total).toBe(4);
    expect(joinStretches([stretch(0, ["a"])])).not.toHaveProperty("total");
  });
});

describe("replacing the last stretch", () => {
  it("keeps the earlier stretches and records the new one by its edges", () => {
    const data = {
      pages: [stretch(0, ["a"]), stretch(1, ["b"])],
      pageParams: [{ at: "offset", offset: 0 } as const, { at: "end" } as const],
    };
    const next = stretch(1, ["b", "c"]);
    expect(withLastReplaced(data, next)).toEqual({
      pages: [data.pages[0], next],
      pageParams: [
        { at: "offset", offset: 0 },
        { at: "offset", offset: 1, limit: 2 },
      ],
    });
  });

  it("leaves a chat with nothing loaded alone", () => {
    expect(withLastReplaced(undefined, stretch(0, ["a"]))).toBeUndefined();
  });

  it("reads a stretch again from where it starts", () => {
    expect(rereadPlace(stretch(5, ["a"]), 50)).toEqual({ at: "offset", offset: 5, limit: 50 });
  });
});

describe("what a reread says", () => {
  const last = stretch(2, ["c", "d"], 4);

  it("sees messages added after the last stretch", () => {
    expect(rereadUpdate([], [], last, stretch(2, ["c", "d", "e"], 5))).toEqual({
      kind: "appended",
      added: 1,
      onPage: true,
    });
  });

  it("counts messages added beyond the stretch by the chat's length", () => {
    expect(rereadUpdate([], [], last, stretch(2, ["c", "d"], 7))).toEqual({
      kind: "appended",
      added: 3,
      onPage: false,
    });
  });

  it("finds nothing new when the stretch reads the same", () => {
    expect(rereadUpdate([], [], last, stretch(2, ["c", "d"], 4))).toEqual({ kind: "same" });
  });

  it("calls the chat rewritten when an earlier stretch changed or did not come back", () => {
    const earlier = [stretch(0, ["a", "b"])];
    const same = stretch(2, ["c", "d"], 4);
    expect(rereadUpdate(earlier, [stretch(0, ["a", "b"])], last, same)).toEqual({ kind: "same" });
    const edited: ChatStretch = {
      ...earlier[0],
      start: 0,
      end: 2,
      page: { messages: [message("a", "edited"), message("b")] },
    };
    expect(rereadUpdate(earlier, [edited], last, same)).toEqual({ kind: "rewritten" });
    expect(rereadUpdate(earlier, [undefined], last, same)).toEqual({ kind: "rewritten" });
  });

  it("calls the chat rewritten when the last stretch's text changed", () => {
    const next: ChatStretch = {
      ...last,
      page: { messages: [message("c", "edited"), message("d")] },
    };
    expect(rereadUpdate([], [], last, next)).toEqual({ kind: "rewritten" });
  });
});
