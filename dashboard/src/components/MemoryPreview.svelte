<script lang="ts">
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import Icon from "$lib/components/Icon.svelte";
import { agoLabel } from "../lib/browse";
import { chatOf, currentOf } from "../lib/memoryView";
import { kindIcons } from "../lib/recordIcons";
import { label, type NativeRecord, standing } from "../lib/records";
import { recordHref } from "../lib/route.svelte";
import { lifecycleTone, standingTone } from "../lib/tone";
import type { ChatSummary } from "../lib/types";
import EvidenceCard from "./EvidenceCard.svelte";
import StatusBadge from "./StatusBadge.svelte";

// The memory the reader is pointing at in the list, beside it: what it says
// and pv-why, the first message behind it, and its revisions, with the way to
// its own page. Only what the list already read is shown.
let {
  record,
  back,
  chats,
  now,
}: {
  record: NativeRecord;
  back: string;
  chats: readonly ChatSummary[] | undefined;
  now: number;
} = $props();
const current = $derived(currentOf(record));
const EVIDENCE = 2;
</script>

{#if current}
  {@const Glyph = kindIcons[current.details.kind]}
  <aside class="pv" aria-label="Selected memory">
    <div class="pv-h">
      <span class="pv-k"><span class="ink-{current.details.kind} pv-ki"><Icon icon={Glyph} size={14} /></span>{label(current.details.kind)}</span>
      <StatusBadge tone={lifecycleTone(current.details.state)} words={label(current.details.state)} size={12} />
      <StatusBadge tone={standingTone(standing(current))} words={standing(current)} size={12} />
    </div>
    <p class="pv-st">{current.statement}</p>
    {#if current.rationale}<p class="pv-why">{current.rationale}</p>{/if}
    {#if current.evidence.length}
      <div class="pv-s">
        <span class="lbl">Evidence</span>
        {#each current.evidence.slice(0, EVIDENCE) as source, index (index)}
          <EvidenceCard {source} node={record.node} chat={chatOf(chats, source.chatId, record.node)} brief />
        {/each}
        {#if current.evidence.length > EVIDENCE}<span class="meta">{current.evidence.length - EVIDENCE} more on its page</span>{/if}
      </div>
    {/if}
    <div class="pv-s">
      <span class="lbl">History</span>
      <ol class="hist">
        {#each record.revisions as revision (revision.revision)}
          {@const latest = revision.revision === record.currentRevision}
          <li class:cur={latest}>
            <span class="pip" aria-hidden="true"></span>
            <span><b>Revision {revision.revision}{latest ? " · current" : ""}</b><small>Saved by {revision.recordedBy || "an agent"} · {agoLabel(revision.recordedAt, now)}</small></span>
          </li>
        {/each}
      </ol>
    </div>
    <a class="pv-open" href={recordHref(record.id, record.node, back)}>Open record<Icon icon={CaretRightIcon} size={14} /></a>
  </aside>
{/if}

<style>
  /* It fills the memory page's side panel, which draws the edge and fill. */
  .pv {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px 24px 32px;
    color: var(--text-body);
  }
  .pv-h {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }
  .pv-h .meta {
    font-size: 12px;
  }
  .pv-k {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 26px;
    padding: 0 10px 0 8px;
    border: var(--stroke-outline) solid var(--linked-line);
    border-radius: var(--radius-5);
    background: var(--surface-raised);
    font-size: 13px;
    font-weight: var(--weight-label);
    color: var(--text-strong);
  }
  .pv-ki {
    display: inline-flex;
  }
  .pv-h :global(.st) {
    font-size: 12px;
  }
  .pv-st {
    font-size: 20px;
    font-weight: var(--weight-strong);
    line-height: 1.35;
    letter-spacing: -0.02em;
    color: var(--text-strong);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .pv-why {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text-muted);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .pv-s {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .lbl {
    font-size: 12px;
    font-weight: var(--weight-label);
    color: var(--text-faint);
  }
  .hist {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin: 4px 0 0;
    padding: 0;
    list-style: none;
  }
  .hist li {
    display: grid;
    grid-template-columns: 12px 1fr;
    gap: 10px;
    align-items: start;
  }
  .hist .pip {
    width: 8px;
    height: 8px;
    margin-top: 4px;
    border: 1.5px solid var(--text-faint);
    border-radius: var(--radius-round);
  }
  .hist .cur .pip {
    border-color: var(--find-current-bg);
    background: var(--find-current-bg);
    box-shadow: 0 0 8px var(--find-current-bg);
  }
  .hist b {
    display: block;
    font-size: 13px;
    font-weight: var(--weight-label);
    line-height: 1.3;
    color: var(--text-strong);
  }
  .hist small {
    display: block;
    margin-top: 3px;
    font-size: 12px;
    line-height: 1.3;
    color: var(--text-muted);
  }
  .pv-open {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    align-self: flex-start;
    font-size: 13px;
    font-weight: var(--weight-label);
    color: var(--text-strong);
  }
  .pv-open:hover {
    color: var(--accent);
  }
</style>
