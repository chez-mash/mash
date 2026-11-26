<script lang="ts">
import Check from "@lucide/svelte/icons/check";
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import type { Snippet } from "svelte";

// A filter chip that opens a menu of choices. `draft` marks a choice that
// has been changed on the page but not yet applied to the results.
let {
  label,
  icon,
  active = false,
  draft = false,
  multiple = false,
  options,
  selected,
  choose,
  optionIcon,
}: {
  label: string;
  icon: Snippet;
  active?: boolean;
  draft?: boolean;
  multiple?: boolean;
  options: readonly { value: string; label: string; count?: number }[];
  selected: readonly string[];
  choose: (value: string) => void;
  /** Each option's icon, given its value; options otherwise repeat the chip's icon. */
  optionIcon?: Snippet<[string]>;
} = $props();
let open = $state(false);
let root: HTMLDivElement;
let trigger: HTMLButtonElement;
</script>

<svelte:window
  onclick={(event) => {
    if (open && event.target instanceof Node && !root.contains(event.target)) open = false;
  }}
  onkeydown={(event) => {
    // Escape closes the menu and puts focus back on its chip, so the reader
    // carries on from where they were rather than from the top of the page.
    if (event.key !== "Escape" || !open) return;
    open = false;
    if (root.contains(document.activeElement) || document.activeElement === document.body)
      trigger.focus();
  }}
/>
<div class="fwrap" bind:this={root}>
  <button class="chip" class:on={active} class:draft type="button" bind:this={trigger} aria-haspopup="true" aria-expanded={open} onclick={() => (open = !open)}>
    {@render icon()}{label}<ChevronDown size={13} aria-hidden="true" />
  </button>
  {#if open}
    <div class="fmenu" role="group" aria-label={label}>
      {#each options as option (option.value)}
        {@const on = selected.includes(option.value)}
        <button
          type="button"
          role={multiple ? "menuitemcheckbox" : "menuitemradio"}
          aria-checked={on}
          onclick={() => {
            choose(option.value);
            if (!multiple) open = false;
          }}
        >
          <span style:flex="none" style:width="16px" style:display="inline-flex">{#if on}<Check size={14} aria-hidden="true" />{/if}</span>
          <span class="oi">{#if optionIcon}{@render optionIcon(option.value)}{:else}{@render icon()}{/if}</span>
          <span class="g">{option.label}</span>
          {#if option.count !== undefined}<span class="n">{option.count.toLocaleString()}</span>{/if}
        </button>
      {:else}
        <p class="meta" style:padding="8px 9px">Nothing to choose from yet.</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* A filter chip is a small raised control with a hairline ring; a chosen
     filter carries a butter ring and tint. Its menu is a floating panel of
     rows, the chosen ones on the neutral selected tint. */
  .fwrap > :global(.chip) {
    min-height: 30px;
    gap: 7px;
    padding: 0 9px 0 10px;
    border: 0;
    border-radius: var(--radius-5);
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-strong);
    color: var(--text-strong);
    font-size: 13px;
    font-weight: var(--weight-regular);
  }
  .fwrap > :global(.chip:hover) {
    background: var(--surface-hover);
  }
  .fwrap > :global(.chip.on),
  .fwrap > :global(.chip[aria-expanded="true"].on) {
    background: var(--chip-on-bg);
    box-shadow: 0 0 0 var(--stroke-outline) var(--chip-on-line);
    font-weight: var(--weight-label);
  }
  .fwrap > :global(.chip[aria-expanded="true"]) {
    font-weight: var(--weight-label);
  }
  .fwrap > :global(.chip > svg:last-child) {
    color: var(--text-muted);
  }
  /* While a menu is open the page behind it dims, its chip staying bright
     above the dimming. The dimming is the menu's own shadow, so a click
     anywhere still reaches the page and closes the menu. */
  .fmenu {
    top: calc(100% + 6px);
    padding: 5px;
    border: 0;
    border-radius: var(--radius-7);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-outline),
      var(--shadow-float),
      0 0 0 100vmax var(--scrim);
  }
  .fwrap:has(> .fmenu) {
    z-index: 16;
  }
  .fwrap:has(> .fmenu) > :global(.chip) {
    position: relative;
    z-index: 16;
  }
  .fmenu button {
    min-height: 32px;
    border-radius: var(--radius-5);
    font-size: 13px;
  }
  .fmenu button[aria-checked="true"] {
    background: var(--selected-surface);
    font-weight: var(--weight-label);
  }
  .fmenu .n {
    font-size: 12px;
  }
</style>
