<script lang="ts">
import { Tooltip as TooltipPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

// A tooltip's label: a small dark tag (light in the dark theme) beside what
// it names, above everything else on the page.
let {
  ref = $bindable(null),
  class: className,
  sideOffset = 6,
  side = "top",
  ...restProps
}: TooltipPrimitive.ContentProps = $props();
</script>

<TooltipPrimitive.Portal>
  <TooltipPrimitive.Content
    bind:ref
    data-slot="tooltip-content"
    {sideOffset}
    {side}
    class={cn(
      "z-80 flex w-fit max-w-[280px] items-center gap-1.5 rounded-(--radius-5) bg-primary px-2 py-1.5 text-[12px] leading-[1.3] whitespace-pre-line text-primary-foreground",
      className,
    )}
    {...restProps}
  />
</TooltipPrimitive.Portal>
