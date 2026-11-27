<script lang="ts">
import { Select as SelectPrimitive } from "bits-ui";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import Icon from "$lib/components/Icon.svelte";
import { cn, type WithoutChild } from "$lib/utils.js";

// One choice: its label, with a tick when it is the chosen one.
let {
  ref = $bindable(null),
  class: className,
  children: childrenProp,
  ...restProps
}: WithoutChild<SelectPrimitive.ItemProps> = $props();
</script>

<SelectPrimitive.Item
  bind:ref
  data-slot="select-item"
  class={cn(
    "flex min-h-9 w-full cursor-default select-none items-center justify-between gap-3 rounded-(--radius-4) px-2.5 text-[13.5px] -outline-offset-2 data-highlighted:bg-accent data-selected:[font-weight:var(--weight-label)] data-disabled:cursor-default data-disabled:opacity-50",
    className,
  )}
  {...restProps}
>
  {#snippet children({ selected, highlighted })}
    {#if childrenProp}{@render childrenProp({ selected, highlighted })}{:else}{restProps.label || restProps.value}{/if}
    <span class="inline-flex w-3.5 flex-none">{#if selected}<Icon icon={CheckIcon} size={14} />{/if}</span>
  {/snippet}
</SelectPrimitive.Item>
