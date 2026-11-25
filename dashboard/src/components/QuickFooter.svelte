<script lang="ts">
import { asset } from "../lib/asset";
import { ariaShortcut, type QuickKeys, type QuickStatus } from "../lib/quick";
import Keycaps from "./Keycaps.svelte";

// The quick panel's footer: Mash's mark and what search can do right now (or
// what was just copied), then opening the selection and the actions menu.
// The dot shows only when something is not as it should be.
let {
  label,
  tone,
  open,
  menuOpen,
  keys,
  onopen,
  ontoggle,
  actionsButton = $bindable(),
}: {
  label: string;
  tone: QuickStatus["tone"];
  /** What Enter does to the selection, or nothing when nothing is selected. */
  open: string;
  menuOpen: boolean;
  keys: QuickKeys;
  onopen: () => void;
  ontoggle: () => void;
  actionsButton?: HTMLButtonElement | undefined;
} = $props();
</script>

<footer>
  <img class="q-mark" src={asset("mash-mark.svg")} alt="" width="20" height="20" />
  <span class="q-status" role="status">{#if tone !== "ready"}<span class="q-dot" data-tone={tone} aria-hidden="true"></span>{/if}<span class="q-label">{label}</span></span>
  <span class="q-end">
    {#if open}
      <button type="button" class="q-ghost q-open" onclick={onopen}>{open}<Keycaps shortcut="↵" /></button>
      <span class="q-sep" aria-hidden="true"></span>
    {/if}
    <button bind:this={actionsButton} type="button" class="q-ghost" aria-label="Actions" aria-haspopup="menu" aria-expanded={menuOpen} aria-keyshortcuts={ariaShortcut(keys.actions, keys.modifier)} onclick={ontoggle}>Actions<Keycaps shortcut={keys.actions} /></button>
  </span>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    height: 44px;
    padding: 0 8px 0 14px;
    border-top: var(--border-divider);
    background: var(--surface-top);
    color: var(--text-muted);
    font-size: 12px;
  }
  :global(.quick.material) footer {
    background: none;
  }
  .q-mark {
    flex: none;
    width: 20px;
    height: 20px;
  }
  .q-status {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .q-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .q-dot {
    flex: none;
    width: 7px;
    height: 7px;
    border-radius: var(--radius-round);
    background: var(--text-faint);
  }
  .q-dot[data-tone="busy"] {
    background: var(--status-warn-dot);
  }
  .q-dot[data-tone="problem"] {
    background: var(--status-bad-dot);
  }
  .q-end {
    display: flex;
    flex: none;
    align-items: center;
    gap: 4px;
  }
  .q-ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-5);
    background: none;
    color: var(--text-strong);
    font: 500 12px / 1 var(--type-sans);
    cursor: pointer;
  }
  .q-ghost:hover,
  .q-ghost[aria-expanded="true"] {
    background: var(--selected-soft-bg);
  }
  .q-sep {
    width: var(--stroke-divider);
    height: 14px;
    margin: 0 4px;
    background: var(--line-strong);
  }
  /* The desktop panel is narrow: Enter is enough said for opening. */
  @media (max-width: 420px) {
    .q-open,
    .q-sep {
      display: none;
    }
  }
</style>
