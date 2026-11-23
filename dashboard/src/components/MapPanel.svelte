<script lang="ts">
import ExternalLink from "@lucide/svelte/icons/external-link";
import X from "@lucide/svelte/icons/x";
import type { Snippet } from "svelte";
import { agoLabel } from "../lib/browse";
import { publishWords, type ServerReport, staleWords } from "../lib/serverStatus";
import { checkedTime, type Service, servicesHref } from "../lib/services.svelte";
import { dependents, type MachineMap, type Place, placeHref, placeLabel } from "../lib/topology";
import ServiceStatus from "./ServiceStatus.svelte";
import { placeIcon } from "./ServicesMap.svelte";

// What a chosen place on the map is, how its last check went, what a fault
// there stops, and the controls the machine offers for it. The full page of a
// service (diagnostics, schema, log) stays one link away.
let {
  map,
  place,
  controls,
  report = { kind: "none" },
}: {
  map: MachineMap;
  place: Place;
  controls: Snippet<[Service]>;
  report?: ServerReport;
} = $props();

const Icon = $derived(placeIcon[place.id]);
const machine = $derived(map.machine.id);
const stops = $derived(dependents(map, place.id));
const about: Record<Place["id"], string> = {
  histories:
    "The chat files your agents write. Mash reads them and leaves them unchanged; agent setup below shows which agents it reads.",
  indexing: "Reads new chats from your agents' history files and writes them to the SQLite index.",
  sqlite: "Mash's copy of your chats on this machine. It is a file, not a process.",
  search: "The Qdrant search index on this machine, built from the SQLite copy of your chats.",
  mcp: "Agents connected to Mash over MCP. Each session is the agent's own Mash process.",
  dashboard: "A Mash dashboard on this machine.",
  server:
    "The shared server this machine publishes to. Its status is its own report of this machine's group.",
  serverSearch: "The search index the shared server builds from what every machine published.",
};
</script>

<aside class="card mpanel" aria-label="{place.label} details">
  <header class="mp-h">
    <span class="ico"><Icon size={18} aria-hidden="true" /></span>
    <div><h2 class="h2">{place.label}</h2><p class="meta">{machine}</p></div>
    <a class="btn sm x" href={placeHref(machine)} aria-label="Close details"><X size={15} aria-hidden="true" /></a>
  </header>
  <p>{about[place.id]}</p>
  {#if place.affectedBy}
    <p class="card tight warn">This works, but it cannot search while {placeLabel[place.affectedBy]} is down.</p>
  {/if}
  {#if stops.length}
    <div class="card tight cream">
      <strong>What this stops</strong>
      <ul>{#each stops as stopped (stopped.id)}<li>{stopped.label}: searching</li>{/each}</ul>
      <p class="meta">Chats already indexed stay in the SQLite index.</p>
    </div>
  {/if}
  {#each place.services as service (service.id)}
    <section class="mp-s" aria-label={service.name}>
      <div class="mp-row"><strong>{service.name}</strong><ServiceStatus state={service.state} label={service.summary} /></div>
      <dl class="kvs">
        <dt>Checked</dt><dd title={checkedTime(service.checked_at)}>{agoLabel(service.checked_at * 1000, Date.now())}</dd>
        {#if service.pid}<dt>Process</dt><dd>PID {service.pid}</dd>{/if}
        {#if service.compatibility}<dt>Build</dt><dd class="mono">{service.compatibility.running_build.slice(0, 12) || "unknown"}</dd>{/if}
      </dl>
      <p class="meta">{service.control_note}</p>
      {@render controls(service)}
      <a class="a" href={servicesHref(machine, service.id)}><ExternalLink size={13} aria-hidden="true" />Full details</a>
    </section>
  {/each}
  {#if place.id === "server"}
    {#if report.kind === "reported"}
      <dl class="kvs">
        <dt>Address</dt><dd><code class="i">{report.address}</code></dd>
        <dt>Group</dt><dd>{report.group}</dd>
        <dt>Server build</dt><dd class="mono">{report.build.slice(0, 12)}</dd>
      </dl>
      {#if report.restartNeeded}<p class="card tight warn" role="status">The server needs a restart before it accepts machines of this group.</p>{/if}
      <section class="mp-s" aria-label="Machines publishing here">
        <strong>Publishing here</strong>
        <ul class="mp-m">
          {#each report.machines as entry (entry.id)}
            <li>
              <span><b>{entry.id}</b>{#if entry.id === machine} <span class="meta">this machine</span>{/if}</span>
              <span class="meta">{publishWords(entry, report.now)}{#if staleWords(entry, report.now)} <span class="pill warn">{staleWords(entry, report.now)}</span>{/if}</span>
            </li>
          {/each}
        </ul>
      </section>
    {:else if report.kind === "unreachable"}
      <p class="card tight warn" role="status">{report.message}</p>
      <p class="meta">Not reported: its machines and when each last published.</p>
    {:else}
      <p class="meta">Not reported here: the server's address, its machines and when each last published.</p>
    {/if}
  {/if}
  {#if place.id === "serverSearch"}
    <p class="meta">Not reported: the server doesn't include its search index's state in its status yet.</p>
  {/if}
  {#if place.id === "histories"}<a class="a" href="#/agents">Agents</a>{/if}
</aside>

<style>
  .mpanel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .mp-h {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .mp-h > div {
    flex: 1;
  }
  .mp-s {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: var(--border-divider);
  }
  .mp-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 6px 10px;
  }
  .mp-s .a {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  ul {
    margin: 6px 0;
    padding-left: 18px;
  }
  .mp-m {
    list-style: none;
    padding: 0;
  }
  .mp-m li {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 2px 10px;
    padding: 6px 0;
    border-bottom: var(--border-divider);
  }
</style>
