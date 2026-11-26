<script lang="ts">
// The overview's side column: memories waiting for a review (or, when none
// are, the latest ones), then the machines and agents the history comes from.

import Bookmark from "@lucide/svelte/icons/bookmark";
import Bot from "@lucide/svelte/icons/bot";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import Laptop from "@lucide/svelte/icons/laptop";
import { agents as agentInfo, agentNamed } from "../lib/agents";
import type { Group } from "../lib/browse";
import { whenLabel } from "../lib/browse";
import { createMachineEvidence } from "../lib/machineEvidence.svelte";
import { machinePulse } from "../lib/machinePulse";
import type { NativeRecord } from "../lib/records";
import { agentHref, machineHref, recordHref } from "../lib/route.svelte";
import { pulseTone } from "../lib/tone";
import AgentMark from "./AgentMark.svelte";
import KindIcon from "./KindIcon.svelte";
import MachineIcon from "./MachineIcon.svelte";
import MemoryRow from "./MemoryRow.svelte";
import Skeleton from "./Skeleton.svelte";
import ToneIcon from "./ToneIcon.svelte";

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
/** Who saved a memory, by agent when the name is one, and how long ago. */
function source(record: NativeRecord) {
  const revision = current(record);
  if (!revision) return undefined;
  const agent = agentNamed(revision.recordedBy);
  return {
    kind: revision.details.kind,
    agent,
    who: agent ? agentInfo[agent].name : revision.recordedBy,
    when: whenLabel(revision.recordedAt, now),
  };
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
      <div class="os-h"><h2><CircleDashed class="os-hw" size={15} aria-hidden="true" />Unconfirmed</h2><a href="#/records?review=candidate">Memory<ChevronRight size={13} aria-hidden="true" /></a></div>
      {#each unconfirmed.slice(0, 3) as record (record.node + record.id)}
        {@const revision = current(record)}
        {@const from = source(record)}
        {#if revision}
          <a class="os-card os-mem" href={recordHref(record.id, record.node, "#/")}><span class="os-mem-t">{revision.statement}</span>{#if from}<span class="os-q os-src"><KindIcon kind={from.kind} /><span class="os-who">{#if from.agent}<AgentMark agent={from.agent} size="sm" />{/if}{from.who}</span><span>{from.when}</span></span>{/if}</a>
        {/if}
      {/each}
      <p class="os-note">Agents confirm these in a review: ask your agent to run <code>/mash:review</code>.</p>
    </section>
  {:else}
    <section class="os-sec">
      <div class="os-h"><h2><Bookmark size={15} aria-hidden="true" />Recent memories</h2><a href="#/records?">Memory<ChevronRight size={13} aria-hidden="true" /></a></div>
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
      <div class="os-h"><h2><Laptop size={15} aria-hidden="true" />Machines</h2><a href="#/machines">Manage<ChevronRight size={13} aria-hidden="true" /></a></div>
      <div class="os-card os-tally">
        {#each machines as machine (machine.key)}
          {@const said = state(machine.key)}
          {@const pulse = machinePulse(machine.key, evidence)}
          <a class="os-li" href={machineHref(machine.key)}><span title={pulse.words}><ToneIcon tone={pulseTone(pulse)} size={13} /></span><span class="os-mt"><b><MachineIcon machine={machine.key} size={13} />{machine.key}</b><span class="os-q tone-{pulseTone(pulse)}" title={said.title}>{said.words}</span></span><span class="os-n">{machine.chats.length.toLocaleString()}<span class="os-u">{" chats"}</span></span></a>
        {/each}
      </div>
    </section>
  {/if}
  {#if providers.length}
    <section class="os-sec">
      <div class="os-h"><h2><Bot size={15} aria-hidden="true" />Agents</h2></div>
      <div class="os-agents">
        {#each providers as provider (provider.key)}
          {@const agent = provider.chats[0]?.agent}
          {#if agent}<a class="os-card os-agent" href={agentHref(agent)}><AgentMark {agent} size="sm" /><span class="os-at"><b>{agentInfo[agent].name}</b><span class="os-u">{provider.chats.length.toLocaleString()} chats</span></span><span class="os-n">{provider.chats.length.toLocaleString()}</span></a>{/if}
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
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 14px;
    font-weight: var(--weight-strong);
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
    font-weight: var(--weight-label);
  }
  .os-q {
    font-size: 12px;
    color: var(--text-faint);
  }
  .os-src,
  .os-who {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
  }
  .os-who {
    gap: 5px;
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
  /* A machine's name and its state share a line beside the rail; on a
     phone the state goes under the name. */
  .os-mt {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .os-q.tone-warn {
    color: var(--status-warn-text);
  }
  .os-q.tone-bad {
    color: var(--status-bad-text);
  }
  .os-li b {
    flex: 1;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: var(--weight-label);
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
  /* An agent's name is never cut short; a long one wraps. */
  .os-at {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .os-agent b {
    font-weight: var(--weight-label);
    overflow-wrap: anywhere;
  }
  .os-u {
    display: none;
  }
  /* Section icons show only on a phone, as the phone wireframe draws them. */
  .os-h h2 :global(svg) {
    display: none;
    color: var(--text-muted);
  }
  @media (max-width: 640px) {
    .os-h h2 :global(svg) {
      display: block;
    }
    .os-h h2 :global(svg.os-hw) {
      color: var(--status-warn-text);
    }
    .os-li {
      min-height: 54px;
      font-size: 15px;
    }
    .os-mt {
      flex-direction: column;
      align-items: flex-start;
      gap: 2px;
    }
    .os-li b {
      flex: none;
      max-width: 100%;
    }
    .os-n {
      font: 12.5px var(--type-sans);
    }
    .os-u {
      display: inline;
    }
    .os-agent {
      min-height: 56px;
      font-size: 14px;
    }
    .os-agent > .os-n {
      display: none;
    }
    .os-at .os-u {
      font-size: 12px;
      color: var(--text-muted);
    }
  }
</style>
