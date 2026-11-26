<script lang="ts">
import { Switch as SwitchPrimitive } from "bits-ui";
import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

// A switch: a quiet grey track and knob when off; butter with a light knob
// when on. The button is 44px tall so a finger finds it; the track inside is
// what the reader sees.
let {
  ref = $bindable(null),
  class: className,
  checked = $bindable(false),
  ...restProps
}: WithoutChildrenOrChild<SwitchPrimitive.RootProps> = $props();
</script>

<SwitchPrimitive.Root
  bind:ref
  bind:checked
  data-slot="switch"
  class={cn("group/switch inline-flex min-h-11 shrink-0 items-center", className)}
  {...restProps}
>
  <span
    class="relative block h-[22px] w-10 rounded-(--radius-pill) border-(length:--stroke-strong) border-solid border-(--line-ink) bg-(--surface-muted) transition-[background] duration-[120ms] group-data-[state=checked]/switch:bg-(--toggle-on) group-disabled/switch:opacity-55"
  >
    <SwitchPrimitive.Thumb
      data-slot="switch-thumb"
      class="absolute top-0.5 left-0.5 block size-3.5 rounded-(--radius-round) bg-(--text-faint) shadow-(--shadow-knob) transition-[left] duration-[120ms] data-[state=checked]:left-5 data-[state=checked]:bg-(--toggle-knob)"
    />
  </span>
</SwitchPrimitive.Root>
