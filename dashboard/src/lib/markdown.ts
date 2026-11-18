import DOMPurify from "dompurify";
import { Marked, type RendererObject } from "marked";

const escapes: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;" };
function asText(text: string): string {
  return text.replace(/[&<>]/g, (character) => escapes[character] ?? character);
}

// Archived text is shown as it was written. Markdown is rendered; nothing in
// it is loaded, followed inside the dashboard or silently dropped.
const renderer: RendererObject = {
  /** Anything that looks like a tag or a comment is text someone wrote. */
  html: ({ text }) => asText(text),
  /** An image is never fetched: its description stands in for it. */
  image: ({ href, text }) => asText(text || href),
  /** Only a web address is a link. A relative one would open a page of the
   * dashboard itself, so it stays the words it was written with. */
  link({ href, tokens }) {
    return /^https?:\/\//i.test(href) ? false : this.parser.parseInline(tokens);
  },
  checkbox: ({ checked }) => (checked ? "[x] " : "[ ] "),
};
const replies = new Marked({ renderer });
/** Mashy's answers, which also link to pages of the dashboard itself. */
const guided = new Marked({
  renderer: {
    ...renderer,
    link({ href, tokens }) {
      return /^(https?:\/\/|#\/)/i.test(href) ? false : this.parser.parseInline(tokens);
    },
  },
});
/** What a person typed: a line break is a line break. */
const typed = new Marked({ breaks: true, renderer });

/** The elements a message may contain; everything else is removed. */
export const ALLOWED_TAGS = [
  "p",
  "br",
  "strong",
  "em",
  "del",
  "blockquote",
  "ul",
  "ol",
  "li",
  "pre",
  "code",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "hr",
  "a",
  "table",
  "thead",
  "tbody",
  "tr",
  "th",
  "td",
] as const;

/** A chat title on one line: emphasis and code marks are noise there. */
export function plainTitle(title: string): string {
  return title.replace(/\*\*|`/g, "");
}

/** Markdown as HTML, before it is sanitized. `own` is a message the reader
 * typed; `inApp` keeps links to the dashboard's own pages. */
export function render(text: string, own = false, inApp = false): string {
  return (inApp ? guided : own ? typed : replies).parse(text, { async: false });
}

/** Archived messages cannot load remote images or run HTML, scripts or styles.
 * `own` renders a message the reader typed themselves. `inApp` is for Mashy's
 * answers only: archived text never links into the dashboard. */
export function markdown(text: string, own = false, inApp = false): string {
  return DOMPurify.sanitize(render(text, own, inApp), {
    ALLOWED_TAGS: [...ALLOWED_TAGS],
    ALLOWED_ATTR: ["href", "title"],
    ALLOWED_URI_REGEXP: inApp ? /^(https?:\/\/|#\/)/i : /^https?:\/\//i,
  });
}
