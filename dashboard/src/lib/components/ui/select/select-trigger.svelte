<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import { Select as SelectPrimitive } from "bits-ui";
import { cn, type WithoutChild } from "$lib/utils.js";

// A select's button looks like the text fields beside it: the same outline
// and fill, a drawn chevron, and its placeholder as faint as a field's.
let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithoutChild<SelectPrimitive.TriggerProps> = $props();
</script>

<SelectPrimitive.Trigger
  bind:ref
  data-slot="select-trigger"
  class={cn(
    "inline-flex min-h-[38px] min-w-[140px] cursor-pointer items-center justify-between gap-2 rounded-(--radius-5) border-(length:--stroke-strong) border-solid border-input bg-(--surface-field) pr-2.5 pl-[11px] text-left font-normal text-strong data-placeholder:text-muted aria-invalid:border-destructive",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
  <ChevronDown class="flex-none text-muted" size={15} aria-hidden="true" />
</SelectPrimitive.Trigger>
