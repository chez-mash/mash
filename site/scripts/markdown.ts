import path from "node:path";
import type { Root } from "mdast";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import { visit } from "unist-util-visit";
import { type LayoutStrings, layoutPage } from "./layouts.ts";

export const markdown = unified().use(remarkParse).use(remarkGfm).use(remarkStringify);

/** Extract the README's quick start for the website and translation drafts. */
export function readmeQuickStart(readme: string): string {
  const body = /^## Quick start\r?\n([\s\S]*?)^<!-- quick-start-end -->\r?$/m.exec(readme)?.[1];
  if (!body) {
    throw new Error("Expected a Quick start heading and quick-start-end marker in README.md");
  }

  const tree = markdown.parse(body);
  for (const child of tree.children) {
    if (child.type === "heading" && child.depth === 3) child.depth = 2;
  }
  return markdown.stringify(tree).trimEnd();
}

export function rewriteMarkdownUrls(tree: Root, rewrite: (url: string) => string): void {
  visit(tree, (node) => {
    if (node.type === "link" || node.type === "image" || node.type === "definition") {
      node.url = rewrite(node.url);
    }
  });
}

/**
 * French sets a space before ? ! : ; and » and after «, and the space must not
 * break, or the mark can start a line on its own. Ordinary spaces in those
 * places become no-break spaces; code is left alone.
 */
export function spaceFrenchPunctuation(text: string): string {
  return text.replace(/ (?=[?!:;»])/g, "\u00a0").replace(/« /g, "«\u00a0");
}

/** The website layout to apply to a page, named by its route's last part. */
export interface Layout {
  readonly page: string;
  readonly strings: LayoutStrings;
}

/**
 * Resolve repository Markdown links without changing examples inside code
 * blocks, then lay the page out for the website.
 */
export function renderMarkdown(
  body: string,
  source: string,
  routes: ReadonlyMap<string, string>,
  layout?: Layout,
) {
  const tree = markdown.parse(body);
  rewriteMarkdownUrls(tree, (url) => {
    if (/^(?:[a-z][a-z\d+.-]*:|\/|#)/i.test(url)) return url;
    const target = new URL(url, `https://github.com/mashhq/mash/blob/main/${source}`);
    const file = path.posix.normalize(
      path.posix.join(path.posix.dirname(source), url.split(/[?#]/)[0] ?? ""),
    );
    const route = routes.get(file);
    return route ? `${route}${target.search}${target.hash}` : target.href;
  });
  if (source.startsWith("docs/fr/")) {
    visit(tree, "text", (node) => {
      node.value = spaceFrenchPunctuation(node.value);
    });
  }
  if (layout) layoutPage(tree, layout.page, layout.strings);
  return markdown.stringify(tree);
}
