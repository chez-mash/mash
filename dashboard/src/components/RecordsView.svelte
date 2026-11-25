<script lang="ts">
import BadgeCheck from "@lucide/svelte/icons/badge-check";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import ListChecks from "@lucide/svelte/icons/list-checks";
import SearchIcon from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { asset } from "../lib/asset";
import { groupByWeek, recordKey } from "../lib/memoryView";
import { navigate } from "../lib/navigation";
import { stableRead } from "../lib/queries";
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

<header class="mem-h">
  <div class="mem-t"><h1 class="h1">Memory</h1><p class="meta">What your agents saved, with the messages behind it</p></div>
  <ScopeNote scoped={scoped.data} fallback={result.data?.fallback} />
</header>
<div class="mem-bar">
  <nav class="mem-seg" aria-label="Memory kind">
    <a href={kindHref("")} aria-current={!kind ? "page" : undefined}>All</a>
    {#each kinds as value (value)}
      <a href={kindHref(value)} aria-current={kind === value ? "page" : undefined}>{label(value)}s</a>
    {/each}
  </nav>
  <form onsubmit={submit} class="mem-f" aria-label="Memory filters">
    <label class="find mem-q"><SearchIcon size={15} aria-hidden="true" /><span class="sr-only">Search statements</span><input id="memory-q" type="search" bind:value={query} placeholder="Find a statement" /></label>
    <FilterMenu label={lifecycle ? label(lifecycle).toLowerCase() : "any status"} active={!!lifecycle} draft={lifecycle !== (initial.get("state") ?? "")} options={[{ value: "", label: "any status" }, ...states.map((value) => ({ value, label: label(value).toLowerCase() }))]} selected={[lifecycle]} choose={(value) => (lifecycle = value)}>
      {#snippet icon()}<ListChecks size={15} aria-hidden="true" />{/snippet}
    </FilterMenu>
    <FilterMenu label={review || "any review"} active={!!review} draft={review !== (initial.get("review") ?? "")} options={[{ value: "", label: "any review" }, { value: "established", label: "established" }, { value: "candidate", label: "candidate (unconfirmed)" }]} selected={[review]} choose={(value) => (review = value)}>
      {#snippet icon()}<BadgeCheck size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if value === "candidate"}<CircleDashed size={15} aria-hidden="true" />{:else}<BadgeCheck size={15} aria-hidden="true" />{/if}{/snippet}
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
      {#if previewed}
        <div class="mem-p">{#key recordKey(previewed)}<MemoryPreview record={previewed} {back} {now} chats={scoped.data?.chats} />{/key}</div>
      {/if}
    </div>
  {/if}
  {#if initial.has("offset") || initial.has("cursor_node") || hasNext}
    <nav aria-label="Record pages" class="hd">
      <span class="meta">{page.data.records.length} on this page</span>
      <span class="acts">
        <button class="btn sm" type="button" disabled={!(initial.has("offset") || initial.has("cursor_node"))} onclick={firstPage}><ChevronsLeft size={14} aria-hidden="true" />First page</button>
        <button class="btn sm" type="button" disabled={!hasNext} onclick={nextPage}>Next page<ChevronRight size={14} aria-hidden="true" /></button>
      </span>
    </nav>
  {/if}
{/if}

<style>
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
    height: 28px;
    padding: 0 11px;
    border-radius: var(--radius-5);
    font-size: 13px;
    font-weight: 500;
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
  /* The list and, on a wide page, the memory pointed at beside it. */
  .mem-split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(300px, 380px);
    gap: 24px;
    align-items: start;
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
    font-weight: 500;
    color: var(--text-faint);
  }
  .mem-c + .mem-g {
    padding-top: 14px;
  }
  .mem-p {
    position: sticky;
    top: 16px;
    min-width: 0;
  }
  @container (max-width: 860px) {
    .mem-split {
      grid-template-columns: minmax(0, 1fr);
    }
    .mem-p {
      display: none;
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
