/**
 * A search passage as plain words. Search returns a window of a message's raw
 * text: Markdown as the agent wrote it, cut wherever the window fell, with
 * `…` where it was cut. On one quiet line the Markdown marks are noise and a
 * part-word at a cut reads as a typo, so both go.
 */

import { highlightParts } from "./highlight";

const CUT = "…";
const WORD_CHARACTER = /[\p{L}\p{N}_]/u;

/**
 * Starts and ends a cut passage at whole words. When the window began on a
 * letter, the word it began in may be cut, so that word is dropped: the
 * server opens the window well before the match, so the dropped word is never
 * the one that matched (crates/mash-storage/src/search_results.rs).
 */
function wholeWords(text: string): string {
  let words = text;
  if (words.startsWith(CUT)) {
    const rest = words.slice(CUT.length);
    const space = rest.search(/\s/);
    words = WORD_CHARACTER.test(rest.charAt(0)) && space > 0 ? rest.slice(space + 1) : rest;
    words = CUT + words.trimStart();
  }
  if (words.endsWith(CUT) && words.length > 2 * CUT.length) {
    const body = words.slice(0, -CUT.length);
    const space = body.search(/\s\S*$/);
    const cutInWord = WORD_CHARACTER.test(body.charAt(body.length - 1));
    words = `${(cutInWord && space > CUT.length ? body.slice(0, space) : body).trimEnd()}${CUT}`;
  }
  return words;
}

/** Markdown's marks removed, keeping the words they mark. */
export function withoutMarkdown(text: string): string {
  return (
    text
      // Links and images keep their words.
      .replace(/!?\[([^\]\n]*)\]\([^)\s]*\)/g, "$1")
      // Headings, quotes and list bullets at the start of a line.
      .replace(/^[ \t]{0,3}(?:#{1,6}[ \t]+|>[ \t]?|[-*+][ \t]+|\d+[.)][ \t]+)/gm, "")
      .replace(/\*\*|__|~~|`+/g, "")
      // Single-mark emphasis: *word* and _word_.
      .replace(/(^|[^*\w])\*([^*\n]+)\*(?![*\w])/g, "$1$2")
      .replace(/(^|[^_\w])_([^_\n]+)_(?![_\w])/g, "$1$2")
  );
}

/** A passage on one line, without Markdown, starting and ending on whole words. */
export function plainPassage(text: string): string {
  return withoutMarkdown(wholeWords(text.trim())).replace(/\s+/g, " ").trim();
}

/**
 * A stretch of about `limit` characters of a whole message, from a word a
 * little before the first word of `query` it holds, with `…` where it was cut.
 */
export function passageAround(text: string, query: string, limit: number): string {
  const flat = plainPassage(text);
  if (flat.length <= limit) return flat;
  let offset = 0;
  for (const part of highlightParts(flat, query)) {
    if (part.hit) break;
    offset += part.text.length;
  }
  const start = offset > limit / 3 ? flat.lastIndexOf(" ", offset - 60) + 1 : 0;
  let end = Math.min(flat.length, start + limit);
  const space = flat.lastIndexOf(" ", end);
  if (end < flat.length && space > start) end = space;
  return `${start > 0 ? CUT : ""}${flat.slice(start, end).trim()}${end < flat.length ? CUT : ""}`;
}
