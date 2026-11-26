<script lang="ts">
// A chat's own bar on a phone, above its title: the way back to where the
// reader came from, named for that page, and a menu holding the actions on
// the whole chat that the two buttons under the title leave out.
import CaretLeftIcon from "phosphor-svelte/lib/CaretLeftIcon";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import DotsThreeIcon from "phosphor-svelte/lib/DotsThreeIcon";
import FileCodeIcon from "phosphor-svelte/lib/FileCodeIcon";
import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import PrinterIcon from "phosphor-svelte/lib/PrinterIcon";
import SidebarSimpleIcon from "phosphor-svelte/lib/SidebarSimpleIcon";
import Icon from "$lib/components/Icon.svelte";
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
    <button class="cpb-back" type="button" onclick={() => history.back()}><Icon icon={CaretLeftIcon} size={18} />{back}</button>
  {:else}
    <a class="cpb-back" href="#/"><Icon icon={CaretLeftIcon} size={18} />{back}</a>
  {/if}
  <span class="cpb-more">
    <DropdownMenu.Root bind:open>
      <DropdownMenu.Trigger class="cpb-b" aria-label="More chat actions"><Icon icon={DotsThreeIcon} size={18} /></DropdownMenu.Trigger>
      <DropdownMenu.ContentStatic class="right-0 min-w-[220px]" aria-label="More chat actions">
        <DropdownMenu.Item aria-label="{sessionsShown ? 'Hide' : 'Show'} session list" onSelect={ontogglesessions}><Icon icon={SidebarSimpleIcon} size={15} />Sessions</DropdownMenu.Item>
        <DropdownMenu.Item onSelect={onfind}><Icon icon={MagnifyingGlassIcon} size={15} />Find in chat</DropdownMenu.Item>
        <DropdownMenu.Item onSelect={oncopy}><Icon icon={CopyIcon} size={15} />Copy session ID</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("pdf")}><Icon icon={PrinterIcon} size={15} />Export to PDF</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("html")}><Icon icon={FileCodeIcon} size={15} />Export to HTML</DropdownMenu.Item>
        <DropdownMenu.Item disabled={!canExport} onSelect={() => onexport("markdown")}><Icon icon={FileTextIcon} size={15} />Export to Markdown</DropdownMenu.Item>
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
