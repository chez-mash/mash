<script module lang="ts">
import DatabaseIcon from "phosphor-svelte/lib/DatabaseIcon";
import EyeIcon from "phosphor-svelte/lib/EyeIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import LayoutIcon from "phosphor-svelte/lib/LayoutIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import PlugIcon from "phosphor-svelte/lib/PlugIcon";
import type { Service } from "../lib/services.svelte";

/** The icon for each kind of service, shared with a service's own page. */
export const kindIcon = {
  sqlite: DatabaseIcon,
  qdrant: MagnifyingGlassIcon,
  server: HardDrivesIcon,
  watcher: EyeIcon,
  mcp: PlugIcon,
  dashboard: LayoutIcon,
} as const;
</script>

<script lang="ts">
import Icon from "$lib/components/Icon.svelte";
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

{#if showMachine}
  <div class="rws">
    <div class="rw head" aria-hidden="true"><span>service</span><span>machine</span><span>kind</span><span>status</span></div>
    {#each machines as machine (machine.id)}
      {#each machine.services as entry (entry.id)}
        {@const Glyph = kindIcon[entry.kind]}
        <a class="rw" href={servicesHref(machine.id, entry.id)}>
          <span class="rw-n"><Icon icon={Glyph} size={17} /><strong>{entry.name}</strong></span>
          <span>{machine.id}</span>
          <span class="rw-kind">{entry.kind}</span>
          <ServiceStatus state={entry.state} label={entry.summary} />
        </a>
      {/each}
    {/each}
  </div>
{:else}
  <!-- One machine's services as the wireframes draw them: a plain list of
       names with what each reports underneath, and its state at the right. -->
  <ul class="card sv">
    {#each machines as machine (machine.id)}
      {#each machine.services as entry (entry.id)}
        <li>
          <a href={servicesHref(machine.id, entry.id)}>
            <span class="sv-n"><strong>{entry.name}</strong><small>{entry.summary}</small></span>
            <ServiceStatus state={entry.state} />
          </a>
        </li>
      {/each}
    {/each}
  </ul>
{/if}

<style>
  .sv {
    margin: 0;
    padding: 0;
    overflow: hidden;
    list-style: none;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
  }
  .sv li + li {
    border-top: var(--border-divider);
  }
  .sv a {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 11px 16px;
    color: var(--text-body);
    text-decoration: none;
  }
  .sv a:hover {
    background: var(--surface-hover);
  }
  .sv-n {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .sv-n strong {
    font-size: 13.5px;
    font-weight: var(--weight-label);
    color: var(--text-strong);
  }
  .sv-n small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
