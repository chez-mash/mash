<script lang="ts">
import BrainIcon from "phosphor-svelte/lib/BrainIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import type { Snippet } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { projectLabel, whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { plainPassage } from "../lib/passage";
import {
  ariaShortcut,
  type QuickCommand,
  type QuickKeys,
  type QuickRow,
  rowTarget,
} from "../lib/quick";
import { recency } from "../lib/tone";
import AgentMark from "./AgentMark.svelte";
import Highlight from "./Highlight.svelte";
import Keycaps from "./Keycaps.svelte";

// The quick panel's two lists: chats (recent ones, or matches once the reader
// types), then suggestions of Mash's own pages. One selection runs through
// both, chats first; the panel owns it and the keys that move it.
let {
  rows,
  commands,
  chats,
  heading,
  label,
  selected,
  query,
  desktop,
  keys,
  now,
  onhover,
  notes,
}: {
  rows: readonly QuickRow[];
  commands: readonly QuickCommand[];
  /** Whether the chats are listed; a note stands in for them when Mash can't be read. */
  chats: boolean;
  heading: string;
  label: string;
  selected: number;
  query: string;
  desktop: boolean;
  keys: QuickKeys;
  now: number;
  onhover: (index: number) => void;
  /** Said under the chats: why there are none, or that results are still coming. */
  notes: Snippet;
} = $props();
const offset = $derived(chats ? rows.length : 0);
</script>

{#snippet commandIcon(command: QuickCommand)}
  {#if command.id === "memory"}<Icon icon={BrainIcon} size={12} />{:else if command.id === "machines"}<Icon icon={LaptopIcon} size={12} />{:else}<Icon icon={MagnifyingGlassIcon} size={12} />{/if}
{/snippet}

{#if chats}
  <h2>{heading}</h2>
  <ul role="listbox" aria-label={label}>
    {#each rows as row, index (row.chat.machine + row.chat.id)}
      <li role="option" aria-selected={index === selected}>
        <a class="q-row" class:q-sel={index === selected} href={rowTarget(row, desktop)} onmouseenter={() => onhover(index)}>
          <AgentMark agent={row.chat.agent} size="sm" />
          <span class="q-t">
            <span class="q-line"><span class="q-title">{plainTitle(row.chat.title)}</span>{#if row.chat.project}<span class="q-sub" title="Project: {row.chat.project}"><Icon icon={FolderIcon} size={11} /><span>{projectLabel(row.chat.project)}</span></span>{/if}</span>
            {#if row.passage}<span class="q-passage"><Highlight text={plainPassage(row.passage)} {query} /></span>{/if}
          </span>
          {#if row.byMeaning}<span class="q-age">by meaning</span>{/if}
          <span class="q-age {recency(row.chat.updatedMs, now)}">{whenLabel(row.chat.updatedMs, now)}</span>
        </a>
      </li>
    {/each}
  </ul>
{/if}
{@render notes()}
<h2>Suggestions</h2>
<ul role="listbox" aria-label="Suggestions">
  {#each commands as command, at (command.id)}
    {@const index = offset + at}
    <li role="option" aria-selected={index === selected}>
      <a class="q-row" class:q-sel={index === selected} href={command.target} aria-keyshortcuts={command.shortcut ? ariaShortcut(command.shortcut, keys.modifier) : undefined} onmouseenter={() => onhover(index)}>
        <span class="q-tile" aria-hidden="true">{@render commandIcon(command)}</span>
        <span class="q-t"><span class="q-title">{command.title}</span></span>
        {#if command.shortcut}<Keycaps shortcut={command.shortcut} />{:else}<span class="q-age q-kind">Command</span>{/if}
      </a>
    </li>
  {/each}
</ul>

<style>
  h2 {
    margin: 0;
    padding: 10px 10px 6px;
    color: var(--text-faint);
    font-size: 12px;
    font-weight: var(--weight-strong);
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
    min-height: 40px;
    padding: 4px 10px;
    box-sizing: border-box;
    border-radius: var(--radius-5);
    color: var(--text-strong);
    text-decoration: none;
  }
  :global(.quick.wide) .q-row {
    min-height: 44px;
  }
  .q-row.q-sel {
    background: var(--selected-soft-bg);
  }
  .q-t {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .q-line {
    display: flex;
    align-items: baseline;
    gap: 7px;
    min-width: 0;
    white-space: nowrap;
  }
  .q-title {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: var(--weight-label);
  }
  .q-sub,
  .q-age {
    flex: none;
    color: var(--text-faint);
    font-size: 12px;
  }
  .q-sub {
    flex: 0 1 auto;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .q-sub > :global(svg) {
    flex: none;
  }
  .q-sub > span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-passage {
    overflow: hidden;
    color: var(--text-muted);
    font-size: 12px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Wide, the preview shows the passage, so each row keeps to one line; narrow,
     the keycaps say enough that a suggestion is a command. */
  :global(.quick.wide) .q-passage,
  :global(.quick:not(.wide)) .q-kind {
    display: none;
  }
  .q-tile {
    display: grid;
    flex: none;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-tile-sm);
    background: var(--memory-tile);
    box-shadow: var(--agent-tile-shadow);
    color: var(--memory-tile-text);
  }
</style>
