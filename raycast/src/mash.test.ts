import { describe, expect, it } from "vitest";
import {
  MashError,
  chatLink,
  findMash,
  installLocations,
  literal,
  liveSearchArguments,
  parseSearch,
  parseView,
  searchLink,
} from "./mash";

describe("finding mash", () => {
  const home = "/Users/ada";
  it("uses the configured location only when it exists", () => {
    expect(findMash("/opt/mash", home, () => true)).toBe("/opt/mash");
    expect(findMash("/opt/mash", home, () => false)).toBeUndefined();
  });
  it("otherwise tries the install locations in order", () => {
    const installed = new Set(["/opt/homebrew/bin/mash", "/Applications/Mash.app/Contents/MacOS/mash"]);
    expect(findMash(undefined, home, (path) => installed.has(path))).toBe("/opt/homebrew/bin/mash");
    expect(findMash("  ", home, () => false)).toBeUndefined();
    expect(installLocations(home)[0]).toBe("/Users/ada/.local/bin/mash");
  });
});

describe("reading Mash's answers", () => {
  it("asks for a live search of this machine", () => {
    expect(liveSearchArguments("deplo")).toEqual(["search", "deplo", "--live", "--limit", "20"]);
  });

  it("reads each chat at its best matching message", () => {
    const hits = parseSearch(
      JSON.stringify({
        results: [
          {
            chat_id: "c1",
            provider: "claude-code",
            title: "  Fix the deploy retry loop ",
            project: "/work/infra",
            updated_at: 1_794_690_968_860,
            matches: [{ message_id: "m3", snippet: " the deploy step retries forever " }],
          },
          { chat_id: "c2", title: "No match recorded", matches: [] },
        ],
      }),
    );
    expect(hits).toEqual([
      {
        chatId: "c1",
        messageId: "m3",
        title: "Fix the deploy retry loop",
        agent: "claude-code",
        project: "/work/infra",
        updatedAt: new Date(1_794_690_968_860),
        passage: "the deploy step retries forever",
      },
    ]);
  });

  it("passes on Mash's own message when it fails", () => {
    const failed = JSON.stringify({
      error: { kind: "unavailable", message: "Start Qdrant with `mash qdrant install`." },
    });
    expect(() => parseSearch(failed)).toThrow(new MashError("Start Qdrant with `mash qdrant install`."));
    expect(() => parseSearch("not json")).toThrow(MashError);
  });

  it("reads the messages around a match", () => {
    const view = JSON.stringify({ messages: [{ id: "m2", role: "user", content: "why?" }, { id: "m3" }] });
    expect(parseView(view)).toEqual([
      { id: "m2", role: "user", content: "why?" },
      { id: "m3", role: "message", content: "" },
    ]);
  });
});

describe("links and untrusted text", () => {
  it("opens a chat or the full search in the desktop app", () => {
    expect(chatLink({ chatId: "a/b", messageId: "m 1" })).toBe("mash://chat/a%2Fb?message=m+1");
    expect(searchLink(" checkout outage ")).toBe("mash://search?q=checkout+outage");
  });

  it("shows history as literal text, never as Markdown that loads or links", () => {
    const shown = literal("![x](https://example.com/p.png) ``` escape");
    expect(shown.startsWith("````text\n")).toBe(true);
    expect(shown.endsWith("\n````")).toBe(true);
    expect(literal("abcdef", 3)).toBe("```text\nabc…\n```");
  });
});
