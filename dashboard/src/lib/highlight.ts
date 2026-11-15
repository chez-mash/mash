export interface HighlightPart {
  readonly text: string;
  readonly hit: boolean;
}

/** Splits `text` into the stretches that match a word of `query` and those
 * between them. Words of one character are ignored. */
export function highlightParts(text: string, query: string): HighlightPart[] {
  const terms = query
    .split(/\s+/)
    .filter((term) => term.length > 1)
    .map((term) => term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  if (!terms.length) return [{ text, hit: false }];
  // Splitting on a capturing group puts every match at an odd index, so the
  // empty stretches are dropped only after each part is told apart.
  return text
    .split(new RegExp(`(${terms.join("|")})`, "gi"))
    .map((part, index) => ({ text: part, hit: index % 2 === 1 }))
    .filter((part) => part.text !== "");
}
