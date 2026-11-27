<script lang="ts">
// The bar across the top of every page: the mark and name at the left, the
// search field in the middle and the viewing switcher at the right. In the
// Mac app the traffic lights take the mark's place. On a
// phone it keeps the mark, a search button that shows the field on a row of
// its own, and the menu button that opens the navigation sheet.

import { createQuery } from "@tanstack/svelte-query";
import ListIcon from "phosphor-svelte/lib/ListIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import { tick } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { asset } from "../lib/asset";
import { FIND_EVENT } from "../lib/findField";
import { catalogOptions } from "../lib/queries";
import { router } from "../lib/route.svelte";
import { shell } from "../lib/shell";
import TopSearch from "./TopSearch.svelte";
import ViewingSwitcher from "./ViewingSwitcher.svelte";

let { railOpen, toggleRail }: { railOpen: boolean; toggleRail: () => void } = $props();
const catalogue = createQuery(() => catalogOptions);

// On a phone the field takes a row of its own: always on the results page,
// where it holds the query, and elsewhere once the search button asks for it.
let asked = $state(false);
const onResults = $derived(router.current.page === "search");
const searching = $derived(asked || onResults);
$effect(() => {
  void router.current;
  asked = false;
});

async function showField(): Promise<void> {
  asked = true;
  await tick();
  const field = document.getElementById("q");
  if (!(field instanceof HTMLInputElement)) return;
  field.focus();
  field.select();
}
function toggleField(): void {
  if (searching && !onResults) asked = false;
  else void showField();
}
// The rail's Search row asks for the field from outside the bar.
$effect(() => {
  const listener = () => void showField();
  addEventListener(FIND_EVENT, listener);
  return () => removeEventListener(FIND_EVENT, listener);
});

// ⌘K on a Mac, Ctrl+K elsewhere, focuses the search field from anywhere in
// the dashboard, as the hint inside the field says.
const modifier = /Mac|iPhone|iPad/.test(globalThis.navigator?.platform ?? "") ? "⌘" : "Ctrl";
function focusSearch(event: KeyboardEvent): void {
  if (
    event.key.toLowerCase() !== "k" ||
    !(event.metaKey || event.ctrlKey) ||
    event.altKey ||
    event.shiftKey
  )
    return;
  if (!document.getElementById("q")) return;
  event.preventDefault();
  void showField();
}
</script>

<svelte:window onkeydown={focusSearch} />

<!-- In the desktop app the bar is also the window's title bar: dragging an
     empty part of it moves the window and a double-click zooms it. On macOS
     the traffic lights take the mark's place, over the sidebar's colour. -->
<header id="top" class:searching inert={railOpen} data-tauri-drag-region={shell === "browser" ? undefined : ""}>
  {#if shell === "macos"}
    <span class="lights" data-tauri-drag-region aria-hidden="true"></span>
  {:else}
    <a class="brand" href="#/" aria-label="Mash overview"><img src={asset("mash-mark.svg")} alt="" width="24" height="24" /><span>mash<i class="pat" aria-hidden="true"></i></span></a>
  {/if}
  <TopSearch {modifier} />
  <div class="top-end">
    {#if !catalogue.error}<div class="top-viewing"><ViewingSwitcher place="bar" /></div>{/if}
    <button id="find" class="top-btn" type="button" aria-label="Show search" aria-expanded={searching} onclick={toggleField}><Icon icon={MagnifyingGlassIcon} size={18} /></button>
    <button id="burger" class="top-btn" type="button" aria-label="Open navigation" aria-expanded={railOpen} onclick={toggleRail}><span class="menu-ico"><Icon icon={ListIcon} size={18} /></span></button>
  </div>
</header>

<style>
  /* The bar's labels are controls, not text: a drag across them selects nothing. */
  #top {
    -webkit-user-select: none;
    user-select: none;
  }
  /* The Mac app hides the system title bar, so this bar is the window's: its
     left cell takes the sidebar's width and colour, holding the traffic
     lights (desktop/src-tauri/src/frame.rs), and the sidebar seems to run to
     the top of the window, as in Finder and Mail. */
  @media (min-width: 801px) {
    :global(:root[data-shell="macos"]) #top {
      grid-template-columns: var(--rail-width, 220px) minmax(0, 1fr) minmax(0, 440px) minmax(0, 1fr);
      column-gap: 0;
      padding-inline: 0;
    }
    /* Over the bar's padding and its bottom line, so it meets the sidebar. */
    :global(:root[data-shell="macos"]) .lights {
      align-self: stretch;
      margin: -6px 0 -7px;
      background: var(--surface-rail);
      border-right: var(--border-outline);
    }
    /* The field sits centred over the page beside the sidebar; the bar's side
       padding moves onto its controls. */
    :global(:root[data-shell="macos"]) #top > :global(.search) {
      grid-column: 3;
    }
    :global(:root[data-shell="macos"]) .top-end {
      padding-right: 16px;
    }
  }
  /* A narrow Mac window keeps room at the left for the traffic lights. */
  @media (max-width: 800px) {
    :global(:root[data-shell="macos"]) .lights {
      flex: none;
      width: 64px;
    }
  }
</style>
