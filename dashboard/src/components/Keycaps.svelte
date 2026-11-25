<script lang="ts">
import { keycaps } from "../lib/quick";

// A shortcut drawn as keycaps, one per key: ⌘ and K, or Ctrl and Enter.
let { shortcut }: { shortcut: string } = $props();
</script>

<span class="caps" aria-hidden="true">{#each keycaps(shortcut) as key, index (index)}<kbd class="kbd">{key}</kbd>{/each}</span>

<style>
  .caps {
    display: inline-flex;
    flex: none;
    gap: 3px;
  }
  .caps .kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    height: 18px;
    padding: 0 4px;
    line-height: 1;
  }
</style>
