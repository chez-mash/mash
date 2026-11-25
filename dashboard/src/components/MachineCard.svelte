<script lang="ts">
import type { Group } from "../lib/browse";
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machinePulse } from "../lib/machinePulse";
import { machineOs, osName } from "../lib/os";
import { machineHref } from "../lib/route.svelte";
import MachineIcon from "./MachineIcon.svelte";

// One machine as a card: its system's mark, its name and what its status dot
// says, the system's name, a line on how it was last heard from, then its
// chats and agents. A machine that needs a look gets an amber edge, and one
// that is not answering is drawn quieter; one nothing reports on is not. Everything here is what the
// machine's pulse and the chat list already say; nothing is assumed.
let { group, local }: { group: Group; local: string } = $props();
const evidence = createMachineEvidence();
const pulse = $derived(machinePulse(group.key, evidence));
const os = $derived(machineOs(group.key, evidence.fleet, evidence.report));
// "Running · checked in 2m": the state names the dot, the rest is the line.
const state = $derived(pulse.words.split(" · ")[0] ?? pulse.words);
const detail = $derived(pulse.words.split(" · ").slice(1).join(" · "));
const line = $derived(
  [group.key === local ? "This machine" : "", detail].filter(Boolean).join(" · "),
);
</script>

<a class="card mc" class:warn={pulse.tone === "warn" || pulse.tone === "bad"} class:quiet={pulse.tone === "none" && !pulse.unreported} href={machineHref(group.key)}>
  <span class="mc-h">
    <span class="mc-i" aria-hidden="true"><MachineIcon machine={group.key} size={16} /></span>
    <span class="mc-n">
      <b>{group.key}</b>
      <span class="mc-s {pulse.tone}" title={pulse.words}><i class="dot {pulse.tone}" aria-hidden="true"></i>{state}</span>
    </span>
    {#if os}<span class="mc-os">{osName(os)}</span>{/if}
  </span>
  {#if line}<span class="mc-l">{line}</span>{/if}
  <span class="mc-f">
    <span><b>{group.chats.length.toLocaleString()}</b><small>chat{group.chats.length === 1 ? "" : "s"}</small></span>
    <span><b>{group.agents.length.toLocaleString()}</b><small>agent{group.agents.length === 1 ? "" : "s"}</small></span>
  </span>
</a>

<style>
  .mc {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
    color: var(--text-body);
  }
  a.mc:hover {
    color: var(--text-body);
    background: var(--surface-hover);
  }
  .mc.warn {
    border-color: var(--status-warn-dot);
  }
  .mc.quiet {
    background: var(--surface-rail);
    box-shadow: none;
  }
  .mc-h {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .mc-i {
    flex: none;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-5);
    background: var(--surface-muted);
    color: var(--text-strong);
    box-shadow: var(--shadow-pick);
  }
  .mc-n {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .mc-n b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .mc-s {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .mc-s.ok {
    color: var(--status-ok-text);
  }
  .mc-s.warn,
  .mc-s.bad {
    color: var(--status-warn-text);
  }
  .mc-os {
    align-self: flex-start;
    padding-top: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .mc-l {
    font-size: 13px;
    color: var(--text-muted);
  }
  .warn .mc-l {
    color: var(--status-warn-text);
  }
  .mc-f {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    padding-top: 14px;
    border-top: var(--border-divider);
  }
  .mc-f > span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .mc-f b {
    font-size: 17px;
    font-weight: 500;
    color: var(--text-strong);
  }
  .mc-f small {
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
