import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import GithubSlugger from "github-slugger";
import type { Heading, Root } from "mdast";
import { toString as markdownText } from "mdast-util-to-string";
import { visit } from "unist-util-visit";
import { markdown, rewriteMarkdownUrls } from "./markdown.ts";

const repository = new URL("../../", import.meta.url);
const linkBase = new URL("https://mash-docs.invalid/");
export type LocaleGuides = ReadonlyMap<string, ReadonlySet<string>>;

export function sectionSlugs(tree: Root): Map<Heading, string> {
  const slugger = new GithubSlugger();
  // The site moves the first H1 into frontmatter before assigning section IDs.
  const title = tree.children.find((node) => node.type === "heading" && node.depth === 1);
  const slugs = new Map<Heading, string>();
  visit(tree, "heading", (node) => {
    if (node !== title) {
      slugs.set(
        node,
        slugger.slug(markdownText(node, { includeHtml: false, includeImageAlt: false })),
      );
    }
  });
  return slugs;
}

function explicitIds(tree: Root): Set<string> {
  const ids = new Set<string>();
  visit(tree, "html", ({ value }) => {
    const match = /^<span id="([^\s"<>&]+)">(?:<\/span>)?$/.exec(value);
    if (match?.[1]) ids.add(match[1]);
  });
  return ids;
}

function fragments(tree: Root): Set<string> {
  return new Set(
    [...sectionSlugs(tree).values(), ...explicitIds(tree)]
      .filter(Boolean)
      .map((id) => new URL(`#${id}`, linkBase).hash),
  );
}

export async function readLocaleGuides(locale: string): Promise<LocaleGuides> {
  const directory = new URL(`docs/${locale}/`, repository);
  const names = await readdir(directory).catch((error: unknown) => {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") return [];
    throw error;
  });
  const guides = names.filter((name) => name.endsWith(".md"));
  return new Map(
    await Promise.all(
      guides.map(
        async (name) =>
          [
            name,
            fragments(markdown.parse(await readFile(new URL(name, directory), "utf8"))),
          ] as const,
      ),
    ),
  );
}

export function localizeLinks(
  tree: Root,
  source: string,
  destination: string,
  localeGuides: LocaleGuides,
): string[] {
  const sections = fragments(tree);
  const base = new URL(source, linkBase);
  const englishFallbacks = new Set<string>();
  rewriteMarkdownUrls(tree, (url) => {
    if (/^(?:[a-z][a-z\d+.-]*:|\/)/i.test(url)) return url;
    const target = new URL(url, base);
    const file = target.pathname.slice(1);
    if (file === source && sections.has(target.hash)) return target.search + target.hash;

    if (file === "README.md") {
      const quickStart = localeGuides.get("quick-start.md");
      if (quickStart && target.hash === "#quick-start") return `quick-start.md${target.search}`;
      if (quickStart?.has(target.hash)) return `quick-start.md${target.search}${target.hash}`;
    }
    const name =
      file === "docs/README.md" ? "guides.md" : /^docs\/([a-z0-9-]+\.md)$/.exec(file)?.[1];
    if (name) {
      const translated = localeGuides.get(name);
      if (translated && (!target.hash || translated.has(target.hash))) {
        return name + target.search + target.hash;
      }
    }

    const relative = path.posix.relative(path.posix.dirname(destination), file);
    const resolved = relative + target.search + target.hash;
    if (name || file === "README.md") englishFallbacks.add(resolved);
    return resolved;
  });
  return [...englishFallbacks].sort();
}

export function retainSectionIds(tree: Root, sourceSlugs: readonly string[]): void {
  const headings = sectionSlugs(tree);
  const headingIds = new Set(headings.values());
  const existing = explicitIds(tree);
  const aliases = new Map<Heading, string>();
  for (const [index, [heading, translatedId]] of [...headings].entries()) {
    const sourceId = sourceSlugs[index];
    if (existing.has(translatedId)) {
      throw new Error(`A translated heading duplicates the existing anchor #${translatedId}.`);
    }
    if (!sourceId || sourceId === translatedId || existing.has(sourceId)) continue;
    if (headingIds.has(sourceId)) {
      throw new Error(`A translated heading conflicts with section link #${sourceId}.`);
    }
    aliases.set(heading, sourceId);
  }
  visit(tree, "heading", (heading, index, parent) => {
    const id = aliases.get(heading);
    if (!id || index === undefined || !parent) return;
    parent.children.splice(index, 0, { type: "html", value: `<span id="${id}"></span>` });
    return index + 2;
  });
}
