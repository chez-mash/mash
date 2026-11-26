<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import { cn } from "$lib/utils.js";
import { row } from "./styles.js";

let {
  ref = $bindable(null),
  class: className,
  ...restProps
}: DropdownMenuPrimitive.ItemProps = $props();
</script>

<DropdownMenuPrimitive.Item
  bind:ref
  data-slot="dropdown-menu-item"
  class={cn(row, className)}
  {...restProps}
/>
