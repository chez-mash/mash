<script lang="ts">
import { ToggleGroup as ToggleGroupPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: ToggleGroupPrimitive.ItemProps = $props();
</script>

<ToggleGroupPrimitive.Item
  bind:ref
  data-slot="toggle-group-item"
  class={cn(
    "min-h-8 rounded-(--radius-4) px-3 text-[13px] [font-weight:var(--weight-label)] text-muted hover:text-strong data-[state=on]:bg-(--selected-bg) data-[state=on]:text-(--selected-text) data-[state=on]:shadow-(--shadow-pick)",
    className,
  )}
  {...restProps}
/>
