// Which language a block of archived text is written in, when that is
// certain from how the text was produced: a code fence's tag, the path of the
// file a tool read or wrote, a command that printed exactly one file, or output
// that is a diff. Everything else stays plain text.
//
// Guessing a language from the content alone is deliberately out of scope.
// Most command output mixes several things (logs, file excerpts, the output of
// several commands), and a wrong guess reads worse than no colour at all.
import type { ToolArgument, ToolCall } from "./toolcalls";

/** The languages the dashboard can colour, by their Shiki names. */
export const LANGUAGES = [
  "astro",
  "c",
  "cpp",
  "csharp",
  "css",
  "go",
  "html",
  "java",
  "javascript",
  "json",
  "jsx",
  "kotlin",
  "lua",
  "markdown",
  "php",
  "powershell",
  "python",
  "ruby",
  "rust",
  "scss",
  "shellscript",
  "sql",
  "svelte",
  "swift",
  "toml",
  "tsx",
  "typescript",
  "xml",
  "yaml",
] as const;
export type Language = (typeof LANGUAGES)[number];
/** A language Shiki colours, or a diff, which the dashboard colours itself. */
export type Highlight = Language | "diff";

const EXTENSIONS: Readonly<Record<string, Language>> = {
  astro: "astro",
  c: "c",
  h: "c",
  cc: "cpp",
  cpp: "cpp",
  cxx: "cpp",
  hpp: "cpp",
  cs: "csharp",
  css: "css",
  go: "go",
  htm: "html",
  html: "html",
  java: "java",
  cjs: "javascript",
  js: "javascript",
  mjs: "javascript",
  json: "json",
  jsonc: "json",
  jsonl: "json",
  jsx: "jsx",
  kt: "kotlin",
  kts: "kotlin",
  lua: "lua",
  md: "markdown",
  mdx: "markdown",
  php: "php",
  ps1: "powershell",
  py: "python",
  pyi: "python",
  rb: "ruby",
  rs: "rust",
  scss: "scss",
  bash: "shellscript",
  sh: "shellscript",
  zsh: "shellscript",
  sql: "sql",
  svelte: "svelte",
  swift: "swift",
  toml: "toml",
  tsx: "tsx",
  cts: "typescript",
  mts: "typescript",
  ts: "typescript",
  svg: "xml",
  xml: "xml",
  yaml: "yaml",
  yml: "yaml",
};

/** Fence tags people write, beyond the extensions and Shiki's own names. */
const FENCE_NAMES: Readonly<Record<string, Highlight>> = {
  console: "shellscript",
  diff: "diff",
  golang: "go",
  javascript: "javascript",
  patch: "diff",
  py3: "python",
  python3: "python",
  shell: "shellscript",
  typescript: "typescript",
};

function isLanguage(value: string): value is Language {
  return (LANGUAGES as readonly string[]).includes(value);
}

/** The language of a file, from its extension; nothing for anything else. */
export function pathLanguage(path: string): Language | undefined {
  const name = path.trim().split(/[\\/]/).pop() ?? "";
  const dot = name.lastIndexOf(".");
  if (dot <= 0 && !name.startsWith(".")) return undefined;
  return EXTENSIONS[name.slice(dot + 1).toLowerCase()];
}

/** The language a code fence names, such as ```rust; untagged fences are plain. */
export function fenceLanguage(tag: string | undefined): Highlight | undefined {
  const name = tag
    ?.trim()
    .split(/\s+/)[0]
    ?.toLowerCase()
    .replace(/^\{?\.?|\}$/g, "");
  if (!name) return undefined;
  if (FENCE_NAMES[name]) return FENCE_NAMES[name];
  if (isLanguage(name)) return name;
  return EXTENSIONS[name];
}

/** Text that is a unified diff from its first line. */
export function isDiff(text: string): boolean {
  const start = text.trimStart();
  // A `---` line alone is common in logs; a diff follows it with `+++`.
  return /^(diff --git |@@ -\d)/.test(start) || /^--- \S.*\n\+\+\+ \S/.test(start);
}

/** Text that is JSON, whole or one value per line. */
export function isJson(text: string): boolean {
  const trimmed = text.trim();
  if (!/^[[{]/.test(trimmed)) return false;
  try {
    JSON.parse(trimmed);
    return true;
  } catch {
    const lines = trimmed.split("\n").filter((line) => line.trim());
    if (lines.length < 2) return false;
    try {
      for (const line of lines) JSON.parse(line);
      return true;
    } catch {
      return false;
    }
  }
}

// Commands that print a file unchanged. `nl` and `cat -n` add numbers, and
// every other command's output is not the file's language for certain.
const VIEWERS = new Set(["cat", "head", "tail", "sed", "bat", "batcat"]);
/** Steps that set up a command without printing anything. */
const QUIET = /^(cd|export|set|unset|pushd|popd|true|:)(\s|$)/;
/** Filters that keep a file's text as it is, only shorter. */
const TRIMS = /^(head|tail)(\s+(-n\s*)?-?\d+|\s+-c\s*\d+)?\s*$/;

function words(segment: string): string[] | undefined {
  const found: string[] = [];
  const pattern = /'([^']*)'|"((?:[^"\\$`]|\\.)*)"|([^\s'"`$<>|&;()]+)|(\S)/g;
  for (let match = pattern.exec(segment); match; match = pattern.exec(segment)) {
    // A substitution, redirection or glob makes the printed text uncertain.
    if (match[4] !== undefined) return undefined;
    const word = match[1] ?? match[2] ?? match[3] ?? "";
    if (/[*?[\]{}]/.test(match[3] ?? "")) return undefined;
    found.push(word);
  }
  return found;
}

/** The one file a shell command prints unchanged, such as `sed -n 1,80p a.rs`.
 * Anything that prints more than that one file has no file. */
export function commandFile(command: string): string | undefined {
  if (/<<|\$\(|`|>/.test(command)) return undefined;
  const steps = command
    .split(/&&|\|\||;|\n/)
    .map((step) => step.trim())
    .filter((step) => step && !QUIET.test(step));
  if (steps.length !== 1) return undefined;
  const [first, ...filters] = (steps[0] ?? "").split("|").map((part) => part.trim());
  if (!first || !filters.every((filter) => TRIMS.test(filter))) return undefined;
  const parts = words(first);
  const name = parts?.[0];
  if (!parts || !name || !VIEWERS.has(name)) return undefined;
  const rest = parts.slice(1);
  const operands: string[] = [];
  for (let index = 0; index < rest.length; index++) {
    const part = rest[index] ?? "";
    if (part === "-n" && name === "cat") return undefined;
    if (part.startsWith("-")) {
      // `head -n 40`, `tail -c 200`: the flag's number is not a file.
      if (name !== "sed" && /^-[nc]$/.test(part)) index++;
      continue;
    }
    operands.push(part);
  }
  // sed's first operand is its script, and only a print-only script is safe.
  if (name === "sed") {
    const script = operands.shift();
    if (!rest.includes("-n") || !script || !/^[\d,$]*p$/.test(script)) return undefined;
  }
  const file = operands.length === 1 ? operands[0] : undefined;
  return file && !/^\d+$/.test(file) ? file : undefined;
}

/** Argument names that hold a file's path, across agents' tools. */
const PATH_KEYS = [
  "file_path",
  "path",
  "target_file",
  "filePath",
  "file",
  "filename",
  "absolute_path",
];
/** Argument names that hold text written into that file. */
const CONTENT_KEYS = new Set([
  "content",
  "contents",
  "file_text",
  "code_edit",
  "new_string",
  "new_str",
  "old_string",
  "old_str",
  "text",
]);
/** An edit's replaced text and its replacement. */
const TONES: Readonly<Record<string, "removed" | "added">> = {
  old_string: "removed",
  old_str: "removed",
  new_string: "added",
  new_str: "added",
};
const COMMAND_KEYS = ["command", "cmd"];
/** Everything a plain file read may also say: which lines, and why. */
const READ_KEYS = new Set([
  ...PATH_KEYS,
  "offset",
  "limit",
  "start_line",
  "end_line",
  "view_range",
  "start_line_one_indexed",
  "end_line_one_indexed_inclusive",
  "should_read_entire_file",
  "explanation",
]);

function argument(call: ToolCall, keys: readonly string[]): string | undefined {
  for (const key of keys) {
    const found = call.args.find((arg) => arg.key === key)?.text.trim();
    if (found) return found;
  }
  return undefined;
}

/** A command argument as one shell line: Codex stores `["bash", "-lc", "…"]`. */
function commandOf(call: ToolCall): string | undefined {
  const value = argument(call, COMMAND_KEYS);
  if (!value?.startsWith("[")) return value;
  try {
    const parts: unknown = JSON.parse(value);
    if (!Array.isArray(parts) || !parts.every((part) => typeof part === "string")) return undefined;
    const shell = /^(bash|sh|zsh)$/.test(parts[0] ?? "") && /^-l?c$/.test(parts[1] ?? "");
    return shell && parts.length === 3 ? parts[2] : parts.join(" ");
  } catch {
    return undefined;
  }
}

/** A Codex patch: `*** Begin Patch`, file headers, hunks, `*** End Patch`. */
function isPatch(text: string): boolean {
  const trimmed = text.trim();
  return trimmed.startsWith("*** Begin Patch\n") && trimmed.endsWith("\n*** End Patch");
}

/**
 * The text of one JavaScript string literal starting at `text[at]`, and where
 * it ends: a quoted string, a template without `${…}`, or `String.raw`. Null
 * for anything it can't read exactly, so a doubtful literal stays as written.
 */
function literal(text: string, at: number): { value: string; end: number } | null {
  const raw = text.startsWith("String.raw`", at);
  const start = raw ? at + "String.raw".length : at;
  const quote = text[start];
  if (quote !== '"' && quote !== "'" && quote !== "`") return null;
  const escapes: Readonly<Record<string, string>> = { n: "\n", t: "\t", r: "\r", "0": "\0" };
  let value = "";
  for (let index = start + 1; index < text.length; index++) {
    const char = text[index] as string;
    if (char === quote) return { value, end: index + 1 };
    if (quote === "`" && char === "$" && text[index + 1] === "{") return null;
    if (quote !== "`" && char === "\n") return null;
    if (char !== "\\") {
      value += char;
      continue;
    }
    const next = text[index + 1];
    if (next === undefined) return null;
    if (raw) {
      value += char + next;
    } else if (next === "u") {
      const hex = text.slice(index + 2, index + 6);
      if (!/^[0-9a-fA-F]{4}$/.test(hex)) return null;
      value += String.fromCharCode(Number.parseInt(hex, 16));
      index += 4;
    } else if (escapes[next] !== undefined) {
      value += escapes[next];
    } else if ("\\'\"`$\n".includes(next)) {
      // An escaped newline continues the literal on the next line.
      if (next !== "\n") value += next;
    } else {
      return null;
    }
    index++;
  }
  return null;
}

/** One line of Codex `exec` code that applies one patch and does nothing else:
 * `text(await tools.apply_patch(…));` or `await tools.apply_patch(…)`. */
const APPLY_PATCH_LINE = /^[ \t]*(text\(\s*)?await\s+tools\.apply_patch\(\s*/gm;

/**
 * What a Codex call's argument shows when it applies patches: the raw patch,
 * or the `exec` code with each line that only applies a patch replaced by
 * that patch. Every other line is kept exactly as written. Undefined when the
 * argument applies no patch this can read exactly.
 */
export function codexPatch(text: string): string | undefined {
  if (isPatch(text)) return text;
  let shown = "";
  let from = 0;
  let replaced = 0;
  for (const match of text.matchAll(APPLY_PATCH_LINE)) {
    const opened = match.index + match[0].length;
    const read = literal(text, opened);
    if (!read || !isPatch(read.value)) continue;
    const closing = text
      .slice(read.end)
      .match(match[1] ? /^\s*\)\s*\)[ \t]*;?[ \t]*(?=\n|$)/ : /^\s*\)[ \t]*;?[ \t]*(?=\n|$)/);
    if (!closing) continue;
    shown += text.slice(from, match.index) + read.value.trim();
    from = read.end + closing[0].length;
    replaced++;
  }
  return replaced ? shown + text.slice(from) : undefined;
}

/** How one argument of a call is coloured, and whether an edit took it out or put it in. */
export interface ArgumentHighlight {
  readonly language: Highlight;
  readonly tone?: "removed" | "added";
  /** What to show instead of the argument's text: a patch decoded from the
   * code that applied it. */
  readonly text?: string;
}
export function argumentHighlight(
  call: ToolCall,
  arg: ToolArgument,
): ArgumentHighlight | undefined {
  if (!arg.block) return undefined;
  if (CONTENT_KEYS.has(arg.key)) {
    const path = argument(call, PATH_KEYS);
    const language = path ? pathLanguage(path) : undefined;
    const tone = TONES[arg.key];
    if (language) return tone ? { language, tone } : { language };
  }
  if (arg.key === "patch" || arg.key === "diff")
    return isDiff(arg.text) || isPatch(arg.text) ? { language: "diff" } : undefined;
  const patch = codexPatch(arg.text);
  if (patch) return patch === arg.text ? { language: "diff" } : { language: "diff", text: patch };
  return isJson(arg.text) && /^[[{]/.test(arg.text.trim()) ? { language: "json" } : undefined;
}

/** Read tools number each line as `12\t…`; that gutter is not the file's text. */
export const NUMBERED_LINE = /^\s*\d+\t/;

/** How a call's output is coloured, when that is certain. */
export interface OutputHighlight {
  readonly language: Highlight;
  /** Every line starts with a line number to keep out of the colouring. */
  readonly numbered?: boolean;
}
export function outputHighlight(call: ToolCall): OutputHighlight | undefined {
  const text = call.output?.text ?? "";
  if (!text.trim() || call.output?.json !== undefined) return undefined;
  if (isDiff(text)) return { language: "diff" };
  if (isJson(text)) return { language: "json" };
  const command = commandOf(call);
  const shown = command ? commandFile(command) : argument(call, PATH_KEYS);
  // A call that ran a command is coloured only when it printed one file. One
  // that ran nothing is a read only when it names a file and at most which
  // lines of it: a pattern or new content means the output is something else.
  const language = shown ? pathLanguage(shown) : undefined;
  if (!language || (!command && !call.args.every((arg) => READ_KEYS.has(arg.key))))
    return undefined;
  const lines = text.split("\n").filter((line) => line.trim());
  const numbered = lines.filter((line) => NUMBERED_LINE.test(line)).length;
  if (numbered === lines.length) return { language, numbered: true };
  // Partly numbered output has notes mixed into the file; leave it plain.
  return numbered ? undefined : { language };
}
