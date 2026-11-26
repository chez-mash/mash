<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { chosen, row } from "./styles.js";

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: DropdownMenuPrimitive.RadioItemProps = $props();
</script>

<DropdownMenuPrimitive.RadioItem
  bind:ref
  data-slot="dropdown-menu-radio-item"
  class={cn(row, chosen, className)}
  {...restProps}
/>
