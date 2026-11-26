<script lang="ts">
// A machine's status mark, as `machinePulse` decides it: a coloured icon whose
// shape says the state as well as its colour. Its tooltip says what it stands
// for and how it was learned.
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machinePulse } from "../lib/machinePulse";
import { pulseTone } from "../lib/tone";
import ToneIcon from "./ToneIcon.svelte";

let { machine }: { machine: string } = $props();
const evidence = createMachineEvidence();
const pulse = $derived(machinePulse(machine, evidence));
</script>

<span title={pulse.words}><ToneIcon tone={pulseTone(pulse)} size={13} /></span>
