<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui";
import type { Snippet } from "svelte";
import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
import DialogOverlay from "./dialog-overlay.svelte";

// A dialog's panel: a raised surface with a hairline ring and the floating
// shadow, centred over the dimmed page. Its size and inner layout are the
// caller's, given as classes.
let {
  ref = $bindable(null),
  class: className,
  children,
  onOpenAutoFocus,
  ...restProps
}: WithoutChildrenOrChild<DialogPrimitive.ContentProps> & { children: Snippet } = $props();

// The panel itself takes focus when the dialog opens, not its first control,
// so no control's hint shows before the reader reaches for one.
function focusPanel(event: Event): void {
  onOpenAutoFocus?.(event);
  if (event.defaultPrevented) return;
  event.preventDefault();
  ref?.focus();
}
</script>

<DialogPrimitive.Portal>
  <DialogOverlay />
  <DialogPrimitive.Content
    bind:ref
    data-slot="dialog-content"
    onOpenAutoFocus={focusPanel}
    class={cn(
      "fixed inset-0 z-70 m-auto overflow-hidden rounded-(--radius-9) bg-(--surface-raised) text-(--text-body) shadow-[0_0_0_var(--stroke-outline)_var(--line-outline),var(--shadow-float)] outline-none",
      className,
    )}
    {...restProps}
  >
    {@render children()}
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
