<script lang="ts">
// A state's icon in its tone's colour (see lib/tone.ts). It is decoration: the
// state's word sits beside it, or `label` names it for screen readers and the
// tooltip when the icon stands alone.
import { type Tone, toneIcons } from "../lib/tone";

let {
  tone,
  size = 14,
  label,
}: { tone: Tone; size?: number; label?: string | undefined } = $props();
const Icon = $derived(toneIcons[tone]);
</script>

{#if label}
  <span class="tone-i tone-{tone}" title={label}><Icon {size} aria-hidden="true" /><span class="sr-only">{label}</span></span>
{:else}
  <span class="tone-i tone-{tone}" aria-hidden="true"><Icon {size} /></span>
{/if}
