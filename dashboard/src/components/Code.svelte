<script lang="ts">
import { type Highlight, NUMBERED_LINE } from "../lib/codeLanguage";
import { highlight, type Token } from "../lib/highlighter";

// A block of archived text, coloured when its language is certain. The plain
// text shows at once; colours replace it only once the block is on screen and
// its language has loaded, with exactly the same characters, so nothing moves
// and selecting or copying gives the original text.
let {
  text,
  language,
  numbered = false,
  tone,
  class: className = "",
  wrap = false,
  fence = false,
}: {
  text: string;
  language?: Highlight | undefined;
  /** Lines start with `12\t` line numbers, shown apart and left uncoloured. */
  numbered?: boolean | undefined;
  /** Text taken out by an edit, or put in. */
  tone?: "removed" | "added" | undefined;
  class?: string;
  wrap?: boolean;
  /** A code fence from a message, which keeps its `<code>` element. */
  fence?: boolean;
} = $props();

/** Larger blocks stay plain: colouring them would hold up the page. */
const LIMIT = 100_000;
const LONGEST_LINE = 2_000;

const gutters = $derived(
  numbered ? text.split("\n").map((line) => line.match(NUMBERED_LINE)?.[0] ?? "") : [],
);
const code = $derived(
  numbered
    ? text
        .split("\n")
        .map((line) => line.replace(NUMBERED_LINE, ""))
        .join("\n")
    : text,
);
const colourable = $derived(
  language !== undefined &&
    language !== "diff" &&
    code.length <= LIMIT &&
    code.split("\n").every((line) => line.length <= LONGEST_LINE),
);
let element = $state<HTMLPreElement>();
let tokens = $state<Token[][]>();

$effect(() => {
  const target = element;
  const wanted = colourable ? language : undefined;
  const source = code;
  tokens = undefined;
  if (!target || !wanted || wanted === "diff") return;
  let live = true;
  const observer = new IntersectionObserver(
    (entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return;
      observer.disconnect();
      highlight(source, wanted).then(
        (lines) => {
          if (live) tokens = lines;
        },
        // Without a highlighter the plain text is still right.
        () => {},
      );
    },
    { rootMargin: "400px 0px" },
  );
  observer.observe(target);
  return () => {
    live = false;
    observer.disconnect();
  };
});

/** A diff's lines, each marked by what it does. */
const diffLines = $derived(
  language === "diff"
    ? text.split("\n").map((line) => ({
        line,
        kind: /^(\+\+\+|---|diff --git|index )/.test(line)
          ? "head"
          : line.startsWith("@@")
            ? "hunk"
            : line.startsWith("+")
              ? "added"
              : line.startsWith("-")
                ? "removed"
                : "",
      }))
    : [],
);
</script>

<pre bind:this={element} class="hl {className}" class:wrap class:tone-removed={tone === "removed"} class:tone-added={tone === "added"} data-highlight={language ?? "plain"}>{#snippet body()}{#if language === "diff"}{#each diffLines as { line, kind }, index (index)}<span class="hl-line {kind}">{line}</span>{#if index < diffLines.length - 1}{"\n"}{/if}{/each}{:else if tokens}{#each tokens as line, index (index)}{#if numbered}<span class="hl-gutter">{gutters[index]}</span>{/if}{#each line as token, at (at)}<span style:color={token.color}>{token.content}</span>{/each}{#if index < tokens.length - 1}{"\n"}{/if}{/each}{:else}{text}{/if}{/snippet}{#if fence}<code>{@render body()}</code>{:else}{@render body()}{/if}</pre>

<style>
.hl-gutter {
  color: var(--text-faint);
}
.hl-line.added {
  color: var(--status-ok);
}
.hl-line.removed {
  color: var(--status-bad);
}
.hl-line.hunk {
  color: var(--code-key);
}
.hl-line.head {
  color: var(--text-muted);
  font-weight: 700;
}
.tone-removed {
  box-shadow: inset 3px 0 0 var(--status-bad);
}
.tone-added {
  box-shadow: inset 3px 0 0 var(--status-ok);
}
</style>
