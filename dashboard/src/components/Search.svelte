<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
import CircleHelp from "@lucide/svelte/icons/circle-help";
import FunnelX from "@lucide/svelte/icons/funnel-x";
import GitFork from "@lucide/svelte/icons/git-fork";
import ListChecks from "@lucide/svelte/icons/list-checks";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import Terminal from "@lucide/svelte/icons/terminal";
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { agents as agentInfo } from "../lib/agents";
import { browse, isAgent, search } from "../lib/api";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { statusLine } from "../lib/federated";
import { FederatedSearch, takeAfresh } from "../lib/federatedSearch.svelte";
import { searchQueryKey, stableRead } from "../lib/queries";
import { decisionOptions, decisionsByChat } from "../lib/records";
import { chatHref, router } from "../lib/route.svelte";
import { inScope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { type Applied, appliedHref, filtersOf, parseApplied } from "../lib/search";
import { withoutCopies } from "../lib/sshCopies";
import { foldedNote, parentOf } from "../lib/subagents";
import { followTopBar } from "../lib/topBar";
import type { ChatSummary } from "../lib/types";
import MemoryMatches from "./MemoryMatches.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import SearchFilters from "./SearchFilters.svelte";
import SearchPreview from "./SearchPreview.svelte";
import SearchRow from "./SearchRow.svelte";

// Chats and search are one page: without a query it lists chats, newest
// first. The address is the applied search; the chips below are a draft
// until "Apply filters" is chosen, so results never change under the reader.
let { params }: { params: string } = $props();
const pageSize = 10;
const applied = $derived(parseApplied(params));
const filtered = $derived(
  applied.agents.length > 0 ||
    applied.machines.length > 0 ||
    applied.projects.length > 0 ||
    applied.days !== null,
);
const clearFiltersHref = $derived(
  appliedHref({ ...applied, agents: [], machines: [], projects: [], days: null, offset: 0 }),
);
const browseChatsHref = $derived(
  appliedHref({ ...applied, query: "", rank: "relevance", offset: 0 }),
);
let draft = $state<Applied>(untrack(() => parseApplied(params)));
// The list and the search follow the viewing scope chosen in the sidebar.
const catalogue = createScoped();
const scopeKey = $derived(catalogue.data ? scopeText(catalogue.data.scope) : "");
const chats = $derived(catalogue.data?.chats ?? []);

const options = $derived({
  agent: groupChats(chats, (chat) => chat.agent).map((group) => ({
    value: group.key,
    label: isAgent(group.key) ? agentInfo[group.key].name : group.key,
    count: group.chats.length,
  })),
  machine: groupChats(chats, (chat) => chat.machine).map((group) => ({
    value: group.key,
    label: group.key,
    count: group.chats.length,
  })),
  project: groupChats(
    chats.filter((chat) => chat.project),
    (chat) => chat.project,
  ).map((group) => ({
    value: group.key,
    label: projectLabel(group.key),
    count: group.chats.length,
  })),
});

// A search of the default kind, by relevance and without a machine named,
// asks every source in the viewing scope at once and shows one list as they
// answer (lib/federated.ts). Other searches run as one request, as before.
const streamed = $derived(
  !!applied.query &&
    applied.offset === 0 &&
    applied.intent === "default" &&
    applied.rank === "relevance" &&
    !applied.machines.length &&
    !!catalogue.data &&
    !(catalogue.data.live && catalogue.data.scope.kind === "node"),
);
const federated = new FederatedSearch();
// Pressing Enter on an unchanged search asks again (see TopBar).
let again = $state(0);
let lastAgain = 0;
$effect(() => {
  const asked = () => again++;
  addEventListener("mash:search-again", asked);
  return () => removeEventListener("mash:search-again", asked);
});
$effect(() => {
  void params;
  void scopeKey;
  // Asked again, it starts afresh rather than keeping the rows it showed.
  const fresh = again !== untrack(() => lastAgain) || takeAfresh(params);
  if (fresh) {
    lastAgain = again;
    untrack(() => federated.stop());
  }
  const data = untrack(() => catalogue.data);
  if (!untrack(() => streamed) || !data) return void untrack(() => federated.stop());
  const question = {
    query: applied.query,
    scope: data.scope,
    live: false,
    limit: pageSize,
    agents: applied.agents,
    projects: applied.projects,
    days: applied.days,
    includeSubagents: applied.subagentHits === "included",
  };
  // Back to a search already answered shows it again without asking.
  untrack(() => {
    if (fresh || !federated.reuse(question, data.reach.localNode))
      federated.start(question, data.reach.localNode);
  });
});
const streamedHits = $derived.by(() => {
  const data = catalogue.data;
  const inside = data
    ? federated.hits.filter((hit) => inScope(hit.chat, data.scope, data.reach.localNode))
    : federated.hits;
  // Mash leaves subagent chats out of the search itself, unless asked,
  // other machines' ones too: each machine publishes its chats' markers.
  return withoutCopies(inside, data?.all ?? []);
});
const status = $derived(statusLine(federated.sources, federated.hits.length, federated.done));
const result = createQuery(() => ({
  ...stableRead,
  // A list of chats without a question follows the chat list as it updates;
  // a submitted search keeps its results until it is submitted again.
  queryKey: applied.query
    ? [...searchQueryKey(params), scopeKey]
    : ["chat-list", params, catalogue.dataUpdatedAt, scopeKey],
  placeholderData: (previous, query) => (query?.queryKey[1] === params ? previous : undefined),
  enabled: catalogue.data !== undefined && !streamed,
  queryFn: async ({ signal }: { signal: AbortSignal }) => {
    const data = catalogue.data;
    if (!data) throw new Error("The chat list is not loaded yet.");
    if (applied.query) {
      // A machine read live is searched on its own dashboard; otherwise the
      // scope narrows the search to the machines it covers.
      const peer = data.live && data.scope.kind === "node" ? data.scope.node : undefined;
      // This machine's own scope narrows only when other machines are there.
      const others = data.reach.joined || data.reach.sharedOnly;
      const covered =
        data.scope.kind === "local" && others
          ? [data.localNode]
          : data.scope.kind === "node"
            ? [data.scope.node]
            : [];
      const machines = applied.machines.length ? applied.machines : covered;
      const page = await search({ ...applied, machines }, data.localNode, signal, peer);
      // A session copied over SSH is shown once, from the machine it ran on.
      // Mash itself leaves subagent chats out, unless asked, before it cuts
      // the page, so a page stays full.
      return {
        ...page,
        hits: withoutCopies(page.hits, data.all),
        total: null,
        first: applied.offset,
        folded: 0,
      };
    }
    const all = browse(data, filtersOf(applied), Date.now(), applied.subagents);
    const ordered = applied.sort === "old" ? [...all.hits].reverse() : all.hits;
    // A link to a page past the end, say after chats were forgotten, shows
    // the last page instead of claiming nothing matched.
    const lastPage = Math.max(0, Math.floor((ordered.length - 1) / pageSize) * pageSize);
    const first = Math.min(applied.offset, lastPage);
    return {
      ...all,
      hits: ordered.slice(first, first + pageSize),
      nextOffset: first + pageSize < ordered.length ? first + pageSize : null,
      total: ordered.length,
      first,
    };
  },
}));
// What the page shows, from whichever path ran.
const view = $derived(
  streamed
    ? federated.sources.length || federated.error
      ? {
          hits: streamedHits,
          warnings: federated.error ? [federated.error] : [],
          first: 0,
          nextOffset: null,
          total: null,
          intent: undefined,
          folded: 0,
        }
      : undefined
    : result.data,
);
const hits = $derived(view?.hits ?? []);
// Chats that produced decisions say how many; the memory page reads the same list.
const decisions = createQuery(() => decisionOptions);
const decided = $derived(decisionsByChat(decisions.data?.records ?? []));
function page(offset: number): string {
  return appliedHref({ ...applied, offset });
}
const rangeLabel = $derived.by(() => {
  if (!view) return "";
  const first = view.first + 1;
  const last = view.first + hits.length;
  if (view.total !== null)
    return view.total > pageSize
      ? `${first.toLocaleString()}–${last.toLocaleString()} of ${view.total.toLocaleString()} chats`
      : `${view.total.toLocaleString()} chat${view.total === 1 ? "" : "s"}`;
  return `results ${first.toLocaleString()}–${last.toLocaleString()} for “${applied.query}”`;
});
// "19 chats · 11 subagent chats folded in": the list counts its own rows.
const foldedLabel = $derived(foldedNote(view?.folded ?? 0));
const hasSubagents = $derived((catalogue.data?.all ?? []).some((chat) => chat.subagent));
// A search that found nothing may have matched only chats agents started as
// subagents, which are left out unless asked for; the empty page says so.
const leftOutSubagents = $derived(
  hasSubagents && !!applied.query && applied.subagentHits === "hidden",
);
const withSubagentsHref = $derived(
  appliedHref({ ...applied, subagentHits: "included", offset: 0 }),
);
const parentTitle = (chat: ChatSummary): string | undefined =>
  chat.subagent
    ? catalogue.data
      ? parentOf(catalogue.data.all, chat)?.title
      : undefined
    : undefined;
// The row the preview beside the list shows: the first, until the reader
// moves to another with the pointer or the keyboard.
let selected = $state(0);
$effect(() => {
  void params;
  selected = 0;
});
const current = $derived(hits[Math.min(selected, hits.length - 1)]);
// The preview needs width beside the list; a narrower screen opens the chat.
const wideQuery =
  typeof window === "undefined" ? undefined : window.matchMedia("(min-width: 1100px)");
let wide = $state(wideQuery?.matches ?? false);
$effect(() => {
  if (!wideQuery) return;
  const update = (): void => {
    wide = wideQuery.matches;
  };
  wideQuery.addEventListener("change", update);
  return () => wideQuery.removeEventListener("change", update);
});
const split = $derived(!!applied.query && wide && hits.length > 0);
// The page is a full-width pair of panes only while it is the page shown:
// it stays mounted, hidden, while a chat is open.
const shown = $derived(router.current.page === "search");
// The preview stays in view under the sticky top bar, whatever its height.
$effect(followTopBar);
/** Up and down move between rows, as they would in a list box. */
function step(event: KeyboardEvent): void {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  const rows = [
    ...(event.currentTarget as HTMLElement).querySelectorAll<HTMLElement>("[data-row]"),
  ];
  const at = rows.indexOf(document.activeElement as HTMLElement);
  if (at < 0) return;
  const next = rows[at + (event.key === "ArrowDown" ? 1 : -1)];
  if (!next) return;
  event.preventDefault();
  next.focus();
}
</script>

<div class="sx" class:shown class:split>
<section class="sx-l" aria-label={applied.query ? "Results" : undefined}>
  <header class="sx-h">
    {#if applied.query}<h1 class="sr-only">Search results</h1>{:else}<div><h1 class="h1">Chats</h1><p class="meta">Every chat Mash has indexed, from all your agents.</p></div>{/if}
    <ScopeNote scoped={catalogue.data} />
    <SearchFilters {applied} bind:draft {options} {hasSubagents} resolvedIntent={view?.intent} />
    {#if view && hits.length}
      <div class="sx-count">
        {#if applied.query}
          <p class="meta" role="status">{streamed && status.text ? status.text : rangeLabel}{#if streamed && status.retry}{" · "}<button class="a" type="button" onclick={() => federated.retry()}>Retry</button>{/if}</p>
          <span class="meta">{applied.rank === "recent" ? "Newest first" : "Sorted by best match"}</span>
        {:else}
          <p class="meta" role="status">{rangeLabel}{foldedLabel}</p>
          <span class="meta">{applied.sort === "old" ? "Oldest first" : "Newest first"}</span>
        {/if}
      </div>
    {/if}
  </header>

  <div class="sx-b">
  <QueryError title="Couldn’t load chats" query={catalogue} />
  {#if !streamed}<QueryError title="Couldn’t load results" query={result} />{/if}
  {#if (streamed ? !view && !federated.done : result.isPending || catalogue.isPending) && !result.error && !catalogue.error}
    <div class="card" role="status">
      <p style:display="flex" style:align-items="center" style:gap="10px" style:font="600 13px var(--type-mono)"><Terminal size={16} aria-hidden="true" />{applied.query ? "Searching…" : "Loading chats…"}</p>
      <div class="skel" style:width="62%"></div><div class="skel" style:width="86%"></div><div class="skel" style:width="54%"></div>
    </div>
  {/if}
  {#if view}
    {#each view.warnings as warning (warning)}<p class="meta" role="status">{warning}</p>{/each}
    {#if !hits.length}
      <div class="card empty">
        <img src={asset("mashy.png")} alt="" />
        <h2>
          {#if view.first > 0}No results on this page
          {:else if applied.query || filtered}No results
          {:else}No chats to show{/if}
        </h2>
        {#if applied.query && view.first === 0}
          <p>
            {#if filtered}Nothing matched “{applied.query}” with the selected filters.
            {:else if view.intent === "related-discussion"}Try a short question in English about the chat.
            {:else}Try one distinctive word you remember from the chat.{/if}
          </p>
          {#if leftOutSubagents}<p class="meta">Chats your agents started as subagents are left out of search.</p>{/if}
        {/if}
        <div class="acts" style:justify-content="center">
          {#if view.first > 0}
            <a class="btn p" href={page(0)}><ChevronsLeft size={14} aria-hidden="true" />First page</a>
          {/if}
          {#if filtered}
            <a class="btn p" href={clearFiltersHref}><FunnelX size={14} aria-hidden="true" />Clear filters</a>
          {:else if view.first === 0 && !applied.query}
            <a class="btn p" href="https://getmash.io/troubleshooting/#nothing-is-indexed-yet" target="_blank" rel="noreferrer"><ListChecks size={14} aria-hidden="true" />Check history setup</a>
          {/if}
          {#if applied.query && view.first === 0}
            <!-- Two ways on when a search finds nothing: look through the chats
                 themselves, or learn why one might not be indexed. -->
            {#if leftOutSubagents}<a class="btn" href={withSubagentsHref}><GitFork size={15} aria-hidden="true" />Include subagent chats</a>{/if}
            <a class="btn" href={browseChatsHref}><MessagesSquare size={15} aria-hidden="true" />Browse chats</a>
            <a class="btn" href="https://getmash.io/find-missing-chat/" target="_blank" rel="noreferrer"><CircleHelp size={15} aria-hidden="true" />Find a missing chat</a>
          {/if}
        </div>
      </div>
    {:else}
      {#if applied.query && view.first === 0 && catalogue.data}<MemoryMatches query={applied.query} scoped={catalogue.data} />{/if}
      <section id="reslist" aria-label={applied.query ? "Chats" : undefined}>
        {#if applied.query}<h2 class="sx-label">Chats</h2>{/if}
        <!-- Moving to a result keeps the list where it is while more arrive. -->
        <div class="sx-rows" class:results={!!applied.query} role="presentation" onmouseenter={() => federated.interact()} onfocusin={() => federated.interact()} onkeydown={step}>
          {#each hits as hit, index (JSON.stringify([hit.chat.machine, hit.chat.id, hit.messageId]))}
            <SearchRow
              chat={hit.chat}
              href={chatHref(hit.chat.id, hit.chat.machine, hit.messageId)}
              query={applied.query}
              snippet={applied.query ? hit.before : ""}
              decisions={decided.get(hit.chat.id) ?? 0}
              parentTitle={parentTitle(hit.chat)}
              byMeaning={"byMeaning" in hit && !!hit.byMeaning}
              selected={split && index === selected}
              onselect={() => (selected = index)}
            />
          {/each}
        </div>
        {#if streamed && federated.done && hits.length && federated.hits.length % pageSize === 0}
          <!-- More continues the same merged ranking; results already shown stay put. -->
          <div class="acts" style:margin-top="14px"><button class="btn sm" type="button" onclick={() => federated.more(pageSize)}><ChevronDown size={14} aria-hidden="true" />Show more results</button></div>
        {:else if view.first > 0 || view.nextOffset !== null}
          <nav class="hd sx-pages" aria-label="Pages">
            <span class="meta">{rangeLabel}</span>
            <span class="acts">
              {#if view.first > 0}<a class="btn sm" rel="prev" href={page(Math.max(0, view.first - pageSize))}><ChevronLeft size={14} aria-hidden="true" />Previous</a>{:else}<button class="btn sm" type="button" disabled><ChevronLeft size={14} aria-hidden="true" />Previous</button>{/if}
              {#if view.nextOffset !== null}<a class="btn sm" rel="next" href={page(view.nextOffset)}>Next<ChevronRight size={14} aria-hidden="true" /></a>{:else}<button class="btn sm" type="button" disabled>Next<ChevronRight size={14} aria-hidden="true" /></button>{/if}
            </span>
          </nav>
        {/if}
      </section>
    {/if}
  {/if}
  </div>
</section>
{#if split && shown && current}
  <div class="sx-p">
    {#key JSON.stringify([current.chat.machine, current.chat.id, current.messageId])}
      <SearchPreview hit={current} query={applied.query} href={chatHref(current.chat.id, current.chat.machine, current.messageId)} />
    {/key}
  </div>
{/if}
</div>

<style>
  /* While it is the page shown, search takes the whole column beside the
     rail: the list on the left and, when there is room, the preview on the
     right, split by a hairline, as the wireframes draw them. */
  :global(#app #main:has(> * > .sx.shown)) {
    max-width: none;
    padding-block: 0;
    padding-left: 0;
    gap: 0;
  }
  /* Beside Mashy's corner the panes keep its column clear, as other pages do. */
  @media (min-width: 801px) {
    :global(#app #main:has(> * > .sx.shown)) {
      padding-right: calc(env(safe-area-inset-right, 0px) + 96px);
    }
  }
  :global(body:not(:has(#mashy-fab)) #main:has(> * > .sx.shown)) {
    padding-right: 0;
  }
  .sx {
    display: flex;
    align-items: stretch;
    min-height: 100%;
    flex: 1;
  }
  .sx-l {
    flex: 2 1 440px;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .split .sx-l {
    border-right: var(--border-divider);
  }
  .sx-h {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px 20px 12px;
    border-bottom: var(--border-divider);
  }
  .sx-count {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .sx-count .meta {
    font-size: 12px;
    color: var(--text-faint);
  }
  .sx-b {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 12px 56px;
  }
  .sx-b > :global(.card) {
    margin: 8px;
  }
  .sx-rows {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  :global(.sx-label) {
    padding: 4px 8px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  :global(.sx-label a) {
    color: inherit;
  }
  .sx-pages {
    margin: 14px 8px 0;
  }
  .sx-p {
    flex: 1 1 340px;
    min-width: 0;
  }
  .sx-p > :global(.pv) {
    position: sticky;
    top: var(--top-h, 52px);
    min-height: 0;
    height: calc(100dvh - var(--top-h, 52px));
    overflow-y: auto;
  }
  @media (max-width: 800px) {
    :global(#app #main:has(> * > .sx.shown)) {
      padding-right: 0;
    }
    .sx-h {
      padding: 14px 16px 12px;
    }
    .sx-b {
      padding: 8px 8px 96px;
    }
  }
</style>
