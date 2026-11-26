<script lang="ts">
// The search field in the top bar: typing suggests filters and chats, and
// submitting opens the results page. The bar around it (TopBar.svelte) owns
// the ⌘K shortcut and, on a phone, whether the field is shown.

import { useQueryClient } from "@tanstack/svelte-query";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import RobotIcon from "phosphor-svelte/lib/RobotIcon";
import XIcon from "phosphor-svelte/lib/XIcon";
import Icon from "$lib/components/Icon.svelte";
import { agents as agentInfo } from "../lib/agents";
import { groupChats, projectLabel } from "../lib/browse";
import { askAfresh } from "../lib/federatedSearch.svelte";
import { createLiveSearch } from "../lib/liveSearch.svelte";
import { navigate } from "../lib/navigation";
import { createProjectChoices } from "../lib/projectChoices.svelte";
import { searchQueryKey } from "../lib/queries";
import { chatHref, router, searchHref } from "../lib/route.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { type Applied, appliedHref, dateRanges, parseApplied } from "../lib/search";
import AgentMark from "./AgentMark.svelte";
import MachineIcon from "./MachineIcon.svelte";

/** The key the shortcut hint names: ⌘ on a Mac, Ctrl elsewhere. */
let { modifier }: { modifier: string } = $props();

const client = useQueryClient();
// Suggestions follow the viewing scope, as the results page does.
const catalogue = createScoped();
// Projects the reader hid are not suggested.
const choices = createProjectChoices();
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
      chats.filter((chat) => chat.project && !choices.isHidden(chat.project)),
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
    <Icon icon={MagnifyingGlassIcon} size={15} />
    {#each tags as tag (tag.label + tag.href)}<a class="tag" href={tag.href} aria-label="Remove filter {tag.label}" style:display="inline-flex" style:align-items="center" style:gap="4px">{tag.label}<Icon icon={XIcon} size={13} /></a>{/each}
    <label for="q" class="sr-only">Search your history</label>
    <input id="q" autocomplete="off" placeholder="Search every chat" bind:value={text} onkeydown={(event) => { if (event.key === "Escape") focused = false; }} />
    <!-- At rest the field shows its shortcut; once in use, its submit button. -->
    {#if !focused && !text}<span class="keys" aria-hidden="true"><kbd class="kbd">{modifier}</kbd><kbd class="kbd">K</kbd></span>{/if}
    <button class="go" class:rest={!focused && !text} type="submit" aria-label="Search"><Icon icon={MagnifyingGlassIcon} size={14} /></button>
  </form>
  {#if showDrop && suggestions}
    <div class="drop">
      {#if suggestions.filters.length}
        <h4>Filters</h4>
        {#each suggestions.filters as filter (filter.kind + filter.value)}
          <a class="opt" href={searchHref({ [filter.kind]: filter.value })} onclick={() => (focused = false)}>{#if filter.kind === "agent"}<Icon icon={RobotIcon} size={14} />{:else if filter.kind === "machine"}<MachineIcon machine={filter.value} size={14} />{:else}<Icon icon={FolderIcon} size={14} />{/if}<span class="t"><span>{filter.label}</span></span><small>{filter.kind}</small></a>
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
        <h4>Chats</h4>
        {#each suggestions.chats as chat (chat.machine + chat.id)}
          <a class="opt" href={chatHref(chat.id, chat.machine)} onclick={() => (focused = false)}><AgentMark agent={chat.agent} size="sm" /><span class="t"><span>{chat.title}</span></span></a>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  /* While suggestions are open the page behind them dims, as the wireframes
     draw it, and the field stays bright above the dimming. The dimming is
     the list's own shadow, so a click anywhere still reaches the page. */
  .drop {
    z-index: 1;
    box-shadow:
      0 0 0 100vmax var(--scrim),
      var(--shadow-float);
  }
  .box.open {
    position: relative;
    z-index: 2;
  }
  .drop h4 {
    margin: 4px 10px 2px;
    font-size: 12px;
    font-weight: var(--weight-regular);
    color: var(--text-faint);
    text-transform: lowercase;
  }
  .opt + .said-list {
    margin-top: 6px;
    padding-top: 6px;
    border-top: var(--border-divider);
  }
  .drop-status {
    padding: 6px 12px 2px;
  }
  .by-meaning {
    flex: none;
  }
</style>
