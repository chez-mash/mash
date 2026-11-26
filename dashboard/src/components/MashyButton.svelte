<script lang="ts">
// Mashy's circle, floating in the screen's bottom right corner like a help
// widget, so it is hard to overlook. Mashy.svelte draws the window it opens.
// The stylesheet keeps the circle off what the page shows: pages end with
// room to scroll clear of it, and on a phone it tucks to the edge while the
// reader scrolls down.
import { asset } from "../lib/asset";
import { mashy } from "../lib/mashy.svelte";
import { watchScrolling } from "../lib/tuck";

interface Props {
  /** Open Mashy, or close it. */
  readonly ontoggle: () => void;
}
const { ontoggle }: Props = $props();

$effect(() => watchScrolling());
</script>

<button id="mashy-fab" type="button" aria-label={mashy.open ? "Close Mashy" : "Open Mashy"} aria-expanded={mashy.open} onclick={ontoggle}>
  <img src={asset("mashy.png")} alt="" />
  {#if mashy.unread}<span class="dot"></span><span class="sr-only">New answer</span>{/if}
</button>
