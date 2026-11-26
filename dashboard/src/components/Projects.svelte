<script module lang="ts">
import type { ProjectView } from "../lib/browse";

// Kept for the life of the page, so returning from a project keeps the view.
const view: ProjectView = $state({ find: "", agent: "", machine: "", sort: "recent" });
</script>

<script lang="ts">
import { Tip } from "$lib/components/ui/tooltip";
import MachineIcon from "./MachineIcon.svelte";
import ArrowDownAZ from "@lucide/svelte/icons/arrow-down-a-z";
import Bot from "@lucide/svelte/icons/bot";
import ClockArrowDown from "@lucide/svelte/icons/clock-arrow-down";
import Eye from "@lucide/svelte/icons/eye";
import EyeOff from "@lucide/svelte/icons/eye-off";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import Pin from "@lucide/svelte/icons/pin";
import SearchIcon from "@lucide/svelte/icons/search";
import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
import { createQuery } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { isAgent } from "../lib/api";
import { arrangeProjects, groupChats, projectLabel, whenLabel } from "../lib/browse";
import { createScoped } from "../lib/scoped.svelte";
import { scope } from "../lib/scope.svelte";
import { createProjectChoices, pinnedFirst, withoutHidden } from "../lib/projectChoices.svelte";
import { projectHref } from "../lib/route.svelte";
import AgentMark from "./AgentMark.svelte";
import EmptyState from "./EmptyState.svelte";
import FilterMenu from "./FilterMenu.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";

// Projects follow the viewing scope chosen in the sidebar.
const catalogue = createScoped();
const chats = $derived(catalogue.data?.chats ?? []);
const projects = $derived(
  groupChats(
    chats.filter((chat) => chat.project),
    (chat) => chat.project,
  ),
);
const choices = createProjectChoices();
// Hidden projects stay out of the page until the reader asks to see them.
let showHidden = $state(false);
const hiddenHere = $derived(projects.filter((project) => choices.isHidden(project.key)).length);
const shown = $derived(
  pinnedFirst(
    arrangeProjects(showHidden ? projects : withoutHidden(projects, choices.hidden), view),
    choices.pinned,
  ),
);
const largest = $derived(projects[0]?.chats.length ?? 1);
const agentChoices = $derived([...new Set(chats.map((chat) => chat.agent))].sort());
const machineChoices = $derived([...new Set(chats.map((chat) => chat.machine))].sort());
const narrowed = $derived(!!(view.find.trim() || view.agent || view.machine));
const chosenAgent = $derived(agentChoices.find((agent) => agent === view.agent));
// Drawing every tile at once makes the page slow to open, so tiles are added
// as the reader nears the end of those drawn so far.
const STEP = 60;
let limit = $state(STEP);
let more = $state<HTMLElement>();
$effect(() => {
  if (!more) return;
  const watcher = new IntersectionObserver(
    (entries) => {
      if (entries.some((entry) => entry.isIntersecting)) limit += STEP;
    },
    { rootMargin: "600px" },
  );
  watcher.observe(more);
  return () => watcher.disconnect();
});
const sorts = [
  { value: "recent", label: "newest" },
  { value: "chats", label: "most chats" },
  { value: "name", label: "name" },
] as const;
const now = Date.now();
</script>

<div><h1 class="h1">Projects</h1><p class="meta" role="status">{narrowed ? `${shown.length.toLocaleString()} of ${projects.length.toLocaleString()} projects` : `${projects.length.toLocaleString()} projects across ${chats.length.toLocaleString()} chats`}</p><ScopeNote scoped={catalogue.data} /></div>
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if catalogue.isPending}<Skeleton rows={5} />{/if}
{#if projects.length}
  <div class="chips" role="group" aria-label="Project filters">
    <label class="find pfind"><SearchIcon size={15} aria-hidden="true" /><input type="search" aria-label="Find a project" placeholder="find a project" autocomplete="off" bind:value={view.find} /></label>
    <FilterMenu label={chosenAgent ? agentInfo[chosenAgent].name : "any agent"} active={!!view.agent} options={[{ value: "", label: "any agent" }, ...agentChoices.map((agent) => ({ value: agent, label: agentInfo[agent].name }))]} selected={[view.agent]} choose={(value) => (view.agent = value)}>
      {#snippet icon()}<Bot size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if isAgent(value)}<AgentMark agent={value} size="sm" />{:else}<Bot size={15} aria-hidden="true" />{/if}{/snippet}
    </FilterMenu>
    <FilterMenu label={view.machine || "any machine"} active={!!view.machine} options={[{ value: "", label: "any machine" }, ...machineChoices.map((machine) => ({ value: machine, label: machine }))]} selected={[view.machine]} choose={(value) => (view.machine = value)}>
      {#snippet icon()}<Laptop size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if value}<MachineIcon machine={value} size={15} />{:else}<Laptop size={15} aria-hidden="true" />{/if}{/snippet}
    </FilterMenu>
    <FilterMenu label="sort: {sorts.find((sort) => sort.value === view.sort)?.label}" options={sorts} selected={[view.sort]} choose={(value) => { const sort = sorts.find((sort) => sort.value === value); if (sort) view.sort = sort.value; }}>
      {#snippet icon()}<SlidersHorizontal size={15} aria-hidden="true" />{/snippet}
      {#snippet optionIcon(value: string)}{#if value === "chats"}<MessagesSquare size={15} aria-hidden="true" />{:else if value === "name"}<ArrowDownAZ size={15} aria-hidden="true" />{:else}<ClockArrowDown size={15} aria-hidden="true" />{/if}{/snippet}
    </FilterMenu>
    {#if hiddenHere}<button class="a" type="button" style:min-height="30px" style:padding="0 6px" style:font-size="13.5px" aria-pressed={showHidden} onclick={() => (showHidden = !showHidden)}>{showHidden ? "Leave out hidden" : `Show ${hiddenHere} hidden`}</button>{/if}
    {#if narrowed}<button class="a" type="button" style:min-height="30px" style:padding="0 6px" style:font-size="13.5px" onclick={() => { view.find = ""; view.agent = ""; view.machine = ""; }}>Show all</button>{/if}
  </div>
{/if}
{#if projects.length && !shown.length}<EmptyState card>No project matches. Change the name, agent or machine above.</EmptyState>{/if}
<div class="sums g2">
  {#each shown.slice(0, limit) as project (project.key)}
    {@const pinned = choices.isPinned(project.key)}
    {@const hidden = choices.isHidden(project.key)}
    <div class="pinned-card" class:hidden>
    <a class="card sum" href={projectHref(project.key)} title={project.key}>
      <span class="sum-h"><span class="ico lg projects"><Folder size={20} aria-hidden="true" /></span><span class="sum-t"><b>{projectLabel(project.key)}</b><code>{project.key}</code></span></span>
      <span class="big"><b>{project.chats.length.toLocaleString()}</b><span>chat{project.chats.length === 1 ? "" : "s"}</span></span>
      <span class="bar"><i style:width="{Math.max(2, Math.round((project.chats.length / largest) * 100))}%"></i></span>
      <span class="sum-f">
        {#each project.agents.slice(0, 5) as [agent] (agent)}<span title={agentInfo[agent].name}><AgentMark {agent} size="sm" /></span>{/each}
        {#each project.machines as [machine] (machine)}<span class="mchip"><MachineIcon {machine} size={13} />{machine}</span>{/each}
        <span class="sp">{whenLabel(project.latestMs, now)}</span>
      </span>
    </a>
    <!-- Beside the link rather than in it: a button cannot sit inside a link. -->
    {#if choices.editable}
      <span class="acts">
        {#if !hidden}<Tip text={pinned ? "Unpin" : "Pin to the top"}>{#snippet children(tip)}<button {...tip} class="pin" class:on={pinned} type="button" aria-pressed={pinned} aria-label="{pinned ? 'Unpin' : 'Pin'} {projectLabel(project.key)}" onclick={() => void choices.togglePin(project.key)}><Pin size={16} aria-hidden="true" /></button>{/snippet}</Tip>{/if}
        <Tip text={hidden ? "Show in project lists again" : "Hide from project lists"}>{#snippet children(tip)}<button {...tip} class="pin" class:on={hidden} type="button" aria-pressed={hidden} aria-label="{hidden ? 'Show' : 'Hide'} {projectLabel(project.key)}" onclick={() => void choices.toggleHidden(project.key)}>{#if hidden}<Eye size={16} aria-hidden="true" />{:else}<EyeOff size={16} aria-hidden="true" />{/if}</button>{/snippet}</Tip>
      </span>
    {:else if pinned}
      <span class="acts"><Tip text="Pinned" passive>{#snippet children(tip)}<span {...tip} class="pin on" role="img" aria-label="Pinned"><Pin size={16} aria-hidden="true" /></span>{/snippet}</Tip></span>
    {/if}
    </div>
  {/each}
</div>
{#if shown.length > limit}{#key limit}<div bind:this={more} aria-hidden="true" style:height="1px"></div>{/key}{/if}

<style>
  /* The search box is shaped like the filter chips beside it. */
  :global(.chips) > .pfind {
    min-height: 32px;
    border: var(--border-outline);
    border-radius: var(--radius-5);
  }
  /* The card fills its grid cell; the pin sits in its top corner. */
  .pinned-card {
    position: relative;
    display: flex;
  }
  .pinned-card > :global(.card) {
    flex: 1;
    min-width: 0;
  }
  .acts {
    position: absolute;
    top: 14px;
    right: 14px;
    display: flex;
    gap: 2px;
  }
  /* A hidden project, shown on request, reads as set aside. */
  .pinned-card.hidden > :global(.card) {
    opacity: 0.55;
  }
  .pin {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-pill);
    color: var(--text-muted);
  }
  button.pin:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .pin.on {
    color: var(--text-strong);
  }
  .pin.on :global(svg) {
    fill: currentColor;
  }
  /* With a mouse, an unpinned project's pin waits until the card is pointed
     at or the button is reached by keyboard; a touch screen always shows it. */
  @media (hover: hover) and (pointer: fine) {
    .pinned-card:not(.hidden) button.pin:not(.on) {
      opacity: 0;
    }
    .pinned-card:hover button.pin,
    button.pin:focus-visible {
      opacity: 1;
    }
  }
  /* The card as in the wireframes: a ring and a soft shadow, a small tile,
     the name over its path, the count, a thin bar, then quiet chips. */
  .pinned-card > :global(.card.sum) {
    gap: 12px;
    padding: 16px 18px;
    border-color: var(--line-outline);
    border-radius: var(--radius-8);
    box-shadow: var(--shadow-panel);
  }
  .pinned-card :global(.ico.lg) {
    width: 36px;
    height: 36px;
    border: 0;
    background: var(--selected-surface);
    color: var(--text-muted);
  }
  .pinned-card :global(.sum-t) {
    gap: 3px;
  }
  .pinned-card :global(.sum-h b) {
    font-size: 14px;
    font-weight: var(--weight-strong);
    letter-spacing: 0;
  }
  .pinned-card :global(.big b) {
    font-size: 22px;
    font-weight: var(--weight-strong);
    letter-spacing: -0.02em;
  }
  .pinned-card :global(.big span) {
    font-size: 13px;
    color: var(--text-muted);
  }
  .pinned-card :global(.bar) {
    height: 4px;
    border: 0;
    background: var(--selected-surface);
  }
  .pinned-card :global(.bar i) {
    border: 0;
    border-radius: var(--radius-pill);
    background: var(--text-muted);
  }
  .pinned-card :global(.mchip) {
    height: 22px;
    padding: 0 7px;
    border: 0;
    border-radius: var(--radius-3);
    background: var(--selected-surface);
    font-size: 12px;
    font-weight: var(--weight-regular);
    color: var(--text-muted);
  }
  .pinned-card :global(.sum-f) {
    font-size: 12px;
  }
  /* Room for the pin, so a long name never runs under it. */
  .pinned-card :global(.sum-h) {
    padding-right: 70px;
  }
</style>
