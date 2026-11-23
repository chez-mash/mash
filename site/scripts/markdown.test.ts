import { describe, expect, it } from "vitest";
import { renderMarkdown } from "./markdown.ts";

const routes = new Map([
  ["README.md", "/quick-start/"],
  ["docs/search.md", "/search/"],
]);
describe("repository Markdown on the website", () => {
  it("keeps navigation and anchors on the site, and source links on GitHub", () => {
    const result = renderMarkdown(
      "[Search](search.md#filters) [Start](../README.md#quick-start) [Code](../app/main.rs)",
      "docs/installation.md",
      routes,
    );
    expect(result).toContain("](/search/#filters)");
    expect(result).toContain("](/quick-start/#quick-start)");
    expect(result).toContain("](https://github.com/mashium/mash/blob/main/app/main.rs)");
  });
  it("resolves reference-style links and leaves literal code examples alone", () => {
    const result = renderMarkdown(
      "[Search][s]\n\n[s]: search.md\n\n```md\n[Example](search.md)\n```\n\n`[Example](search.md)`",
      "docs/installation.md",
      routes,
    );
    expect(result).toContain("[s]: /search/");
    expect(result.match(/\[Example\]\(search.md\)/g)).toHaveLength(2);
  });
  it("preserves external URLs, page anchors and absolute site links", () => {
    const result = renderMarkdown(
      "[Web](https://example.com/a?q=1#b) [Here](#install) [Home](/)",
      "docs/installation.md",
      routes,
    );
    expect(result).toContain("(https://example.com/a?q=1#b)");
    expect(result).toContain("(#install)");
    expect(result).toContain("(/)");
  });
  it("keeps French punctuation on the line of the word before it, outside code", () => {
    const result = renderMarkdown(
      "Où chercher ? « Mash » : oui ; non !\n\n`a ? b`\n\n```sh\nmash search 'x ?'\n```",
      "docs/fr/search.md",
      routes,
    );
    expect(result).toContain("Où chercher\u00a0? «\u00a0Mash\u00a0»\u00a0: oui\u00a0; non\u00a0!");
    expect(result).toContain("`a ? b`");
    expect(result).toContain("mash search 'x ?'");
  });
  it("leaves other languages' spacing alone", () => {
    expect(renderMarkdown("Where ? Here !", "docs/search.md", routes)).toContain("Where ? Here !");
  });
});
