<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import Menu from "@lucide/svelte/icons/menu";
import Search from "@lucide/svelte/icons/search";
import X from "@lucide/svelte/icons/x";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { liveSearch } from "../lib/api";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { navigate } from "../lib/navigation";
import { searchQueryKey } from "../lib/queries";
import { chatHref, router, searchHref } from "../lib/route.svelte";
import { inScope, scope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { type Applied, appliedHref, dateRanges, parseApplied } from "../lib/search";
import AgentMark from "./AgentMark.svelte";
import ViewingSwitcher from "./ViewingSwitcher.svelte";

let { railOpen, toggleRail }: { railOpen: boolean; toggleRail: () => void } = $props();
const client = useQueryClient();
// Suggestions follow the viewing scope, as the results page does.
const catalogue = createScoped();
let text = $state("");
let focused = $state(false);
let root: HTMLDivElement;
// The box shows the submitted search; typing is a draft until it is submitted.
$effect(() => {
  const route = router.current;
  text = route.page === "search" ? (new URLSearchParams(route.params).get("q") ?? "") : "";
});
// Applied filters show as tags in the box; removing one reruns the search.
const applied = $derived(
  router.current.page === "search" ? parseApplied(router.current.params) : null,
);
const tags = $derived.by(() => {
  if (!applied) return [];
  const without = (change: Partial<Applied>) => appliedHref({ ...applied, ...change, offset: 0 });
  return [
    ...applied.agents.map((agent) => ({
      label: agentInfo[agent].name,
      href: without({ agents: applied.agents.filter((item) => item !== agent) }),
    })),
    ...applied.machines.map((machine) => ({
      label: machine,
      href: without({ machines: applied.machines.filter((item) => item !== machine) }),
    })),
    ...applied.projects.map((project) => ({
      label: projectLabel(project),
      href: without({ projects: applied.projects.filter((item) => item !== project) }),
    })),
    ...(applied.days === null
      ? []
      : [
          {
            label: dateRanges.find((range) => range.days === applied.days)?.label ?? "date",
            href: without({ days: null }),
          },
        ]),
  ];
});
const needle = $derived(text.trim().toLowerCase());
const suggestions = $derived.by(() => {
  const chats = catalogue.data?.chats ?? [];
  if (needle.length < 2) return null;
  const has = (value: string) => value.toLowerCase().includes(needle);
  const filters = [
    ...groupChats(chats, (chat) => chat.agent)
      .filter((group) => has(agentInfo[group.chats[0]?.agent ?? "codex"].name))
      .map((group) => ({ kind: "agent", value: group.key, label: group.key })),
    ...groupChats(chats, (chat) => chat.machine)
      .filter((group) => has(group.key))
      .map((group) => ({ kind: "machine", value: group.key, label: group.key })),
    ...groupChats(
      chats.filter((chat) => chat.project),
      (chat) => chat.project,
    )
      .filter((group) => has(projectLabel(group.key)))
      .map((group) => ({ kind: "project", value: group.key, label: projectLabel(group.key) })),
  ].slice(0, 5);
  return { filters, chats: chats.filter((chat) => has(chat.title)).slice(0, 4) };
});
// While the reader types, the box also searches what chats said: this
// machine only, by words, once typing pauses. Where that is unavailable (the
// demo, or a dashboard serving only the shared index) the box falls back to
// matching chat titles above.
let typed = $state("");
$effect(() => {
  const value = needle;
  const timer = setTimeout(() => (typed = value), 60);
  return () => clearTimeout(timer);
});
// A machine read live is asked itself; otherwise this machine's words search
// stands in, kept to what the scope covers.
const live = createQuery(() => ({
  queryKey: ["live-search", typed, catalogue.data ? scopeText(catalogue.data.scope) : ""],
  queryFn: async ({ signal }: { signal: AbortSignal }) => {
    const data = catalogue.data;
    if (!data) return [];
    const peer = data.live && data.scope.kind === "node" ? data.scope.node : undefined;
    const hits = await liveSearch(typed, data.reach.localNode, signal, peer);
    return peer ? hits : hits.filter((hit) => inScope(hit.chat, data.scope, data.reach.localNode));
  },
  enabled: focused && typed.length >= 2 && catalogue.data !== undefined,
  networkMode: "always" as const,
  staleTime: 30_000,
  retry: false,
  // Keep the last answer on screen while the next keystroke's is on its way.
  placeholderData: (previous: Awaited<ReturnType<typeof liveSearch>> | undefined) => previous,
}));
const said = $derived(needle.length >= 2 && !live.isError ? (live.data ?? []) : []);
const showDrop = $derived(
  focused &&
    suggestions !== null &&
    suggestions.filters.length + suggestions.chats.length + said.length > 0,
);
function submit(event: SubmitEvent): void {
  event.preventDefault();
  const route = router.current;
  const params = new URLSearchParams(route.page === "search" ? route.params : "");
  const query = text.trim();
  if (query !== params.get("q")) params.delete("intent");
  if (query) params.set("q", query);
  else params.delete("q");
  // A new query is a new search: it starts at the first result.
  params.delete("offset");
  const submitted = params.toString();
  focused = false;
  if (query) {
    // Every scope's results for this address: the scope is the key's last part.
    const target = { queryKey: searchQueryKey(submitted) };
    if (route.page === "search" && route.params === submitted) {
      void client.refetchQueries(target, { cancelRefetch: false });
      return;
    }
    // A typed submission should read current results even if we visited its
    // address before. Ordinary Back and Forward still reuse the saved query.
    void client.resetQueries(target);
  }
  navigate(submitted ? `#/search?${submitted}` : "#/search");
}
</script>

<header id="top">
  <button id="burger" type="button" aria-label="Open navigation" aria-expanded={railOpen} onclick={toggleRail}><span class="menu-ico"><Menu size={20} aria-hidden="true" /></span><img src={asset("mash-mark.svg")} alt="" width="30" height="28" /></button>
  <div
    class="search"
    role="search"
    bind:this={root}
    onfocusin={() => (focused = true)}
    onfocusout={(event) => {
      if (!(event.relatedTarget instanceof Node) || !root.contains(event.relatedTarget)) focused = false;
    }}
  >
    <form class="box" class:open={showDrop} onsubmit={submit}>
      <Search size={15} aria-hidden="true" />
      {#each tags as tag (tag.label + tag.href)}<a class="tag" href={tag.href} aria-label="Remove filter {tag.label}" style:display="inline-flex" style:align-items="center" style:gap="4px">{tag.label}<X size={13} aria-hidden="true" /></a>{/each}
      <label for="q" class="sr-only">Search your history</label>
      <input id="q" autocomplete="off" placeholder="Search your history" bind:value={text} onkeydown={(event) => { if (event.key === "Escape") focused = false; }} />
      <button class="go" type="submit" aria-label="Search"><Search size={14} /></button>
    </form>
    {#if showDrop && suggestions}
      <div class="drop">
        {#if suggestions.filters.length}
          <h4>filters</h4>
          {#each suggestions.filters as filter (filter.kind + filter.value)}
            <a class="opt" href={searchHref({ [filter.kind]: filter.value })} onclick={() => (focused = false)}>{#if filter.kind === "agent"}<Bot size={14} aria-hidden="true" />{:else if filter.kind === "machine"}<Laptop size={14} aria-hidden="true" />{:else}<Folder size={14} aria-hidden="true" />{/if}<span class="t"><span>{filter.label}</span></span><small>{filter.kind}</small></a>
          {/each}
        {/if}
        {#if said.length}
          <h4>chats that mention it</h4>
          {#each said as hit (hit.chat.machine + hit.chat.id)}
            <a class="opt said" href={chatHref(hit.chat.id, hit.chat.machine, hit.messageId)} onclick={() => (focused = false)}><AgentMark agent={hit.chat.agent} size="sm" /><span class="t"><span>{hit.chat.title}</span>{#if hit.before}<small>{hit.before}</small>{/if}</span></a>
          {/each}
        {:else if suggestions.chats.length}
          <h4>chats</h4>
          {#each suggestions.chats as chat (chat.machine + chat.id)}
            <a class="opt" href={chatHref(chat.id, chat.machine)} onclick={() => (focused = false)}><AgentMark agent={chat.agent} size="sm" /><span class="t"><span>{chat.title}</span></span></a>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
  <!-- On a phone the sidebar is a drawer, so the viewing scope sits here. -->
  <div class="top-scope"><ViewingSwitcher variant="top" /></div>
</header>

<style>
  .top-scope {
    display: none;
  }
  @media (max-width: 800px) {
    .top-scope {
      display: block;
      flex: none;
    }
    /* With the scope beside it, the box's text field may be narrow before
       the search button wraps under it. */
    #top :global(.box input) {
      flex-basis: 40px;
    }
  }
</style>
