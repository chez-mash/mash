<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { panel } from "./styles.js";

// A menu's panel kept where it is written, under its trigger's own wrapper,
// for a menu the page's layout places: as wide as a row of filters on a
// phone, or dimming the page around a trigger that stays bright above it.
let {
  ref = $bindable(null),
  preventScroll = false,
  class: className,
  onOpenAutoFocus,
  ...restProps
}: DropdownMenuPrimitive.ContentStaticProps = $props();

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

<DropdownMenuPrimitive.ContentStatic
  bind:ref
  data-slot="dropdown-menu-content"
  {preventScroll}
  onOpenAutoFocus={focusPanel}
  class={cn(panel, "absolute top-[calc(100%+6px)]", className)}
  {...restProps}
/>
