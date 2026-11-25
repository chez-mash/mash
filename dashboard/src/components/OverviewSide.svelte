<script lang="ts">
// The overview's side column: memories waiting for a review (or, when none
// are, the latest ones), then the machines and agents the history comes from.
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import { agents as agentInfo, agentNamed } from "../lib/agents";
import type { Group } from "../lib/browse";
import { whenLabel } from "../lib/browse";
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machinePulse } from "../lib/machinePulse";
import { label, type NativeRecord } from "../lib/records";
import { agentHref, machineHref, recordHref } from "../lib/route.svelte";
import AgentMark from "./AgentMark.svelte";
import MemoryRow from "./MemoryRow.svelte";
import Skeleton from "./Skeleton.svelte";

let {
  unconfirmed,
  recent,
  memoriesPending,
  machines,
  providers,
  localNode,
}: {
  /** Memories agents saved that nobody has confirmed yet. */
  unconfirmed: readonly NativeRecord[];
  /** The latest memories, shown when none are waiting. */
  recent: readonly NativeRecord[] | undefined;
  memoriesPending: boolean;
  machines: readonly Group[];
  providers: readonly Group[];
  localNode: string;
} = $props();
const evidence = createMachineEvidence();
const now = Date.now();
function current(record: NativeRecord) {
  return (
    record.revisions.find((revision) => revision.revision === record.currentRevision) ??
    record.revisions[0]
  );
}
function source(record: NativeRecord): string {
  const revision = current(record);
  if (!revision) return "";
  const agent = agentNamed(revision.recordedBy);
  const who = agent ? agentInfo[agent].name : revision.recordedBy;
  return `${label(revision.details.kind)} · ${who}, ${whenLabel(revision.recordedAt, now)}`;
}
/** A machine's state in a word or two, from the evidence its dot shows. */
function state(machine: string): { words: string; title: string } {
  if (machine === localNode) return { words: "this machine", title: "This machine" };
  const pulse = machinePulse(machine, evidence);
  const first = pulse.words.split(" · ")[0] ?? pulse.words;
  return { words: first.toLowerCase(), title: pulse.words };
}
</script>

<div class="os">
  {#if unconfirmed.length}
    <section class="os-sec">
      <div class="os-h"><h2>Unconfirmed</h2><a href="#/records?review=candidate">Memory<ChevronRight size={13} aria-hidden="true" /></a></div>
      {#each unconfirmed.slice(0, 3) as record (record.node + record.id)}
        {@const revision = current(record)}
        {#if revision}
          <a class="os-card os-mem" href={recordHref(record.id, record.node, "#/")}><span class="os-mem-t">{revision.statement}</span><span class="os-q">{source(record)}</span></a>
        {/if}
      {/each}
      <p class="os-note">Agents confirm these in a review: ask your agent to run <code>/mash:review</code>.</p>
    </section>
  {:else}
    <section class="os-sec">
      <div class="os-h"><h2>Recent memories</h2><a href="#/records?">Memory<ChevronRight size={13} aria-hidden="true" /></a></div>
      {#if memoriesPending}<div class="card pad0"><Skeleton rows={3} /></div>{/if}
      {#if recent?.length}
        <div class="card pad0 list">
          {#each recent.slice(0, 4) as record (record.node + record.id)}<MemoryRow {record} />{/each}
        </div>
      {:else if recent}
        <p class="card tight cream">Nothing saved yet. Agents save decisions and preferences as you state them, and notes when you ask.</p>
      {/if}
    </section>
  {/if}
  {#if machines.length}
    <section class="os-sec">
      <div class="os-h"><h2>Machines</h2><a href="#/machines">Manage<ChevronRight size={13} aria-hidden="true" /></a></div>
      <div class="os-card os-tally">
        {#each machines as machine (machine.key)}
          {@const said = state(machine.key)}
          {@const pulse = machinePulse(machine.key, evidence)}
          <a class="os-li" href={machineHref(machine.key)}><i class="dot {pulse.tone}" aria-hidden="true" title={pulse.words}></i><b>{machine.key}</b><span class="os-q" title={said.title}>{said.words}</span><span class="os-n">{machine.chats.length.toLocaleString()}</span></a>
        {/each}
      </div>
    </section>
  {/if}
  {#if providers.length}
    <section class="os-sec">
      <div class="os-h"><h2>Agents</h2></div>
      <div class="os-agents">
        {#each providers as provider (provider.key)}
          {@const agent = provider.chats[0]?.agent}
          {#if agent}<a class="os-card os-agent" href={agentHref(agent)}><AgentMark {agent} size="sm" /><b>{agentInfo[agent].name}</b><span class="os-n">{provider.chats.length.toLocaleString()}</span></a>{/if}
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .os {
    display: flex;
    flex-direction: column;
    gap: 24px;
    min-width: 0;
  }
  .os-sec {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .os-h {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .os-h h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .os-h a {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .os-h a:hover {
    color: var(--text-strong);
  }
  .os-card {
    border-radius: var(--radius-8);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-outline),
      var(--shadow-panel);
    color: var(--text-strong);
  }
  .os-mem {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px;
  }
  .os-mem:hover {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
  .os-mem-t {
    font-size: 13px;
    line-height: 1.5;
    font-weight: 500;
  }
  .os-q {
    font-size: 12px;
    color: var(--text-faint);
  }
  .os-note {
    margin: 2px 2px 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-faint);
  }
  .os-note code {
    font: 11px var(--type-mono);
    color: var(--text-muted);
  }
  .os-tally {
    padding: 6px 14px;
  }
  .os-li {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 34px;
    font-size: 13px;
    color: var(--text-strong);
  }
  .os-li + .os-li {
    border-top: var(--border-divider);
  }
  .os-li b {
    flex: 1;
    min-width: 0;
    font-weight: 400;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .os-li:hover b {
    text-decoration: underline;
  }
  .os-n {
    flex: none;
    min-width: 36px;
    font: 12px var(--type-mono);
    color: var(--text-muted);
    text-align: right;
  }
  .os-agents {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }
  .os-agent {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 40px;
    padding: 10px 12px;
    border-radius: var(--radius-7);
    font-size: 13px;
  }
  .os-agent:hover {
    color: var(--text-strong);
    background: var(--surface-hover);
  }
  .os-agent b {
    flex: 1;
    min-width: 0;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  @media (max-width: 640px) {
    .os-li {
      min-height: 52px;
      font-size: 15px;
    }
  }
</style>
