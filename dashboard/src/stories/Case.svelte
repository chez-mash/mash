<script lang="ts">
// One state of a component, under the name of the state.
import type { Snippet } from "svelte";

let { label, note, children }: { label: string; note?: string; children: Snippet } = $props();
</script>

<figure class="case">
  <figcaption><b>{label}</b>{#if note}<span>{note}</span>{/if}</figcaption>
  <div class="case-b">{@render children()}</div>
</figure>

<style>
  .case {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    margin: 0;
  }
  figcaption {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 8px;
    font-family: var(--type-mono);
    font-size: 11.5px;
    line-height: 1.4;
    color: var(--text-faint);
  }
  figcaption b {
    font-weight: var(--weight-label);
    color: var(--text-muted);
  }
  .case-b {
    min-width: 0;
  }
</style>
