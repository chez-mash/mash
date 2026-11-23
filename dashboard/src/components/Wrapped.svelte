<script lang="ts">
import { wrapPoints } from "../lib/wrap";

// A path or address that may wrap only between its parts, never inside a name.
let { value }: { value: string } = $props();
</script>

{#each wrapPoints(value) as piece, index (index)}{#if index}<wbr />{/if}{piece}{/each}
