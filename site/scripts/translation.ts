import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import type { Root, Text } from "mdast";
import { visit } from "unist-util-visit";
import { markdown, readmeQuickStart } from "./markdown.ts";
import {
  type LocaleGuides,
  localizeLinks,
  retainSectionIds,
  sectionSlugs,
} from "./translation-links.ts";

export const translationPages = [
  "quick-start",
  "installation",
  "install-methods",
  "local-search",
  "troubleshooting",
  "privacy",
  "dashboard",
  "pick-up-work",
  "agents",
  "plugin",
  "supported-agents",
  "shared-index",
  "search-languages",
] as const;
export type TranslationPage = (typeof translationPages)[number];

export type TranslationSource = {
  file: string;
  body: string;
  revision: string;
};

const repo = new URL("../../", import.meta.url);
const sourceMarker = /^<!-- mash-docs-source: ([a-f0-9]{64}) -->$/m;
const uiLabels = new Set([
  "Apply filters",
  "Ask Mashy",
  "Browse chats",
  "Check search service again",
  "Clear filters",
  "Copy",
  "Copy session ID",
  "Connect to an existing shared server",
  "Custom number of days",
  "Customize setup",
  "Export",
  "Export to PDF",
  "JSON",
  "Markdown",
  "N new messages",
  "Next",
  "Next page",
  "Retry",
  "Review and edit settings",
  "Save as PDF",
  "Save configuration only",
  "See all",
  "Set up and index",
  "Set up and index without connecting agents",
  "Set up without indexing",
  "Show history folders",
  "Show updated messages",
  "Skip agent connections",
  "Skip to the dashboard",
  "Just this computer (recommended)",
  "Search your history",
  "Use existing settings",
  "Which agents should get access to Mash search?",
  "agents",
  "check for updates",
  "chats",
  "export chats",
  "find a statement",
  "fleet manager",
  "machines",
  "meaning",
  "memory",
  "memory from this chat",
  "open the message",
  "overview",
  "projects",
  "reload chat list",
  "Saved as",
  "search: default",
  "services",
  "settings",
  "tool explorer",
  "viewing history",
  "words",
]);

export function isTranslationPage(page: string | undefined): page is TranslationPage {
  return translationPages.some((candidate) => candidate === page);
}

export async function readTranslationSource(page: TranslationPage): Promise<TranslationSource> {
  const file = page === "quick-start" ? "README.md" : `docs/${page}.md`;
  let body = await readFile(new URL(file, repo), "utf8");
  if (page === "quick-start") {
    body = `# Quick start\n\n${readmeQuickStart(body)}`;
  }
  // The site already has a language picker. Leave the repository's language
  // shortcuts out of generated drafts.
  body = body.replace(/^\[Español\]\([^\n]+\)\n\n/m, "").trimEnd();
  const revision = createHash("sha256").update(body).digest("hex");
  return { file, body, revision };
}

export function recordedTranslationRevision(body: string): string | undefined {
  return sourceMarker.exec(body)?.[1];
}

function protectedContent(tree: Root): string {
  const content: (string | number | null | undefined)[][] = [];
  visit(tree, (node) => {
    if (node.type === "heading") {
      content.push([node.type, node.depth]);
    } else if (node.type === "link" || node.type === "image" || node.type === "definition") {
      content.push([node.type, node.url, node.title]);
    } else if (node.type === "code") {
      content.push([node.type, node.value, node.lang, node.meta]);
    } else if (node.type === "inlineCode" || node.type === "html") {
      content.push([node.type, node.value]);
    }
  });
  return JSON.stringify(content);
}

export function prepareTranslation(
  source: TranslationSource,
  destination: string,
  localeGuides: LocaleGuides = new Map(),
) {
  const tree = markdown.parse(source.body);
  const sourceSlugs = [...sectionSlugs(tree).values()];
  const nodes: Text[] = [];
  visit(tree, "text", (node, _index, parent) => {
    if (!node.value.trim()) return;
    if (
      parent?.type === "strong" &&
      uiLabels.has(node.value.trim().replace(/\s+/g, " ").replace(/:$/, ""))
    )
      return;
    if (parent?.type === "link" && parent.url === node.value) return;
    nodes.push(node);
  });
  const passages = nodes.map((node) => node.value.trim());
  localizeLinks(tree, source.file, destination, localeGuides);

  return {
    passages,
    render(value: unknown): string {
      if (!value || typeof value !== "object" || !("translations" in value)) {
        throw new Error("The translator did not return a translations array.");
      }
      const translations = value.translations;
      if (!Array.isArray(translations) || translations.length !== passages.length) {
        throw new Error(`Expected exactly ${passages.length} translated passages.`);
      }
      for (const [index, node] of nodes.entries()) {
        const text: unknown = translations[index];
        if (typeof text !== "string" || !text.trim()) {
          throw new Error(`Translation passage ${index + 1} is missing or empty.`);
        }
        const leading = node.value.match(/^\s*/)?.[0] ?? "";
        const trailing = node.value.match(/\s*$/)?.[0] ?? "";
        node.value = leading + text.trim().replace(/\s+/g, " ") + trailing;
      }
      const translated = markdown.parse(markdown.stringify(tree));
      if (protectedContent(translated) !== protectedContent(tree)) {
        throw new Error("The translated prose changed a heading, link or code example.");
      }
      retainSectionIds(translated, sourceSlugs);
      return `${markdown.stringify(translated)}\n<!-- mash-docs-source: ${source.revision} -->\n`;
    },
  };
}
