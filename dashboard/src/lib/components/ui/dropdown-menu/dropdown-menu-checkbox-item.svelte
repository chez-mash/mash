<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { chosen, row } from "./styles.js";

let {
  ref = $bindable(null),
  checked = $bindable(false),
  class: className,
  ...restProps
}: DropdownMenuPrimitive.CheckboxItemProps = $props();
</script>

<DropdownMenuPrimitive.CheckboxItem
  bind:ref
  bind:checked
  data-slot="dropdown-menu-checkbox-item"
  class={cn(row, chosen, className)}
  {...restProps}
/>
