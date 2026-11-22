<script module lang="ts">
import Database from "@lucide/svelte/icons/database";
import Eye from "@lucide/svelte/icons/eye";
import LayoutDashboard from "@lucide/svelte/icons/layout-dashboard";
import Plug from "@lucide/svelte/icons/plug";
import Search from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import type { Service } from "../lib/services.svelte";

/** The icon for each kind of service, shared with a service's own page. */
export const kindIcon = {
  sqlite: Database,
  qdrant: Search,
  server: Server,
  watcher: Eye,
  mcp: Plug,
  dashboard: LayoutDashboard,
} as const;
</script>

<script lang="ts">
import { servicesHref } from "../lib/services.svelte";
import ServiceStatus from "./ServiceStatus.svelte";

// Services as one list of rows, each opening that service's page. With
// several machines the machine is a column, not a section of its own.
let {
  machines,
  showMachine = true,
}: {
  machines: readonly { readonly id: string; readonly services: readonly Service[] }[];
  showMachine?: boolean;
} = $props();
</script>

<div class="rws" class:solo={!showMachine}>
  <div class="rw head" aria-hidden="true"><span>service</span>{#if showMachine}<span>machine</span>{/if}<span>kind</span><span>status</span></div>
  {#each machines as machine (machine.id)}
    {#each machine.services as entry (entry.id)}
      {@const Icon = kindIcon[entry.kind]}
      <a class="rw" href={servicesHref(machine.id, entry.id)}>
        <span class="rw-n"><Icon size={17} aria-hidden="true" /><strong>{entry.name}</strong></span>
        {#if showMachine}<span>{machine.id}</span>{/if}
        <span class="rw-kind">{entry.kind}</span>
        <ServiceStatus state={entry.state} label={entry.summary} />
      </a>
    {/each}
  {/each}
</div>
