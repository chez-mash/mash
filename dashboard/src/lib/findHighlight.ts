// Paints Find in chat matches on the rendered page with the CSS Custom
// Highlight API, which marks text without changing the page's markup.
// Browsers without it still scroll to each match; they only lose the colour.
import { tick } from "svelte";
import type { FindMatch } from "./find";

const ALL = "mash-find";
const CURRENT = "mash-find-current";

function supported(): boolean {
  return typeof Highlight !== "undefined" && typeof CSS !== "undefined" && "highlights" in CSS;
}

function pattern(query: string): RegExp | undefined {
  const words = query.trim().split(/\s+/).filter(Boolean);
  if (!words.length) return undefined;
  const escaped = words.map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  return new RegExp(escaped.join("\\s+"), "giu");
}

/** Matches in text nodes read together as one string. A coloured code block
 * splits its text into a node per token, and a match may cross them. */
function matchRun(nodes: readonly Text[], expression: RegExp, found: Range[]): void {
  const ends: number[] = [];
  let text = "";
  for (const node of nodes) {
    text += node.nodeValue ?? "";
    ends.push(text.length);
  }
  // The node holding a position; an end on a boundary stays in the node before.
  const at = (offset: number, end: boolean): [Text, number] => {
    let index = ends.findIndex((nodeEnd) => (end ? offset <= nodeEnd : offset < nodeEnd));
    if (index < 0) index = nodes.length - 1;
    const start = (ends[index] ?? 0) - (nodes[index]?.nodeValue?.length ?? 0);
    return [nodes[index] as Text, offset - start];
  };
  expression.lastIndex = 0;
  for (let match = expression.exec(text); match; match = expression.exec(text)) {
    if (!match[0]) break;
    const range = document.createRange();
    range.setStart(...at(match.index, false));
    range.setEnd(...at(match.index + match[0].length, true));
    found.push(range);
  }
}

function ranges(root: Node, expression: RegExp): Range[] {
  const found: Range[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let block: Element | null = null;
  let run: Text[] = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = node as Text;
    // Text inside one highlighted block is searched as a whole; any other
    // text node stands alone, as before.
    const coloured = text.parentElement?.closest("pre[data-highlight]") ?? null;
    if (!coloured || coloured !== block) {
      if (run.length) matchRun(run, expression, found);
      run = [];
    }
    block = coloured;
    run.push(text);
  }
  if (run.length) matchRun(run, expression, found);
  return found;
}

export function clearMatches(): void {
  if (!supported()) return;
  CSS.highlights.delete(ALL);
  CSS.highlights.delete(CURRENT);
}

/**
 * Highlights every match under `root`, and the `occurrence`-th one inside
 * `current` more strongly. Returns that match, or `current` itself when
 * its text is not on the page (for example, cut from a long tool output).
 */
export function paintMatches(
  root: HTMLElement,
  query: string,
  current?: { readonly element: Element; readonly occurrence: number },
): Range | Element | undefined {
  const expression = pattern(query);
  if (!expression) {
    clearMatches();
    return undefined;
  }
  const all = ranges(root, expression);
  let target: Range | undefined;
  if (current) {
    const inside = all.filter((range) => current.element.contains(range.startContainer));
    target = inside[Math.min(current.occurrence, inside.length - 1)];
  }
  if (supported()) {
    CSS.highlights.set(ALL, new Highlight(...all.filter((range) => range !== target)));
    if (target) CSS.highlights.set(CURRENT, new Highlight(target));
    else CSS.highlights.delete(CURRENT);
  }
  return target ?? current?.element;
}

/**
 * Scrolls `target` to the middle of the window, first scrolling any box it
 * sits in that scrolls on its own (a long tool output, a wide command).
 */
function bringIntoView(target: Range | Element): void {
  const start = target instanceof Range ? target.startContainer.parentElement : target;
  for (let box = start; box && box !== document.body; box = box.parentElement) {
    const style = getComputedStyle(box);
    const scrolls =
      /(auto|scroll)/.test(`${style.overflowX} ${style.overflowY}`) &&
      (box.scrollWidth > box.clientWidth || box.scrollHeight > box.clientHeight);
    if (!scrolls) continue;
    const spot = target.getBoundingClientRect();
    const frame = box.getBoundingClientRect();
    box.scrollLeft += spot.left - frame.left - box.clientWidth / 2;
    box.scrollTop += spot.top - frame.top - box.clientHeight / 2;
  }
  const spot = target.getBoundingClientRect();
  window.scrollTo({ top: window.scrollY + spot.top - window.innerHeight / 2, behavior: "instant" });
}

/** Opens every fold from `element` outward that `inside` allows. */
function openFolds(from: Element | null | undefined, inside: (fold: Element) => boolean): void {
  for (
    let fold = from?.closest("details");
    fold && inside(fold);
    fold = fold.parentElement?.closest("details")
  )
    fold.open = true;
}

/**
 * Shows the match `target` under `pane`: its message and any collapsed step
 * around it are opened, every match is painted, and the window scrolls to it.
 */
export async function revealMatch(
  pane: HTMLElement,
  query: string,
  target: FindMatch,
): Promise<void> {
  const element = pane.querySelector(`[data-message="${CSS.escape(target.messageId)}"]`);
  if (!element) return;
  if (element instanceof HTMLDetailsElement) element.open = true;
  openFolds(element, () => true);
  await tick();
  const spot = paintMatches(pane, query, { element, occurrence: target.occurrence });
  // The match may sit in a fold inside the message, such as its added context.
  if (spot instanceof Range) {
    openFolds(spot.startContainer.parentElement, (fold) => element.contains(fold));
    await tick();
  }
  if (spot) bringIntoView(spot);
}
