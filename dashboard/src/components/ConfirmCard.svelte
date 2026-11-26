<script lang="ts">
// The question asked before a change is made somewhere: a raised card in an
// amber ring, holding what will happen, the button that does it and Cancel.
// The caller writes all three (a `span` for the words, then the buttons).
import type { Snippet } from "svelte";

let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="card tight confirm" role="group" aria-label={label}>{@render children()}</div>

<style>
  .confirm {
    border-color: transparent;
    border-radius: var(--radius-8);
    box-shadow:
      0 0 0 1px var(--memory-ring),
      var(--memory-glow);
  }
</style>
