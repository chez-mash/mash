<script lang="ts">
import ClockIcon from "phosphor-svelte/lib/ClockIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import FunnelIcon from "phosphor-svelte/lib/FunnelIcon";
import GitForkIcon from "phosphor-svelte/lib/GitForkIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import RobotIcon from "phosphor-svelte/lib/RobotIcon";
import SortAscendingIcon from "phosphor-svelte/lib/SortAscendingIcon";
import SortDescendingIcon from "phosphor-svelte/lib/SortDescendingIcon";
import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
import Icon from "$lib/components/Icon.svelte";
import { agents as agentInfo } from "../lib/agents";
import { isAgent } from "../lib/api";
import { projectLabel } from "../lib/browse";
import { navigate } from "../lib/navigation";
import {
  type Applied,
  appliedHref,
  dateRanges,
  parseIntent,
  rankings,
  type Sort,
  sameSearch,
  searchIntents,
  sorts,
  subagentHitViews,
  subagentViews,
} from "../lib/search";
import AgentMark from "./AgentMark.svelte";
import FilterMenu from "./FilterMenu.svelte";

type Field = "agents" | "machines" | "projects";
type Option = { value: string; label: string; count?: number };

// The search page's filter chips. They edit a draft of the search; "Apply
// filters" puts the draft in the address, so results never change under the
// reader. On a phone the chips fold behind one button that counts them.
let {
  applied,
  draft = $bindable(),
  options,
  hasSubagents,
  resolvedIntent,
}: {
  applied: Applied;
  draft: Applied;
  options: { agent: Option[]; machine: Option[]; project: Option[] };
  /** Whether any chat in view is a subagent, so the subagent chip means something. */
  hasSubagents: boolean;
  /** The kind of search Mash chose for a default search, once it answered. */
  resolvedIntent: string | undefined;
} = $props();
let open = $state(false);
const dirty = $derived(!sameSearch(draft, applied));
// How many filters narrow the results. The sort order is not a filter.
const active = $derived(
  [
    applied.agents.length > 0,
    applied.machines.length > 0,
    applied.projects.length > 0,
    applied.days !== null,
    !!applied.query && applied.intent !== "default",
    !!applied.query && applied.subagentHits === "included",
  ].filter(Boolean).length,
);
/** The one agent chosen, whose mark stands in for the chip's icon. */
const onlyAgent = $derived(draft.agents.length === 1 ? draft.agents[0] : undefined);
const intentLabel = $derived.by(() => {
  const chosen = searchIntents.find((intent) => intent.value === draft.intent)?.label;
  const resolved =
    draft.intent === "default" && applied.intent === "default"
      ? searchIntents.find((intent) => intent.value === resolvedIntent)?.label
      : undefined;
  return resolved ? `search: default (${resolved})` : `search: ${chosen}`;
});

function toggle(field: Field, value: string): void {
  const current: readonly string[] = draft[field];
  const next = current.includes(value)
    ? current.filter((item) => item !== value)
    : [...current, value];
  draft =
    field === "agents" ? { ...draft, agents: next.filter(isAgent) } : { ...draft, [field]: next };
}
function chipLabel(field: Field, noun: string): string {
  const chosen = draft[field];
  if (chosen.length === 0) return `any ${noun}`;
  if (chosen.length > 1) return `${chosen.length} ${noun}s`;
  const [only] = chosen;
  if (only === undefined) return `any ${noun}`;
  if (field === "agents") return isAgent(only) ? agentInfo[only].name : only;
  return field === "projects" ? projectLabel(only) : only;
}
const changed = (field: Field) =>
  [...draft[field]].sort().join() !== [...applied[field]].sort().join();
function chooseSort(value: string): void {
  const sort: Sort | undefined = sorts.find((option) => option.value === value)?.value;
  if (sort) draft = { ...draft, sort };
}
function apply(): void {
  navigate(appliedHref({ ...draft, query: applied.query, offset: 0 }));
}
</script>

<div class="sf">
  <button class="btn sm fold-t" type="button" aria-expanded={open || dirty} aria-controls="search-filters" onclick={() => (open = !open)}><Icon icon={FunnelIcon} size={14} />Filters{#if active}<span class="pill">{active}</span>{/if}</button>
  <div class="chips" class:folded={!open && !dirty} id="search-filters" role="group" aria-label="Filters">
    <FilterMenu label={chipLabel("agents", "agent")} active={draft.agents.length > 0} draft={changed("agents")} multiple options={options.agent} selected={draft.agents} choose={(value) => toggle("agents", value)}>
      {#snippet icon()}{#if onlyAgent}<AgentMark agent={onlyAgent} size="sm" />{:else}<Icon icon={RobotIcon} size={15} />{/if}{/snippet}
      {#snippet optionIcon(value: string)}{#if isAgent(value)}<AgentMark agent={value} size="sm" />{:else}<Icon icon={RobotIcon} size={15} />{/if}{/snippet}
    </FilterMenu>
    <!-- Which machines is the viewing scope's to choose; a link may still name one. -->
    {#if applied.machines.length || draft.machines.length}<FilterMenu label={chipLabel("machines", "machine")} active={draft.machines.length > 0} draft={changed("machines")} multiple options={options.machine} selected={draft.machines} choose={(value) => toggle("machines", value)}>
      {#snippet icon()}<Icon icon={LaptopIcon} size={15} />{/snippet}
    </FilterMenu>{/if}
    <FilterMenu label={chipLabel("projects", "project")} active={draft.projects.length > 0} draft={changed("projects")} multiple options={options.project} selected={draft.projects} choose={(value) => toggle("projects", value)}>
      {#snippet icon()}<Icon icon={FolderIcon} size={15} />{/snippet}
    </FilterMenu>
    <FilterMenu
      label={dateRanges.find((range) => range.days === draft.days)?.label ?? "any time"}
      active={draft.days !== null}
      draft={draft.days !== applied.days}
      options={dateRanges.map((range) => ({ value: String(range.days ?? ""), label: range.label }))}
      selected={[String(draft.days ?? "")]}
      choose={(value) => (draft = { ...draft, days: value ? Number(value) : null })}
    >
      {#snippet icon()}<Icon icon={ClockIcon} size={15} />{/snippet}
    </FilterMenu>
    {#if !applied.query}
      <FilterMenu label="sort: {sorts.find((sort) => sort.value === draft.sort)?.label}" draft={draft.sort !== applied.sort} options={sorts} selected={[draft.sort]} choose={chooseSort}>
        {#snippet icon()}<Icon icon={FunnelIcon} size={15} />{/snippet}
        {#snippet optionIcon(value: string)}{#if value === "old"}<Icon icon={SortAscendingIcon} size={15} />{:else}<Icon icon={SortDescendingIcon} size={15} />{/if}{/snippet}
      </FilterMenu>
    {:else}
      <FilterMenu
        label={intentLabel}
        active={draft.intent !== "default"}
        draft={draft.intent !== applied.intent}
        options={searchIntents}
        selected={[draft.intent]}
        choose={(value) => (draft = { ...draft, intent: parseIntent(value) })}
      >
        {#snippet icon()}<Icon icon={MagnifyingGlassIcon} size={15} />{/snippet}
      </FilterMenu>
      <FilterMenu
        label="sort: {rankings.find((ranking) => ranking.value === draft.rank)?.label}"
        draft={draft.rank !== applied.rank}
        options={rankings}
        selected={[draft.rank]}
        choose={(value) => (draft = { ...draft, rank: value === "recent" ? "recent" : "relevance" })}
      >
        {#snippet icon()}<Icon icon={FunnelIcon} size={15} />{/snippet}
        {#snippet optionIcon(value: string)}{#if value === "recent"}<Icon icon={SortDescendingIcon} size={15} />{:else}<Icon icon={SparkleIcon} size={15} />{/if}{/snippet}
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
        {#snippet icon()}<Icon icon={GitForkIcon} size={15} />{/snippet}
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
        {#snippet icon()}<Icon icon={GitForkIcon} size={15} />{/snippet}
      </FilterMenu>
    {/if}
    {#if dirty}<button class="btn sm p apply" id="applyf" type="button" onclick={apply}>Apply filters</button>{/if}
  </div>
</div>

<style>
  .sf {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .apply {
    min-height: 30px;
    margin-left: 4px;
  }
  @media (max-width: 640px) {
    .chips.folded {
      display: none;
    }
  }
</style>
