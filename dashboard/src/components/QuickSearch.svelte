<script lang="ts">
import Laptop from "@lucide/svelte/icons/laptop";
import SearchIcon from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { onMount } from "svelte";
import { createLiveSearch } from "../lib/liveSearch.svelte";
import { onThisMachine } from "../lib/openIn";
import {
  ariaShortcut,
  browserPlatform,
  fullSearchTarget,
  hidePanel,
  moveSelection,
  pageTarget,
  type QuickRow,
  quickCommands,
  quickEmpty,
  quickKeys,
  quickRows,
  quickStatus,
  rowRoute,
  rowTarget,
  searching,
} from "../lib/quick";
import { scopeName } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { inDesktopApp, shareLink } from "../lib/shareLink";
import { type Status, statusOptions } from "../lib/status";
import QuickActions from "./QuickActions.svelte";
import QuickFooter from "./QuickFooter.svelte";
import QuickList from "./QuickList.svelte";
import QuickPreview from "./QuickPreview.svelte";

// The panel the desktop app drops down from its menu bar icon (and shows for
// its global shortcut): live search over this machine, or the latest chats
// and a few of Mash's pages before anything is typed. It is drawn in the
// dashboard's own theme, light or dark as the system is. The desktop window
// is small (menubar.rs), so the preview of the selected chat appears only
// where the panel is wide enough, as in a browser.
const desktop = inDesktopApp();
// The desktop app on macOS draws the panel's window on the system's blurred
// popover material, and says so before the page loads (menubar.rs).
const material =
  (globalThis as { __MASH_PANEL_MATERIAL__?: unknown }).__MASH_PANEL_MATERIAL__ === true;
const keys = quickKeys(browserPlatform());
// Wider than this, the list keeps to the left and the preview has the rest.
const PREVIEW_WIDTH = 600;
// The panel follows the viewing scope chosen in the dashboard, as the top bar's
// suggestions do: the same browser storage holds the choice for both.
const catalogue = createScoped();
let text = $state("");
let typed = $state("");
$effect(() => {
  const value = text.trim();
  const timer = setTimeout(() => (typed = value), 60);
  return () => clearTimeout(timer);
});
// Every source in the viewing scope, as one list (see lib/liveSearch.svelte.ts).
// It waits for the chat list, which names this machine, but searches anyway
// when the list failed, so a typed search fails where the reader can see it.
const live = createLiveSearch(() => ({
  typed,
  enabled: searching(typed),
  catalogue: catalogue.data,
  withoutCatalogue: catalogue.isError,
  limit: 8,
}));
const rows = $derived(quickRows(text, catalogue.data?.chats ?? [], live.hits));
const commands = $derived(quickCommands(text, desktop, keys));
const failed = $derived(searching(text) ? live.error : null);
// The panel stays open in the background, so it reads status again when it
// is shown, and every few seconds while indexing is under way.
const status = createQuery(() => ({
  ...statusOptions,
  staleTime: 10_000,
  refetchOnWindowFocus: true,
  refetchInterval: (query: { state: { data: Status | undefined } }) =>
    query.state.data && !query.state.data.ready && query.state.data.watching ? 5_000 : false,
}));
const health = $derived(quickStatus(status.data, status.isError));
// Mash isn't answering when it cannot list chats, or cannot report status
// before the list has arrived.
const unreachable = $derived(catalogue.isError || (status.isError && !catalogue.data));
const showChats = $derived(!failed && !(unreachable && !searching(text)));
// One selection runs through the chats and then the suggestions under them.
const chatCount = $derived(showChats ? rows.length : 0);
let selected = $state(0);
const chosenRow = $derived(selected < chatCount ? rows[selected] : undefined);
const chosenCommand = $derived(commands[selected - chatCount]);
// A new text starts at the top; results still arriving for the same text
// leave the highlighted row where it is.
$effect(() => {
  void typed;
  selected = 0;
});
const scopeLabel = $derived.by(() => {
  const data = catalogue.data;
  if (!data || (!data.reach.joined && data.reach.live.length === 0)) return "";
  return scopeName(data.scope, data.reach);
});
const local = $derived(
  !!chosenRow &&
    onThisMachine(chosenRow.chat, catalogue.data?.localNode, catalogue.data?.sharedOnly ?? false),
);
// Memory has something to show for a chat only when it is turned on.
const memoryOn = $derived(status.data?.configuration.recordsEnabled ?? false);
let width = $state(0);
const wide = $derived(width >= PREVIEW_WIDTH);
let menuOpen = $state(false);
let input: HTMLInputElement | undefined = $state();
let list: HTMLDivElement | undefined = $state();
let actionsButton: HTMLButtonElement | undefined = $state();
let menu: HTMLDivElement | undefined = $state();
let copied: { label: string; ok: boolean } | null = $state(null);
// The panel lives for the whole app session, so ages are measured from a
// clock that moves on when the panel is shown and every half minute.
let now = $state(Date.now());

// The selected row stays in view as the arrow keys move past the list's edge.
$effect(() => {
  void selected;
  list?.querySelector('[aria-selected="true"]')?.scrollIntoView?.({ block: "nearest" });
});

function shown(): void {
  now = Date.now();
  // A failed or empty list is read again, so the panel catches up once Mash
  // is back or its first index pass has finished.
  if (catalogue.isError || catalogue.data?.chats.length === 0) void catalogue.refetch();
  if (!menuOpen) {
    input?.focus();
    input?.select();
  }
}
onMount(() => {
  shown();
  const timer = setInterval(() => {
    if (document.visibilityState === "visible") now = Date.now();
  }, 30_000);
  // Around the panel: clear in the desktop app's window, whose rounded edge
  // the panel draws; the theme's page in a browser.
  if (desktop) document.documentElement.dataset.window = "panel";
  return () => {
    clearInterval(timer);
    delete document.documentElement.dataset.window;
  };
});

function retry(): void {
  void catalogue.refetch();
  void status.refetch();
  if (searching(typed)) live.retry();
}

function go(target: string): void {
  menuOpen = false;
  location.href = target;
}

function open(): void {
  if (chosenRow) go(rowTarget(chosenRow, desktop));
  else if (chosenCommand) go(chosenCommand.target);
}

let copiedTimer: ReturnType<typeof setTimeout> | undefined;
function notice(label: string, ok: boolean): void {
  menuOpen = false;
  input?.focus();
  copied = { label, ok };
  clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => (copied = null), 1_500);
}

async function copyLink(row: QuickRow): Promise<void> {
  const link = desktop
    ? rowTarget(row, true)
    : shareLink(rowRoute(row), `${location.origin}${location.pathname}`, false);
  try {
    if (!navigator.clipboard) throw new Error("no clipboard");
    await navigator.clipboard.writeText(link);
    notice("Link copied", true);
  } catch {
    notice("Couldn’t copy the link", false);
  }
}

function toggleMenu(): void {
  menuOpen = !menuOpen;
  if (!menuOpen) input?.focus();
}

// Escape closes the menu, then clears the query, then hides the panel,
// wherever focus is in the panel. ⌘K (Ctrl+K) opens and closes the actions.
function windowKeydown(event: KeyboardEvent): void {
  if (event.key === "k" && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    toggleMenu();
    return;
  }
  if (event.key !== "Escape") return;
  event.preventDefault();
  if (menuOpen) {
    menuOpen = false;
    input?.focus();
  } else if (text) text = "";
  else hidePanel();
}

function pointerdown(event: PointerEvent): void {
  const target = event.target as Node | null;
  if (menuOpen && !menu?.contains(target) && !actionsButton?.contains(target)) menuOpen = false;
}

function keydown(event: KeyboardEvent): void {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    // Moving to a result keeps the list where it is while more results arrive.
    live.interact();
    const step = event.key === "ArrowDown" ? 1 : -1;
    selected = moveSelection(selected, step, chatCount + commands.length);
  } else if (event.key === "Enter") {
    event.preventDefault();
    if (event.metaKey || event.ctrlKey) go(fullSearchTarget(text, desktop));
    else open();
  } else if ((event.key === "o" || event.key === ",") && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    go(pageTarget(event.key === "o" ? "#/" : "#/settings", desktop));
  } else if (
    event.key === "c" &&
    (event.metaKey || event.ctrlKey) &&
    chosenRow &&
    input?.selectionStart === input?.selectionEnd
  ) {
    event.preventDefault();
    void copyLink(chosenRow);
  }
}

function hover(index: number): void {
  selected = index;
  live.interact();
}
</script>

<!-- Shown again from the menu bar, the panel is ready to type into. -->
<svelte:window onfocus={shown} onkeydown={windowKeydown} onpointerdown={pointerdown} />

<div class="quick" class:material class:wide role="search" bind:clientWidth={width}>
  <label class="q-field">
    <SearchIcon size={18} aria-hidden="true" />
    <span class="sr-only">Search Mash</span>
    <input bind:this={input} bind:value={text} placeholder="Search Mash" autocomplete="off" spellcheck="false" onkeydown={keydown} />
    {#if scopeLabel}<span class="q-scope" title="The viewing scope chosen in Mash"><Laptop size={14} aria-hidden="true" />{scopeLabel}</span>{/if}
  </label>

  <div class="q-body">
    <div class="q-list" bind:this={list}>
      <QuickList {rows} {commands} chats={showChats} heading={searching(text) ? "Chats" : "Recent"} label={searching(text) ? "Matching chats" : "Recent chats"} {selected} query={typed} {desktop} {keys} {now} onhover={hover}>
        {#snippet notes()}
        {#if failed}
          <div class="q-note" role="alert">
            <strong>Mash couldn’t search.</strong>
            <span>{failed.message}</span>
            <span class="q-acts"><a href={pageTarget("#/", desktop)}>Open Mash</a><button type="button" onclick={retry}>Try again</button></span>
          </div>
        {:else if unreachable && !searching(text)}
          <div class="q-note" role="alert">
            <strong>Mash isn’t responding.</strong>
            <span>The panel can’t read your chats right now.</span>
            <span class="q-acts"><a href={pageTarget("#/", desktop)}>Open Mash</a><button type="button" onclick={retry}>Try again</button></span>
          </div>
        {:else}
          {#if searching(text) && live.status.text}<p class="q-empty">{live.status.text}{#if live.status.retry}{" · "}<button class="a" type="button" onclick={() => live.retry()}>Retry</button>{/if}</p>{/if}
          {#if searching(text) && live.answered && !live.status.text && rows.length === 0}
            <p class="q-empty">No chats mention “{text.trim()}”.</p>
          {:else if !searching(text) && catalogue.data && rows.length === 0}
            {@const empty = quickEmpty(status.data)}
            <p class="q-empty">{empty.text}{#if empty.action}<br /><a href={pageTarget(empty.action.route, desktop)}>{empty.action.label}</a>{/if}</p>
          {/if}
        {/if}
        {/snippet}
      </QuickList>
    </div>
    {#if wide && chosenRow}<QuickPreview row={chosenRow} query={typed} {now} {local} memory={memoryOn} {desktop} />{/if}
  </div>

  <QuickFooter bind:actionsButton label={copied?.label ?? health.label} tone={copied ? (copied.ok ? "ready" : "problem") : health.tone} open={chosenRow ? "Open chat" : chosenCommand ? "Open" : ""} {menuOpen} {keys} onopen={open} ontoggle={toggleMenu} />
  {#if menuOpen}
    <QuickActions bind:element={menu} row={chosenRow} {local} memory={memoryOn} {desktop} {keys} {go} oncopylink={() => chosenRow && void copyLink(chosenRow)} onnotice={notice} />
  {/if}
</div>

<style>
  :global(:root[data-surface="native"]),
  :global(:root[data-surface="native"] body) {
    min-height: 0;
    background: var(--surface-muted);
  }
  :global(:root[data-surface="native"][data-window="panel"]),
  :global(:root[data-surface="native"][data-window="panel"] body) {
    background: var(--quick-backdrop);
  }
  /* In the desktop app the panel is its window; in a browser it floats in
     the middle of the page at the launcher's size. */
  .quick {
    position: relative;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: min(100vw, 720px);
    height: min(100vh, 460px);
    margin: max(0px, calc((100vh - 460px) / 2)) auto 0;
    overflow: hidden;
    border-radius: var(--radius-7);
    background: var(--surface-page);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-strong),
      var(--shadow-float);
    color: var(--text-strong);
    font: 13px / 1.3 var(--type-sans);
    font-feature-settings: "tnum" 1;
  }
  :global(:root[data-window="panel"]) .quick {
    width: 100vw;
    height: 100vh;
    margin: 0;
    box-shadow: inset 0 0 0 var(--stroke-outline) var(--line-strong);
  }
  .quick.material {
    background: var(--quick-surface-material);
  }
  .q-field {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: none;
    height: 48px;
    padding: 0 12px 0 16px;
    border-bottom: var(--border-divider);
    color: var(--text-muted);
  }
  .wide .q-field {
    height: 58px;
    padding: 0 14px 0 20px;
  }
  .q-field input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--text-strong);
    caret-color: var(--accent-strong);
    font: var(--weight-regular) 16px / 1 var(--type-sans);
    letter-spacing: -0.01em;
  }
  .wide .q-field input {
    font-size: 18px;
  }
  .q-field input::placeholder {
    color: var(--text-faint);
  }
  .q-scope {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 7px;
    max-width: 40%;
    overflow: hidden;
    color: var(--text-muted);
    font-size: 12px;
    font-weight: var(--weight-label);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .q-body {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .q-list {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 2px 6px 6px;
  }
  .wide .q-list {
    flex: none;
    width: 330px;
    box-sizing: border-box;
    padding: 4px 8px 8px;
    border-right: var(--border-divider);
  }
  .q-empty,
  .q-note {
    margin: 0;
    padding: 20px 16px;
    color: var(--text-muted);
    text-align: center;
  }
  .q-note {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: var(--text-body);
  }
  .q-note a,
  .q-empty a {
    color: inherit;
    text-decoration: underline;
    text-decoration-color: var(--link-line);
  }
  .q-acts {
    display: flex;
    justify-content: center;
    gap: 14px;
  }
  .q-acts button {
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    text-decoration: underline;
    text-decoration-color: var(--link-line);
    cursor: pointer;
  }
</style>
