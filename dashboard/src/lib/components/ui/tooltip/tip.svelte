<script lang="ts">
import { Tooltip as TooltipPrimitive } from "bits-ui";
import type { Snippet } from "svelte";
import TooltipContent from "./tooltip-content.svelte";

// A hint for one control or icon: the caller draws the element and spreads
// the given attributes on it, so the element keeps its own tag, classes and
// styles. The hint shows on hover and on keyboard focus. It adds no
// accessible name: the element keeps its own label or text.
let {
  text,
  side = "top",
  passive = false,
  children,
}: {
  text: string;
  side?: "top" | "right" | "bottom" | "left";
  /** The element is not a control, so it must not become a tab stop. */
  passive?: boolean;
  children: Snippet<[Record<string, unknown>]>;
} = $props();
</script>

<TooltipPrimitive.Provider delayDuration={400}>
  <TooltipPrimitive.Root>
    <TooltipPrimitive.Trigger>
      {#snippet child({ props })}
        {@render children(passive ? { ...props, tabindex: undefined } : props)}
      {/snippet}
    </TooltipPrimitive.Trigger>
    <TooltipContent {side}>{text}</TooltipContent>
  </TooltipPrimitive.Root>
</TooltipPrimitive.Provider>
