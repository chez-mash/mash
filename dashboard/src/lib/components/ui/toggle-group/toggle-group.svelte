<script lang="ts">
import { ToggleGroup as ToggleGroupPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { slideIndicator } from "./slide";

// A segmented control: a muted track holding the choices, the chosen one
// raised. The arrow keys move between the choices. The raised highlight is
// one element behind the choices that slides to the chosen one (slide.ts).
let {
  ref = $bindable(null),
  value = $bindable(),
  class: className,
  children,
  ...restProps
}: ToggleGroupPrimitive.RootProps = $props();

let indicator = $state<HTMLElement | null>(null);

$effect(() => {
  if (ref && indicator)
    return slideIndicator(ref, indicator, {
      chosen: ":scope > [data-state=on]",
      watch: ["data-state", "disabled"],
    });
});
</script>

<!-- bits-ui types the root as one of two shapes, by `type`; the props pass
     through as given. -->
<ToggleGroupPrimitive.Root
  bind:ref
  bind:value={value as never}
  data-slot="toggle-group"
  class={cn("group/seg relative inline-flex flex-wrap gap-0.5 rounded-(--radius-5) bg-muted p-[3px]", className)}
  {...restProps}
>
  <span
    bind:this={indicator}
    aria-hidden="true"
    data-slot="toggle-group-indicator"
    class="pointer-events-none absolute top-0 left-0 hidden rounded-(--radius-4) bg-(--selected-bg) shadow-(--shadow-pick) transition-[transform,width] duration-[200ms] ease-out group-data-[sliding]/seg:block motion-reduce:transition-none"
  ></span>
  {@render children?.()}
</ToggleGroupPrimitive.Root>
