import { describe, expect, it } from "vitest";
import { ALLOWED_TAGS, plainTitle, render } from "./markdown";

describe("chat titles", () => {
  it("drops emphasis and code marks but keeps the words and other punctuation", () => {
    expect(plainTitle("Master is at **1a7a4d6e1**, see `docs/plan.md`")).toBe(
      "Master is at 1a7a4d6e1, see docs/plan.md",
    );
    expect(plainTitle("2 * 3 and __init__")).toBe("2 * 3 and __init__");
  });
});

// `render` is what the sanitizer receives. Text that reaches it escaped
// cannot be dropped, and the allowed elements are checked by name.
describe("message Markdown", () => {
  it("shows HTML-looking text in a reply as written, and still renders Markdown", () => {
    const html = render("Returns **a** Vec<Token> inside <div>x</div> <!-- note -->");
    expect(html).toContain("<strong>a</strong>");
    expect(html).toContain("Vec&lt;Token&gt;");
    expect(html).toContain("&lt;div&gt;x&lt;/div&gt;");
    expect(html).toContain("&lt;!-- note --&gt;");
  });

  it("shows a block of HTML in a reply as text", () => {
    const html = render('<script>alert(1)</script>\n\n<img src="x" onerror="alert(1)">');
    expect(html).not.toMatch(/<script|<img/);
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(html).toContain('&lt;img src="x" onerror="alert(1)"&gt;');
  });

  it("keeps a person's line breaks and tags, as before", () => {
    expect(render("one\ntwo <b>", true)).toBe("<p>one<br>two &lt;b&gt;</p>\n");
    expect(render("one\ntwo")).toBe("<p>one\ntwo</p>\n");
  });

  it("names an image by its description or address and never loads it", () => {
    const html = render("![A <chart>](https://example.com/a.png) ![](https://example.com/b.png)");
    expect(html).not.toContain("<img");
    expect(html).toBe("<p>A &lt;chart&gt; https://example.com/b.png</p>\n");
  });

  it("links web addresses only; other links stay as their words", () => {
    expect(render("[site](https://example.com/a)")).toContain(
      '<a href="https://example.com/a">site</a>',
    );
    for (const source of [
      "[the **plan**](docs/plan.md)",
      "[the **plan**](/settings)",
      "[the **plan**](#/chat/1)",
      "[the **plan**](javascript:alert(1))",
      "[the **plan**][ref]\n\n[ref]: ../plan.md",
    ])
      expect(render(source)).toBe("<p>the <strong>plan</strong></p>\n");
  });

  it("renders the two smallest headings", () => {
    expect(render("##### Five\n\n###### Six")).toBe("<h5>Five</h5>\n<h6>Six</h6>\n");
    expect(ALLOWED_TAGS).toEqual(expect.arrayContaining(["h5", "h6"]));
  });

  it("shows task items as text, since a checkbox would be removed", () => {
    const html = render("- [ ] write it\n- [x] test it\n\n1. [x] first\n\n   more");
    expect(html).not.toContain("<input");
    expect(html).toContain("<li>[ ] write it</li>");
    expect(html).toContain("<li>[x] test it</li>");
    expect(html).toContain("[x] first");
  });
});
