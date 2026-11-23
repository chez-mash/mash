// Colours code with Shiki, loaded only when a block first needs it: the core,
// its JavaScript regular-expression engine and each language are separate
// chunks, so a chat with no code downloads none of them. Shiki names its
// colours as CSS variables, which are drawn as the dashboard's own code tokens.
import type { HighlighterCore, LanguageInput } from "shiki/core";
import type { Language } from "./codeLanguage";

/** One coloured run of text; `color` is a CSS value such as `var(--…)`. */
export interface Token {
  readonly content: string;
  readonly color?: string;
}

const LOADERS: Readonly<Record<Language, () => Promise<{ default: LanguageInput }>>> = {
  astro: () => import("shiki/langs/astro.mjs"),
  c: () => import("shiki/langs/c.mjs"),
  cpp: () => import("shiki/langs/cpp.mjs"),
  csharp: () => import("shiki/langs/csharp.mjs"),
  css: () => import("shiki/langs/css.mjs"),
  go: () => import("shiki/langs/go.mjs"),
  html: () => import("shiki/langs/html.mjs"),
  java: () => import("shiki/langs/java.mjs"),
  javascript: () => import("shiki/langs/javascript.mjs"),
  json: () => import("shiki/langs/json.mjs"),
  jsx: () => import("shiki/langs/jsx.mjs"),
  kotlin: () => import("shiki/langs/kotlin.mjs"),
  lua: () => import("shiki/langs/lua.mjs"),
  markdown: () => import("shiki/langs/markdown.mjs"),
  php: () => import("shiki/langs/php.mjs"),
  powershell: () => import("shiki/langs/powershell.mjs"),
  python: () => import("shiki/langs/python.mjs"),
  ruby: () => import("shiki/langs/ruby.mjs"),
  rust: () => import("shiki/langs/rust.mjs"),
  scss: () => import("shiki/langs/scss.mjs"),
  shellscript: () => import("shiki/langs/shellscript.mjs"),
  sql: () => import("shiki/langs/sql.mjs"),
  svelte: () => import("shiki/langs/svelte.mjs"),
  swift: () => import("shiki/langs/swift.mjs"),
  toml: () => import("shiki/langs/toml.mjs"),
  tsx: () => import("shiki/langs/tsx.mjs"),
  typescript: () => import("shiki/langs/typescript.mjs"),
  xml: () => import("shiki/langs/xml.mjs"),
  yaml: () => import("shiki/langs/yaml.mjs"),
};

const THEME = "mash";
/** Text the theme leaves uncoloured gets this marker, which is dropped below. */
const PLAIN = "inherit";
/** A TextMate theme whose colours are the dashboard's own code tokens. Names
 * and operators stay in the code's ordinary colour so the kinds that matter
 * (keywords, strings, literals, functions, types) stand apart. */
const SCOPES: ReadonlyArray<readonly [readonly string[], string]> = [
  [["comment", "punctuation.definition.comment"], "var(--text-muted)"],
  [["keyword", "storage", "keyword.control"], "var(--code-keyword)"],
  [["keyword.operator", "punctuation"], PLAIN],
  [["string", "string punctuation.definition.string"], "var(--code-string)"],
  [
    ["constant.numeric", "constant.language", "constant.character", "constant.other.color"],
    "var(--code-literal)",
  ],
  [
    ["entity.name.function", "support.function", "meta.function-call entity.name.function"],
    "var(--code-key)",
  ],
  [
    [
      "entity.name.type",
      "entity.name.class",
      "support.type",
      "support.class",
      "storage.type.primitive",
      "entity.name.type.primitive",
    ],
    "var(--code-type)",
  ],
  [["entity.name.tag", "markup.heading"], "var(--code-keyword)"],
  [
    ["entity.other.attribute-name", "support.type.property-name", "meta.object-literal.key"],
    "var(--code-key)",
  ],
  [["markup.inserted"], "var(--code-string)"],
  [["markup.deleted"], "var(--code-keyword)"],
  [["markup.underline.link", "string.other.link"], "var(--accent)"],
];
const theme = {
  name: THEME,
  type: "light" as const,
  fg: PLAIN,
  bg: "transparent",
  settings: [
    { settings: { foreground: PLAIN } },
    ...SCOPES.map(([scope, foreground]) => ({ scope: [...scope], settings: { foreground } })),
  ],
};
function colour(value: string | undefined): string | undefined {
  return value && value !== PLAIN && value.startsWith("var(--") ? value : undefined;
}
let core: Promise<HighlighterCore> | undefined;
const loaded = new Map<Language, Promise<void>>();

function highlighter(): Promise<HighlighterCore> {
  core ??= (async () => {
    const [{ createHighlighterCore }, { createJavaScriptRegexEngine }] = await Promise.all([
      import("shiki/core"),
      import("shiki/engine/javascript"),
    ]);
    return createHighlighterCore({
      // Colour only: the theme sets no bold or italic, which would change how wide the code is.
      themes: [theme],
      langs: [],
      engine: createJavaScriptRegexEngine(),
    });
  })();
  return core;
}

function load(language: Language): Promise<void> {
  let ready = loaded.get(language);
  if (!ready) {
    ready = Promise.all([highlighter(), LOADERS[language]()]).then(([shiki, grammar]) =>
      shiki.loadLanguage(grammar.default),
    );
    // A chunk that failed to load is tried again the next time.
    ready.catch(() => loaded.delete(language));
    loaded.set(language, ready);
  }
  return ready;
}

/** `text` as lines of coloured runs. */
export async function highlight(text: string, language: Language): Promise<Token[][]> {
  await load(language);
  const shiki = await highlighter();
  return shiki.codeToTokensBase(text, { lang: language, theme: THEME }).map((line) =>
    line.map(({ content, color }) => {
      const shown = colour(color);
      return shown ? { content, color: shown } : { content };
    }),
  );
}
