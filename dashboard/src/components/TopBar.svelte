<script lang="ts">
// The bar across the top of every page: the mark and name at the left, the
// search field in the middle, then the viewing switcher and Mashy's circle at
// the right. On a phone it keeps the mark, a search button that shows the
// field on a row of its own, Mashy, and the menu button that opens the
// navigation sheet.
import Menu from "@lucide/svelte/icons/menu";
import Search from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { tick } from "svelte";
import { asset } from "../lib/asset";
import { FIND_EVENT } from "../lib/findField";
import { catalogOptions } from "../lib/queries";
import { router } from "../lib/route.svelte";
import MashyButton from "./MashyButton.svelte";
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

<header id="top" class:searching inert={railOpen}>
  <a class="brand" href="#/" aria-label="Mash overview"><img src={asset("mash-mark.svg")} alt="" width="24" height="24" /><span>mash<i class="pat" aria-hidden="true"></i></span></a>
  <TopSearch {modifier} />
  <div class="top-end">
    {#if !catalogue.error}<div class="top-viewing"><ViewingSwitcher place="bar" /></div>{/if}
    <button id="find" class="top-btn" type="button" aria-label="Show search" aria-expanded={searching} onclick={toggleField}><Search size={18} aria-hidden="true" /></button>
    <MashyButton />
    <button id="burger" class="top-btn" type="button" aria-label="Open navigation" aria-expanded={railOpen} onclick={toggleRail}><span class="menu-ico"><Menu size={18} aria-hidden="true" /></span></button>
  </div>
</header>
