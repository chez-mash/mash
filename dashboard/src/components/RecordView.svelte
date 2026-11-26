<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import ChevronsUp from "@lucide/svelte/icons/chevrons-up";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import Clock from "@lucide/svelte/icons/clock";
import FileText from "@lucide/svelte/icons/file-text";
import Laptop from "@lucide/svelte/icons/laptop";
import Lightbulb from "@lucide/svelte/icons/lightbulb";
import { createQuery } from "@tanstack/svelte-query";
import { agentNamed } from "../lib/agents";
import { agoLabel } from "../lib/browse";
import { chatOf } from "../lib/memoryView";
import { catalogOptions, stableRead } from "../lib/queries";
import { scopeIcons } from "../lib/recordIcons";
import {
  type Evidence,
  getRecord,
  label,
  type Revision,
  scopeLabel,
  scopeName,
  standing,
  timingLabel,
} from "../lib/records";

import { lifecycleTone, standingTone } from "../lib/tone";
import { updates } from "../lib/updates.svelte";
import AgentMark from "./AgentMark.svelte";
import EvidenceCard from "./EvidenceCard.svelte";
import KindPill from "./KindPill.svelte";
import LoadError from "./LoadError.svelte";
import MachineIcon from "./MachineIcon.svelte";
import RecordFacts from "./RecordFacts.svelte";
import RecordRelated from "./RecordRelated.svelte";
import Skeleton from "./Skeleton.svelte";
import StatusBadge from "./StatusBadge.svelte";
import ToneIcon from "./ToneIcon.svelte";

let { id, node, back }: { id: string; node: string; back: string } = $props();
// The chat list names this machine, as the memory list does.
const catalogue = createQuery(() => catalogOptions);
let before = $state<number | null>(null);
let linkOffset = $state(0);
const latest = createQuery(() => ({
  ...stableRead,
  queryKey: ["record", id, node, null, 0],
  queryFn: ({ signal }) => getRecord(id, node, null, 0, signal),
}));
const history = createQuery(() => ({
  ...stableRead,
  queryKey: ["record-history", id, node, latest.dataUpdatedAt, before, linkOffset],
  enabled: before !== null || linkOffset !== 0,
  queryFn: ({ signal }) => getRecord(id, node, before, linkOffset, signal),
}));
const page = $derived(before === null && linkOffset === 0 ? latest : history);
const record = $derived(latest.data);
const current = $derived(
  record?.revisions.find((revision) => revision.revision === record.currentRevision),
);
async function refresh(): Promise<void> {
  before = null;
  linkOffset = 0;
  await latest.refetch();
}
// The rail's Check for updates returns the history to its first page.
updates.offer({
  check: refresh,
  owns: ([kind, recordId]) => (kind === "record" || kind === "record-history") && recordId === id,
});
function date(value: number | null): string {
  return value === null ? "Unknown" : new Date(value).toLocaleString();
}
</script>

{#snippet ago(value: number)}<time datetime={new Date(value).toISOString()} title={date(value)}>{agoLabel(value, Date.now())}</time>{/snippet}
{#snippet sources(evidence: readonly Evidence[], node: string)}
  <div class="evs">
    {#each evidence as source, index (index)}
      <EvidenceCard {source} {node} chat={chatOf(catalogue.data?.chats, source.chatId, node)} />
    {/each}
  </div>
{/snippet}
{#snippet details(revision: Revision)}
  {#if revision.details.kind === "decision" && (revision.details.alternatives.length || revision.details.consequences.length)}
    <div class="g2 rev-d">
      {#if revision.details.alternatives.length}<div><span class="eyebrow">alternatives</span><ul>{#each revision.details.alternatives as item, index (index)}<li>{item}</li>{/each}</ul></div>{/if}
      {#if revision.details.consequences.length}<div><span class="eyebrow">consequences</span><ul>{#each revision.details.consequences as item, index (index)}<li>{item}</li>{/each}</ul></div>{/if}
    </div>
  {:else if revision.details.kind === "preference" && revision.details.conditions.length}
    <div class="rev-d"><div><span class="eyebrow">conditions</span><ul>{#each revision.details.conditions as item, index (index)}<li>{item}</li>{/each}</ul></div></div>
  {/if}
{/snippet}

<a class="back" href={back} aria-label="Back to memory"><ChevronLeft size={15} aria-hidden="true" />Memory</a>
{#if latest.isPending}<Skeleton rows={4} />{/if}
{#if latest.error}<LoadError title="Couldn’t load this memory" message={latest.error.message} retry={refresh} busy={latest.isFetching}/>{/if}
{#if page.error && !latest.error}<LoadError title="Couldn’t load memory history" message={page.error.message} retry={() => page.refetch()} busy={page.isFetching} retryLabel="Retry history"/>{/if}
{#if record && current && !latest.error && !page.error}
  {@const settled = standing(current)}
  {@const ScopeIcon = scopeIcons[record.scope.kind]}
  {@const recorder = agentNamed(current.recordedBy)}
  <div class="rec">
  <div class="rec-m">
  <header class="rec-h">
    <p class="pills">
      <KindPill kind={current.details.kind} />
      <StatusBadge pill tone={lifecycleTone(current.details.state)} words={label(current.details.state)} size={12} />
      <StatusBadge pill tone={standingTone(settled)} words={label(settled)} size={12} />
    </p>
    <h1 class="h1">{current.statement}</h1>
    <p class="mts">
      <span class="mt" title={scopeLabel(record.scope)}><ScopeIcon size={13} aria-hidden="true" />{scopeName(record.scope)}</span>
      <span class="mt">{#if record.node || catalogue.data?.localNode}<MachineIcon machine={record.node || catalogue.data?.localNode || ""} size={13} />{:else}<Laptop size={13} aria-hidden="true" />{/if}{record.node || catalogue.data?.localNode || "this machine"}</span>
      <span class="mt">{#if recorder}<AgentMark agent={recorder} size="sm" />{/if}Recorded by {current.recordedBy}</span>
      <span class="mt">{@render ago(current.recordedAt)}</span>
    </p>
    {#if settled === "unconfirmed"}
      <!-- The dashboard only reads memory: there is no way to confirm a record
           from here. An agent confirms or corrects it in a review. -->
      <p class="card tight cream rec-u" role="note"><span class="tone-i tone-warn"><CircleDashed size={16} aria-hidden="true" /></span><span>To confirm this, ask your agent to review your memory (<code>/mash:review</code> in Claude Code); opening it here leaves it unconfirmed. <a class="a" href="https://getmash.io/records/#correct-or-replace" target="_blank" rel="noreferrer">How review works</a></span></p>
    {/if}
    {#if timingLabel(current, Date.now()) || record.conflicted || !current.evidenceCurrent}
      <div class="g3 rec-w">
        {#if timingLabel(current, Date.now())}<p class="card tight warn" role="status"><ToneIcon tone="warn" size={16} /><span>{timingLabel(current, Date.now())}. Check the effective date before relying on this.</span></p>{/if}
        {#if record.conflicted}<p class="card tight bad" role="status"><ToneIcon tone="bad" size={16} /><span>Another record contradicts this one. Look at the related records before relying on it.</span></p>{/if}
        {#if !current.evidenceCurrent}<p class="card tight bad" role="status"><ToneIcon tone="bad" size={16} /><span>The quoted messages have changed or are missing. Review the sources before relying on this.</span></p>{/if}
      </div>
    {/if}
  </header>
  {#if current.rationale}<section class="sec"><h2 class="h2"><Lightbulb size={15} aria-hidden="true" />Why</h2><p class="rec-why">{current.rationale}</p></section>{/if}
  <section class="sec">
    <h2 class="h2"><FileText size={15} aria-hidden="true" />Evidence{current.evidenceCurrent ? "" : " (stale or missing)"}</h2>
    {@render sources(current.evidence, record.node)}
  </section>
  <section class="sec">
    <div class="hd"><h2 class="h2"><Clock size={15} aria-hidden="true" />Revision history</h2></div>
    {#if page.isFetching}<p class="meta" role="status">Loading history…</p>{/if}
    {#if page.data}
      {#if !page.data.revisions.length}<p class="meta">No revisions on this page.</p>{/if}
      {#if page.data.revisions.length}
        <ol class="rec-tl">
          {#each page.data.revisions as revision (revision.revision)}
            <li class="rev" class:cur={revision.revision === record.currentRevision}>
              <span class="pip" aria-hidden="true"></span>
              <div class="rev-t"><b>Revision {revision.revision}{revision.revision === record.currentRevision ? " (current)" : ""}: {label(revision.details.state).toLowerCase()}, {standing(revision)}</b><span class="meta">{@render ago(revision.recordedAt)}</span></div>
              <p class="meta rev-by">{revision.recordedBy || "An agent"}{revision.effectiveAt === null ? "" : ` · applies from ${date(revision.effectiveAt)}`}</p>
              {#if revision.revision !== record.currentRevision}
                <p class="rev-s">{revision.statement}</p>
                {#if revision.rationale}<p class="meta" style:white-space="pre-wrap" style:overflow-wrap="anywhere">{revision.rationale}</p>{/if}
              {/if}
              {@render details(revision)}
              {#if revision.revision !== record.currentRevision && revision.evidence.length}
                <span class="eyebrow">evidence{revision.evidenceCurrent ? "" : " (stale or missing)"}</span>
                {@render sources(revision.evidence, record.node)}
              {/if}
            </li>
          {/each}
        </ol>
      {/if}
      {#if before !== null || page.data.nextBeforeRevision !== null}
        <nav aria-label="Revision pages" class="acts">
          {#if before !== null}<button class="btn sm" type="button" onclick={() => (before = null)}><ChevronsUp size={14} aria-hidden="true" />Newest revisions</button>{/if}
          {#if page.data.nextBeforeRevision !== null}<button class="btn sm" type="button" onclick={() => (before = page.data?.nextBeforeRevision ?? null)}>Older revisions<ChevronDown size={14} aria-hidden="true" /></button>{/if}
        </nav>
      {/if}
    {/if}
  </section>
  </div>
  <aside class="rec-s" aria-label="About this memory">
    <RecordFacts {record} revision={current} machine={record.node || catalogue.data?.localNode || "this machine"} />
    {#if page.data && (page.data.links.length || linkOffset > 0)}
    <section class="sec">
      <h2 class="h2 rec-l">Related records</h2>
      {#if !page.data.links.length}<p class="meta">No relationships on this page.</p>{/if}
      {#if page.data.links.length}<RecordRelated {record} links={page.data.links} {back} />{/if}
      {#if linkOffset > 0 || page.data.nextLinkOffset !== null}
        <nav aria-label="Relationship pages" class="acts">
          {#if linkOffset > 0}<button class="btn sm" type="button" onclick={() => (linkOffset = 0)}><ChevronsUp size={14} aria-hidden="true" />First relationships</button>{/if}
          {#if page.data.nextLinkOffset !== null}<button class="btn sm" type="button" onclick={() => (linkOffset = page.data?.nextLinkOffset ?? 0)}>More relationships<ChevronDown size={14} aria-hidden="true" /></button>{/if}
        </nav>
      {/if}
    </section>
  {/if}
  </aside>
  </div>
{/if}

<style>
  /* The memory and its history, with what is known about it beside them. */
  .rec {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(240px, 282px);
    gap: 24px 40px;
    align-items: start;
  }
  .rec-m,
  .rec-s {
    display: flex;
    flex-direction: column;
    gap: 26px;
    min-width: 0;
  }
  .rec-s {
    gap: 18px;
  }
  @container (max-width: 760px) {
    .rec {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .rec-l {
    font-size: 12px;
    font-weight: var(--weight-label);
    color: var(--text-faint);
  }
  .evs {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  /* Revisions as a timeline: a dot per revision on a line down the left,
     butter for the current one. */
  .rec-tl {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .rec-tl > .rev {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 0 22px 26px;
    border: 0;
  }
  .rec-tl > .rev::before {
    content: "";
    position: absolute;
    top: 14px;
    bottom: 0;
    left: 4px;
    border-left: var(--border-divider);
  }
  .rec-tl > .rev:last-child::before {
    display: none;
  }
  .pip {
    position: absolute;
    top: 5px;
    left: 0;
    width: 9px;
    height: 9px;
    border-radius: var(--radius-round);
    background: var(--line-strong);
  }
  .cur > .pip {
    background: var(--find-current-bg);
    box-shadow: 0 0 8px var(--find-current-bg);
  }
  .rec-tl :global(.rev-t) {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 4px 12px;
  }
  .rec-tl :global(.rev-t b) {
    font-size: 13.5px;
    font-weight: var(--weight-label);
    color: var(--text-strong);
  }
  .rev-by {
    margin: -2px 0 0;
    font-size: 12.5px;
  }
</style>
