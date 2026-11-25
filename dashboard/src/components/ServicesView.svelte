<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Check from "@lucide/svelte/icons/check";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import Info from "@lucide/svelte/icons/info";
import Play from "@lucide/svelte/icons/play";
import RefreshCw from "@lucide/svelte/icons/refresh-cw";
import RotateCw from "@lucide/svelte/icons/rotate-cw";
import ScrollText from "@lucide/svelte/icons/scroll-text";
import Search from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import Square from "@lucide/svelte/icons/square";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agoLabel } from "../lib/browse";
import { stableRead } from "../lib/queries";
import { reported, serverReport, serverSearch, serverViewOptions } from "../lib/serverStatus";
import {
  checkedTime,
  type Machine,
  machineState,
  type Operation,
  problems,
  type Service,
  type ServiceAction,
  serviceAccessOptions,
  serviceAction,
  serviceHistory,
  serviceLog,
  serviceOptions,
  servicesHref,
} from "../lib/services.svelte";
import { statusOptions } from "../lib/status";
import { machineMap, type PlaceId } from "../lib/topology";
import { updates } from "../lib/updates.svelte";
import LoadError from "./LoadError.svelte";
import MachineSetup from "./MachineSetup.svelte";
import MapPanel from "./MapPanel.svelte";
import QueryError from "./QueryError.svelte";
import ServiceActivity from "./ServiceActivity.svelte";
import ServiceList from "./ServiceList.svelte";
import { kindIcon } from "./ServiceRows.svelte";
import ServicesMap from "./ServicesMap.svelte";
import SetupLink from "./SetupLink.svelte";
import Skeleton from "./Skeleton.svelte";

let { params }: { params: string } = $props();
const queryClient = useQueryClient();
const selection = $derived(new URLSearchParams(params));
const machineId = $derived(selection.get("machine") ?? "");
const serviceId = $derived(selection.get("service") ?? "");
const access = createQuery(() => serviceAccessOptions);
const enabled = $derived(access.data !== undefined);
const fleet = createQuery(() => serviceOptions(enabled));
// Without a machine in the address, the page shows the local one.
const selectedId = $derived(machineId || access.data?.localNode || "");
// Whether this machine's history reaches the shared server is known only here,
// so the map shows it for this machine alone.
const status = createQuery(() => statusOptions);
const publication = $derived(
  selectedId && selectedId === access.data?.localNode ? (status.data?.publication ?? null) : null,
);
const machine = $derived(fleet.data?.find((machine) => machine.id === selectedId));
const healthy = $derived(
  (fleet.data ?? []).filter((machine) => machineState(machine) === "healthy").length,
);
const service = $derived(machine?.services.find((service) => service.id === serviceId));
// The shared server as it reports this machine's group: its address, its
// machines and when each published. It speaks for the machine it was read
// from and for the machines it lists, not for a machine of another group.
const serverView = createQuery(() => ({ ...serverViewOptions, enabled }));
const report = $derived.by(() => {
  const whole = serverReport(serverView.data);
  const local = selectedId === access.data?.localNode;
  return local || reported(whole, selectedId) ? whole : serverReport(undefined);
});
// The map of the chosen machine, and the place on it whose details are open.
const map = $derived(
  machine ? machineMap(machine, report.kind === "none" ? false : serverSearch(report)) : null,
);
const placeId = $derived((selection.get("place") ?? "") as PlaceId | "");
const place = $derived(placeId ? map?.places.get(placeId) : undefined);
const others = $derived((fleet.data ?? []).filter((candidate) => candidate.id !== selectedId));
const history = createQuery(() => ({
  ...stableRead,
  queryKey: ["service-history", selectedId],
  enabled: enabled && !!selectedId,
  queryFn: ({ signal }) => serviceHistory(selectedId, signal),
}));
let showLog = $state(false);
const log = createQuery(() => ({
  ...stableRead,
  queryKey: ["service-log", selectedId, serviceId],
  enabled: enabled && showLog && !!serviceId,
  queryFn: ({ signal }) => serviceLog(selectedId, serviceId, signal),
}));
interface Target {
  machine: Machine;
  service: Service;
  action: ServiceAction;
  /** Where it was asked, so only that place shows the confirmation. */
  from?: "panel" | "list";
}
let confirm = $state<Target | null>(null);
// Faults are shown with the machine they belong to, not across the fleet.
// The list shows every machine; choosing one in the table narrows it to that one.
const shown = $derived(
  (fleet.data ?? []).filter((candidate) => !machineId || candidate.id === machineId),
);
const issue = $derived(
  problems(machine ? [machine] : []).find((found) => found.service?.id === serviceId),
);
let pending = $state(false);
let actionError = $state("");
let result = $state<Operation | null>(null);
$effect(() => {
  void params;
  showLog = false;
  confirm = null;
  actionError = "";
});
async function refresh(): Promise<void> {
  const refreshed = await fleet.refetch();
  if (!refreshed.error) actionError = "";
  await queryClient.invalidateQueries({ queryKey: ["service-setup"] });
  if (selectedId) await history.refetch();
  if (showLog && serviceId) await log.refetch();
}
// The rail's Check for updates also clears a stale action error and rereads
// every machine's setup, not only the ones on screen.
const serviceKeys: readonly unknown[] = [
  "services",
  "service-history",
  "service-log",
  "service-setup",
];
updates.offer({ check: refresh, owns: ([kind]) => serviceKeys.includes(kind) });
async function act(): Promise<void> {
  const revision = confirm?.service.control_revision;
  if (!confirm || !revision || pending) return;
  const action = confirm.action,
    target = confirm.machine.id,
    id = confirm.service.id;
  confirm = null;
  pending = true;
  actionError = "";
  result = null;
  try {
    result = await serviceAction(target, id, action, revision);
    await queryClient.invalidateQueries({ queryKey: ["services"] });
    await queryClient.invalidateQueries({ queryKey: ["service-history"] });
  } catch (error) {
    actionError =
      error instanceof Error
        ? error.message
        : "Operation outcome is unknown. Check status and recent activity.";
  } finally {
    pending = false;
  }
}
const actionLabel: Record<ServiceAction | "reinstall_plugin", string> = {
  start: "Start",
  stop: "Stop",
  restart: "Restart",
  reinstall_plugin: "Reinstall plugin",
};
const actionIcon = { start: Play, stop: Square, restart: RotateCw } as const;
// Kinds a supervisor runs: the services page can control them once the
// services configuration allows it. A file or the page's own process cannot be.
const supervised: readonly Service["kind"][] = ["watcher", "qdrant", "server"];
function consequences({ service, action }: Target): string {
  if (action === "start") return "";
  return service.kind === "watcher"
    ? "New chats may stop appearing until the watcher is running again. Existing indexed history is retained."
    : service.kind === "qdrant"
      ? "Search that uses this database may be unavailable while it is stopped or restarting."
      : "Clients using this service may lose access while it is stopped or restarting.";
}
const blocked = $derived(pending || !!fleet.error || !!actionError);
const activityItems = $derived(
  history.data?.filter((item) => !serviceId || item.request.service === serviceId) ?? [],
);
const allHealthy = $derived(!!fleet.data && healthy === fleet.data.length);
// The latest check of any machine, said in the page's subtitle.
const checked = $derived(Math.max(0, ...(fleet.data ?? []).map((entry) => entry.checked_at)));
</script>

{#snippet confirmation(target: Target)}
  <div class="card tight confirm" role="group" aria-label="Confirm service action">
    <span>{actionLabel[target.action]} {target.service.name} on {target.machine.id}? {consequences(target)}</span>
    <button class="btn p" type="button" onclick={act}><Check size={14} aria-hidden="true" />Confirm {target.action}</button>
    <button class="btn" type="button" onclick={() => (confirm = null)}>Cancel</button>
  </div>
{/snippet}
{#snippet serviceControls(entry: Service)}
  {#if machine && entry.actions.length}
    <div class="chips" style:gap="8px">
      {#each entry.actions as action (action)}{@const ActionIcon = actionIcon[action]}<button class="btn" class:p={action !== "stop" && entry.state !== "healthy"} type="button" disabled={blocked || !machine.reachable} onclick={() => machine && (confirm = { machine, service: entry, action, from: "panel" })}><ActionIcon size={14} aria-hidden="true" />{actionLabel[action]}</button>{/each}
    </div>
  {/if}
  {#if confirm?.from === "panel" && confirm.service.id === entry.id && confirm.machine.id === machine?.id}{@render confirmation(confirm)}{/if}
{/snippet}
{#snippet activity()}
  <ServiceActivity title="Recent activity{serviceId ? '' : ` on ${selectedId}`}" items={history.data ? activityItems : undefined} error={history.error} pending={history.isPending} />
{/snippet}
{#snippet listConfirmation(entry: Service)}
  {#if confirm?.from === "list" && confirm.service.id === entry.id && confirm.machine.id === machine?.id}{@render confirmation(confirm)}{/if}
{/snippet}

{#snippet attention()}
  <!-- The map shows every machine, so every machine's faults are listed. -->
  {#each fleet.data ?? [] as machine (machine.id)}
    {@const issues = problems([machine])}
    {#if issues.length}
      <section class="sec" aria-label="Needs attention on {machine.id}">
        <h2 class="h2"><TriangleAlert size={15} aria-hidden="true" />Needs attention on {machine.id}</h2>
        {#each issues as issue (`${issue.machine.id}/${issue.service?.id ?? ""}`)}
          {@const entry = issue.service}
          <div class="card next-step">
            <div class="g">
              <p><strong>{entry ? entry.name : issue.machine.id}</strong> <span>{entry ? entry.summary : "Could not be reached"}</span></p>
              <p class="meta">{issue.fix}</p>
            </div>
            <span class="acts">
              {#if entry}
                {#each issue.actions as action (action)}{@const ActionIcon = actionIcon[action]}<button class="btn sm p" type="button" disabled={blocked} onclick={() => (confirm = { machine: issue.machine, service: entry, action })}><ActionIcon size={14} aria-hidden="true" />{actionLabel[action]}</button>{/each}
                <a class="btn sm" href={servicesHref(issue.machine.id, entry.id)}><Info size={14} aria-hidden="true" />Details</a>
              {:else}
                <button class="btn sm" type="button" disabled={fleet.isFetching || pending} onclick={refresh}><RefreshCw size={15} aria-hidden="true" />Check again</button>
              {/if}
            </span>
          </div>
          {#if entry && confirm && !confirm.from && confirm.service.id === entry.id && confirm.machine.id === issue.machine.id}{@render confirmation(confirm)}{/if}
        {/each}
      </section>
    {/if}
  {/each}
{/snippet}
{#if serviceId}
  {@const KindIcon = service ? kindIcon[service.kind] : Server}
  <a class="back" href={servicesHref(selectedId)} aria-label="Back to services"><ChevronLeft size={15} aria-hidden="true" />services</a>
  <div class="hd">
    <div class="dh"><span class="ico lg services"><KindIcon size={20} aria-hidden="true" /></span><div><h1 class="h1">{service?.name ?? serviceId}</h1><p class="meta">{selectedId}{service ? ` / ${service.kind}` : ""}</p></div></div>
  </div>
{:else}
  <div class="hd">
    <div><h1 class="h1">Services</h1><p class="meta">Statuses are from the last check{checked ? `, ${agoLabel(checked * 1000, Date.now())}` : ""}. This page does not refresh on its own.</p></div>
    <span class="acts">
      <span role="status">{#if fleet.data}<span class="pill svc-health" class:ok={allHealthy} class:warn={!allHealthy}><i class="dot" aria-hidden="true"></i>{healthy} of {fleet.data.length} machine{fleet.data.length === 1 ? "" : "s"} healthy</span>{:else if enabled}<span class="meta">checking machines…</span>{/if}</span>
      {#if enabled}<button class="btn" type="button" disabled={fleet.isFetching || pending} onclick={refresh}><RefreshCw size={15} aria-hidden="true" />Check again</button>{/if}
    </span>
  </div>
{/if}
{#if serviceId}<p class="meta">Statuses are from the last check. This page does not refresh on its own.</p>{/if}
<QueryError title="Couldn’t connect to services" query={access} />
{#if enabled && fleet.isPending}<Skeleton rows={3} />{/if}
{#if enabled && fleet.error}<LoadError title="Couldn’t check service status" message={fleet.error.message} retry={refresh} busy={fleet.isFetching}/><p class="meta warn">Statuses shown earlier have not been verified.</p>{/if}
{#if pending}<p class="meta" role="status">Service action in progress. Check recent activity if the connection is interrupted.</p>{/if}
{#if actionError}<p class="fail" role="alert">{actionError}</p>{/if}
{#if result}<p class="ok" role="status" style:font-size="13.5px"><strong>{actionLabel[result.request.action]} on {result.request.machine}: {result.outcome}.</strong> {result.message}</p>{/if}

{#if enabled && fleet.data && serviceId}
  {#if !machine}<div class="state-c"><p role="alert">This machine is not in the configured inventory.</p></div>{/if}
  {#if machine && !service}<div class="state-c"><p role="alert">This service was not reported by the latest check.</p></div>{/if}
  {#if machine && service}
    {@const compatibility = service.compatibility}
    <dl class="stats">
      <div><dt class="eyebrow">status</dt><dd><b><i class="dot" class:warn={service.state !== "healthy"} aria-hidden="true"></i>{machine.reachable ? service.summary : "Machine unreachable"}</b><small title={checkedTime(service.checked_at)}>checked {agoLabel(service.checked_at * 1000, Date.now())}{service.pid ? ` / PID ${service.pid}` : ""}</small></dd></div>
      {#if compatibility}
        <div><dt class="eyebrow">schema</dt><dd><b><i class="dot" class:warn={compatibility.state !== "healthy"} aria-hidden="true"></i>{compatibility.state === "healthy" ? "Compatible" : "Needs attention"}</b><small>database {compatibility.database_schema ?? "unknown"} / supported {compatibility.supported_schema ?? "unknown"}</small></dd></div>
        <div><dt class="eyebrow">build</dt><dd><b class="mono">{compatibility.running_build.slice(0, 12) || "unknown"}</b><small>dashboard {compatibility.observer_build.slice(0, 12)}</small></dd></div>
      {/if}
    </dl>
    {#if compatibility && (compatibility.blocked || compatibility.next_step)}
      <section class="card butter" aria-label="Schema compatibility"><h2 class="h2">Schema compatibility</h2><p style:margin-top="6px">{compatibility.blocked}</p><p style:margin-top="6px"><strong>Next step.</strong> <span>{compatibility.next_step}</span></p></section>
    {/if}
    {@const controllable = service.actions.length > 0 || service.logs_available}
    <section class="sec">
      <h2 class="h2"><Server size={15} aria-hidden="true" />Manage</h2>
      <div class="card stack-s" class:cream={!controllable}>
        {#if !controllable}<p><strong>No actions are available for this service here.</strong></p>{/if}
        {#if issue}<p><span class="warn">Needs attention.</span> {issue.fix}</p>{/if}
        <p>{service.control_note}</p>
        {#if !controllable && supervised.includes(service.kind)}<p><a class="a" href="https://getmash.io/services/#enable-service-access" target="_blank" rel="noreferrer">How to turn on service controls</a></p>{/if}
        {#if service.actions.length || service.logs_available}
          <div class="chips" style:gap="8px">
            {#each service.actions as action (action)}{@const ActionIcon = actionIcon[action]}<button class="btn" class:p={issue?.actions.includes(action)} type="button" disabled={blocked || !machine.reachable} onclick={() => (confirm = { machine, service, action })}><ActionIcon size={14} aria-hidden="true" />{actionLabel[action]}</button>{/each}
            {#if service.logs_available}<button class="btn" type="button" aria-expanded={showLog} onclick={() => (showLog = !showLog)}><ScrollText size={15} aria-hidden="true" />{showLog ? "Hide log" : "View log"}</button>{/if}
          </div>
        {/if}
        {#if confirm}{@render confirmation(confirm)}{/if}
        {#if showLog}
          {#if log.isPending}<p class="meta" role="status">Loading log…</p>{/if}
          {#if log.error}<p class="fail" role="alert">{log.error.message}</p>{/if}
          {#if log.data !== undefined}<div class="blk"><div class="sc"><pre class="service-log" style:margin="0" style:white-space="pre">{log.data || "No recent log entries."}</pre></div></div>{/if}
        {/if}
      </div>
    </section>
    <section class="sec">
      <h2 class="h2"><Info size={15} aria-hidden="true" />Diagnostics</h2>
      <div class="card">
        {#if service.diagnostics.length}
          <dl class="kvs wide">{#each service.diagnostics as item (item.label)}<dt>{item.label}</dt><dd>{item.value}</dd>{/each}</dl>
        {:else}<p class="meta">Nothing further was reported.</p>{/if}
      </div>
    </section>
  {/if}
{:else if enabled && fleet.data}
  {#if machineId && !machine}<div class="state-c"><p role="alert">This machine is not in the configured inventory.</p></div>{/if}
  <div class="cols map-side">
    <div class="c-main stack">
      {#if map}<ServicesMap {map} {others} selected={placeId} {report} />{/if}
      {#if machine}<ServiceList {machine} disabled={blocked} ask={(entry, action) => machine && (confirm = { machine, service: entry, action, from: "list" })} confirmation={listConfirmation} />{/if}
      {#each shown as machine (machine.id)}
        {#if machine.reachable}{#each machine.warnings as warning (warning)}<p class="meta">{machine.id}: {warning}</p>{/each}{/if}
      {/each}
      {#if shown.some((machine) => machine.reachable)}
        <section class="sec">
          <div class="hd"><h2 class="h2"><Bot size={15} aria-hidden="true" />Agent setup</h2>{#if shown.length === 1 && shown[0]}<SetupLink machineId={shown[0].id} />{/if}</div>
          <div class="rws agentset" class:solo={shown.length < 2}>
            <div class="rw head" aria-hidden="true"><span>agent</span>{#if shown.length > 1}<span>machine</span>{/if}<span>parts</span><span>status</span></div>
            {#each shown.filter((machine) => machine.reachable) as machine (machine.id)}<MachineSetup machineId={machine.id} {enabled} showMachine={shown.length > 1} />{/each}
          </div>
          <p class="meta">Mash reads settings and files for this; it does not start the agents.</p>
        </section>
      {/if}
    </div>
    <div class="c-side">
      {#if map && place}<MapPanel {map} {place} {report} {publication} controls={serviceControls} />{/if}
      {#if selectedId}{@render activity()}{/if}
      {@render attention()}
    </div>
  </div>
{/if}
{#if enabled && selectedId && serviceId}{@render activity()}{/if}

<style>
  /* A fault and what to do about it: a raised card with a butter ring. */
  .next-step {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-color: transparent;
    border-radius: var(--radius-9);
    box-shadow:
      0 0 0 1px var(--memory-ring),
      var(--memory-glow);
  }
  .next-step .acts {
    flex-wrap: wrap;
  }
  .svc-health {
    gap: 7px;
  }
  /* Asking before a service is started, stopped or restarted. */
  .confirm {
    border-color: transparent;
    border-radius: var(--radius-8);
    box-shadow:
      0 0 0 1px var(--memory-ring),
      var(--memory-glow);
  }
</style>
