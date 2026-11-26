<script lang="ts">
import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import PlayIcon from "phosphor-svelte/lib/PlayIcon";
import StopIcon from "phosphor-svelte/lib/StopIcon";
import type { Snippet } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { agoLabel } from "../lib/browse";
import {
  checkedTime,
  type Machine,
  type Service,
  type ServiceAction,
  servicesHref,
  stateLabel,
} from "../lib/services.svelte";
import { serviceTone } from "../lib/tone";
import { kindIcon } from "./ServiceRows.svelte";
import ToneIcon from "./ToneIcon.svelte";

// One machine's services as rows: what each one is for, its last check, and
// the controls the machine offers for it. A control only asks; the page
// confirms before anything is sent.
let {
  machine,
  disabled,
  ask,
  confirmation,
}: {
  machine: Machine;
  disabled: boolean;
  ask: (service: Service, action: ServiceAction) => void;
  /** The confirmation for a control asked from a row, under that row. */
  confirmation: Snippet<[Service]>;
} = $props();

const purpose: Record<Service["kind"], string> = {
  watcher: "Watches agent histories and indexes new chats as they appear.",
  qdrant: "Answers searches from this page, the desktop app, Raycast and MCP.",
  sqlite: "Mash's copy of your chats on this machine. A file, not a process.",
  mcp: "Lets your agents search past chats and record decisions.",
  dashboard: "This page and the apps that open it.",
  server: "Keeps history in one place for all your machines.",
};
const label: Record<ServiceAction, string> = { start: "Start", stop: "Stop", restart: "Restart" };
const icon = { start: PlayIcon, stop: StopIcon, restart: ArrowClockwiseIcon } as const;
</script>

<section class="card pad0 svc-list" aria-label="Services on {machine.id}">
  {#each machine.services as entry (entry.id)}
    {@const Glyph = kindIcon[entry.kind]}
    <div class="svc-row">
      <span class="svc-ico" aria-hidden="true"><Icon icon={Glyph} size={16} /></span>
      <span class="svc-name"><strong>{entry.name}</strong><small>{purpose[entry.kind]}</small></span>
      <span class="svc-state {serviceTone(entry.state)}">
        <span class="svc-st"><ToneIcon tone={serviceTone(entry.state)} /><span title={stateLabel[entry.state]}>{machine.reachable ? entry.summary : "Machine unreachable"}</span></span>
        <small class="mono" title={checkedTime(entry.checked_at)}>checked {agoLabel(entry.checked_at * 1000, Date.now())}{entry.pid ? ` · PID ${entry.pid}` : ""}</small>
      </span>
      <span class="svc-acts">
        {#each entry.actions as action (action)}{@const ActionIcon = icon[action]}<button class="btn sm" class:p={action !== "stop" && entry.state !== "healthy"} class:danger={action === "stop"} type="button" disabled={disabled || !machine.reachable} onclick={() => ask(entry, action)}><Icon icon={ActionIcon} size={14} />{label[action]}<span class="sr-only"> {entry.name}</span></button>{/each}
        <a class="btn sm" href={servicesHref(machine.id, entry.id)} aria-label="{entry.name} details">Details<Icon icon={CaretRightIcon} size={14} /></a>
      </span>
      {@render confirmation(entry)}
    </div>
  {/each}
</section>

<style>
  .svc-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 16px;
    padding: 14px 18px;
    border-top: var(--border-divider);
  }
  .svc-row:first-child {
    border-top: 0;
  }
  .svc-ico {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: var(--border-divider);
    border-radius: var(--radius-4);
    background: var(--surface-well);
    color: var(--text-strong);
  }
  .svc-name,
  .svc-state {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .svc-name {
    flex: 1 1 220px;
  }
  .svc-name strong {
    color: var(--text-strong);
    font-size: 14px;
    font-weight: var(--weight-strong);
  }
  .svc-name small {
    color: var(--text-muted);
    font-size: 12px;
  }
  .svc-state {
    flex: 0 1 200px;
  }
  .svc-st {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--text-strong);
    font-size: 13px;
    font-weight: var(--weight-label);
  }
  .svc-state.warn .svc-st {
    color: var(--status-warn-text);
  }
  .svc-state.bad .svc-st {
    color: var(--status-bad-text);
  }
  .svc-state small {
    color: var(--text-faint);
    font-size: 12px;
  }
  .svc-acts {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
    margin-left: auto;
  }
  .svc-row > :global(.confirm) {
    flex-basis: 100%;
  }
</style>
