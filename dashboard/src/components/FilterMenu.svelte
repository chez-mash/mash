<script lang="ts">
import Check from "@lucide/svelte/icons/check";
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import type { Snippet } from "svelte";
import * as DropdownMenu from "$lib/components/ui/dropdown-menu";

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
// A row is shorter than the kit's and keeps the neutral selected tint.
const row =
  "min-h-8 rounded-(--radius-5) px-[9px] text-[13px] data-[state=checked]:bg-(--selected-surface) data-[state=checked]:[font-weight:var(--weight-label)]";
</script>

{#snippet line(option: { value: string; label: string; count?: number }, on: boolean)}
  <span style:flex="none" style:width="16px" style:display="inline-flex">{#if on}<Check size={14} aria-hidden="true" />{/if}</span>
  <span class="oi">{#if optionIcon}{@render optionIcon(option.value)}{:else}{@render icon()}{/if}</span>
  <span class="g">{option.label}</span>
  {#if option.count !== undefined}<span class="n">{option.count.toLocaleString()}</span>{/if}
{/snippet}
<!-- Escape closes the menu and puts focus back on its chip, so the reader
     carries on from where they were rather than from the top of the page. -->
<div class="fwrap">
  <DropdownMenu.Root bind:open>
    <DropdownMenu.Trigger class={["chip", { on: active, draft }]}>
      {@render icon()}{label}<ChevronDown size={13} aria-hidden="true" />
    </DropdownMenu.Trigger>
    <DropdownMenu.ContentStatic class="fmenu" aria-label={label}>
      {#if multiple}
        {#each options as option (option.value)}
          {@const on = selected.includes(option.value)}
          <DropdownMenu.CheckboxItem class={row} closeOnSelect={false} bind:checked={() => on, () => choose(option.value)}>
            {@render line(option, on)}
          </DropdownMenu.CheckboxItem>
        {/each}
      {:else}
        <DropdownMenu.RadioGroup value={selected[0] ?? ""}>
          {#each options as option (option.value)}
            <DropdownMenu.RadioItem class={row} value={option.value} onSelect={() => choose(option.value)}>
              {@render line(option, selected.includes(option.value))}
            </DropdownMenu.RadioItem>
          {/each}
        </DropdownMenu.RadioGroup>
      {/if}
      {#if !options.length}<p class="meta" style:padding="8px 9px">Nothing to choose from yet.</p>{/if}
    </DropdownMenu.ContentStatic>
  </DropdownMenu.Root>
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
  .fwrap > :global(.chip.on) {
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
  .fwrap :global(.fmenu) {
    top: calc(100% + 6px);
    padding: 5px;
    border: 0;
    border-radius: var(--radius-7);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-outline),
      var(--shadow-float),
      0 0 0 100vmax var(--scrim);
  }
  .fwrap:has(:global(.fmenu)) {
    z-index: 16;
  }
  .fwrap:has(:global(.fmenu)) > :global(.chip) {
    position: relative;
    z-index: 16;
  }
  .fwrap :global(.fmenu .n) {
    font-size: 12px;
  }
</style>
