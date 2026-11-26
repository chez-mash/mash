<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import CaretDoubleLeftIcon from "phosphor-svelte/lib/CaretDoubleLeftIcon";
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import CircleDashedIcon from "phosphor-svelte/lib/CircleDashedIcon";
import ListChecksIcon from "phosphor-svelte/lib/ListChecksIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import SealCheckIcon from "phosphor-svelte/lib/SealCheckIcon";
import { untrack } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { asset } from "../lib/asset";
import { groupByWeek, recordKey } from "../lib/memoryView";
import { navigate } from "../lib/navigation";
import { stableRead } from "../lib/queries";
import { kindIcons } from "../lib/recordIcons";
import { kinds, label, scopedRecordPage, states } from "../lib/records";
import { recordHref, recordsHref } from "../lib/route.svelte";
import { readingScope, scope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import FilterMenu from "./FilterMenu.svelte";
import MemoryItem from "./MemoryItem.svelte";
import MemoryPreview from "./MemoryPreview.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";

const now = Date.now();

let { params }: { params: string } = $props();
const initial = untrack(() => new URLSearchParams(params));
let query = $state(initial.get("q") ?? "");
let kind = $state(initial.get("kind") ?? "");
let lifecycle = $state(initial.get("state") ?? "");
let review = $state(initial.get("review") ?? "");
const chat = initial.get("chat") ?? "";
// Whose memories: the viewing scope chosen in the sidebar, the same as every page.
const scoped = createScoped();
// Older links named a source here; the scope has replaced it, so a link's
// source becomes the scope, as a link's scope= does.
if (initial.get("shared") === "true") {
  const node = initial.get("node");
  scope.choose(node ? { kind: "node", node } : { kind: "all" });
}
const own = $derived.by(() => {
  const next = new URLSearchParams(params);
  next.delete("shared");
  next.delete("node");
  return next.toString();
});
// Memory still shows when the chat list cannot be read (see readingScope).
const reading = $derived(readingScope(scoped.data, scoped.isError, scope.chosen));
const result = createQuery(() => ({
  ...stableRead,
  // Its own key: the decisions list caches plain pages under ["records", …].
  queryKey: ["records", "view", reading ? scopeText(reading.scope) : "", own],
  enabled: reading !== undefined,
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    reading
      ? scopedRecordPage(own, reading.scope, reading.reach, signal)
      : Promise.reject(new Error("The viewing scope is not known yet.")),
}));
const page = $derived({
  data: result.data?.page,
  error: result.error,
  isPending: result.isPending,
});
const hasNext = $derived(
  page.data && (page.data.nextCursor !== null || page.data.nextOffset !== null),
);
function submit(event: SubmitEvent): void {
  event.preventDefault();
  const next = new URLSearchParams();
  if (query.trim()) next.set("q", query.trim());
  if (kind) next.set("kind", kind);
  if (lifecycle) next.set("state", lifecycle);
  if (review) next.set("review", review);
  if (chat) next.set("chat", chat);
  navigate(recordsHref(next));
}
function nextPage(): void {
  if (!page.data) return;
  const next = new URLSearchParams(own);
  next.delete("offset");
  next.delete("cursor_node");
  next.delete("cursor_offset");
  if (page.data.nextCursor) {
    next.set("cursor_node", page.data.nextCursor.node);
    next.set("cursor_offset", String(page.data.nextCursor.offset));
  } else if (page.data.nextOffset !== null) next.set("offset", String(page.data.nextOffset));
  navigate(recordsHref(next));
}
function firstPage(): void {
  const next = new URLSearchParams(params);
  next.delete("offset");
  next.delete("cursor_node");
  next.delete("cursor_offset");
  navigate(recordsHref(next));
}
function kindHref(value: string): string {
  const next = new URLSearchParams(params);
  next.delete("offset");
  next.delete("cursor_node");
  next.delete("cursor_offset");
  if (value) next.set("kind", value);
  else next.delete("kind");
  return recordsHref(next);
}
// The chips are a draft until the form is submitted.
const dirty = $derived(
  query.trim() !== (initial.get("q") ?? "") ||
    lifecycle !== (initial.get("state") ?? "") ||
    review !== (initial.get("review") ?? ""),
);
const back = $derived(recordsHref(new URLSearchParams(params)));
const groups = $derived(page.data ? groupByWeek(page.data.records, now) : []);
// The preview shows the row last pointed at or reached by keyboard, and the
// first row until then.
let chosen = $state("");
const previewed = $derived(
  page.data?.records.find((record) => recordKey(record) === chosen) ?? page.data?.records[0],
);
const localNode = $derived(scoped.data?.reach.localNode ?? "");
</script>

<div class="mem-page">
<div class="mem-main">
<header class="mem-h">
  <div class="mem-t"><h1 class="h1">Memory</h1><p class="meta">What your agents saved, with the messages behind it<ScopeNote scoped={scoped.data} fallback={result.data?.fallback} /></p></div>
</header>
<div class="mem-bar">
  <nav class="mem-seg" aria-label="Memory kind">
    <a href={kindHref("")} aria-current={!kind ? "page" : undefined}>All</a>
    {#each kinds as value (value)}
      {@const KindGlyph = kindIcons[value]}<a href={kindHref(value)} aria-current={kind === value ? "page" : undefined}><span class="ink-{value} seg-i"><Icon icon={KindGlyph} size={13} state={kind === value ? "on" : "rest"} /></span>{label(value)}s</a>
    {/each}
  </nav>
  <form onsubmit={submit} class="mem-f" aria-label="Memory filters">
    <label class="find mem-q"><Icon icon={MagnifyingGlassIcon} size={15} /><span class="sr-only">Search statements</span><input id="memory-q" type="search" bind:value={query} placeholder="Find a statement" /></label>
    <FilterMenu label={lifecycle ? label(lifecycle).toLowerCase() : "any status"} active={!!lifecycle} draft={lifecycle !== (initial.get("state") ?? "")} options={[{ value: "", label: "any status" }, ...states.map((value) => ({ value, label: label(value).toLowerCase() }))]} selected={[lifecycle]} choose={(value) => (lifecycle = value)}>
      {#snippet icon()}<Icon icon={ListChecksIcon} size={15} />{/snippet}
    </FilterMenu>
    <FilterMenu label={review || "any review"} active={!!review} draft={review !== (initial.get("review") ?? "")} options={[{ value: "", label: "any review" }, { value: "established", label: "established" }, { value: "candidate", label: "candidate (unconfirmed)" }]} selected={[review]} choose={(value) => (review = value)}>
      {#snippet icon()}<Icon icon={SealCheckIcon} size={15} />{/snippet}
      {#snippet optionIcon(value: string)}{#if value === "candidate"}<Icon icon={CircleDashedIcon} size={15} />{:else}<Icon icon={SealCheckIcon} size={15} />{/if}{/snippet}
    </FilterMenu>
    {#if dirty}<button class="a" id="applyf" type="submit" style:min-height="30px" style:padding="0 6px" style:font-size="12.5px">Apply filters</button>{/if}
  </form>
</div>
{#if chat}<p class="card tight cream meta">Memory citing chat <code style:overflow-wrap="anywhere">{chat}</code>. <a class="a" href="#/records?">Show all memory</a></p>{/if}
{#if page.isPending}<Skeleton rows={5} />{/if}
<QueryError title="Couldn’t load memory" query={result} />
{#if page.data && !page.error}
  {#if !page.data.records.length}
    <p class="card tight note"><img src={asset("mashy.png")} alt="" /><span>{hasNext ? "No matching records on this page. Continue to the next page." : "No memory matches these filters. Agents save decisions and preferences as you state them, and notes when you ask."}</span></p>
  {:else}
    <div class="mem-split">
      <section class="mem-l" aria-label="Memory">
        {#each groups as group (group.label)}
          <h2 class="mem-g">{group.label}</h2>
          <div class="mem-c">
            {#each group.records as record (recordKey(record))}
              <MemoryItem {record} {back} {now} machine={localNode || "this machine"} selected={previewed === record} preview={() => (chosen = recordKey(record))} />
            {/each}
          </div>
        {/each}
      </section>
    </div>
  {/if}
  {#if initial.has("offset") || initial.has("cursor_node") || hasNext}
    <nav aria-label="Record pages" class="hd">
      <span class="meta">{page.data.records.length} on this page</span>
      <span class="acts">
        <button class="btn sm" type="button" disabled={!(initial.has("offset") || initial.has("cursor_node"))} onclick={firstPage}><Icon icon={CaretDoubleLeftIcon} size={14} />First page</button>
        <button class="btn sm" type="button" disabled={!hasNext} onclick={nextPage}>Next page<Icon icon={CaretRightIcon} size={14} /></button>
      </span>
    </nav>
  {/if}
{/if}
</div>
<!-- The memory pointed at, in a full-height panel at the page's right edge. -->
{#if page.data?.records.length && !page.error && previewed}
  <div class="mem-p">{#key recordKey(previewed)}<MemoryPreview record={previewed} {back} {now} chats={scoped.data?.chats} />{/key}</div>
{/if}
</div>

<style>
  /* The page fills the column beside the rail: the list on the left, under
     a header that ends in a hairline, and the memory pointed at in a panel
     reaching from the top bar to the foot of the screen at the right. */
  :global(#app #main:has(> .mem-page)) {
    max-width: none;
    padding: 0;
    gap: 0;
  }
  .mem-page {
    flex: 1;
    display: flex;
    align-items: stretch;
    min-height: calc(100dvh - var(--top-h, 52px));
  }
  .mem-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px 24px 56px 32px;
  }
  .mem-p {
    flex: 0 0 340px;
    position: sticky;
    top: var(--top-h, 52px);
    align-self: flex-start;
    height: calc(100dvh - var(--top-h, 52px));
    overflow-y: auto;
    border-left: var(--border-divider);
    background: var(--surface-rail);
    scrollbar-width: thin;
  }
  .mem-h {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .mem-t {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 6px 12px;
  }
  .mem-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin: 0 -24px 0 -32px;
    padding: 0 24px 16px 32px;
    border-bottom: var(--border-divider);
  }
  /* The kinds as a segmented control: a muted track, the current kind raised. */
  .mem-seg {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-6);
    background: var(--surface-muted);
    max-width: 100%;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .mem-seg a {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 11px;
    border-radius: var(--radius-5);
    font-size: 13px;
    font-weight: var(--weight-label);
    color: var(--text-muted);
    white-space: nowrap;
  }
  .mem-seg a:hover {
    color: var(--text-strong);
  }
  .mem-seg a[aria-current="page"] {
    background: var(--selected-bg);
    color: var(--text-strong);
    box-shadow: var(--shadow-pick);
  }
  .seg-i {
    display: inline-flex;
  }
  .mem-f {
    display: flex;
    flex: 1 1 320px;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .mem-q {
    flex: 1 1 200px;
    min-height: 34px;
    border: var(--border-outline);
    border-radius: var(--radius-5);
  }
  .mem-split {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
  }
  .mem-l {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .mem-c {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .mem-g {
    padding: 8px 12px 4px;
    font-size: 12px;
    font-weight: var(--weight-label);
    color: var(--text-faint);
  }
  .mem-c + .mem-g {
    padding-top: 14px;
  }
  @container (max-width: 860px) {
    .mem-p {
      display: none;
    }
  }
  @media (max-width: 800px) {
    .mem-main {
      padding: 20px 16px 48px;
    }
    .mem-bar {
      margin-inline: -16px;
      padding-inline: 16px;
    }
  }
  /* On a phone each group is a card of rows. */
  @container (max-width: 600px) {
    .mem-g {
      padding: 4px 2px 0;
    }
    .mem-c + .mem-g {
      padding-top: 12px;
    }
    .mem-c {
      gap: 0;
      margin-top: 6px;
      border: var(--border-outline);
      border-radius: var(--radius-7);
      background: var(--surface-raised);
      overflow: hidden;
    }
    .mem-c > :global(article + article) {
      border-top: var(--border-divider);
    }
    .mem-f {
      flex-basis: 100%;
    }
    .mem-q {
      flex-basis: 100%;
    }
  }
</style>
