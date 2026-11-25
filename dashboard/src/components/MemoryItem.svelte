<script lang="ts">
import { agentNamed } from "../lib/agents";
import { whenLabel } from "../lib/browse";
import { currentOf } from "../lib/memoryView";
import { kindIcons } from "../lib/recordIcons";
import {
  label,
  type NativeRecord,
  scopeLabel,
  scopeName,
  standing,
  statePill,
  timingLabel,
} from "../lib/records";
import { recordHref } from "../lib/route.svelte";
import AgentMark from "./AgentMark.svelte";
import MachineIcon from "./MachineIcon.svelte";

// One memory in the memory page's list: its kind on a tile, the statement,
// then its kind, who saved it, what it applies to and the machine, with the
// time at the end. Anything that should give the reader pause (not yet
// confirmed, quotes changed, a conflict, replaced) is a pill under that.
// Pointing at a row or reaching it by keyboard shows it in the preview.
let {
  record,
  back,
  machine,
  now,
  selected = false,
  preview,
}: {
  record: NativeRecord;
  back: string;
  /** The machine to name when the record carries none: this one. */
  machine: string;
  now: number;
  selected?: boolean;
  preview?: () => void;
} = $props();
const revision = $derived(currentOf(record));
</script>

{#if revision}
  {@const Icon = kindIcons[revision.details.kind]}
  {@const settled = standing(revision)}
  {@const recorder = agentNamed(revision.recordedBy)}
  {@const timing = timingLabel(revision, now)}
  {@const where = record.node || machine}
  {@const ended = statePill(revision.details.state) !== "ok"}
  <article class="mi" class:on={selected} aria-current={selected ? "true" : undefined}>
    <span class="tile" aria-hidden="true"><Icon size={15} /></span>
    <span class="body">
      <span class="top"><span class="kindp"><Icon size={12} aria-hidden="true" />{label(revision.details.kind).toLowerCase()}</span><span class="when">{whenLabel(revision.recordedAt, now)}</span></span>
      <a href={recordHref(record.id, record.node, back)} onpointerenter={preview} onfocus={preview}>{revision.statement}</a>
      <span class="meta-l">
        <span class="mi-kind">{label(revision.details.kind)}</span>
        <span class="sep" aria-hidden="true">·</span>
        {#if revision.recordedBy}<span class="mi-who">{#if recorder}<AgentMark agent={recorder} size="sm" />{/if}{revision.recordedBy}</span><span class="sep" aria-hidden="true">·</span>{/if}
        <span title={scopeLabel(record.scope)}>{scopeName(record.scope)}</span>
        {#if where}<span class="sep" aria-hidden="true">·</span><span class="mi-who"><MachineIcon machine={where} size={12} />{where}</span>{/if}
      </span>
      {#if ended || settled !== "established" || !revision.evidenceCurrent || record.conflicted || timing}
        <span class="pills">
          {#if ended}<span class="pill {statePill(revision.details.state)}">{label(revision.details.state).toLowerCase()}</span>{/if}
          {#if settled !== "established"}<span class="pill warn">{settled}</span>{/if}
          {#if !revision.evidenceCurrent}<span class="pill bad">stale evidence</span>{/if}
          {#if record.conflicted}<span class="pill bad">conflicting records</span>{/if}
          {#if timing}<span class="pill warn">{timing.toLowerCase()}</span>{/if}
        </span>
      {/if}
    </span>
    <span class="when side">{whenLabel(revision.recordedAt, now)}</span>
  </article>
{/if}

<style>
  .mi {
    position: relative;
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    padding: 11px 12px;
    border-radius: var(--radius-6);
    color: var(--text-strong);
  }
  .mi:hover,
  .mi.on {
    background: var(--selected-surface);
  }
  .mi:has(a:focus-visible) {
    outline: 2px solid var(--focus);
    outline-offset: -2px;
  }
  .tile {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-5);
    background: var(--surface-raised);
    color: var(--text-body);
    box-shadow: var(--shadow-pick);
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  /* The statement is the row's link; it covers the whole row. */
  a {
    font-size: 14px;
    font-weight: 500;
    line-height: 1.35;
    color: var(--text-strong);
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }
  .on a {
    font-weight: 600;
  }
  a:focus-visible {
    outline: 0;
  }
  a::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
  }
  .meta-l {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 2px 6px;
    min-width: 0;
    font-size: 12px;
    color: var(--text-faint);
  }
  .mi-who {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .mi-who :global(.mark.sm) {
    width: 14px;
    height: 14px;
  }
  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .when {
    font-size: 12px;
    color: var(--text-faint);
    white-space: nowrap;
  }
  .top {
    display: none;
  }
  .kindp {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--radius-pill);
    background: var(--surface-muted);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-body);
  }
  /* On a phone a row is a card row: the kind as a pill and the time over the
     statement, the tile and the kind's word left out. */
  @container (max-width: 600px) {
    .mi {
      grid-template-columns: minmax(0, 1fr);
      padding: 13px 14px;
      border-radius: 0;
    }
    /* Without the preview beside the list, no row is singled out. */
    .mi.on:not(:hover) {
      background: transparent;
    }
    .on a {
      font-weight: 500;
    }
    .tile,
    .side,
    .mi-kind,
    .mi-kind + .sep {
      display: none;
    }
    .top {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
    }
    a {
      font-size: 14.5px;
    }
  }
</style>
