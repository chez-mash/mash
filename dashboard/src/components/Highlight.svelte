<script lang="ts">
import { highlightParts } from "../lib/highlight";

// Marks the words of a query inside archived text. The text is rendered as
// text, never as markup.
let { text, query }: { text: string; query: string } = $props();
const parts = $derived(highlightParts(text, query));
</script>

{#each parts as part, index (index)}{#if part.hit}<mark>{part.text}</mark>{:else}{part.text}{/if}{/each}
