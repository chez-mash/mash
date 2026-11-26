<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

// The dimming behind a dialog: the theme's scrim over the whole page.
let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: DialogPrimitive.OverlayProps = $props();
</script>

<DialogPrimitive.Overlay
  bind:ref
  data-slot="dialog-overlay"
  class={cn("fixed inset-0 z-70 bg-(--scrim)", className)}
  {...restProps}
/>
