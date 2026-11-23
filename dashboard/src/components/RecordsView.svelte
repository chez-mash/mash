<script lang="ts">
import BadgeCheck from "@lucide/svelte/icons/badge-check";
import Bookmark from "@lucide/svelte/icons/bookmark";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import Database from "@lucide/svelte/icons/database";
import Laptop from "@lucide/svelte/icons/laptop";
import ListChecks from "@lucide/svelte/icons/list-checks";
import Network from "@lucide/svelte/icons/network";
import SearchIcon from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { agentNamed } from "../lib/agents";
import { asset } from "../lib/asset";
import { whenLabel } from "../lib/browse";
import { navigate } from "../lib/navigation";
import { catalogOptions, stableRead } from "../lib/queries";
import { kindIcons, scopeIcons } from "../lib/recordIcons";
import {
  kindPill,
  kinds,
  label,
  memorySourceLabel,
  memorySources,
  recordPage,
  scopeLabel,
  scopeName,
  standing,
  statePill,
  states,
  timingLabel,
} from "../lib/records";
import { recordHref, recordsHref } from "../lib/route.svelte";
import AgentMark from "./AgentMark.svelte";
import FilterMenu from "./FilterMenu.svelte";
import QueryError from "./QueryError.svelte";
import Skeleton from "./Skeleton.svelte";

const now = Date.now();

let { params }: { params: string } = $props();
const initial = untrack(() => new URLSearchParams(params));
let query = $state(initial.get("q") ?? "");
let kind = $state(initial.get("kind") ?? "");
let lifecycle = $state(initial.get("state") ?? "");
let review = $state(initial.get("review") ?? "");
let source = $state(
  initial.get("shared") === "true"
    ? initial.get("node")
      ? `node:${initial.get("node")}`
      : "shared"
    : "",
);
const chat = initial.get("chat") ?? "";
const catalogue = createQuery(() => catalogOptions);
const nodes = $derived([
  ...new Set([
    ...(catalogue.data?.chats.map((chat) => chat.machine) ?? []),
    ...(initial.get("node") ? [initial.get("node") ?? ""] : []),
  ]),
]);
// Where records are read from; see `memorySources` for who is "this machine".
const localNode = $derived(catalogue.data?.localNode ?? "");
const sharedOnly = $derived(catalogue.data?.sharedOnly ?? false);
const sources = $derived(memorySources(localNode, sharedOnly, nodes, source));
const sourceLabel = $derived(memorySourceLabel(source, sharedOnly));
const page = createQuery(() => ({
  ...stableRead,
  queryKey: ["records", params],
  queryFn: ({ signal }) => recordPage(params, signal),
}));
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
  if (source) {
    next.set("shared", "true");
    if (source.startsWith("node:")) next.set("node", source.slice(5));
  }
  navigate(recordsHref(next));
}
function nextPage(): void {
  if (!page.data) return;
  const next = new URLSearchParams(params);
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
    review !== (initial.get("review") ?? "") ||
    source !==
      (initial.get("shared") === "true"
        ? initial.get("node")
          ? `node:${initial.get("node")}`
          : "shared"
        : ""),
);
</script>

<div><h1 class="h1">Memory</h1><p class="meta">Decisions, preferences and facts, with the conversations behind them.</p></div>
<nav class="chips" aria-label="Memory kind" style:gap="10px">
  <a class="btn" class:sel={!kind} href={kindHref("")} aria-current={!kind ? "page" : undefined}><Bookmark size={15} aria-hidden="true" />all</a>
  {#each kinds as value (value)}
    {@const Icon = kindIcons[value]}
    <a class="btn" class:sel={kind === value} href={kindHref(value)} aria-current={kind === value ? "page" : undefined}><Icon size={15} aria-hidden="true" />{label(value).toLowerCase()}s</a>
  {/each}
</nav>
<form onsubmit={submit} class="chips" aria-label="Memory filters" style:gap="10px">
  <label class="find"><SearchIcon size={15} aria-hidden="true" /><span class="sr-only">Search statements</span><input id="memory-q" type="search" bind:value={query} placeholder="find a statement" /></label>
  <FilterMenu label={lifecycle ? label(lifecycle).toLowerCase() : "any status"} active={!!lifecycle} draft={lifecycle !== (initial.get("state") ?? "")} options={[{ value: "", label: "any status" }, ...states.map((value) => ({ value, label: label(value).toLowerCase() }))]} selected={[lifecycle]} choose={(value) => (lifecycle = value)}>
    {#snippet icon()}<ListChecks size={15} aria-hidden="true" />{/snippet}
  </FilterMenu>
  <FilterMenu label={review || "any review"} active={!!review} draft={review !== (initial.get("review") ?? "")} options={[{ value: "", label: "any review" }, { value: "established", label: "established" }, { value: "candidate", label: "candidate (unconfirmed)" }]} selected={[review]} choose={(value) => (review = value)}>
    {#snippet icon()}<BadgeCheck size={15} aria-hidden="true" />{/snippet}
    {#snippet optionIcon(value: string)}{#if value === "candidate"}<CircleDashed size={15} aria-hidden="true" />{:else}<BadgeCheck size={15} aria-hidden="true" />{/if}{/snippet}
  </FilterMenu>
  <FilterMenu label={sourceLabel} active={!!source} options={sources} selected={[source]} choose={(value) => (source = value)}>
    {#snippet icon()}<Database size={15} aria-hidden="true" />{/snippet}
    {#snippet optionIcon(value: string)}{#if value === ""}<Laptop size={15} aria-hidden="true" />{:else if value === "shared"}<Network size={15} aria-hidden="true" />{:else}<Server size={15} aria-hidden="true" />{/if}{/snippet}
  </FilterMenu>
  {#if dirty}<button class="a" id="applyf" type="submit" style:min-height="30px" style:padding="0 6px" style:font-size="12.5px">Apply filters</button>{/if}
</form>
{#if chat}<p class="card tight cream meta">Memory citing chat <code style:overflow-wrap="anywhere">{chat}</code>. <a class="a" href="#/records?">Show all memory</a></p>{/if}
{#if page.isPending}<Skeleton rows={5} />{/if}
<QueryError title="Couldn’t load memory" query={page} />
{#if page.data && !page.error}
  {#if !page.data.records.length}
    <p class="card tight note"><img src={asset("mashy.png")} alt="" /><span>{hasNext ? "No matching records on this page. Continue to the next page." : "No memory matches these filters. Agents save decisions, preferences and facts as you state them."}</span></p>
  {:else}
    <section class="card pad0 list" aria-label="Memory">
      {#each page.data.records as record (JSON.stringify([record.node, record.owner, record.id]))}
        {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
        {#if revision}
          {@const Icon = kindIcons[revision.details.kind]}
          {@const ScopeIcon = scopeIcons[record.scope.kind]}
          {@const settled = standing(revision)}
          {@const recorder = agentNamed(revision.recordedBy)}
          <article class="chatrow mrow">
            <span class="pills">
              <span class="pill {kindPill(revision.details.kind)}"><Icon size={12} aria-hidden="true" />{label(revision.details.kind).toLowerCase()}</span>
              <span class="pill {statePill(revision.details.state)}">{label(revision.details.state).toLowerCase()}</span>
              {#if settled !== "established"}<span class="pill warn">{settled}</span>{/if}
              {#if !revision.evidenceCurrent}<span class="pill bad">stale evidence</span>{/if}
              {#if record.conflicted}<span class="pill bad">conflicting records</span>{/if}
              {#if timingLabel(revision, now)}<span class="pill warn">{timingLabel(revision, now)?.toLowerCase()}</span>{/if}
            </span>
            <a href={recordHref(record.id, record.node, recordsHref(new URLSearchParams(params)))}>{revision.statement}</a>
            <span class="mts">
              <span class="mt" title={scopeLabel(record.scope)}><ScopeIcon size={12} aria-hidden="true" />{scopeName(record.scope)}</span>
              <span class="mt"><Laptop size={12} aria-hidden="true" />{record.node || catalogue.data?.localNode || "this machine"}</span>
              {#if revision.recordedBy}<span class="mt">{#if recorder}<AgentMark agent={recorder} size="sm" />{/if}{revision.recordedBy}</span>{/if}
              <span class="mtx">{whenLabel(revision.recordedAt, now)}</span>
            </span>
          </article>
        {/if}
      {/each}
    </section>
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
