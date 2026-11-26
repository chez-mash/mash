<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

// The list of choices: a raised panel under the button, at least as wide.
let {
  ref = $bindable(null),
  class: className,
  sideOffset = 6,
  align = "start",
  preventScroll = false,
  children,
  ...restProps
}: SelectPrimitive.ContentProps = $props();
</script>

<SelectPrimitive.Portal>
  <SelectPrimitive.Content
    bind:ref
    data-slot="select-content"
    {sideOffset}
    {align}
    {preventScroll}
    class={cn(
      "z-60 max-h-(--bits-select-content-available-height) min-w-(--bits-select-anchor-width) overflow-y-auto rounded-(--radius-6) border-(length:--stroke-strong) border-solid border-(--line-strong) bg-popover p-1 text-popover-foreground shadow-(--shadow-float)",
      className,
    )}
    {...restProps}
  >
    <SelectPrimitive.Viewport>{@render children?.()}</SelectPrimitive.Viewport>
  </SelectPrimitive.Content>
</SelectPrimitive.Portal>
