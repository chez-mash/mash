<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import Icon from "$lib/components/Icon.svelte";
import {
  problems,
  serviceAccessOptions,
  serviceOptions,
  servicesHref,
} from "../lib/services.svelte";
import AgentSetupList from "./AgentSetupList.svelte";
import ServiceRows from "./ServiceRows.svelte";
import SetupLink from "./SetupLink.svelte";

// What a machine's own dashboard reports: its agents' setup and its services.
let { machineId }: { machineId: string } = $props();
const access = createQuery(() => serviceAccessOptions);
const enabled = $derived(access.data !== undefined);
const fleet = createQuery(() => serviceOptions(enabled));
const machine = $derived(fleet.data?.find((machine) => machine.id === machineId));
const issues = $derived(problems(machine ? [machine] : []));
</script>

{#if machine?.reachable}
  <AgentSetupList {machineId} {enabled}>
    {#snippet action()}<SetupLink {machineId} />{/snippet}
  </AgentSetupList>
{/if}
<section class="sec">
  <h2 class="h2"><Icon icon={HardDrivesIcon} size={15} />Services</h2>
  {#if fleet.isPending && enabled}<p class="meta" role="status">Checking services…</p>{/if}
  {#if fleet.error}<p class="meta" role="alert">{fleet.error.message}</p>{/if}
  {#if fleet.data && !machine}<p class="card tight cream meta">This machine is not in the dashboard's services configuration, so its services and agent setup are not reported.</p>{/if}
  {#if machine && !machine.reachable}<p class="card tight warn"><strong>Could not be reached.</strong> {issues[0]?.fix}</p>{/if}
  {#if machine?.reachable}
    <ServiceRows machines={[machine]} showMachine={false} />
    {#each issues.filter((issue) => issue.service) as issue (issue.service?.id)}
      <p class="meta"><strong style:font-weight="var(--weight-strong)">{issue.service?.name}:</strong> {issue.fix}{issue.actions.length ? " Open it to do that." : ""}</p>
    {/each}
    {#each machine.warnings as warning (warning)}<p class="meta">{warning}</p>{/each}
  {/if}
</section>
