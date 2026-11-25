<script lang="ts">
// A chat's own bar on a phone, above its title: the way back to where the
// reader came from, named for that page, and a menu holding the actions on
// the whole chat that the two buttons under the title leave out.
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import Copy from "@lucide/svelte/icons/copy";
import Ellipsis from "@lucide/svelte/icons/ellipsis";
import FileCode from "@lucide/svelte/icons/file-code";
import FileText from "@lucide/svelte/icons/file-text";
import PanelLeft from "@lucide/svelte/icons/panel-left";
import Printer from "@lucide/svelte/icons/printer";
import Search from "@lucide/svelte/icons/search";
import type { ExportFormat } from "../lib/export";
import { type Route, router } from "../lib/route.svelte";

let {
  sessionsShown,
  findOpen,
  canExport,
  ontogglesessions,
  onfind,
  oncopy,
  onexport,
}: {
  sessionsShown: boolean;
  findOpen: boolean;
  canExport: boolean;
  ontogglesessions: () => void;
  onfind: () => void;
  oncopy: () => void;
  onexport: (format: ExportFormat) => void;
} = $props();

const names: Partial<Record<Route["page"], string>> = {
  search: "Results",
  overview: "Overview",
  history: "Chats",
  records: "Memory",
  record: "Memory",
  project: "Project",
  agent: "Agent",
  machine: "Machine",
};
// Back returns to the page the reader came from in this visit; a chat opened
// from outside the dashboard goes back to the overview instead.
const previous = $derived(router.previous);
const back = $derived(previous ? (names[previous.page] ?? "Back") : "Overview");

let open = $state(false);
let root = $state<HTMLElement>();
function act(action: () => void): void {
  open = false;
  action();
}
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape" && open) open = false;
  }}
  onpointerdown={(event) => {
    if (open && event.target instanceof Node && !root?.contains(event.target)) open = false;
  }}
/>

<div class="cpb">
  {#if previous}
    <button class="cpb-back" type="button" onclick={() => history.back()}><ChevronLeft size={18} aria-hidden="true" />{back}</button>
  {:else}
    <a class="cpb-back" href="#/"><ChevronLeft size={18} aria-hidden="true" />{back}</a>
  {/if}
  <span class="cpb-more" bind:this={root}>
    <button class="cpb-b" type="button" aria-label="More chat actions" aria-expanded={open} onclick={() => (open = !open)}><Ellipsis size={18} aria-hidden="true" /></button>
    {#if open}
      <div class="menu cpb-m">
        <button type="button" aria-pressed={sessionsShown} aria-label="{sessionsShown ? 'Hide' : 'Show'} session list" onclick={() => act(ontogglesessions)}><PanelLeft size={15} aria-hidden="true" />Sessions</button>
        <button type="button" aria-expanded={findOpen} onclick={() => act(onfind)}><Search size={15} aria-hidden="true" />Find in chat</button>
        <button type="button" onclick={() => act(oncopy)}><Copy size={15} aria-hidden="true" />Copy session ID</button>
        <button type="button" disabled={!canExport} onclick={() => act(() => onexport("pdf"))}><Printer size={15} aria-hidden="true" />Export to PDF</button>
        <button type="button" disabled={!canExport} onclick={() => act(() => onexport("html"))}><FileCode size={15} aria-hidden="true" />Export to HTML</button>
        <button type="button" disabled={!canExport} onclick={() => act(() => onexport("markdown"))}><FileText size={15} aria-hidden="true" />Export to Markdown</button>
      </div>
    {/if}
  </span>
</div>

<style>
  .cpb {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin: -6px -8px 0 -6px;
  }
  .cpb-back {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-height: 40px;
    padding: 0 8px 0 2px;
    border-radius: var(--radius-5);
    color: var(--text-strong);
    font-size: 15px;
  }
  .cpb-more {
    position: relative;
  }
  .cpb-b {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-5);
    color: var(--text-strong);
  }
  .cpb-b[aria-expanded="true"],
  .cpb-b:hover,
  .cpb-back:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .cpb-m {
    right: 0;
    left: auto;
    min-width: 220px;
  }
</style>
