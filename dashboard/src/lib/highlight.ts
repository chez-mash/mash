export interface HighlightPart {
  readonly text: string;
  readonly hit: boolean;
}

/** Letters and digits of any script, which make up a word. */
const WORD = "[\\p{L}\\p{N}_]*";

/** Splits `text` into the stretches that match a word of `query` and those
 * between them. A match is widened to the whole word it falls in, and words
 * matched next to each other are marked as one stretch. Words of one
 * character in the query are ignored. */
export function highlightParts(text: string, query: string): HighlightPart[] {
  const terms = query
    .split(/\s+/)
    .filter((term) => term.length > 1)
    .map((term) => term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  if (!terms.length) return [{ text, hit: false }];
  // Splitting on a capturing group puts every match at an odd index, so the
  // empty stretches are dropped only after each part is told apart.
  const parts = text
    .split(new RegExp(`(${WORD}(?:${terms.join("|")})${WORD})`, "giu"))
    .map((part, index) => ({ text: part, hit: index % 2 === 1 }))
    .filter((part) => part.text !== "");
  // Spaces between two marked words join them.
  const joined: HighlightPart[] = [];
  for (const [index, part] of parts.entries()) {
    const previous = joined.at(-1);
    const next = parts[index + 1];
    const gap = !part.hit && /^[ \t]+$/.test(part.text) && previous?.hit && next?.hit;
    if (previous?.hit && (part.hit || gap))
      joined[joined.length - 1] = { text: previous.text + part.text, hit: true };
    else joined.push(part);
  }
  return joined;
}
