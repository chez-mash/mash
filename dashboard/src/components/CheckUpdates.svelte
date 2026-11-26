<script lang="ts">
import ArrowsClockwiseIcon from "phosphor-svelte/lib/ArrowsClockwiseIcon";
import Icon from "$lib/components/Icon.svelte";
import { Tip } from "$lib/components/ui/tooltip";

// The dashboard's one way to read what is on screen again, shown at the foot
// of the rail (see lib/updates.svelte.ts). Pages do not add their own copy;
// actions that do something else, such as retrying a failed load, keep their
// own words.
let {
  checking,
  disabled,
  check,
  compact = false,
}: {
  /** A check is in progress; the button says so and waits. */
  checking: boolean;
  /** Unavailable for another reason, for instance before the first answer. */
  disabled?: boolean;
  check: () => unknown;
  /** Only the icon, named for assistive technology and in a tooltip. */
  compact?: boolean;
} = $props();
const words = $derived(checking ? "Checking…" : "Check for updates");
</script>

{#if compact}
  <Tip text={words}>{#snippet children(tip)}<button {...tip} class="btn ghost sm icon" type="button" aria-label={words} disabled={disabled ?? checking} onclick={() => check()}><Icon icon={ArrowsClockwiseIcon} size={14} /></button>{/snippet}</Tip>
{:else}
  <button class="btn ghost sm" type="button" disabled={disabled ?? checking} onclick={() => check()}><Icon icon={ArrowsClockwiseIcon} size={14} />{words}</button>
{/if}
