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
import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
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
</script>

<div class="cpb">
  {#if previous}
    <button class="cpb-back" type="button" onclick={() => history.back()}><ChevronLeft size={18} aria-hidden="true" />{back}</button>
  {:else}
    <a class="cpb-back" href="#/"><ChevronLeft size={18} aria-hidden="true" />{back}</a>
  {/if}
  <span class="cpb-more">
    <DropdownMenu.Root bind:open>
      <DropdownMenu.Trigger class="cpb-b" aria-label="More chat actions"><Ellipsis size={18} aria-hidden="true" /></DropdownMenu.Trigger>
      <DropdownMenu.ContentStatic class="right-0 min-w-[220px]" aria-label="More chat actions">
        <DropdownMenu.Item aria-label="{sessionsShown ? 'Hide' : 'Show'} session list" onSelect={ontogglesessions}><PanelLeft size={15} aria-hidden="true" />Sessions</DropdownMenu.Item>
        <DropdownMenu.Item onSelect={onfind}><Search size={15} aria-hidden="true" />Find in chat</DropdownMenu.Item>
        <DropdownMenu.Item onSelect={oncopy}><Copy size={15} aria-hidden="true" />Copy session ID</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("pdf")}><Printer size={15} aria-hidden="true" />Export to PDF</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("html")}><FileCode size={15} aria-hidden="true" />Export to HTML</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("markdown")}><FileText size={15} aria-hidden="true" />Export to Markdown</DropdownMenu.Item>
      </DropdownMenu.ContentStatic>
    </DropdownMenu.Root>
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
  .cpb-more :global(.cpb-b) {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-5);
    color: var(--text-strong);
  }
  .cpb-more :global(.cpb-b[aria-expanded="true"]),
  .cpb-more :global(.cpb-b:hover),
  .cpb-back:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
</style>
