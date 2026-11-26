<script lang="ts">
// How an export ended: a tinted card that leads with its tone's icon. Green
// names what was written and where; red says why nothing was.
import type { Snippet } from "svelte";
import ToneIcon from "./ToneIcon.svelte";

let { outcome, children }: { outcome: "done" | "failed"; children: Snippet } = $props();
</script>

{#if outcome === "failed"}
  <p class="card tight bad xo" role="alert"><ToneIcon tone="bad" size={16} /><span><strong>Export failed.</strong> {@render children()}</span></p>
{:else}
  <p class="card tight ok xo" role="status"><ToneIcon tone="ok" size={16} /><span>{@render children()}</span></p>
{/if}

<style>
  .xo {
    margin: 0;
    font-size: 13px;
    overflow-wrap: anywhere;
  }
</style>
