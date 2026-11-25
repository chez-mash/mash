<script lang="ts">
import ArrowUpFromLine from "@lucide/svelte/icons/arrow-up-from-line";
import Check from "@lucide/svelte/icons/check";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import Clock from "@lucide/svelte/icons/clock";
import Download from "@lucide/svelte/icons/download";
import Eye from "@lucide/svelte/icons/eye";
import FileText from "@lucide/svelte/icons/file-text";
import Laptop from "@lucide/svelte/icons/laptop";
import ListChecks from "@lucide/svelte/icons/list-checks";
import Play from "@lucide/svelte/icons/play";
import RefreshCw from "@lucide/svelte/icons/refresh-cw";
import SearchIcon from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import Square from "@lucide/svelte/icons/square";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agoLabel } from "../lib/browse";
import { stableRead } from "../lib/queries";
import { fleetHref } from "../lib/route.svelte";
import {
  type FleetAction,
  type FleetJob,
  type FleetRequest,
  fleetActionLabel,
  fleetCatalogOptions,
  fleetHistory,
  serviceAccessOptions,
  serviceRequestId,
  submitFleetJob,
} from "../lib/services.svelte";
import QueryError from "./QueryError.svelte";
import Skeleton from "./Skeleton.svelte";

// Fleet actions run through the controller's allowlist. The list page shows
// what this dashboard may start; an action's page picks its machines.
let { action }: { action: string } = $props();
const client = useQueryClient();
const access = createQuery(() => serviceAccessOptions);
const enabled = $derived(access.data !== undefined);
const catalog = createQuery(() => fleetCatalogOptions(enabled));
const history = createQuery(() => ({
  ...stableRead,
  queryKey: ["fleet-jobs"],
  enabled: enabled && catalog.data?.enabled === true,
  queryFn: ({ signal }) => fleetHistory(signal),
  refetchInterval: (query) =>
    query.state.data?.some((job) => job.outcome === "running") ? 2000 : false,
}));

const groups: readonly {
  name: string;
  icon: typeof ListChecks;
  actions: readonly FleetAction[];
}[] = [
  { name: "check", icon: ListChecks, actions: ["status", "preflight", "watch-status"] },
  { name: "update", icon: Download, actions: ["deploy"] },
  { name: "index", icon: RefreshCw, actions: ["sync", "local-views", "replay-views"] },
  { name: "watchers", icon: Eye, actions: ["watch-start", "watch-stop", "watch-logs"] },
];
const blurb: Record<FleetAction, string> = {
  status: "Reads each machine's installed build and readiness. Changes nothing.",
  preflight: "Checks that a deployment could run. Changes nothing.",
  "watch-status": "Reads whether each machine's watcher is running. Changes nothing.",
  deploy: "Builds and installs Mash. Services can be interrupted while it runs.",
  sync: "Indexes new history and publishes it. Can take several minutes.",
  "local-views": "Rebuilds local search from stored history. Can take several minutes.",
  "replay-views": "Rebuilds shared search on the server. Can take several minutes.",
  "watch-start": "Installs the configured supervisor job and starts the watcher.",
  "watch-stop": "Stops the watcher. New chats stop appearing until it is started again.",
  "watch-logs": "Saves recent watcher logs privately on the controller.",
};
// The guide that explains the inventory's `allow_actions` list.
const fleetDocs = "https://getmash.io/services/#run-fleet-operations";
const icons = {
  status: Laptop,
  preflight: ListChecks,
  "watch-status": Eye,
  deploy: Download,
  sync: ArrowUpFromLine,
  "local-views": SearchIcon,
  "replay-views": Server,
  "watch-start": Play,
  "watch-stop": Square,
  "watch-logs": FileText,
} as const;
const readOnly: readonly FleetAction[] = ["status", "preflight", "watch-status"];

const allowed = $derived(catalog.data?.actions ?? []);
const current = $derived(allowed.find((item) => item.action === action));
// A target is a machine unless the inventory lists it as a shared server.
const services = $derived(catalog.data?.services ?? []);
const machineTargets = $derived(
  (current?.targets ?? []).filter((id) => !services.some((service) => service.id === id)),
);
const serviceTargets = $derived(
  services.filter((service) => current?.targets.includes(service.id)),
);
function targetName(id: string): string {
  const service = services.find((item) => item.id === id);
  return service ? `${service.name} (${service.machine})` : id;
}
function countLabel(ids: readonly string[]): string {
  const onlyMachines = ids.every((id) => !services.some((service) => service.id === id));
  const noun = onlyMachines ? "machine" : "target";
  return `${ids.length} ${noun}${ids.length === 1 ? "" : "s"}`;
}
let selected = $state<string[]>([]);
let confirmation = $state<FleetRequest | null>(null);
let pending = $state(false);
let error = $state("");
$effect(() => {
  void action;
  selected = [];
  confirmation = null;
});
const running = $derived(history.data?.find((job) => job.outcome === "running"));
const jobs = $derived(
  (history.data ?? []).filter((job) => !action || job.request.action === action),
);

function toggle(machine: string): void {
  selected = selected.includes(machine)
    ? selected.filter((item) => item !== machine)
    : [...selected, machine];
  confirmation = null;
}
function prepare(): void {
  if (!catalog.data?.revision || !current || !selected.length) return;
  const request = {
    id: serviceRequestId(),
    action: current.action,
    targets: [...selected],
    revision: catalog.data.revision,
  };
  if (readOnly.includes(current.action)) void submit(request);
  else confirmation = request;
}
async function submit(request: FleetRequest): Promise<void> {
  if (pending) return;
  confirmation = null;
  pending = true;
  try {
    const job = await submitFleetJob(request);
    client.setQueryData<Awaited<ReturnType<typeof fleetHistory>>>(["fleet-jobs"], (previous) => [
      job,
      ...(previous ?? []).filter((item) => item.request.id !== job.request.id),
    ]);
    selected = [];
  } catch (cause) {
    error = `${cause instanceof Error ? cause.message : "Fleet request failed."} Request ${request.id}. Check the activity below and the affected machines before starting another operation.`;
  } finally {
    pending = false;
  }
}
// The outcome as a pill: mint done, mustard partial, red failed, plain
// paper while it runs or when the outcome is unknown.
const outcomeClass = {
  running: "",
  succeeded: "ok",
  partial: "warn",
  failed: "bad",
  unknown: "",
} as const;
const local = $derived(access.data?.localNode ?? "");
// Ages are read again whenever the job list is.
const now = $derived(history.dataUpdatedAt >= 0 ? Date.now() : 0);
/** A target's rows are worth showing while the job runs, or when a target
 * reports something the job's own outcome does not already say. */
function detailed(job: FleetJob): boolean {
  return (
    job.outcome === "running" ||
    job.targets.some(
      (target) => !!target.detail || !!target.progress || target.state !== job.outcome,
    )
  );
}
function targetNote(job: FleetJob, target: FleetJob["targets"][number]): string {
  const state = target.state !== job.outcome || job.outcome === "running" ? target.state : "";
  return [state, target.detail ?? ""].filter(Boolean).join(" · ") || target.state;
}
function percent(target: FleetJob["targets"][number]): number | null {
  const progress = target.progress;
  if (!progress?.total) return null;
  return Math.max(0, Math.min(100, Math.round((progress.progress / progress.total) * 100)));
}
</script>

{#snippet when(job: FleetJob, compact: boolean)}
  {job.outcome === "running" ? "started " : ""}{agoLabel(job.requested_at * 1000, now)}{compact ? ` · ${countLabel(job.targets.map((target) => target.id))}` : ""}
{/snippet}
{#snippet activity()}
  <div class="hd"><h2 class="h2"><Clock size={15} aria-hidden="true" />{action ? "recent runs" : "activity"}</h2></div>
  <QueryError title="Couldn’t check fleet jobs" query={history} />
  {#if history.data && !jobs.length}<p class="card tight cream meta">No runs yet.</p>{/if}
  {#if action && jobs.length}
    <div class="card pad0 list">
      {#each jobs as job (job.request.id)}
        <article class="lrow">
          <b>{fleetActionLabel[job.request.action]}</b>
          <span class="sp">{@render when(job, true)}<span class="pill jb {outcomeClass[job.outcome]}">{job.outcome}</span></span>
          {#if detailed(job)}<p>{job.targets.map((target) => `${targetName(target.id)}: ${targetNote(job, target)}`).join("; ")}</p>{/if}
          {#if job.outcome === "unknown"}<p class="warn">Completion could not be confirmed. Inspect the affected machines before starting another operation.</p>{/if}
        </article>
      {/each}
    </div>
  {:else}
    {#each jobs as job (job.request.id)}
      {@const open = detailed(job)}
      <article class="card job">
        <header><b>{fleetActionLabel[job.request.action]}</b><span class="pill jb {outcomeClass[job.outcome]}">{job.outcome}</span></header>
        <p class="meta">{@render when(job, !open)}</p>
        {#if open}
          <div class="jts">
            {#each job.targets as target (target.id)}
              {@const done = percent(target)}
              <div class="jt">
                <Laptop size={14} aria-hidden="true" /><b>{targetName(target.id)}</b>
                {#if done !== null && target.state === "running"}<span class="bar"><i style:width="{done}%"></i></span><span class="num">{done}%</span>
                {:else}<span class="sp" class:fail={target.state === "failed"}>{targetNote(job, target)}</span>{/if}
              </div>
            {/each}
          </div>
        {/if}
        {#if job.outcome === "unknown"}<p class="warn jn">Completion could not be confirmed. Inspect the affected machines before starting another operation.</p>{/if}
      </article>
    {/each}
  {/if}
{/snippet}

{#snippet turnOn(name: string)}
  <p>Fleet operations are turned on in the fleet inventory, the private file this dashboard was started with. To turn this one on, add <code class="i">"{name}"</code> to <code class="i">allow_actions</code> in its <code class="i">[controller]</code> section, then reload this page. {#if name === "deploy"}Deploying also needs build, install and preflight commands in the inventory.{:else if name === "replay-views"}It also needs a shared server whose <code class="i">deployment</code> lists <code class="i">replay_views</code> commands.{/if}</p>
  <p><a class="a" href={fleetDocs} target="_blank" rel="noreferrer">Read how to run fleet operations</a></p>
{/snippet}

{#if action}<a class="back" href="#/fleet" aria-label="Back to fleet manager"><ChevronLeft size={15} aria-hidden="true" />fleet manager</a>{/if}
<QueryError title="Couldn’t connect to services" query={access} />
<QueryError title="Couldn’t load fleet controls" query={catalog} />
{#if (access.isPending || catalog.isPending) && !access.error && !catalog.error}<h1 class="h1">Fleet manager</h1><Skeleton rows={3} />{/if}
{#if catalog.data && !catalog.data.enabled}
  <h1 class="h1">Fleet manager</h1>
  <div class="card cream">
    <p><strong>Fleet control is not set up on this dashboard.</strong></p>
    <p class="meta" style:margin-top="6px">It needs a fleet inventory that names this machine as the controller: start the dashboard with <code class="i">mash dash --fleet-config PATH</code>. Until then, deploys and index runs are started from the command line with <code class="i">tools/fleet.sh</code>.</p>
  </div>
{:else if catalog.data && !action}
  <h1 class="h1">Fleet manager</h1>
  <div class="cols c32">
    <div class="stack">
      {#each groups as group (group.name)}
        <section class="sec">
          <h2 class="h2"><group.icon size={15} aria-hidden="true" />{group.name}</h2>
          <div class="g3">
            {#each group.actions as item (item)}
              {@const Icon = icons[item]}
              {#if allowed.some((entry) => entry.action === item)}
                <a class="card opc" href={fleetHref(item)} aria-label={fleetActionLabel[item]} aria-describedby="fleet-about-{item}"><span class="ico fleet"><Icon size={17} aria-hidden="true" /></span><b>{fleetActionLabel[item]}</b><span id="fleet-about-{item}">{blurb[item]}</span></a>
              {:else}
                <a class="card opc off" href={fleetHref(item)} aria-label="{fleetActionLabel[item]}: turned off" aria-describedby="fleet-about-{item}"><span class="ico fleet"><Icon size={17} aria-hidden="true" /></span><b>{fleetActionLabel[item]}</b><span id="fleet-about-{item}">{blurb[item]}</span><span class="pill">Turned off</span><span class="a">How to turn it on</span></a>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
      {#if !allowed.length}<p class="meta">Every operation is turned off for this dashboard. Operations are turned on in the fleet inventory. <a class="a" href={fleetDocs} target="_blank" rel="noreferrer">How to turn them on</a></p>{/if}
    </div>
    <div class="sec">{@render activity()}</div>
  </div>
{:else if catalog.data && action}
  {#if !current}
    {@const Off = icons[action as FleetAction] ?? ListChecks}
    <div class="dh"><span class="ico lg fleet"><Off size={20} aria-hidden="true" /></span><div><h1 class="h1">{fleetActionLabel[action as FleetAction] ?? action}</h1><p class="meta">turned off</p></div></div>
    <div class="card cream stack-s">
      <p><strong>This operation is turned off for this dashboard.</strong></p>
      {@render turnOn(action)}
    </div>
  {:else}
    {@const Icon = icons[current.action]}
    <div class="dh"><span class="ico lg fleet"><Icon size={20} aria-hidden="true" /></span><div><h1 class="h1">{fleetActionLabel[current.action]}</h1><p class="meta">{groups.find((group) => group.actions.includes(current.action))?.name}</p></div></div>
    <p style:max-width="640px" style:font-size="15px">{blurb[current.action]}{readOnly.includes(current.action) ? "" : " It keeps running if you leave this page."}</p>
    <section class="sec">
      <div class="hd"><h2 class="h2"><Laptop size={15} aria-hidden="true" />Machines</h2><button class="a" type="button" disabled={pending || !!error} onclick={() => { selected = [...current.targets]; confirmation = null; }} style:font-size="12.5px">Select all</button></div>
      <div class="g3">
        {#each machineTargets as machine (machine)}
          {@const on = selected.includes(machine)}
          <button class="card tick" class:butter={on} type="button" role="checkbox" aria-label={machine} aria-describedby={machine === local ? "fleet-local" : undefined} aria-checked={on} disabled={pending || !!error} onclick={() => toggle(machine)}><span class="check" class:on>{#if on}<Check aria-hidden="true" />{/if}</span><span class="g"><b>{machine}</b>{#if machine === local}<small id="fleet-local">this machine</small>{/if}</span></button>
        {/each}
      </div>
      {#if !current.targets.length}<p class="meta">Nothing in the inventory is eligible for this action.</p>{/if}
    </section>
    {#if serviceTargets.length}
      <section class="sec">
        <h2 class="h2"><Server size={15} aria-hidden="true" />Services</h2>
        <div class="g3">
          {#each serviceTargets as service (service.id)}
            {@const on = selected.includes(service.id)}
            <button class="card tick" class:butter={on} type="button" role="checkbox" aria-label={service.name} aria-checked={on} disabled={pending || !!error} onclick={() => toggle(service.id)}><span class="check" class:on>{#if on}<Check aria-hidden="true" />{/if}</span><span class="g"><b>{service.name}</b><small>on {service.machine}</small></span></button>
          {/each}
        </div>
      </section>
    {/if}
    {#if running}<p class="meta" style:font-size="14px" role="status">{fleetActionLabel[running.request.action]} is running. Only one fleet operation can run at a time.</p>{/if}
    {#if confirmation}
      <div class="card butter armed" role="group" aria-label="Confirm">
        <p><strong>{fleetActionLabel[confirmation.action]} on {confirmation.targets.map(targetName).join(", ")}? {blurb[confirmation.action]}</strong></p>
        <button class="btn p" type="button" disabled={pending} onclick={() => { if (confirmation) void submit(confirmation); }}><Play size={14} aria-hidden="true" />Yes, start it</button>
        <button class="btn" type="button" onclick={() => (confirmation = null)}>Cancel</button>
      </div>
    {:else}
      <div><button class="btn p lg" type="button" disabled={!selected.length || pending || !!error || !!running} onclick={prepare}><Icon size={16} aria-hidden="true" />{running ? "Unavailable while another job runs" : selected.length ? `${fleetActionLabel[current.action]} on ${countLabel(selected)}` : serviceTargets.length ? "Choose targets" : "Choose machines"}</button></div>
    {/if}
    {#if pending}<p class="meta" role="status">Recording the fleet operation…</p>{/if}
    {#if error}<p class="fail" role="alert">{error}</p><div><button class="btn" type="button" onclick={() => (error = "")}><Check size={14} aria-hidden="true" />I’ve checked the outcome</button></div>{/if}
    <section class="sec">{@render activity()}</section>
  {/if}
{/if}
