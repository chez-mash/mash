<script lang="ts">
import Lock from "@lucide/svelte/icons/lock";
import SearchIcon from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { read } from "../lib/api";
import { groupOrder, locked, parseConsole } from "../lib/console";
import { stableRead } from "../lib/queries";
import QueryError from "./QueryError.svelte";
import Skeleton from "./Skeleton.svelte";
import ToolDetail from "./ToolDetail.svelte";

// The tools an agent can call on Mash, listed from the server's own
// description of them: a list grouped by what each tool touches, and the
// chosen tool beside it. On a narrow screen there is no room beside the
// list, so the chosen tool opens under its own row instead.
const listing = createQuery(() => ({
  ...stableRead,
  queryKey: ["console-tools"],
  queryFn: async ({ signal }) =>
    parseConsole(await read("/api/console/tools", new URLSearchParams(), signal)),
}));
let filter = $state("");
// No tool is chosen at first, so the page first reads as a list of what exists.
let open = $state("");
let values = $state<Record<string, string>>({});
let wide = $state(false);
$effect(() => {
  const query = matchMedia("(min-width: 900px)");
  const update = () => {
    wide = query.matches;
  };
  update();
  query.addEventListener("change", update);
  return () => query.removeEventListener("change", update);
});

const tools = $derived(listing.data?.tools ?? []);
const matching = $derived.by(() => {
  const words = filter.trim().toLowerCase();
  return words
    ? tools.filter((tool) => `${tool.name} ${tool.about}`.toLowerCase().includes(words))
    : tools;
});
const sections = $derived(
  groupOrder
    .map((name) => ({ name, tools: matching.filter((tool) => tool.group === name) }))
    .filter((section) => section.tools.length),
);
const runnable = $derived(tools.filter((tool) => tool.readOnly).length);
// A chosen tool the filter hides is put away with its row.
const chosen = $derived(matching.find((entry) => entry.name === open));
</script>

{#snippet detail()}
  {#if chosen}
    {#key chosen.name}<ToolDetail entry={chosen} bind:values />{/key}
  {/if}
{/snippet}

<div class="hd" style:align-items="flex-end">
  <div style:min-width="0">
    <h1 class="h1">Tool explorer</h1>
    <p class="meta" style:max-width="64ch" style:margin-top="4px">Every tool your agents can call on Mash, with what it takes and what it returns. Run the read-only ones here and see exactly what comes back.</p>
  </div>
  {#if listing.data}
    <span class="pills" style:gap="8px"><span class="pill plain mono">build {listing.data.build.slice(0, 8)}</span><span class="pill plain mono">contract {listing.data.contract.slice(0, 8)}</span></span>
  {/if}
</div>
<QueryError title="Couldn’t load the tool list" query={listing} />
{#if listing.isPending}<Skeleton rows={5} />{/if}
{#if listing.data}
  <section class="sec">
    <div class="tx-count">
      <span class="meta" role="status">{tools.length} tools · {runnable} can run here · {tools.length - runnable} change data and are locked</span>
    </div>
    <div class="tx" class:picked={!!chosen}>
      <div class="card tx-list">
        <label class="tx-find"><SearchIcon size={15} aria-hidden="true" /><input type="search" aria-label="Filter tools" placeholder="Filter tools" autocomplete="off" bind:value={filter} /></label>
        {#if !sections.length}<p class="meta tx-none">No tool matches “{filter}”.</p>{/if}
        {#each sections as section (section.name)}
          <section class="ops" aria-label={section.name}>
            <h2 class="tx-g">{#if section.name === locked}<Lock size={12} aria-hidden="true" />Changes data · locked{:else}{section.name}{/if}</h2>
            <ul class="opl">
              {#each section.tools as entry (entry.name)}
                {@const expanded = open === entry.name}
                <li>
                  <button class="op-h" type="button" aria-expanded={expanded} onclick={() => (open = expanded ? "" : entry.name)}>
                    <code class="op-n">{entry.name}</code>
                    <span class="op-k" class:ok={entry.readOnly}>{entry.readOnly ? "READ" : "WRITE"}</span>
                  </button>
                  {#if expanded && !wide}<div class="tx-inline">{@render detail()}</div>{/if}
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </div>
      {#if wide}
        <div class="card tx-detail">
          {#if chosen}{@render detail()}{:else}<p class="meta">Choose a tool to see what it takes and what it returns. Tools that only read can be run here.</p>{/if}
        </div>
      {/if}
    </div>
  </section>
{/if}
