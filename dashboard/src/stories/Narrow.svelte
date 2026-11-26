<script lang="ts">
// A column as wide as one tile of a card grid, for a card that fills its cell.
import type { Snippet } from "svelte";

let { children, width = 300 }: { children: Snippet; width?: number } = $props();
</script>

<div style:max-width="{width}px">{@render children()}</div>
