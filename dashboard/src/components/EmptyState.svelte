<script lang="ts">
import type { Snippet } from "svelte";
import { asset } from "../lib/asset";

// Nothing to show, or nowhere to go: Mashy, a heading, one plain sentence and
// the one thing to do next. `card` draws it in an outlined card.
let {
  title,
  heading = "h2",
  card = false,
  children,
  action,
}: {
  title?: string;
  /** The heading's level: "h1" when the state is the whole page. */
  heading?: "h1" | "h2";
  card?: boolean;
  children: Snippet;
  action?: Snippet;
} = $props();
</script>

<div class="empty" class:card>
  <img src={asset("mashy.png")} alt="" />
  {#if title}<svelte:element this={heading} class="empty-h">{title}</svelte:element>{/if}
  <p>{@render children()}</p>
  {#if action}<div class="acts" style:justify-content="center">{@render action()}</div>{/if}
</div>
