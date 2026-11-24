<script lang="ts">
// A machine's status dot, as `machinePulse` decides it. Its tooltip says what
// the colour stands for and how it was learned.
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machinePulse } from "../lib/machinePulse";

let { machine }: { machine: string } = $props();
const evidence = createMachineEvidence();
const pulse = $derived(machinePulse(machine, evidence));
</script>

<i class="dot {pulse.tone}" aria-hidden="true" title={pulse.words}></i>
