import { describe, expect, it } from "vitest";
import {
  agentLogo,
  byActivity,
  chatPosition,
  type Hit,
  lastActive,
  messageCount,
  appLocations,
  chatLink,
  failures,
  findMash,
  hasDesktopApp,
  installLocations,
  literal,
  liveSearchArguments,
  MashError,
  parseChats,
  parseSearch,
  mashEnvironment,
  parseStreamLine,
  resumeCommand,
  rows,
  viewArguments,
  glance,
  copies,
  primaryAction,
  quoted,
  speaker,
  viewChoice,
  parseView,
  recentChatsArguments,
  rendered,
  searchCommand,
  searchLink,
  settle,
  stillComing,
  streamArguments,
} from "./mash";

describe("finding mash", () => {
  const home = "/Users/ada";
  it("uses the configured location only when it exists", () => {
    expect(findMash("/opt/mash", home, () => true)).toBe("/opt/mash");
    expect(findMash("/opt/mash", home, () => false)).toBeUndefined();
  });
  it("reads a chosen location under the home folder", () => {
    const installed = (path: string) => path === "/Users/ada/tools/mash";
    expect(findMash("~/tools/mash", home, installed)).toBe("/Users/ada/tools/mash");
    expect(findMash(" /Users/ada/tools/mash ", home, installed)).toBe("/Users/ada/tools/mash");
    expect(findMash("~/elsewhere/mash", home, installed)).toBeUndefined();
    expect(findMash("~other/mash", home, () => false)).toBeUndefined();
  });
  it("otherwise tries the install locations in order", () => {
    const installed = new Set(["/opt/homebrew/bin/mash", "/Applications/Mash.app/Contents/MacOS/mash"]);
    expect(findMash(undefined, home, (path) => installed.has(path))).toBe("/opt/homebrew/bin/mash");
    expect(findMash("  ", home, () => false)).toBeUndefined();
    expect(findMash("", home, (path) => installed.has(path))).toBe("/opt/homebrew/bin/mash");
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
        matches: [{ messageId: "m3", passage: "the deploy step retries forever" }],
      },
    ]);
  });

  it("lists this machine's recent chats, each at its first message", () => {
    expect(recentChatsArguments()).toEqual(["list-chats", "--local", "--exclude-subagents", "--limit", "20"]);
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

  it("marks tool calls, output and thinking as steps, and keeps the person's own words", () => {
    const view = JSON.stringify({
      messages: [
        {
          id: "t1",
          role: "assistant",
          content: 'Tool Bash: {"command":"ls"}',
          dialogue: "",
          tools: { tool_calls: [] },
        },
        { id: "t2", role: "user", content: "(Bash completed with no output)", tools: { tool_results: [] } },
        { id: "w1", role: "user", content: "<context>…</context>\nship it", dialogue: "ship it" },
        { id: "a1", role: "assistant", content: "Shipped." },
      ],
    });
    expect(parseView(view).messages).toEqual([
      { id: "t1", role: "assistant", content: 'Tool Bash: {"command":"ls"}', step: true },
      { id: "t2", role: "user", content: "(Bash completed with no output)", step: true },
      { id: "w1", role: "user", content: "ship it" },
      { id: "a1", role: "assistant", content: "Shipped." },
    ]);
  });

  it("reads the messages around a match", () => {
    const view = JSON.stringify({ messages: [{ id: "m2", role: "user", content: "why?" }, { id: "m3" }] });
    expect(parseView(view)).toEqual({
      messages: [
        { id: "m2", role: "user", content: "why?" },
        { id: "m3", role: "message", content: "" },
      ],
    });
  });

  it("reads the chat's message count and each message's place in it", () => {
    const view = JSON.stringify({
      message_count: 14,
      messages: [
        { id: "m3", role: "user", content: "why?", ordinal: 2 },
        { id: "m4", role: "assistant", content: "because", ordinal: 3 },
        { id: "bad", role: "user", content: "x", ordinal: -1 },
      ],
    });
    expect(parseView(view)).toEqual({
      total: 14,
      messages: [
        { id: "m3", role: "user", content: "why?", ordinal: 2 },
        { id: "m4", role: "assistant", content: "because", ordinal: 3 },
        { id: "bad", role: "user", content: "x" },
      ],
    });
  });
});

describe("subagent chats", () => {
  const chat = (id: string, subagent?: object) => ({
    chat_id: id,
    provider: "claude-code",
    title: id,
    lookup: { view: { id: `${id}-m` } },
    ...(subagent ? { subagent } : {}),
  });
  const fork = { parent_chat_id: "p", label: "Fix the build", role: "fork", depth: 1 };

  it("marks search matches in subagent chats, and only those", () => {
    const row = (id: string, subagent?: object) => ({
      chat_id: id,
      provider: "claude-code",
      title: id,
      matches: [{ message_id: `${id}-m`, snippet: "x" }],
      ...(subagent ? { subagent } : {}),
    });
    const hits = parseSearch(JSON.stringify({ results: [row("mine"), row("sub", fork)] }));
    expect(hits.map((hit) => [hit.chatId, hit.subagent ?? false])).toEqual([
      ["mine", false],
      ["sub", true],
    ]);
  });

  it("names a subagent chat by its task label, not its prompt", () => {
    const prompted = { ...chat("sub", fork), title: "<fork-boilerplate> You are a worker fork." };
    const unlabelled = { ...chat("bare", { parent_chat_id: "p" }), title: "Its own prompt" };
    const [labelled, bare] = parseChats(JSON.stringify({ chats: [prompted, unlabelled] }));
    expect(labelled?.title).toBe("Fix the build");
    expect(bare?.title).toBe("Its own prompt");
  });

  it("does not take a null subagent field for a subagent", () => {
    const [hit] = parseChats(JSON.stringify({ chats: [{ ...chat("mine"), subagent: null }] }));
    expect(hit?.subagent).toBeUndefined();
  });
});

describe("links and untrusted text", () => {
  it("opens a chat or the full search in the desktop app", () => {
    expect(chatLink({ chatId: "a/b", messageId: "m 1", matched: true })).toBe("mash://chat/a%2Fb?message=m+1");
    expect(chatLink({ chatId: "a/b", messageId: "first", matched: false })).toBe("mash://chat/a%2Fb");
    // Another machine's chat names it, as the desktop app's links do.
    expect(chatLink({ chatId: "c", messageId: "m", matched: true, machine: "build box" })).toBe(
      "mash://chat/c?node=build+box&message=m",
    );
    expect(chatLink({ chatId: "c", messageId: "first", matched: false, machine: "xenia" })).toBe(
      "mash://chat/c?node=xenia",
    );
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
    // An unmatched backtick is text, so the image after it is still defused.
    expect(rendered("a ` then ![x](https://example.com/p.png)")).toBe("a ` then \\![x](https://example.com/p.png)");
    expect(rendered("``a` ![x](u)`` ![y](v)")).toBe("``a` ![x](u)`` \\![y](v)");
  });

  it("closes a code block left open, so what follows the history is not code", () => {
    expect(rendered("```ts\nconst a = 1;")).toBe("```ts\nconst a = 1;\n```");
    expect(rendered("~~~\nabcdef", 6)).toBe("~~~\nab…\n~~~");
    expect(rendered("```ts\nconst a = 1;\n```\nafter")).toBe("```ts\nconst a = 1;\n```\nafter");
  });

  it("does not cut a character pair in half", () => {
    expect(rendered("ab😀cd", 3)).toBe("ab…");
    expect(literal("ab😀cd", 4)).toBe("```text\nab😀…\n```");
  });

  it("sets the searched words in bold, as whole words in any case", () => {
    const terms = ["checkout", "retry"];
    expect(rendered("Checkout fails; retry the checkouts", 4000, terms)).toBe(
      "**Checkout** fails; **retry** the checkouts",
    );
    expect(rendered("no terms, no bold: checkout")).toBe("no terms, no bold: checkout");
    // A term is matched as written, never as a pattern.
    expect(rendered("a.b and axb", 4000, ["a.b"])).toBe("**a.b** and axb");
  });

  it("adds no bold inside code, addresses, link targets or existing emphasis", () => {
    const terms = ["checkout"];
    expect(rendered("run `checkout --fast` on checkout", 4000, terms)).toBe("run `checkout --fast` on **checkout**");
    expect(rendered("```sh\ncheckout\n```\n    checkout indented\ncheckout", 4000, terms)).toBe(
      "```sh\ncheckout\n```\n    checkout indented\n**checkout**",
    );
    expect(rendered("[checkout docs](https://example.com/checkout) at https://example.com/checkout", 4000, terms)).toBe(
      "[**checkout** docs](https://example.com/checkout) at https://example.com/checkout",
    );
    expect(rendered("[docs][checkout]\n\n[checkout]: /docs/checkout", 4000, terms)).toBe(
      "[docs][checkout]\n\n[checkout]: /docs/checkout",
    );
    expect(rendered("mail checkout@example.com or www.example.com/checkout", 4000, terms)).toBe(
      "mail checkout@example.com or www.example.com/checkout",
    );
    expect(rendered("**the checkout page** *checkout* _checkout_ ~~checkout~~", 4000, terms)).toBe(
      "**the checkout page** *checkout* _checkout_ ~~checkout~~",
    );
  });

  it("still defuses images and HTML around bold words, and never bolds inside an entity", () => {
    expect(rendered("checkout ![checkout](https://example.com/checkout.png)", 4000, ["checkout"])).toBe(
      "**checkout** \\![**checkout**](https://example.com/checkout.png)",
    );
    expect(rendered('<img src="x"> lt <b>', 4000, ["lt", "img"])).toBe('&lt;**img** src="x"> **lt** &lt;b>');
  });

  it("quotes every line, so a quoted message cannot end its own quote", () => {
    expect(quoted("one\n\n```\ncode\n```\ntwo")).toBe("> one\n>\n> ```\n> code\n> ```\n> two");
  });

  it("knows each agent's logo, and nothing for an unknown one", () => {
    expect(agentLogo("claude-code")).toEqual({ file: "claude.svg", tint: "#d97757" });
    expect(agentLogo("codex")).toEqual({ file: "openai.svg" });
    expect(agentLogo("someone-else")).toBeUndefined();
  });
});

describe("federated search", () => {
  const row = (chat: string, matchedBy: string[], node?: string) => ({
    chat_id: chat,
    provider: "codex",
    title: chat,
    ...(node ? { node_id: node } : {}),
    matches: [{ message_id: `${chat}-m`, snippet: "words" }],
    matched_by: matchedBy,
  });

  it("asks every source in the scope while typing", () => {
    expect(streamArguments("invoice exports", "all")).toEqual([
      "search",
      "invoice exports",
      "--live",
      "--stream",
      "--limit",
      "20",
      "--scope",
      "all",
    ]);
  });

  it("leaves subagent chats to Mash to leave out, unless the reader asks for them", () => {
    expect(streamArguments("invoice exports", "local", true).slice(-3)).toEqual([
      "--scope",
      "local",
      "--include-subagents",
    ]);
  });

  it("reads one line: the merged list, which chats only meaning found, and each source", () => {
    const update = parseStreamLine(
      JSON.stringify({
        done: false,
        results: [row("a", ["words", "meaning"]), row("b", ["meaning"]), row("c", ["machines"], "xenia")],
        sources: [
          { label: "laptop", state: "done", count: 2 },
          { label: "xenia", state: "searching" },
        ],
      }),
    );
    expect(update.hits.map((hit) => [hit.chatId, hit.byMeaning ?? false, hit.machine])).toEqual([
      ["a", false, undefined],
      ["b", true, undefined],
      ["c", false, "xenia"],
    ]);
    expect(update.done).toBe(false);
    expect(stillComing(update.sources)).toBe("Searching xenia…");
    expect(() => parseStreamLine('{"error":{"message":"no index"}}')).toThrow("no index");
  });

  it("keeps rows in place once frozen and adds new ones below", () => {
    const hits = (...ids: string[]) =>
      ids.map((chatId) => ({ ...parseStreamLine(JSON.stringify({ results: [row(chatId, ["words"])] })).hits[0]! }));
    const shown = hits("a", "b");
    expect(settle(shown, hits("c", "a", "b"), false).map((hit) => hit.chatId)).toEqual(["c", "a", "b"]);
    expect(settle(shown, hits("c", "a", "b"), true).map((hit) => hit.chatId)).toEqual(["a", "b", "c"]);
  });

  it("names sources that failed so the reader can try again", () => {
    expect(
      failures([
        { label: "build-box", state: "failed", timedOut: true },
        { label: "xenia", state: "failed", message: "offline" },
        { label: "laptop", state: "done" },
      ]),
    ).toBe("build-box didn’t answer · xenia: offline");
  });
});

describe("chats or messages", () => {
  const result = {
    chat_id: "c1",
    provider: "codex",
    title: "Rotate the logs",
    matches: [
      { message_id: "m1", snippet: "rotate daily" },
      { message_id: "m2", snippet: "keep a week" },
    ],
  };
  const other = { chat_id: "c2", provider: "codex", title: "Other", matches: [{ message_id: "m3", snippet: "logs" }] };

  it("keeps every match a chat result carries, the best first", () => {
    const [hit] = parseSearch(JSON.stringify({ results: [result] }));
    expect(hit?.messageId).toBe("m1");
    expect(hit?.matches?.map((match) => match.messageId)).toEqual(["m1", "m2"]);
  });

  it("lists one row per chat, or one per matching message in the chats' order", () => {
    const hits = parseSearch(JSON.stringify({ results: [result, other] }));
    expect(rows(hits, "chats").map((hit) => hit.chatId)).toEqual(["c1", "c2"]);
    expect(rows(hits, "messages").map((hit) => [hit.chatId, hit.messageId, hit.passage])).toEqual([
      ["c1", "m1", "rotate daily"],
      ["c1", "m2", "keep a week"],
      ["c2", "m3", "logs"],
    ]);
  });

  it("reads the dropdown's choice, falling back to chats on every machine", () => {
    expect(viewChoice("messages:local")).toEqual({ show: "messages", scope: "local" });
    expect(viewChoice("chats:all")).toEqual({ show: "chats", scope: "all" });
    expect(viewChoice("sideways")).toEqual({ show: "chats", scope: "all" });
  });
});

describe("resuming a chat in its agent", () => {
  const chat = { agent: "claude-code", chatId: "3cd8e223-aea3", project: "/work/it's here" };

  it("resumes Claude Code and Codex sessions in the folder they ran in", () => {
    expect(resumeCommand(chat)).toBe("cd '/work/it'\\''s here' && claude --resume 3cd8e223-aea3");
    expect(resumeCommand({ ...chat, agent: "codex", project: "" })).toBe("codex resume 3cd8e223-aea3");
  });

  it("offers nothing it could not resume here", () => {
    expect(resumeCommand({ ...chat, subagent: true })).toBeUndefined();
    expect(resumeCommand({ ...chat, machine: "xenia" })).toBeUndefined();
    expect(resumeCommand({ ...chat, agent: "cursor" })).toBeUndefined();
    expect(resumeCommand({ ...chat, chatId: "x; rm -rf ~" })).toBeUndefined();
  });
});

describe("the environment mash runs in", () => {
  it("names Raycast as the caller for Mash's usage log, keeping the rest", () => {
    expect(mashEnvironment({ PATH: "/usr/bin", HOME: "/Users/me" })).toEqual({
      PATH: "/usr/bin",
      HOME: "/Users/me",
      MASH_CLIENT: "raycast",
    });
  });
});

describe("reading a chat's messages", () => {
  const said = (id: string, role = "assistant") => ({ id, role, content: id });
  const step = (id: string) => ({ id, role: "assistant", content: `Tool ${id}`, step: true });

  it("shows the match with the message said before and after it, passing over steps", () => {
    const messages = [
      said("q0", "user"),
      said("q1", "user"),
      step("s2"),
      step("s3"),
      said("m"),
      step("s4"),
      said("q2", "user"),
      said("a2"),
    ];
    expect(glance(messages, "m")).toEqual([
      { message: messages[1], match: false },
      { message: messages[4], match: true },
      { message: messages[6], match: false },
    ]);
  });

  it("still shows a match that is itself a step, and a chat's first or last message alone on that side", () => {
    const messages = [step("s0"), said("q", "user"), step("m"), step("s1"), said("a")];
    expect(glance(messages, "m").map((glimpse) => [glimpse.message.id, glimpse.match])).toEqual([
      ["q", false],
      ["m", true],
      ["a", false],
    ]);
    expect(glance(messages, "q").map((glimpse) => glimpse.message.id)).toEqual(["q", "a"]);
    expect(glance(messages, "a").map((glimpse) => glimpse.message.id)).toEqual(["q", "a"]);
  });

  it("starts at the first thing said when the message is not among those read, marking no match", () => {
    const messages = [step("s0"), said("q", "user"), said("a")];
    expect(glance(messages, "gone")).toEqual([
      { message: messages[1], match: false },
      { message: messages[2], match: false },
    ]);
    expect(glance([step("s0")], "gone")).toEqual([]);
  });

  it("names who spoke", () => {
    expect(speaker("user", "codex")).toBe("You");
    expect(speaker("assistant", "claude-code")).toBe("Claude Code");
    expect(speaker("system", "codex")).toBe("System");
    expect(speaker("assistant", "")).toBe("Assistant");
  });

  it("reads another machine's message from the shared index, by that machine", () => {
    expect(viewArguments({ messageId: "m1" })).toEqual(["view", "m1", "--context", "12"]);
    expect(viewArguments({ messageId: "m1", machine: "dev-macbook" })).toEqual([
      "view",
      "m1",
      "--context",
      "12",
      "--shared",
      "dev-macbook",
    ]);
  });
});

describe("where the match sits in a chat", () => {
  const at = (id: string, ordinal?: number) => ({
    id,
    role: "assistant",
    content: id,
    ...(ordinal === undefined ? {} : { ordinal }),
  });

  it("says the chat's message count and the match's place", () => {
    const messages = [at("q", 2), at("m", 3), at("a", 4)];
    const position = chatPosition({ total: 14, messages }, glance(messages, "m"));
    expect(position).toEqual({ total: 14, match: 4 });
    expect(messageCount(position)).toBe("14 · match is message 4");
  });

  it("says nothing it was not told", () => {
    const messages = [at("q"), at("m")];
    expect(chatPosition({ messages }, glance(messages, "m"))).toEqual({});
    expect(messageCount({})).toBeUndefined();
    // A count without the match's place is just the count.
    expect(messageCount({ total: 3 })).toBe("3");
  });
});

describe("what a row does without the desktop app", () => {
  const chat = { agent: "claude-code", chatId: "abc-123", project: "/work/shop", passage: "the words" };

  it("opens the chat in Mash when the app is installed, and offers its link to copy", () => {
    expect(primaryAction(chat, true)).toBe("open");
    expect(copies(chat, true)).toEqual(["resume", "link", "passage"]);
  });

  it("otherwise copies the resume command, or the message ID for a chat that cannot be resumed here", () => {
    expect(primaryAction(chat, false)).toBe("resume");
    expect(copies(chat, false)).toEqual(["resume", "id", "passage"]);
    for (const other of [
      { ...chat, agent: "cursor" },
      { ...chat, machine: "dev-macbook" },
      { ...chat, subagent: true },
    ]) {
      expect(primaryAction(other, false)).toBe("id");
      expect(copies(other, false)).toEqual(["id", "passage"]);
    }
    // A recent chat has no matching passage to copy.
    expect(copies({ ...chat, passage: "" }, false)).toEqual(["resume", "id"]);
  });
});

describe("grouping rows by last activity", () => {
  // Wednesday 25 November 2026, 15:00 local time.
  const now = new Date(2026, 10, 25, 15, 0);
  const hit = (chatId: string, updatedAt?: Date): Hit => ({
    chatId,
    messageId: `${chatId}-m`,
    matched: true,
    title: chatId,
    agent: "codex",
    project: "",
    updatedAt,
    passage: "",
  });

  it("groups Today, This week and Earlier, keeping the ranked order within each", () => {
    const hits = [
      hit("old", new Date(2026, 10, 12, 9, 0)),
      hit("morning", new Date(2026, 10, 25, 0, 5)),
      hit("thursday", new Date(2026, 10, 19, 8, 0)),
      hit("monday", new Date(2026, 10, 23, 18, 0)),
      hit("last-night", new Date(2026, 10, 24, 23, 59)),
      hit("week-ago", new Date(2026, 10, 18, 23, 59)),
    ];
    expect(byActivity(hits, now).map((group) => [group.title, group.hits.map((h) => h.chatId)])).toEqual([
      ["Today", ["morning"]],
      ["This week", ["thursday", "monday", "last-night"]],
      ["Earlier", ["old", "week-ago"]],
    ]);
  });

  it("lists a row without a time last, under no heading, and leaves out empty groups", () => {
    const groups = byActivity([hit("undated"), hit("today", new Date(2026, 10, 25, 14, 0))], now);
    expect(groups.map((group) => group.title)).toEqual(["Today", undefined]);
    expect(groups[1]?.hits.map((h) => h.chatId)).toEqual(["undated"]);
    expect(byActivity([], now)).toEqual([]);
  });

  it("says a time today as Today, and any other day in full", () => {
    expect(lastActive(new Date(2026, 10, 25, 10, 12), now)).toMatch(/^Today, /);
    expect(lastActive(new Date(2026, 10, 24, 10, 12), now)).not.toMatch(/^Today/);
  });
});
