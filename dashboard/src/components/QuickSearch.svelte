<script lang="ts">
import Ellipsis from "@lucide/svelte/icons/ellipsis";
import SearchIcon from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { onMount } from "svelte";
import { agents } from "../lib/agents";
import { whenLabel } from "../lib/browse";
import { createLiveSearch } from "../lib/liveSearch.svelte";
import { plainTitle } from "../lib/markdown";
import {
  browserPlatform,
  fullSearchTarget,
  hidePanel,
  inkLogo,
  moveSelection,
  pageTarget,
  type QuickRow,
  quickEmpty,
  quickKeys,
  quickRows,
  quickStatus,
  rowRoute,
  rowTarget,
  searching,
} from "../lib/quick";
import { createScoped } from "../lib/scoped.svelte";
import { inDesktopApp, shareLink } from "../lib/shareLink";
import { type Status, statusOptions } from "../lib/status";

// The panel the desktop app drops down from its menu bar icon (and shows for
// its global shortcut): live search over this machine, or the latest chats
// before anything is typed. It looks like the operating system, not like the
// dashboard; its colours are the theme's native roles.
const desktop = inDesktopApp();
// The desktop app on macOS draws the panel's window on the system's blurred
// popover material, and says so before the page loads (menubar.rs).
const material =
  (globalThis as { __MASH_PANEL_MATERIAL__?: unknown }).__MASH_PANEL_MATERIAL__ === true;
const keys = quickKeys(browserPlatform());
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
let selected = $state(0);
// A new text starts at the top; results still arriving for the same text
// leave the highlighted row where it is.
$effect(() => {
  void typed;
  selected = 0;
});
let menuOpen = $state(false);
let input: HTMLInputElement | undefined = $state();
let more: HTMLButtonElement | undefined = $state();
let menu: HTMLDivElement | undefined = $state();
let copied: { label: string; ok: boolean } | null = $state(null);
// The panel lives for the whole app session, so ages are measured from a
// clock that moves on when the panel is shown and every half minute.
let now = $state(Date.now());

function shown(): void {
  now = Date.now();
  // A failed or empty list is read again, so the panel catches up once Mash
  // is back or its first index pass has finished.
  if (catalogue.isError || catalogue.data?.chats.length === 0) void catalogue.refetch();
  input?.focus();
  input?.select();
}
onMount(() => {
  shown();
  const timer = setInterval(() => {
    if (document.visibilityState === "visible") now = Date.now();
  }, 30_000);
  return () => clearInterval(timer);
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

let copiedTimer: ReturnType<typeof setTimeout> | undefined;
async function copyLink(row: QuickRow): Promise<void> {
  const link = desktop
    ? rowTarget(row, true)
    : shareLink(rowRoute(row), `${location.origin}${location.pathname}`, false);
  try {
    if (!navigator.clipboard) throw new Error("no clipboard");
    await navigator.clipboard.writeText(link);
    copied = { label: "Link copied", ok: true };
  } catch {
    copied = { label: "Couldn’t copy the link", ok: false };
  }
  clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => (copied = null), 1_500);
}

// Escape closes the menu, then clears the query, then hides the panel,
// wherever focus is in the panel.
function closeOnEscape(event: KeyboardEvent): void {
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
  if (menuOpen && !menu?.contains(target) && !more?.contains(target)) menuOpen = false;
}

function keydown(event: KeyboardEvent): void {
  const row = rows[selected];
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    // Moving to a result keeps the list where it is while more results arrive.
    live.interact();
    selected = moveSelection(selected, event.key === "ArrowDown" ? 1 : -1, rows.length);
  } else if (event.key === "Enter") {
    event.preventDefault();
    if (event.metaKey || event.ctrlKey) go(fullSearchTarget(text, desktop));
    else if (row) go(rowTarget(row, desktop));
  } else if ((event.key === "o" || event.key === ",") && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    go(pageTarget(event.key === "o" ? "#/" : "#/settings", desktop));
  } else if (
    event.key === "c" &&
    (event.metaKey || event.ctrlKey) &&
    row &&
    input?.selectionStart === input?.selectionEnd
  ) {
    event.preventDefault();
    void copyLink(row);
  }
}
</script>

<!-- Shown again from the menu bar, the panel is ready to type into. -->
<svelte:window onfocus={shown} onkeydown={closeOnEscape} onpointerdown={pointerdown} />

<div class="quick" class:material role="search">
  <label class="q-field">
    <SearchIcon size={17} aria-hidden="true" />
    <span class="sr-only">Search Mash</span>
    <input bind:this={input} bind:value={text} placeholder="Search Mash" autocomplete="off" spellcheck="false" onkeydown={keydown} />
    {#if searching(text) && live.answered}<small>{live.hits.length === 1 ? "1 chat" : `${live.hits.length} chats`}</small>{/if}
  </label>

  <div class="q-list">
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
      <h2>{searching(text) ? "Chats" : "Recent chats"}</h2>
      <ul role="listbox" aria-label={searching(text) ? "Matching chats" : "Recent chats"}>
        {#each rows as row, index (row.chat.machine + row.chat.id)}
          <li role="option" aria-selected={index === selected}>
            <a class="q-row" class:q-sel={index === selected} href={rowTarget(row, desktop)} onmouseenter={() => {
              selected = index;
              live.interact();
            }}>
              <span class="qmark" aria-hidden="true" style:color={index === selected || inkLogo(row.chat.agent) ? undefined : agents[row.chat.agent].tint}>{@html agents[row.chat.agent].logo}</span>
              <span class="q-t">
                <span class="q-title">{plainTitle(row.chat.title)}</span>
                {#if row.passage}<span class="q-passage">{row.passage}</span>{/if}
              </span>
              {#if row.byMeaning}<span class="q-age">by meaning</span>{/if}
              <span class="q-age">{whenLabel(row.chat.updatedMs, now)}</span>
            </a>
          </li>
        {/each}
      </ul>
      {#if searching(text) && live.status.text}<p class="q-empty" role="status">{live.status.text}{#if live.status.retry}{" · "}<button class="a" type="button" onclick={() => live.retry()}>Retry</button>{/if}</p>{/if}
      {#if searching(text) && live.answered && !live.status.text && rows.length === 0}
        <p class="q-empty">No chats mention “{text.trim()}”.</p>
      {:else if !searching(text) && catalogue.data && rows.length === 0}
        {@const empty = quickEmpty(status.data)}
        <p class="q-empty">{empty.text}{#if empty.action}<br /><a href={pageTarget(empty.action.route, desktop)}>{empty.action.label}</a>{/if}</p>
      {/if}
    {/if}
  </div>

  <footer>
    <span class="q-status" role="status"><span class="q-dot" data-tone={copied ? (copied.ok ? "ready" : "problem") : health.tone} aria-hidden="true"></span>{copied?.label ?? health.label}</span>
    <span class="q-end">
      <a href={fullSearchTarget(text, desktop)}>Search Everywhere {keys.everywhere}</a>
      <button bind:this={more} type="button" class="q-more" aria-label="More" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}><Ellipsis size={16} aria-hidden="true" /></button>
    </span>
    {#if menuOpen}
      <div bind:this={menu} class="q-menu" role="menu">
        <a role="menuitem" href={pageTarget("#/", desktop)} aria-keyshortcuts="{keys.modifier}+O">Open Mash<kbd>{keys.open}</kbd></a>
        <a role="menuitem" href={pageTarget("#/settings", desktop)} aria-keyshortcuts="{keys.modifier}+,">Settings…<kbd>{keys.settings}</kbd></a>
      </div>
    {/if}
  </footer>
</div>

<style>
  :global(:root[data-surface="native"]) {
    color-scheme: var(--native-scheme);
  }
  :global(:root[data-surface="native"]),
  :global(:root[data-surface="native"] body) {
    min-height: 0;
    background: var(--native-backdrop);
  }
  .quick {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    box-sizing: border-box;
    overflow: hidden;
    border: 1px solid var(--native-line);
    border-radius: var(--native-radius-panel);
    background: var(--native-surface);
    color: var(--native-text);
    font: 13px / 1.3 var(--type-native);
  }
  .quick.material {
    background: var(--native-surface-material);
  }
  .q-field {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    height: 40px;
    padding: 0 12px;
    border-bottom: 1px solid var(--native-line);
    color: var(--native-muted);
  }
  .q-field input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--native-text);
    font: 15px var(--type-native);
  }
  .q-field small,
  .q-age,
  .q-passage,
  footer,
  .q-empty,
  h2 {
    color: var(--native-muted);
  }
  .q-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 4px 5px;
  }
  h2 {
    margin: 3px 8px 2px;
    font-size: 11px;
    font-weight: 600;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .q-row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 5px 8px;
    border-radius: var(--native-radius-row);
    color: var(--native-text);
    text-decoration: none;
  }
  .q-row.q-sel {
    background: var(--native-selected-fallback-bg);
    color: var(--native-selected-fallback-text);
  }
  @supports (color: AccentColor) {
    .q-row.q-sel {
      background: var(--native-selected-bg);
      color: var(--native-selected-text);
    }
  }
  .q-row.q-sel .q-passage,
  .q-row.q-sel .q-age {
    color: inherit;
    opacity: 0.85;
  }
  .qmark {
    flex: none;
    width: 16px;
    height: 16px;
  }
  .qmark :global(svg) {
    display: block;
    width: 100%;
    height: 100%;
  }
  .q-t {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .q-title,
  .q-passage {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-title {
    font-weight: 500;
  }
  .q-passage {
    font-size: 12px;
  }
  .q-age {
    flex: none;
    font-size: 12px;
  }
  .q-empty,
  .q-note {
    margin: 0;
    padding: 24px 16px;
    text-align: center;
  }
  .q-note {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  footer {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: none;
    height: 28px;
    padding: 0 6px 0 12px;
    border-top: 1px solid var(--native-line);
    font-size: 11px;
  }
  footer a,
  .q-note a,
  .q-empty a {
    color: inherit;
  }
  .q-note a,
  .q-empty a {
    text-decoration: underline;
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
    cursor: pointer;
  }
  .q-end {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .q-more {
    display: grid;
    place-items: center;
    width: 26px;
    height: 22px;
    border: 0;
    border-radius: var(--native-radius-row);
    background: none;
    color: var(--native-text);
  }
  .q-more:hover,
  .q-more[aria-expanded="true"] {
    background: var(--native-hover);
  }
  .q-menu {
    position: absolute;
    right: 6px;
    bottom: 25px;
    display: flex;
    flex-direction: column;
    min-width: 180px;
    padding: 5px;
    border: 1px solid var(--native-line);
    border-radius: var(--native-radius-row);
    background: var(--native-surface);
  }
  .q-menu a {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 4px 9px;
    border-radius: var(--native-radius-row);
    color: var(--native-text);
    font-size: 13px;
    text-decoration: none;
  }
  .q-status {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    white-space: nowrap;
  }
  .q-dot {
    flex: none;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--native-muted);
  }
  .q-dot[data-tone="ready"] {
    background: var(--native-status-ready);
  }
  .q-dot[data-tone="busy"] {
    background: var(--native-status-busy);
  }
  .q-dot[data-tone="problem"] {
    background: var(--native-status-problem);
  }
  .q-menu kbd {
    color: var(--native-muted);
    font: inherit;
  }
  .q-menu a:hover kbd {
    color: inherit;
  }
  .q-menu a:hover {
    background: var(--native-hover);
  }
</style>
