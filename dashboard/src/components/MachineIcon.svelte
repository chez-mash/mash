<script lang="ts">
// A machine's icon: the mark of the system it runs when that is known, the
// plain laptop otherwise. The system's name is in the tooltip.
import Laptop from "@lucide/svelte/icons/laptop";
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machineOs, osLogo, osName } from "../lib/os";

let { machine, size = 20 }: { machine: string; size?: number } = $props();
const evidence = createMachineEvidence();
const os = $derived(machineOs(machine, evidence.fleet, evidence.report));
</script>

{#if os}
  <span class="os" style:width="{size}px" style:height="{size}px" title={osName(os)}>{@html osLogo(os)}</span>
{:else}
  <Laptop {size} aria-hidden="true" />
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
