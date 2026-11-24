<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import Menu from "@lucide/svelte/icons/menu";
import Search from "@lucide/svelte/icons/search";
import X from "@lucide/svelte/icons/x";
import { useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { askAfresh } from "../lib/federatedSearch.svelte";
import { createLiveSearch } from "../lib/liveSearch.svelte";
import { navigate } from "../lib/navigation";
import { searchQueryKey } from "../lib/queries";
import { chatHref, router, searchHref } from "../lib/route.svelte";
import { scope } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { type Applied, appliedHref, dateRanges, parseApplied } from "../lib/search";
import AgentMark from "./AgentMark.svelte";

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
// While the reader types, the box also searches what chats said, in every
// source the viewing scope covers (see lib/federated.ts): this machine's
// words at once, search by meaning once typing pauses, other machines as they
// answer, all in one list. Where that is unavailable (the demo, or a
// dashboard serving only the shared index) the box falls back to matching
// chat titles above.
let typed = $state("");
$effect(() => {
  const value = needle;
  const timer = setTimeout(() => (typed = value), 60);
  return () => clearTimeout(timer);
});
const live = createLiveSearch(() => ({
  typed,
  enabled: focused && typed.length >= 2,
  catalogue: catalogue.data,
  limit: 5,
}));
const said = $derived(needle.length >= 2 && !live.error ? live.hits : []);
const status = $derived(needle.length >= 2 ? live.status : { text: "", retry: false });
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
      // A streamed search is not a query; ask it to run again.
      dispatchEvent(new Event("mash:search-again"));
      return;
    }
    // A typed submission should read current results even if we visited its
    // address before. Ordinary Back and Forward still reuse the saved query.
    void client.resetQueries(target);
    askAfresh(submitted);
  }
  navigate(submitted ? `#/search?${submitted}` : "#/search");
}
</script>

<header id="top">
  <button id="burger" type="button" aria-label="Open navigation" aria-expanded={railOpen} onclick={toggleRail}><span class="menu-ico"><Menu size={20} aria-hidden="true" /></span><img src={asset("mash-mark.svg")} alt="" width="40" height="40" /></button>
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
          <!-- One list from every source; moving to a result keeps it where it is. -->
          <div class="said-list" role="presentation" onmouseenter={() => live.interact()} onfocusin={() => live.interact()}>
            {#each said as hit (hit.chat.machine + hit.chat.id)}
              <a class="opt said" href={chatHref(hit.chat.id, hit.chat.machine, hit.messageId)} onclick={() => (focused = false)}><AgentMark agent={hit.chat.agent} size="sm" /><span class="t"><span>{hit.chat.title}</span>{#if hit.before}<small>{hit.before}</small>{/if}</span>{#if hit.byMeaning}<small class="by-meaning">by meaning</small>{/if}<small>{hit.chat.machine}</small></a>
            {/each}
          </div>
          {#if status.text}<p class="meta drop-status" role="status">{status.text}{#if status.retry}{" · "}<button class="a" type="button" onclick={() => live.retry()}>Retry</button>{/if}</p>{/if}
        {:else if suggestions.chats.length}
          <h4>chats</h4>
          {#each suggestions.chats as chat (chat.machine + chat.id)}
            <a class="opt" href={chatHref(chat.id, chat.machine)} onclick={() => (focused = false)}><AgentMark agent={chat.agent} size="sm" /><span class="t"><span>{chat.title}</span></span></a>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</header>

<style>
  .drop-status {
    padding: 6px 12px 2px;
  }
  .by-meaning {
    flex: none;
  }
</style>
