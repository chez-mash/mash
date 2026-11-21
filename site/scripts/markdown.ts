import path from "node:path";
import type { Root } from "mdast";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import { visit } from "unist-util-visit";

export const markdown = unified().use(remarkParse).use(remarkGfm).use(remarkStringify);

export function rewriteMarkdownUrls(tree: Root, rewrite: (url: string) => string): void {
  visit(tree, (node) => {
    if (node.type === "link" || node.type === "image" || node.type === "definition") {
      node.url = rewrite(node.url);
    }
  });
}

/** Resolve repository Markdown links without changing examples inside code blocks. */
export function renderMarkdown(body: string, source: string, routes: ReadonlyMap<string, string>) {
  const tree = markdown.parse(body);
  rewriteMarkdownUrls(tree, (url) => {
    if (/^(?:[a-z][a-z\d+.-]*:|\/|#)/i.test(url)) return url;
    const target = new URL(url, `https://github.com/mashium/mash/blob/main/${source}`);
    const file = path.posix.normalize(
      path.posix.join(path.posix.dirname(source), url.split(/[?#]/)[0] ?? ""),
    );
    const route = routes.get(file);
    return route ? `${route}${target.search}${target.hash}` : target.href;
  });
  return markdown.stringify(tree);
}
