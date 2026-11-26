<script lang="ts">
import Icon from "$lib/components/Icon.svelte";
import { Tip } from "$lib/components/ui/tooltip";
// A state's icon in its tone's colour (see lib/tone.ts). It is decoration: the
// state's word sits beside it, or `label` names it for screen readers and the
// tooltip when the icon stands alone.
import { type Tone, toneIcons } from "../lib/tone";

let {
  tone,
  size = 14,
  label,
}: { tone: Tone; size?: number; label?: string | undefined } = $props();
const Glyph = $derived(toneIcons[tone]);
// A warning and a failure are drawn bold at any size.
const state = $derived(tone === "warn" || tone === "bad" ? "strong" : "rest");
</script>

{#if label}
  <Tip text={label} passive>{#snippet children(tip)}<span {...tip} class="tone-i tone-{tone}"><Icon icon={Glyph} {size} {state} /><span class="sr-only">{label}</span></span>{/snippet}</Tip>
{:else}
  <span class="tone-i tone-{tone}" aria-hidden="true"><Icon icon={Glyph} {size} {state} /></span>
{/if}
