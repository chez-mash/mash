<script lang="ts">
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
import ArrowsClockwiseIcon from "phosphor-svelte/lib/ArrowsClockwiseIcon";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import DownloadSimpleIcon from "phosphor-svelte/lib/DownloadSimpleIcon";
import WarningIcon from "phosphor-svelte/lib/WarningIcon";
import WrenchIcon from "phosphor-svelte/lib/WrenchIcon";
import { onDestroy } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { type RepairResult, type RepairStage, repairPlugin } from "../lib/pluginRepair";
import {
  fleetCatalogOptions,
  fleetHistory,
  reinstallPlugin,
  serviceHistory,
  serviceRequestId,
  setupBuild,
  submitFleetJob,
} from "../lib/services.svelte";
import ToneIcon from "./ToneIcon.svelte";

// The button that does the plugin fix on the machine: update Mash there when
// Fleet manager may, reinstall the plugin, then say the one step left, which
// is restarting the agent. The result is the page's, so it outlives the
// warning once a check finds the plugin current.
let {
  machineId,
  build,
  agentName,
  result,
  onresult,
}: {
  machineId: string;
  build: string;
  agentName: string;
  result: RepairResult | undefined;
  onresult: (result: RepairResult | undefined) => void;
} = $props();
const client = useQueryClient();
const catalog = createQuery(() => fleetCatalogOptions(true));
const deployRevision = $derived(
  catalog.data?.enabled &&
    catalog.data.actions.some(
      (item) => item.action === "deploy" && item.targets.includes(machineId),
    )
    ? catalog.data.revision
    : null,
);
const restartsDashboard = $derived(!!catalog.data?.dashboardRestarts.includes(machineId));
let confirming = $state(false);
let stage = $state<RepairStage | null>(null);
const controller = new AbortController();
onDestroy(() => controller.abort());

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => {
      clearTimeout(timer);
      reject(new Error("Stopped watching; check the agent setup again."));
    });
  });
}
async function run(): Promise<void> {
  confirming = false;
  onresult(undefined);
  const outcome = await repairPlugin(
    { machine: machineId, deployRevision, build, restartsDashboard },
    {
      submitFleetJob,
      fleetHistory,
      reinstall: reinstallPlugin,
      serviceHistory,
      setupBuild,
      requestId: serviceRequestId,
      sleep,
    },
    (next) => {
      stage = next;
    },
    controller.signal,
  );
  stage = null;
  onresult(outcome);
}
function start(): void {
  if (deployRevision) confirming = true;
  else void run();
}
// After a fix, the setup is read again to see whether the plugin is current.
let checking = $state(false);
async function checkAgain(): Promise<void> {
  checking = true;
  try {
    await client.invalidateQueries({ queryKey: ["service-setup", machineId] });
  } finally {
    checking = false;
  }
}

type Mark = "todo" | "running" | "done" | "failed";
const updateMark = $derived<Mark>(
  stage?.stage === "updating"
    ? "running"
    : result?.failed === "update"
      ? "failed"
      : stage?.stage === "reinstalling" || result?.ok || result?.failed === "reinstall"
        ? "done"
        : "todo",
);
const reinstallMark = $derived<Mark>(
  stage?.stage === "reinstalling"
    ? "running"
    : result?.ok
      ? "done"
      : result?.failed === "reinstall"
        ? "failed"
        : "todo",
);
const word: Record<Mark, string> = {
  todo: "",
  running: "Running…",
  done: "Done",
  failed: "Did not finish",
};
const busy = $derived(stage !== null);
</script>

{#snippet mark(state: Mark)}
  {#if state !== "todo"}
    {@const tone = state === "done" ? "ok" : state === "failed" ? "warn" : "run"}
    <span class="rp-mark tone-{tone}" role={state === "running" ? "status" : undefined}><ToneIcon {tone} size={14} />{word[state]}</span>
  {/if}
{/snippet}

<div class="rp" role="group" aria-label="Fix the plugin on {machineId}">
  <ol>
    {#if deployRevision}
      <li>
        <span>Update Mash on {machineId} through Fleet manager. {@render mark(updateMark)}</span>
        {#if stage?.stage === "updating" && stage.detail}<span class="meta rp-detail">{stage.detail}</span>{/if}
      </li>
    {/if}
    <li><span>Reinstall the plugin there, as <code>mash init --yes --no-index</code> does. {@render mark(reinstallMark)}</span></li>
    <li>
      <span class:rp-left={result?.ok}>Restart {agentName} on {machineId}. {#if result?.ok}<strong>This one is yours:</strong> {agentName} loads its plugins when it starts, so Mash cannot do it for you.{:else}You do this one: Mash cannot restart it.{/if}</span>
    </li>
  </ol>
  {#if !deployRevision && !result}
    <p class="meta rp-note">This dashboard cannot update Mash on {machineId}. If Mash itself is out of date there, update it first: see the steps by hand.</p>
  {/if}
  {#if result?.ok}
    <p class="ok rp-out" role="status"><Icon icon={CheckIcon} size={16} /><span><strong>Plugin reinstalled on {machineId}.</strong> {result.message}</span></p>
    <div class="rp-acts"><button class="btn sm" type="button" disabled={checking} onclick={checkAgain}><Icon icon={ArrowsClockwiseIcon} size={14} />{checking ? "Checking…" : "Check the plugin again"}</button></div>
  {:else if result}
    <p class="warn rp-out" role="alert"><Icon icon={WarningIcon} size={16} state="strong" /><span>{result.message} You can also do it by hand: see the steps below.</span></p>
    <div class="rp-acts"><button class="btn sm" type="button" onclick={start}><Icon icon={ArrowClockwiseIcon} size={14} />Try again</button></div>
  {:else if confirming}
    <div class="card tight confirm rp-confirm" role="group" aria-label="Confirm plugin fix">
      <span>Deploy Mash to {machineId}, then reinstall the plugin there? Deploying builds and installs Mash, and services there can be interrupted while it runs.{#if restartsDashboard} Its dashboard restarts on the new release.{/if}</span>
      <button class="btn p" type="button" onclick={run}><Icon icon={DownloadSimpleIcon} size={14} state="strong" />Update and reinstall</button>
      <button class="btn" type="button" onclick={() => (confirming = false)}>Cancel</button>
    </div>
  {:else}
    <div class="rp-acts">
      <button class="btn p" type="button" disabled={busy} onclick={start}><Icon icon={WrenchIcon} size={15} state="strong" />{deployRevision ? "Update Mash and the plugin" : "Reinstall the plugin"}</button>
    </div>
  {/if}
</div>

<style>
  /* The automated steps reuse the warning's numbered list; these place the
     button, the result and each step's state beside it. */
  .rp {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .rp-mark {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: 4px;
    font-size: 13px;
    font-weight: var(--weight-strong);
    vertical-align: middle;
  }
  .rp-detail {
    grid-column: 2;
    overflow-wrap: anywhere;
  }
  .rp-left {
    color: var(--text-strong);
  }
  .rp-note,
  .rp-out {
    margin: 0;
  }
  .rp-out {
    display: flex;
    gap: 8px;
    overflow-wrap: anywhere;
  }
  .rp-out :global(svg) {
    flex: none;
    margin-top: 3px;
  }
  .rp-out span {
    color: var(--text-body);
  }
  .rp-acts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .rp-acts .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  /* Asking before a deploy: a raised card with a butter ring, as every
     confirmation of a change on another machine is. */
  .rp-confirm {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 10px;
    padding: 16px;
    border-color: transparent;
    border-radius: var(--radius-8);
    box-shadow:
      0 0 0 1px var(--memory-ring),
      var(--memory-glow);
  }
  .rp-confirm span {
    flex: 1 1 100%;
  }
</style>
