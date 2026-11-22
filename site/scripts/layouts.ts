/**
 * Page layouts for the website, applied to the Markdown the site is built
 * from. The guides stay plain Markdown that reads well on GitHub; these steps
 * add the website's cards, callouts and pills by recognising each page's
 * structure (a list under a heading, a lead paragraph, a quoted prompt), never
 * its words, so the translations get the same layout. When a page no longer
 * has the structure a step looks for, the step leaves it unchanged.
 */
import type { BlockContent, Code, Html, List, Paragraph, Root, RootContent, Table } from "mdast";
import { toString as text } from "mdast-util-to-string";

/** Words the layouts add, in the page's language. */
export interface LayoutStrings {
  readonly tipBefore: string;
  readonly tipAfter: string;
  readonly copy: string;
  readonly copied: string;
  readonly copyPrompt: string;
}

const html = (value: string): Html => ({ type: "html", value });

const escapeHtml = (value: string): string =>
  value.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** Line icons for the cards' icon tiles. */
function icon(paths: readonly string[]): string {
  const drawn = paths.map((d) => `<path d="${d}"/>`).join("");
  return `<span class="mash-ico" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">${drawn}</svg></span>`;
}

const icons = {
  start: icon(["M5 21V4", "M5 4h11l-2 4 2 4H5"]),
  install: icon(["M12 3v12", "m7 10 5 5 5-5", "M5 21h14"]),
  dashboard: icon(["M4 5h16v11H4z", "M2 19h20"]),
  search: icon(["M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z", "m20 20-3.5-3.5"]),
  agents: icon(["M5 5h14v10H9l-4 4z"]),
  maintain: icon(["M14.5 6.5a4 4 0 0 0-5 5L4 17l3 3 5.5-5.5a4 4 0 0 0 5-5l-2.5 2.5-3-3z"]),
  help: icon([
    "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z",
    "M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6",
    "M12 17h.01",
  ]),
  reference: icon(["M5 4h9a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3z", "M5 17a3 3 0 0 1 3-3h9"]),
  reload: icon([
    "M20 11a8 8 0 0 0-14.5-4.5L4 8",
    "M4 4v4h4",
    "M4 13a8 8 0 0 0 14.5 4.5L20 16",
    "M20 20v-4h-4",
  ]),
};

/** A Mashy sticker, decorative beside the words it sits with. */
const mashy = '<img class="mash-mashy-face" src="/mashy.png" alt="" width="64" height="58">';

/**
 * Raw HTML on its own line, such as the `<span id="…"></span>` anchors kept
 * for old links. Markdown reads a lone inline tag as a paragraph of HTML.
 */
function isRawHtml(node: RootContent | undefined): boolean {
  return (
    node?.type === "html" ||
    (node?.type === "paragraph" &&
      node.children.every(
        (child) => child.type === "html" || (child.type === "text" && child.value.trim() === ""),
      ))
  );
}

/** The markup of a raw HTML node or paragraph, or nothing for anything else. */
function rawHtml(node: RootContent | undefined): string {
  if (node?.type === "html") return node.value;
  if (node?.type !== "paragraph" || !isRawHtml(node)) return "";
  return node.children.map((child) => (child.type === "html" ? child.value : "")).join("");
}

/** The node after `index` that is not an anchor or other raw HTML. */
function nextContent(children: readonly RootContent[], index: number): RootContent | undefined {
  return children.slice(index + 1).find((node) => !isRawHtml(node));
}

/**
 * A bold line on its own just before a code block names the block: it
 * becomes the block's title, drawn in its window frame. On the quick start a
 * short note may sit between the two; it moves under the block.
 */
export function titleCodeBlocks(tree: Root, allowNote: boolean): void {
  const children = tree.children;
  for (let index = 0; index < children.length; index += 1) {
    const label = children[index];
    const solo =
      label?.type === "paragraph" &&
      label.children.length === 1 &&
      label.children[0]?.type === "strong";
    if (!solo) continue;
    const after = children[index + 1];
    const later = children[index + 2];
    let code: Code | undefined;
    let note: Paragraph | undefined;
    if (after?.type === "code") code = after;
    else if (allowNote && after?.type === "paragraph" && later?.type === "code") {
      code = later;
      note = after;
    }
    if (!code || /\btitle=/.test(code.meta ?? "")) continue;
    code.meta = `${code.meta ? `${code.meta} ` : ""}title="${text(label).replace(/"/g, "'")}"`;
    const moved = note ? [html('<div class="mash-note">'), note, html("</div>")] : [];
    children.splice(index, note ? 3 : 2, code, ...moved);
  }
}

/** READ and WRITE in bold are the tool reference's two kinds: draw them as its pills. */
export function badgePills(tree: Root): void {
  const visit = (node: Root | RootContent): void => {
    if (!("children" in node)) return;
    node.children = node.children.map((child) => {
      const kind = child.type === "strong" ? text(child) : "";
      if (kind === "READ" || kind === "WRITE") {
        return html(`<span class="mash-badge ${kind.toLowerCase()}">${kind}</span>`);
      }
      visit(child);
      return child;
    }) as typeof node.children;
  };
  visit(tree);
}

/**
 * The tool reference: each run of tool rows sits in one stack, and the last
 * group, the short maintenance tools, sits two across.
 */
function toolStacks(tree: Root): void {
  const isTool = (node: RootContent | undefined): boolean =>
    node?.type === "html" && node.value.startsWith('<details class="mash-tool');
  const runs: [number, number][] = [];
  tree.children.forEach((node, index) => {
    if (!isTool(node)) return;
    const last = runs.at(-1);
    if (last && last[1] === index - 1) last[1] = index;
    else runs.push([index, index]);
  });
  runs.reverse().forEach(([first, last], order) => {
    const kind = order === 0 && runs.length > 1 ? "mash-tools pair" : "mash-tools";
    tree.children.splice(last + 1, 0, html("</div>"));
    tree.children.splice(first, 0, html(`<div class="${kind}">`));
  });
}

/**
 * The guides directory: the lead and a search tip, then one card per group of
 * guides. A heading with no list of its own ("Using Mash") still heads the
 * cards after it for screen readers, but takes no card. The last group, the
 * reference, spans the grid. Icons follow the cards' order, which every
 * translation keeps.
 */
function guideCards(tree: Root, strings: LayoutStrings): void {
  const order = [
    icons.start,
    icons.install,
    icons.dashboard,
    icons.search,
    icons.agents,
    icons.maintain,
    icons.help,
    icons.reference,
  ];
  const out: RootContent[] = [];
  const children = tree.children;
  const firstHeading = children.findIndex((node) => node.type === "heading");
  const lead = children.slice(0, firstHeading === -1 ? children.length : firstHeading);
  out.push(
    html('<div class="mash-guides-intro">'),
    html('<div class="lead">'),
    ...lead,
    html("</div>"),
  );
  out.push(
    html(
      `<aside class="mash-mashy butter">${mashy}<p>${escapeHtml(strings.tipBefore)} <kbd>Ctrl K</kbd> ${escapeHtml(strings.tipAfter)}</p></aside>`,
    ),
    html("</div>"),
    html('<div class="mash-guides">'),
  );
  const cardHeadings = children.flatMap((node, index) =>
    node.type === "heading" && nextContent(children, index)?.type === "list" ? [index] : [],
  );
  let card = 0;
  // Anchors kept for old links wait for the card they lead to, so the grid
  // holds only cards.
  let anchors: RootContent[] = [];
  for (let index = Math.max(firstHeading, 0); index < children.length; index += 1) {
    const node = children[index];
    if (!node || firstHeading === -1) break;
    if (node.type !== "heading") {
      if (isRawHtml(node)) anchors.push(node);
      else out.push(node);
      continue;
    }
    if (!cardHeadings.includes(index)) {
      out.push(html('<div class="mash-guide-group">'), node, html("</div>"));
      continue;
    }
    const listAt = children.findIndex((child, at) => at > index && child.type === "list");
    const wide = card === cardHeadings.length - 1 ? " wide" : "";
    out.push(
      html(`<section class="mash-guide${wide}">`),
      ...anchors,
      html(`<div class="mash-guide-head">${order[card] ?? icons.reference}`),
      node,
      html("</div>"),
      ...children.slice(index + 1, listAt + 1),
      html("</section>"),
    );
    anchors = [];
    card += 1;
    index = listAt;
  }
  out.push(...anchors);
  out.push(html("</div>"));
  tree.children = out;
}

/** The agents Mash reads, by the name a table row starts with. */
const agentTiles: readonly [RegExp, string][] = [
  [/^Claude Code/, "claude-code"],
  [/^Codex/, "codex"],
  [/^GitHub Copilot/, "copilot"],
  [/^OpenCode/, "opencode"],
  [/^Cursor/, "cursor"],
  [/^Antigravity/, "antigravity"],
];

/** How well an agent was tested, in English, by the pill colour it takes. */
const testedPills: Readonly<Record<string, string>> = {
  "Daily use": "ok",
  Partial: "warn",
};

/**
 * Supported agents: each agent's row starts with its mark on a light tile; a
 * plain "yes" is a mint pill, and the testing levels are pills.
 */
function agentTables(tree: Root): void {
  const tables = tree.children.filter((node): node is Table => node.type === "table");
  for (const [order, table] of tables.entries()) {
    for (const row of table.children.slice(1)) {
      row.children.forEach((cell, column) => {
        const value = text(cell).trim();
        if (column === 0) {
          const agent = agentTiles.find(([name]) => name.test(value))?.[1];
          if (agent) {
            cell.children = [
              html(
                `<span class="mash-agent"><span class="mash-agent-tile" data-agent="${agent}" aria-hidden="true"></span><span>`,
              ),
              ...cell.children,
              html("</span></span>"),
            ];
          }
          return;
        }
        const plain = cell.children.length === 1 && cell.children[0]?.type === "text";
        if (!plain) return;
        if (
          order === 0 &&
          column === table.children[0].children.length - 1 &&
          /^(Yes|Oui|Sí|是|はい)$/.test(value)
        ) {
          cell.children = [html(`<span class="mash-pill ok">${escapeHtml(value)}</span>`)];
        } else if (order > 0) {
          const tone = testedPills[value] ?? "plain";
          cell.children = [html(`<span class="mash-pill ${tone}">${escapeHtml(value)}</span>`)];
        }
      });
    }
  }
}

/**
 * The quick start: the lead line is a cream callout; the closing paragraph
 * of the first-chat section is a tip beside Mashy; the prompt to paste into
 * an agent is a butter card with a copy button.
 */
function quickStart(tree: Root, strings: LayoutStrings): void {
  const children = tree.children;
  const firstHeading = children.findIndex((node) => node.type === "heading");
  const lead = children.findIndex(
    (node, index) => node.type === "paragraph" && !isRawHtml(node) && index < firstHeading,
  );
  if (lead !== -1) {
    children.splice(lead + 1, 0, html("</div>"));
    children.splice(lead, 0, html(`<div class="mash-callout">${icons.reload}`));
  }

  const tryIt = children.findIndex((node) => rawHtml(node).includes('id="try-it"'));
  const heading = children.findIndex((node, index) => index > tryIt && node.type === "heading");
  const end = children.findIndex(
    (node, index) => index > heading && (node.type === "heading" || isRawHtml(node)),
  );
  const section = children.slice(heading + 1, end === -1 ? children.length : end);
  const paragraphs = section.filter((node) => node.type === "paragraph");
  const tip = paragraphs.length > 1 ? paragraphs.at(-1) : undefined;
  if (tryIt !== -1 && heading !== -1 && tip) {
    const at = children.indexOf(tip);
    children.splice(
      at,
      1,
      html(`<div class="mash-mashy">${mashy}<div>`),
      tip,
      html("</div></div>"),
    );
  }

  // Every quote in the quick start is something to say to an agent, so each
  // one becomes a prompt with a copy button.
  for (const [quote, prompt] of children.entries()) {
    if (prompt.type !== "blockquote") continue;
    const words = text(prompt).replace(/\s+/g, " ").trim();
    children.splice(
      quote,
      1,
      html(
        `<div class="mash-ask"><p>${escapeHtml(words)}</p><button type="button" data-copy="${escapeHtml(words)}" data-copied="${escapeHtml(strings.copied)}" aria-label="${escapeHtml(strings.copyPrompt)}">${escapeHtml(strings.copy)}</button></div>`,
      ),
    );
  }
}

/** Applies the website layout for one page, named by its route's last part. */
export function layoutPage(tree: Root, page: string, strings: LayoutStrings): void {
  titleCodeBlocks(tree, page === "quick-start");
  badgePills(tree);
  if (page === "guides") guideCards(tree, strings);
  if (page === "tools") toolStacks(tree);
  if (page === "supported-agents") agentTables(tree);
  if (page === "quick-start") quickStart(tree, strings);
}
