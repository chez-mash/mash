<script lang="ts">
import { ToggleGroup as ToggleGroupPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";

// A segmented control: a muted track holding the choices, the chosen one
// raised. The arrow keys move between the choices.
let {
  ref = $bindable(null),
  value = $bindable(),
  class: className,
  ...restProps
}: ToggleGroupPrimitive.RootProps = $props();
</script>

<!-- bits-ui types the root as one of two shapes, by `type`; the props pass
     through as given. -->
<ToggleGroupPrimitive.Root
  bind:ref
  bind:value={value as never}
  data-slot="toggle-group"
  class={cn("inline-flex flex-wrap gap-0.5 rounded-(--radius-5) bg-muted p-[3px]", className)}
  {...restProps}
/>
