<script lang="ts">
// A machine's icon: the mark of the system it runs when that is known, the
// plain laptop otherwise. The system's name is in the tooltip. Reading the
// system never starts a timer (see lib/machineOs), so the logo can go
// wherever a machine is named.
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import Icon from "$lib/components/Icon.svelte";
import { Tip } from "$lib/components/ui/tooltip";
import { createMachineOs } from "../lib/machineOs.svelte";
import { osLogo, osName } from "../lib/os";

let { machine, size = 20 }: { machine: string; size?: number } = $props();
const systems = createMachineOs();
const os = $derived(systems.of(machine));
</script>

{#if os}
  <Tip text={osName(os)} passive>{#snippet children(tip)}<span {...tip} class="os" style:width="{size}px" style:height="{size}px">{@html osLogo(os)}</span>{/snippet}</Tip>
{:else}
  <Icon icon={LaptopIcon} {size} />
{/if}

<style>
  .os {
    display: inline-grid;
    place-items: center;
  }
  .os :global(svg) {
    width: 100%;
    height: 100%;
  }
</style>
