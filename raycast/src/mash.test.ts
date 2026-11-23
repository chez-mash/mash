import { describe, expect, it } from "vitest";
import {
  MashError,
  appLocations,
  chatLink,
  findMash,
  hasDesktopApp,
  installLocations,
  agentLogo,
  literal,
  liveSearchArguments,
  parseChats,
  parseSearch,
  parseView,
  recentChatsArguments,
  rendered,
  searchCommand,
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
    expect(installLocations(home)).toContain("/Users/ada/Applications/Mash.app/Contents/MacOS/mash");
  });
  it("finds the desktop app in either Applications folder", () => {
    expect(appLocations(home)).toEqual(["/Applications/Mash.app", "/Users/ada/Applications/Mash.app"]);
    expect(hasDesktopApp(home, (path) => path === "/Users/ada/Applications/Mash.app")).toBe(true);
    expect(hasDesktopApp(home, () => false)).toBe(false);
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
        matched: true,
        title: "Fix the deploy retry loop",
        agent: "claude-code",
        project: "/work/infra",
        updatedAt: new Date(1_794_690_968_860),
        passage: "the deploy step retries forever",
      },
    ]);
  });

  it("lists this machine's recent chats, each at its first message", () => {
    expect(recentChatsArguments()).toEqual(["list-chats", "--local", "--limit", "20"]);
    const chats = parseChats(
      JSON.stringify({
        chats: [
          {
            chat_id: "c9",
            provider: "codex",
            title: "Rotate the logs",
            project: "/work/ops",
            updated_at: 1_795_409_863_721,
            lookup: { view: { id: "first" } },
          },
          { chat_id: "c10", title: "Nothing to read" },
        ],
        next_offset: null,
      }),
    );
    expect(chats).toEqual([
      {
        chatId: "c9",
        messageId: "first",
        matched: false,
        title: "Rotate the logs",
        agent: "codex",
        project: "/work/ops",
        updatedAt: new Date(1_795_409_863_721),
        passage: "",
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
    expect(chatLink({ chatId: "a/b", messageId: "m 1", matched: true })).toBe("mash://chat/a%2Fb?message=m+1");
    expect(chatLink({ chatId: "a/b", messageId: "first", matched: false })).toBe("mash://chat/a%2Fb");
    expect(searchLink(" checkout outage ")).toBe("mash://search?q=checkout+outage");
  });

  it("quotes the words of the terminal search command", () => {
    expect(searchCommand(" deploy retry ")).toBe("mash search 'deploy retry'");
    expect(searchCommand("don't")).toBe("mash search 'don'\\''t'");
  });

  it("shows history as literal text, never as Markdown that loads or links", () => {
    const shown = literal("![x](https://example.com/p.png) ``` escape");
    expect(shown.startsWith("````text\n")).toBe(true);
    expect(shown.endsWith("\n````")).toBe(true);
    expect(literal("abcdef", 3)).toBe("```text\nabc…\n```");
  });

  it("renders history as Markdown without loading images or HTML", () => {
    expect(rendered("**bold** and `Vec<T>` stay")).toBe("**bold** and `Vec<T>` stay");
    expect(rendered("see ![x](https://example.com/p.png)")).toBe("see \\![x](https://example.com/p.png)");
    expect(rendered('<img src="https://example.com/p.png"> a < b')).toBe(
      '&lt;img src="https://example.com/p.png"> a < b',
    );
    expect(rendered("```html\n<img src=x>\n```\n<b>")).toBe("```html\n<img src=x>\n```\n&lt;b>");
    expect(rendered("abcdef", 3)).toBe("abc…");
  });

  it("knows each agent's logo, and nothing for an unknown one", () => {
    expect(agentLogo("claude-code")).toEqual({ file: "claude.svg", tint: "#d97757" });
    expect(agentLogo("codex")).toEqual({ file: "openai.svg" });
    expect(agentLogo("someone-else")).toBeUndefined();
  });
});
