<script lang="ts">
// Mashy's circle, at the right of the top bar. It lives in the bar rather
// than floating over the page, so it never covers what the page shows; the
// window it opens is drawn by Mashy.svelte, which listens for MASHY_TOGGLE.
import { asset } from "../lib/asset";
import { MASHY_TOGGLE, mashy } from "../lib/mashy.svelte";
</script>

<button id="mashy-fab" type="button" aria-label={mashy.open ? "Close Mashy" : "Open Mashy"} aria-expanded={mashy.open} onclick={() => dispatchEvent(new Event(MASHY_TOGGLE))}>
  <img src={asset("mashy.png")} alt="" />
  {#if mashy.unread}<span class="dot"></span><span class="sr-only">New answer</span>{/if}
</button>
