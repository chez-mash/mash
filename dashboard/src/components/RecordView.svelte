<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import ChevronsUp from "@lucide/svelte/icons/chevrons-up";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import Clock from "@lucide/svelte/icons/clock";
import FileText from "@lucide/svelte/icons/file-text";
import Laptop from "@lucide/svelte/icons/laptop";
import Lightbulb from "@lucide/svelte/icons/lightbulb";
import Link2 from "@lucide/svelte/icons/link-2";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery } from "@tanstack/svelte-query";
import { agentNamed } from "../lib/agents";
import { agoLabel } from "../lib/browse";
import { catalogOptions, stableRead } from "../lib/queries";
import { kindIcons, scopeIcons } from "../lib/recordIcons";
import {
  type Evidence,
  getRecord,
  kindPill,
  label,
  type Revision,
  scopeLabel,
  scopeName,
  standing,
  statePill,
  timingLabel,
} from "../lib/records";
import { chatHref, recordHref } from "../lib/route.svelte";
import { updates } from "../lib/updates.svelte";
import AgentMark from "./AgentMark.svelte";
import LoadError from "./LoadError.svelte";
import Skeleton from "./Skeleton.svelte";

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
{#snippet sources(evidence: readonly Evidence[], node: string, owner: string)}
  {#each evidence as source, index (index)}
    <figure class="card butter quote">
      <blockquote>“{source.quote}”</blockquote>
      <figcaption>
        <span class="who">{#if source.sourceRole === "user"}<span class="me" aria-hidden="true">{(owner[0] ?? "Y").toUpperCase()}</span>{/if}{source.sourceRole === "user" ? "you said this" : `${source.sourceRole || "someone"} said this`}{source.purpose ? ` (${label(source.purpose).toLowerCase()})` : ""}</span>
        <a class="a" href={chatHref(source.chatId, node, source.messageId, Boolean(node))}>open the message</a>
      </figcaption>
    </figure>
  {/each}
{/snippet}
{#snippet details(revision: Revision)}
  {#if revision.details.kind === "decision" && (revision.details.alternatives.length || revision.details.consequences.length)}
    <div class="g2 rev-d">
      {#if revision.details.alternatives.length}<div><span class="eyebrow">alternatives</span><ul>{#each revision.details.alternatives as item, index (index)}<li>{item}</li>{/each}</ul></div>{/if}
      {#if revision.details.consequences.length}<div><span class="eyebrow">consequences</span><ul>{#each revision.details.consequences as item, index (index)}<li>{item}</li>{/each}</ul></div>{/if}
    </div>
  {:else if revision.details.kind === "preference" && revision.details.conditions.length}
    <div class="rev-d"><div><span class="eyebrow">conditions</span><ul>{#each revision.details.conditions as item, index (index)}<li>{item}</li>{/each}</ul></div></div>
  {:else if revision.details.kind === "fact"}
    <p class="meta">reported by {revision.details.reportedBy}{revision.details.validUntil === null ? "" : ` / valid until ${date(revision.details.validUntil)}`}</p>
  {/if}
{/snippet}

<a class="back" href={back} aria-label="Back to memory"><ChevronLeft size={15} aria-hidden="true" />memory</a>
{#if latest.isPending}<Skeleton rows={4} />{/if}
{#if latest.error}<LoadError title="Couldn’t load this memory" message={latest.error.message} retry={refresh} busy={latest.isFetching}/>{/if}
{#if page.error && !latest.error}<LoadError title="Couldn’t load memory history" message={page.error.message} retry={() => page.refetch()} busy={page.isFetching} retryLabel="Retry history"/>{/if}
{#if record && current && !latest.error && !page.error}
  {@const settled = standing(current)}
  {@const Kind = kindIcons[current.details.kind]}
  {@const ScopeIcon = scopeIcons[record.scope.kind]}
  {@const recorder = agentNamed(current.recordedBy)}
  <header class="rec-h">
    <p class="pills">
      <span class="pill {kindPill(current.details.kind)}"><Kind size={12} aria-hidden="true" />{label(current.details.kind).toLowerCase()}</span>
      <span class="pill {statePill(current.details.state)}">{label(current.details.state).toLowerCase()}</span>
      <span class="pill {settled === 'established' ? '' : 'warn'}">{settled}</span>
    </p>
    <h1 class="h1">{current.statement}</h1>
    <p class="mts">
      <span class="mt" title={scopeLabel(record.scope)}><ScopeIcon size={13} aria-hidden="true" />{scopeName(record.scope)}</span>
      <span class="mt"><Laptop size={13} aria-hidden="true" />{record.node || catalogue.data?.localNode || "this machine"}</span>
      <span class="mt">{#if recorder}<AgentMark agent={recorder} size="sm" />{/if}recorded by {current.recordedBy}</span>
      <span class="mt">{@render ago(current.recordedAt)}</span>
    </p>
    {#if settled === "unconfirmed"}
      <!-- The dashboard only reads memory: there is no way to confirm a record
           from here. An agent confirms or corrects it in a review. -->
      <p class="card tight cream rec-u" role="note"><CircleDashed size={16} aria-hidden="true" /><span>To confirm this, ask your agent to review your memory (<code>/mash:review</code> in Claude Code); opening it here leaves it unconfirmed. <a class="a" href="https://getmash.io/records/#correct-or-replace" target="_blank" rel="noreferrer">How review works</a></span></p>
    {/if}
    {#if timingLabel(current, Date.now()) || record.conflicted || !current.evidenceCurrent}
      <div class="g3 rec-w">
        {#if timingLabel(current, Date.now())}<p class="card tight warn" role="status"><TriangleAlert size={16} aria-hidden="true" /><span>{timingLabel(current, Date.now())}. Check the effective and expiry dates before relying on this.</span></p>{/if}
        {#if record.conflicted}<p class="card tight bad" role="status"><TriangleAlert size={16} aria-hidden="true" /><span>Another record contradicts this one. Look at the related records before relying on it.</span></p>{/if}
        {#if !current.evidenceCurrent}<p class="card tight bad" role="status"><TriangleAlert size={16} aria-hidden="true" /><span>The quoted messages have changed or are missing. Review the sources before relying on this.</span></p>{/if}
      </div>
    {/if}
  </header>
  {#if current.rationale}<section class="sec"><h2 class="h2"><Lightbulb size={15} aria-hidden="true" />why</h2><p class="rec-why">{current.rationale}</p></section>{/if}
  <section class="sec">
    <h2 class="h2"><FileText size={15} aria-hidden="true" />evidence{current.evidenceCurrent ? "" : " (stale or missing)"}</h2>
    {@render sources(current.evidence, record.node, record.owner)}
  </section>
  <section class="sec">
    <div class="hd"><h2 class="h2"><Clock size={15} aria-hidden="true" />revision history</h2></div>
    {#if page.isFetching}<p class="meta" role="status">Loading history…</p>{/if}
    {#if page.data}
      {#if !page.data.revisions.length}<p class="meta">No revisions on this page.</p>{/if}
      {#if page.data.revisions.length}
        <div class="card pad0 list">
          {#each page.data.revisions as revision (revision.revision)}
            <div class="rev">
              <div class="rev-t"><b>Revision {revision.revision}{revision.revision === record.currentRevision ? " (current)" : ""}: {label(revision.details.state).toLowerCase()}, {standing(revision)}</b><span class="meta">{@render ago(revision.recordedAt)} / {revision.recordedBy}{revision.effectiveAt === null ? "" : ` / applies from ${date(revision.effectiveAt)}`}</span></div>
              {#if revision.revision !== record.currentRevision}
                <p class="rev-s">{revision.statement}</p>
                {#if revision.rationale}<p class="meta" style:white-space="pre-wrap" style:overflow-wrap="anywhere">{revision.rationale}</p>{/if}
              {/if}
              {@render details(revision)}
              {#if revision.revision !== record.currentRevision && revision.evidence.length}
                <span class="eyebrow">evidence{revision.evidenceCurrent ? "" : " (stale or missing)"}</span>
                {@render sources(revision.evidence, record.node, record.owner)}
              {/if}
            </div>
          {/each}
        </div>
      {/if}
      {#if before !== null || page.data.nextBeforeRevision !== null}
        <nav aria-label="Revision pages" class="acts">
          {#if before !== null}<button class="btn sm" type="button" onclick={() => (before = null)}><ChevronsUp size={14} aria-hidden="true" />Newest revisions</button>{/if}
          {#if page.data.nextBeforeRevision !== null}<button class="btn sm" type="button" onclick={() => (before = page.data?.nextBeforeRevision ?? null)}>Older revisions<ChevronDown size={14} aria-hidden="true" /></button>{/if}
        </nav>
      {/if}
    {/if}
  </section>
  {#if page.data && (page.data.links.length || linkOffset > 0)}
    <section class="sec">
      <h2 class="h2"><Link2 size={15} aria-hidden="true" />related records</h2>
      {#if !page.data.links.length}<p class="meta">No relationships on this page.</p>{/if}
      {#if page.data.links.length}
        <div class="card pad0 list">
          {#each page.data.links as link (JSON.stringify(link))}
            {@const outgoing = link.sourceId === record.id}
            {@const related = outgoing ? link.targetId : link.sourceId}
            <a class="lrow" href={recordHref(related, record.node, back)}><span class="pill {link.relation === 'supports' ? 'ok' : link.relation === 'contradicts' ? 'bad' : ''}">{outgoing ? `this record ${link.relation}` : `${link.relation} this record`}</span><span class="sp mono">…{related.slice(-12)}</span></a>
          {/each}
        </div>
      {/if}
      {#if linkOffset > 0 || page.data.nextLinkOffset !== null}
        <nav aria-label="Relationship pages" class="acts">
          {#if linkOffset > 0}<button class="btn sm" type="button" onclick={() => (linkOffset = 0)}><ChevronsUp size={14} aria-hidden="true" />First relationships</button>{/if}
          {#if page.data.nextLinkOffset !== null}<button class="btn sm" type="button" onclick={() => (linkOffset = page.data?.nextLinkOffset ?? 0)}>More relationships<ChevronDown size={14} aria-hidden="true" /></button>{/if}
        </nav>
      {/if}
    </section>
  {/if}
  <p class="meta mono" style:overflow-wrap="anywhere">record {record.id} / owner {record.owner}</p>
{/if}
