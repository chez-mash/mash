import { describe, expect, it } from "vitest";
import { renderMarkdown } from "./markdown.ts";

const strings = {
  tip: "Not sure where to look?",
  tipSearch: "Search the docs",
  copy: "Copy",
  copied: "Copied",
  copyPrompt: "Copy the prompt for your agent",
};
const render = (body: string, page: string) =>
  renderMarkdown(body, "docs/example.md", new Map(), { page, strings });

describe("website page layouts", () => {
  it("titles a code block with the bold line before it, and leaves other bold lines", () => {
    const result = render("**Linux**\n\n```sh\nmash dash\n```\n\n**Note** text.\n", "search");
    expect(result).toContain('```sh title="Linux"');
    expect(result).not.toContain("**Linux**");
    expect(result).toContain("**Note** text.");
  });

  it("moves the quick start's note under the block it qualifies", () => {
    const result = render("**macOS**\n\nNeeds a session.\n\n```sh\ncurl x\n```\n", "quick-start");
    expect(result.indexOf("curl x")).toBeLessThan(result.indexOf("Needs a session."));
    expect(result).toContain('<div class="mash-note">');
  });

  it("draws READ and WRITE in bold as pills", () => {
    expect(render("**READ** tools never change it.", "tools")).toContain(
      '<span class="mash-badge read">READ</span>',
    );
  });

  it("gives each group of guides a card and the last one the full width", () => {
    const result = render(
      "Lead.\n\n## Start here\n\n- [A](a.md)\n\n## Using Mash\n\n### Search\n\n- [B](b.md)\n\n### Reference\n\n- [C](c.md)\n",
      "guides",
    );
    expect(result.match(/<section class="mash-guide/g)).toHaveLength(3);
    expect(result).toContain('<section class="mash-guide wide">');
    expect(result).toContain('<div class="mash-guide-group">');
    expect(result).toContain(
      '<button type="button" class="mash-search-tip" data-mash-search>Search the docs<kbd>Ctrl K</kbd></button>',
    );
  });

  it("turns a quoted prompt into a card whose button copies it", () => {
    const result = render("## Ask your agent\n\n> Read the\n> setup.\n", "quick-start");
    expect(result).toContain('data-copy="Read the setup."');
    expect(result).not.toContain("> Read");
  });

  it("puts each agent's mark beside its name and a plain yes in a pill", () => {
    const result = render(
      "| Agent | Reads | Connect |\n| --- | --- | --- |\n| Codex | Chats | Yes |\n| Other | Chats | Yes |\n",
      "supported-agents",
    );
    expect(result).toContain('data-agent="codex"');
    expect(result.match(/mash-pill ok/g)).toHaveLength(2);
    expect(result.match(/data-agent=/g)).toHaveLength(1);
  });

  it("puts a system's mark before a table row that starts with its name", () => {
    const result = render(
      "| Computer | Download |\n| --- | --- |\n| Mac with Apple silicon | [a](a) |\n| Linux ARM64 | [b](b) |\n| Other | [c](c) |\n",
      "downloads",
    );
    expect(result).toContain('data-system="macos"');
    expect(result).toContain('data-system="linux"');
    expect(result.match(/data-system=/g)).toHaveLength(2);
  });
});
