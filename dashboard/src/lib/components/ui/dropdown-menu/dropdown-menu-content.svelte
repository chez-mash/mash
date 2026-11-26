<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { panel } from "./styles.js";

// A menu's panel, floated beside its trigger above the rest of the page.
let {
  ref = $bindable(null),
  sideOffset = 6,
  align = "start",
  preventScroll = false,
  class: className,
  onOpenAutoFocus,
  ...restProps
}: DropdownMenuPrimitive.ContentProps = $props();

// Opened with the pointer, the panel itself takes focus, so no row is tinted
// until the pointer or an arrow key reaches one; opened from the keyboard,
// bits-ui moves focus to the first row.
function focusPanel(event: Event): void {
  onOpenAutoFocus?.(event);
  if (event.defaultPrevented) return;
  event.preventDefault();
  ref?.focus();
}
</script>

<DropdownMenuPrimitive.Portal>
  <DropdownMenuPrimitive.Content
    bind:ref
    data-slot="dropdown-menu-content"
    {sideOffset}
    {align}
    {preventScroll}
  onOpenAutoFocus={focusPanel}
    class={cn(panel, "z-60 max-h-(--bits-dropdown-menu-content-available-height) min-w-[210px]", className)}
    {...restProps}
  />
</DropdownMenuPrimitive.Portal>
