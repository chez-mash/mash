<script lang="ts">
import Ellipsis from "@lucide/svelte/icons/ellipsis";
import SearchIcon from "@lucide/svelte/icons/search";
import { createQuery } from "@tanstack/svelte-query";
import { onMount } from "svelte";
import { agents } from "../lib/agents";
import { liveSearch } from "../lib/api";
import { whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { catalogOptions } from "../lib/queries";
import {
  fullSearchTarget,
  hidePanel,
  inkLogo,
  moveSelection,
  pageTarget,
  type QuickRow,
  quickRows,
  quickStatus,
  rowTarget,
  searching,
} from "../lib/quick";
import { chatHref } from "../lib/route.svelte";
import { inDesktopApp, shareLink } from "../lib/shareLink";
import { type Status, statusOptions } from "../lib/status";

// The panel the desktop app drops down from its menu bar icon (and shows for
// its global shortcut): live search over this machine, or the latest chats
// before anything is typed. It looks like the operating system, not like the
// dashboard; its colours are the theme's native roles.
const desktop = inDesktopApp();
const catalogue = createQuery(() => catalogOptions);
let text = $state("");
let typed = $state("");
$effect(() => {
  const value = text.trim();
  const timer = setTimeout(() => (typed = value), 60);
  return () => clearTimeout(timer);
});
const live = createQuery(() => ({
  queryKey: ["live-search", typed],
  queryFn: ({ signal }) => liveSearch(typed, catalogue.data?.localNode ?? "", signal),
  enabled: searching(typed) && catalogue.data !== undefined,
  networkMode: "always" as const,
  staleTime: 30_000,
  retry: false,
  placeholderData: (previous: Awaited<ReturnType<typeof liveSearch>> | undefined) => previous,
}));
const rows = $derived(quickRows(text, catalogue.data?.chats ?? [], live.data ?? []));
const failed = $derived(searching(text) && live.isError ? live.error : null);
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
let selected = $state(0);
$effect(() => {
  void rows;
  selected = 0;
});
let menuOpen = $state(false);
let input: HTMLInputElement | undefined = $state();
const now = Date.now();

function focusField(): void {
  input?.focus();
  input?.select();
}
onMount(focusField);

function go(target: string): void {
  menuOpen = false;
  location.href = target;
}

function copyLink(row: QuickRow): void {
  const route = chatHref(row.chat.id, row.chat.machine, row.messageId);
  const link = desktop
    ? rowTarget(row, true)
    : shareLink(route, `${location.origin}${location.pathname}`, false);
  void navigator.clipboard?.writeText(link);
}

function keydown(event: KeyboardEvent): void {
  const row = rows[selected];
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    selected = moveSelection(selected, event.key === "ArrowDown" ? 1 : -1, rows.length);
  } else if (event.key === "Enter") {
    event.preventDefault();
    if (event.metaKey || event.ctrlKey) go(fullSearchTarget(text, desktop));
    else if (row) go(rowTarget(row, desktop));
  } else if ((event.key === "o" || event.key === ",") && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    go(pageTarget(event.key === "o" ? "#/" : "#/settings", desktop));
  } else if (event.key === "Escape") {
    event.preventDefault();
    if (menuOpen) menuOpen = false;
    else if (text) text = "";
    else hidePanel();
  } else if (
    event.key === "c" &&
    (event.metaKey || event.ctrlKey) &&
    row &&
    input?.selectionStart === input?.selectionEnd
  ) {
    event.preventDefault();
    copyLink(row);
  }
}
</script>

<!-- Shown again from the menu bar, the panel is ready to type into. -->
<svelte:window onfocus={focusField} />

<div class="quick" role="search">
  <label class="q-field">
    <SearchIcon size={17} aria-hidden="true" />
    <span class="sr-only">Search Mash</span>
    <input bind:this={input} bind:value={text} placeholder="Search Mash" autocomplete="off" spellcheck="false" onkeydown={keydown} />
    {#if searching(text) && live.data}<small>{live.data.length === 1 ? "1 chat" : `${live.data.length} chats`}</small>{/if}
  </label>

  <div class="q-list">
    {#if failed}
      <div class="q-note" role="alert">
        <strong>Mash couldn’t search.</strong>
        <span>{failed.message}</span>
        <a href={pageTarget("#/", desktop)}>Open Mash</a>
      </div>
    {:else}
      <h2>{searching(text) ? "Chats on this machine" : "Recent chats"}</h2>
      <ul role="listbox" aria-label={searching(text) ? "Matching chats" : "Recent chats"}>
        {#each rows as row, index (row.chat.machine + row.chat.id)}
          <li role="option" aria-selected={index === selected}>
            <a class="q-row" class:q-sel={index === selected} href={rowTarget(row, desktop)} onmouseenter={() => (selected = index)}>
              <span class="qmark" aria-hidden="true" style:color={index === selected || inkLogo(row.chat.agent) ? undefined : agents[row.chat.agent].tint}>{@html agents[row.chat.agent].logo}</span>
              <span class="q-t">
                <span class="q-title">{plainTitle(row.chat.title)}</span>
                {#if row.passage}<span class="q-passage">{row.passage}</span>{/if}
              </span>
              <span class="q-age">{whenLabel(row.chat.updatedMs, now)}</span>
            </a>
          </li>
        {/each}
      </ul>
      {#if searching(text) && live.data && rows.length === 0}
        <p class="q-empty">No chats on this machine mention “{text.trim()}”.</p>
      {:else if !searching(text) && catalogue.data && rows.length === 0}
        <p class="q-empty">No chats yet.</p>
      {/if}
    {/if}
  </div>

  <footer>
    <span class="q-status" role="status"><span class="q-dot" data-tone={health.tone} aria-hidden="true"></span>{health.label}</span>
    <span class="q-end">
      <a href={fullSearchTarget(text, desktop)}>Search Everywhere ⌘↵</a>
      <button type="button" class="q-more" aria-label="More" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}><Ellipsis size={16} aria-hidden="true" /></button>
    </span>
    {#if menuOpen}
      <div class="q-menu" role="menu">
        <a role="menuitem" href={pageTarget("#/", desktop)} aria-keyshortcuts="Meta+O">Open Mash<kbd>⌘O</kbd></a>
        <a role="menuitem" href={pageTarget("#/settings", desktop)} aria-keyshortcuts="Meta+,">Settings…<kbd>⌘,</kbd></a>
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
    font: 13px / 1.35 var(--type-native);
  }
  .q-field {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    height: 46px;
    padding: 0 14px;
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
    font: 17px var(--type-native);
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
    padding: 6px;
  }
  h2 {
    margin: 4px 10px;
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
    gap: 10px;
    padding: 7px 10px;
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
    font-weight: 600;
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
    height: 34px;
    padding: 0 8px 0 14px;
    border-top: 1px solid var(--native-line);
    font-size: 12px;
  }
  footer a,
  .q-note a {
    color: inherit;
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
    bottom: 30px;
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
