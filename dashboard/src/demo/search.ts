const words = /[\p{Alphabetic}\p{Number}]+/gu;
const boundary = "[^\\p{Alphabetic}\\p{Number}_+\\-]";

/** Find where the query first matches one message in the sample history. */
export function messageMatcher(input: string): (text: string) => number | undefined {
  const query = input.trim();
  const wanted = query.match(words)?.map((word) => word.toLowerCase()) ?? [];
  if (wanted.length === 0) return () => undefined;

  // Preserve punctuation in quoted phrases and single terms such as a filename.
  const quoted = query.startsWith('"') && query.endsWith('"');
  if (quoted || !/\s/u.test(query)) {
    const literal = quoted ? query.slice(1, -1) : query;
    const escaped = literal.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
    const pattern = new RegExp(`(^|${boundary})${escaped}(?=$|${boundary})`, "iu");
    return (text) => {
      const match = pattern.exec(text);
      return match ? match.index + (match[1]?.length ?? 0) : undefined;
    };
  }

  return (text) => {
    const missing = new Set(wanted);
    let first: number | undefined;
    for (const match of text.matchAll(words)) {
      if (missing.delete(match[0].toLowerCase())) first ??= match.index;
      if (missing.size === 0) return first;
    }
    return undefined;
  };
}

export function searchSnippet(text: string, matchOffset: number): string {
  const plain = text.replace(/\s+/g, " ").trim();
  const at = text.slice(0, matchOffset).replace(/\s+/g, " ").trimStart().length;
  const start = Math.max(0, at - 70);
  const piece = plain.slice(start, start + 220);
  return (start > 0 ? "…" : "") + piece + (start + 220 < plain.length ? "…" : "");
}
