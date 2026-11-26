import { describe, expect, it } from "vitest";
import { glanceMarkdown, queryTerms, speakerPicture } from "./glance";

describe("a glance at a chat", () => {
  const you = { id: "q", role: "user", content: "Why does checkout time out?\nIt worked yesterday." };
  const agent = { id: "a", role: "assistant", content: "The checkout request waits on three retries." };
  const picture = (who: { kind: string; agent?: string }) => `![](${who.kind === "you" ? "you" : who.agent}.svg)`;

  it("names each speaker on a line with a picture, quotes the reader, and labels the match", () => {
    const shown = glanceMarkdown(
      [
        { message: you, match: false },
        { message: agent, match: true },
      ],
      { agent: "claude-code", matched: true, terms: ["checkout"], picture },
    );
    expect(shown).toBe(
      [
        "![](you.svg) **You**",
        "> Why does checkout time out?\n> It worked yesterday.",
        "![](claude-code.svg) **Claude Code** · `Match`",
        "The **checkout** request waits on three retries.",
      ].join("\n\n"),
    );
  });

  it("marks nothing in a recent chat, which was not opened at a match", () => {
    const shown = glanceMarkdown([{ message: agent, match: true }], {
      agent: "codex",
      matched: false,
      terms: ["checkout"],
    });
    expect(shown).toBe("**Codex**\n\nThe checkout request waits on three retries.");
  });

  it("keeps chat text from loading pictures or leaving its quote", () => {
    const hostile = { id: "h", role: "user", content: "![x](https://example.com/p.png)\n\n<img src=x>\n```\nopen" };
    const shown = glanceMarkdown([{ message: hostile, match: false }], { agent: "", matched: true, picture });
    expect(shown).toBe(
      "![](you.svg) **You**\n\n> \\![x](https://example.com/p.png)\n>\n> &lt;img src=x>\n> ```\n> open\n> ```",
    );
  });

  it("caps each message, the match longer than its neighbours, and shows a matched step literally", () => {
    const long = { id: "l", role: "assistant", content: "a".repeat(5000) };
    const cut = (match: boolean) =>
      glanceMarkdown([{ message: long, match }], { agent: "", matched: true }).split("\n\n")[1];
    expect(cut(false)).toBe(`${"a".repeat(400)}…`);
    expect(cut(true)).toBe(`${"a".repeat(1200)}…`);
    const step = { id: "s", role: "assistant", content: "Tool Bash: ![x](https://example.com/p.png)", step: true };
    expect(glanceMarkdown([{ message: step, match: true }], { agent: "", matched: true })).toBe(
      "**Assistant** · `Match`\n\n```text\nTool Bash: ![x](https://example.com/p.png)\n```",
    );
  });

  it("takes the words to mark from what was typed", () => {
    expect(queryTerms("The checkout, and RETRY retry of it")).toEqual(["checkout", "retry"]);
    expect(queryTerms("")).toEqual([]);
  });

  it("draws a speaker from the assets folder, for the appearance, at a line's height", () => {
    expect(speakerPicture({ kind: "you" }, "/ext/My Assets", false)).toBe(
      "![](file:///ext/My%20Assets/agent-tiles/you-light.svg?raycast-width=16&raycast-height=16)",
    );
    expect(speakerPicture({ kind: "agent", agent: "codex" }, "/ext/assets", true)).toBe(
      "![](file:///ext/assets/agent-tiles/openai-dark.svg?raycast-width=16&raycast-height=16)",
    );
    expect(speakerPicture({ kind: "agent", agent: "unknown" }, "/ext/assets", true)).toBe("");
  });
});
