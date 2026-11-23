<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import CircleHelp from "@lucide/svelte/icons/circle-help";
import Clock from "@lucide/svelte/icons/clock";
import ClockArrowDown from "@lucide/svelte/icons/clock-arrow-down";
import ClockArrowUp from "@lucide/svelte/icons/clock-arrow-up";
import Folder from "@lucide/svelte/icons/folder";
import GitFork from "@lucide/svelte/icons/git-fork";
import Laptop from "@lucide/svelte/icons/laptop";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import SearchIcon from "@lucide/svelte/icons/search";
import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
import Sparkles from "@lucide/svelte/icons/sparkles";
import Terminal from "@lucide/svelte/icons/terminal";
import { createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import { agents as agentInfo } from "../lib/agents";
import { browse, isAgent, search } from "../lib/api";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { statusLine } from "../lib/federated";
import { FederatedSearch, takeAfresh } from "../lib/federatedSearch.svelte";
import { navigate } from "../lib/navigation";
import { searchQueryKey, stableRead } from "../lib/queries";
import { decisionOptions, decisionsByChat } from "../lib/records";
import { chatHref } from "../lib/route.svelte";
import { inScope, scope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import {
  type Applied,
  appliedHref,
  dateRanges,
  filtersOf,
  parseApplied,
  parseIntent,
  rankings,
  type Sort,
  sameSearch,
  searchIntents,
  sorts,
  subagentHitViews,
  subagentViews,
} from "../lib/search";
import { withoutCopies } from "../lib/sshCopies";
import { foldedNote, parentOf } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatRow from "./ChatRow.svelte";
import FilterMenu from "./FilterMenu.svelte";
import Highlight from "./Highlight.svelte";
import MetaChips from "./MetaChips.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import SubagentPill from "./SubagentPill.svelte";

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
// How many filters narrow the results, for the button that folds them away
// on a phone. The sort order is not a filter.
const activeFilters = $derived(
  [
    applied.agents.length > 0,
    applied.machines.length > 0,
    applied.projects.length > 0,
    applied.days !== null,
    !!applied.query && applied.intent !== "default",
    !!applied.query && applied.subagentHits === "included",
  ].filter(Boolean).length,
);
let filtersOpen = $state(false);
const clearFiltersHref = $derived(
  appliedHref({ ...applied, agents: [], machines: [], projects: [], days: null, offset: 0 }),
);
const browseChatsHref = $derived(
  appliedHref({ ...applied, query: "", rank: "relevance", offset: 0 }),
);
let draft = $state<Applied>(untrack(() => parseApplied(params)));
const dirty = $derived(!sameSearch(draft, applied));
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
/** The one agent chosen, whose mark stands in for the chip's icon. */
const onlyAgent = $derived(draft.agents.length === 1 ? draft.agents[0] : undefined);
const draftIntentLabel = $derived(
  searchIntents.find((intent) => intent.value === draft.intent)?.label,
);
const resolvedDefaultLabel = $derived(
  draft.intent === "default" && applied.intent === "default"
    ? searchIntents.find((intent) => intent.value === view?.intent)?.label
    : undefined,
);

function toggle(field: "agents" | "machines" | "projects", value: string): void {
  const current: readonly string[] = draft[field];
  const next = current.includes(value)
    ? current.filter((item) => item !== value)
    : [...current, value];
  draft =
    field === "agents" ? { ...draft, agents: next.filter(isAgent) } : { ...draft, [field]: next };
}
function chipLabel(field: "agents" | "machines" | "projects", noun: string): string {
  const chosen = draft[field];
  if (chosen.length === 0) return `any ${noun}`;
  if (chosen.length > 1) return `${chosen.length} ${noun}s`;
  const [only] = chosen;
  if (only === undefined) return `any ${noun}`;
  if (field === "agents") return isAgent(only) ? agentInfo[only].name : only;
  return field === "projects" ? projectLabel(only) : only;
}
const changed = (field: "agents" | "machines" | "projects") =>
  [...draft[field]].sort().join() !== [...applied[field]].sort().join();
function chooseSort(value: string): void {
  const sort: Sort | undefined = sorts.find((option) => option.value === value)?.value;
  if (sort) draft = { ...draft, sort };
}
function apply(): void {
  navigate(appliedHref({ ...draft, query: applied.query, offset: 0 }));
}
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
const parentTitle = (chat: ChatSummary): string | undefined =>
  chat.subagent
    ? catalogue.data
      ? parentOf(catalogue.data.all, chat)?.title
      : undefined
    : undefined;
</script>

{#if applied.query}<h1 class="sr-only">Search results</h1><ScopeNote scoped={catalogue.data} />{:else}<div><h1 class="h1">Chats</h1><p class="meta">Every chat Mash has indexed, from all your agents.</p><ScopeNote scoped={catalogue.data} /></div>{/if}
<div class="hd" style:align-items="flex-start">
<!-- On a phone the filters fold behind one button, so results start near the top. -->
<button class="btn sm fold-t" type="button" aria-expanded={filtersOpen || dirty} aria-controls="search-filters" onclick={() => (filtersOpen = !filtersOpen)}><SlidersHorizontal size={14} aria-hidden="true" />Filters{#if activeFilters}<span class="pill">{activeFilters}</span>{/if}</button>
<div class="chips" class:folded={!filtersOpen && !dirty} id="search-filters" role="group" aria-label="Filters" style:flex="1 1 480px">
  <FilterMenu label={chipLabel("agents", "agent")} active={draft.agents.length > 0} draft={changed("agents")} multiple options={options.agent} selected={draft.agents} choose={(value) => toggle("agents", value)}>
    {#snippet icon()}{#if onlyAgent}<AgentMark agent={onlyAgent} size="sm" />{:else}<Bot size={15} aria-hidden="true" />{/if}{/snippet}
    {#snippet optionIcon(value: string)}{#if isAgent(value)}<AgentMark agent={value} size="sm" />{:else}<Bot size={15} aria-hidden="true" />{/if}{/snippet}
  </FilterMenu>
  <!-- Which machines is the viewing scope's to choose; a link may still name one. -->
  {#if applied.machines.length || draft.machines.length}<FilterMenu label={chipLabel("machines", "machine")} active={draft.machines.length > 0} draft={changed("machines")} multiple options={options.machine} selected={draft.machines} choose={(value) => toggle("machines", value)}>
    {#snippet icon()}<Laptop size={15} aria-hidden="true" />{/snippet}
  </FilterMenu>{/if}
  <FilterMenu label={chipLabel("projects", "project")} active={draft.projects.length > 0} draft={changed("projects")} multiple options={options.project} selected={draft.projects} choose={(value) => toggle("projects", value)}>
    {#snippet icon()}<Folder size={15} aria-hidden="true" />{/snippet}
  </FilterMenu>
  <FilterMenu
    label={dateRanges.find((range) => range.days === draft.days)?.label ?? "any time"}
    active={draft.days !== null}
    draft={draft.days !== applied.days}
    options={dateRanges.map((range) => ({ value: String(range.days ?? ""), label: range.label }))}
    selected={[String(draft.days ?? "")]}
    choose={(value) => (draft = { ...draft, days: value ? Number(value) : null })}
  >
    {#snippet icon()}<Clock size={15} aria-hidden="true" />{/snippet}
  </FilterMenu>
  {#if !applied.query}
    <FilterMenu
      label="sort: {sorts.find((sort) => sort.value === draft.sort)?.label}"
      draft={draft.sort !== applied.sort}
      options={sorts}
      selected={[draft.sort]}
      choose={chooseSort}
    >
      {#snippet icon()}<SlidersHorizontal size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if value === "old"}<ClockArrowUp size={15} aria-hidden="true" />{:else}<ClockArrowDown size={15} aria-hidden="true" />{/if}{/snippet}
    </FilterMenu>
  {:else}
    <FilterMenu
      label={resolvedDefaultLabel ? `search: default (${resolvedDefaultLabel})` : `search: ${draftIntentLabel}`}
      active={draft.intent !== "default"}
      draft={draft.intent !== applied.intent}
      options={searchIntents}
      selected={[draft.intent]}
      choose={(value) => (draft = { ...draft, intent: parseIntent(value) })}
    >
      {#snippet icon()}<SearchIcon size={15} aria-hidden="true" />{/snippet}
    </FilterMenu>
    <FilterMenu
      label="sort: {rankings.find((ranking) => ranking.value === draft.rank)?.label}"
      draft={draft.rank !== applied.rank}
      options={rankings}
      selected={[draft.rank]}
      choose={(value) => (draft = { ...draft, rank: value === "recent" ? "recent" : "relevance" })}
    >
      {#snippet icon()}<SlidersHorizontal size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if value === "recent"}<ClockArrowDown size={15} aria-hidden="true" />{:else}<Sparkles size={15} aria-hidden="true" />{/if}{/snippet}
    </FilterMenu>
  {/if}
  {#if hasSubagents && draft.query}
    <!-- A query leaves subagent chats out unless asked to include them. -->
    <FilterMenu
      label="subagents: {draft.subagentHits}"
      active={draft.subagentHits === "included"}
      draft={draft.subagentHits !== applied.subagentHits}
      options={subagentHitViews}
      selected={[draft.subagentHits]}
      choose={(value) => (draft = { ...draft, subagentHits: value === "included" ? "included" : "hidden" })}
    >
      {#snippet icon()}<GitFork size={15} aria-hidden="true" />{/snippet}
    </FilterMenu>
  {:else if hasSubagents}
    <!-- Subagents are chats an agent started for part of a task; the list
         keeps them with the chat that started them unless asked. -->
    <FilterMenu
      label="subagents: {draft.subagents}"
      active={draft.subagents === "shown"}
      draft={draft.subagents !== applied.subagents}
      options={subagentViews}
      selected={[draft.subagents]}
      choose={(value) => (draft = { ...draft, subagents: value === "shown" ? "shown" : "folded" })}
    >
      {#snippet icon()}<GitFork size={15} aria-hidden="true" />{/snippet}
    </FilterMenu>
  {/if}
  {#if dirty}<button class="a" id="applyf" type="button" onclick={apply} style:min-height="30px" style:padding="0 4px" style:margin-left="4px" style:font-size="13px">Apply filters</button>{/if}
</div>
{#if !applied.query && view && hits.length}
  <p class="meta" role="status" style:margin-top="8px">{rangeLabel}{foldedLabel}</p>
{/if}
</div>

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
      {/if}
      <div class="acts" style:justify-content="center">
        {#if view.first > 0}
          <a class="btn p" href={page(0)}>First page</a>
        {/if}
        {#if filtered}
          <a class="btn p" href={clearFiltersHref}>Clear filters</a>
        {:else if view.first === 0 && !applied.query}
          <a class="btn p" href="https://getmash.io/troubleshooting/#nothing-is-indexed-yet" target="_blank" rel="noreferrer">Check history setup</a>
        {/if}
        {#if applied.query && view.first === 0}
          <!-- Two ways on when a search finds nothing: look through the chats
               themselves, or learn why one might not be indexed. -->
          <a class="btn" href={browseChatsHref}><MessagesSquare size={15} aria-hidden="true" />Browse chats</a>
          <a class="btn" href="https://getmash.io/find-missing-chat/" target="_blank" rel="noreferrer"><CircleHelp size={15} aria-hidden="true" />Find a missing chat</a>
        {/if}
      </div>
    </div>
  {:else}
    <section id="reslist">
      {#if applied.query}
        <!-- Each match is a card of its own: the title and the passage that
             matched, with the searched words marked. -->
        <p class="meta" role="status" style:margin-bottom="14px">{streamed && status.text ? status.text : rangeLabel}{#if streamed && status.retry}{" · "}<button class="a" type="button" onclick={() => federated.retry()}>Retry</button>{/if}</p>
        <!-- Moving to a result keeps the list where it is while more arrive. -->
        <div class="results" role="presentation" onmouseenter={() => federated.interact()} onfocusin={() => federated.interact()}>
          {#each hits as hit (JSON.stringify([hit.chat.machine, hit.chat.id, hit.messageId]))}
            <a class="card pad0" href={chatHref(hit.chat.id, hit.chat.machine, hit.messageId)}>
              <span class="chatrow">
                <span class="chatrow-t"><AgentMark agent={hit.chat.agent} size="sm" /><b><Highlight text={hit.chat.title} query={applied.query} /></b>{#if "byMeaning" in hit && hit.byMeaning}<span class="pill">by meaning</span>{/if}<SubagentPill chat={hit.chat} parentTitle={parentTitle(hit.chat)} />{#if decided.get(hit.chat.id)}{@const count = decided.get(hit.chat.id) ?? 0}<span class="pill decision">{count} decision{count === 1 ? "" : "s"}</span>{/if}</span>
                {#if hit.before}<span class="snip res-s"><Highlight text={hit.before} query={applied.query} /></span>{/if}
                <MetaChips chat={hit.chat} />
              </span>
            </a>
          {/each}
        </div>
      {:else}
        <div class="card pad0 list">
          {#each hits as hit (JSON.stringify([hit.chat.machine, hit.chat.id, hit.messageId]))}
            <ChatRow chat={hit.chat} decisions={decided.get(hit.chat.id) ?? 0} to={chatHref(hit.chat.id, hit.chat.machine, hit.messageId)} />
          {/each}
        </div>
      {/if}
      {#if streamed && federated.done && hits.length && federated.hits.length % pageSize === 0}
        <!-- More continues the same merged ranking; results already shown stay put. -->
        <div class="acts" style:margin-top="14px"><button class="btn sm" type="button" onclick={() => federated.more(pageSize)}>Show more results</button></div>
      {:else if view.first > 0 || view.nextOffset !== null}
        <nav class="hd" aria-label="Pages" style:margin-top="14px">
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
