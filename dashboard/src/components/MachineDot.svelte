<script lang="ts">
// A machine's status dot, as `machinePulse` decides it. Its tooltip
// says what the colour stands for and how it was learned.
import { createQuery } from "@tanstack/svelte-query";
import { machinePulse } from "../lib/machinePulse";
import { serverReport, serverViewOptions } from "../lib/serverStatus";
import { serviceAccessOptions, serviceOptions } from "../lib/services.svelte";

let { machine }: { machine: string } = $props();
/** The checks repeat while a dot is on screen, so its colour stays current. */
const EVERY_MINUTE = 60_000;
const access = createQuery(() => serviceAccessOptions);
const fleet = createQuery(() => ({
  ...serviceOptions(access.data !== undefined),
  refetchInterval: EVERY_MINUTE,
}));
const server = createQuery(() => ({ ...serverViewOptions, refetchInterval: EVERY_MINUTE }));
const pulse = $derived(
  machinePulse(machine, {
    fleet: fleet.data,
    report: serverReport(server.data),
    now: fleet.dataUpdatedAt || Date.now(),
  }),
);
</script>

<i class="dot {pulse.tone}" aria-hidden="true" title={pulse.words}></i>
