<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { serviceAccessOptions } from "../lib/services.svelte";
import { settingsOptions } from "../lib/settings";

// The way to connect another agent, beside an agent setup list. Setup writes
// the configuration of the machine running this dashboard, so the link shows
// only for that machine, and only when its settings can be changed here.
let { machineId }: { machineId: string } = $props();
const access = createQuery(() => serviceAccessOptions);
const settings = createQuery(() => settingsOptions);
const local = $derived(access.data?.localNode === machineId);
</script>

{#if local && settings.data?.editable}<a class="a" href="#/setup" style:font-size="12.5px">Connect an agent</a>{/if}
