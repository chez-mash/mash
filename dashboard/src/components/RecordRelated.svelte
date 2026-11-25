<script lang="ts">
import { createQueries } from "@tanstack/svelte-query";
import { currentOf } from "../lib/memoryView";
import { stableRead } from "../lib/queries";
import { getRecord, label, type NativeRecord, scopeName } from "../lib/records";
import { recordHref } from "../lib/route.svelte";

// The records a memory is linked to, each by what it says. A related record
// is read the way its own page reads it, so opening it costs nothing more;
// until it arrives, or when it cannot be read, its id stands in.
let { record, links, back }: { record: NativeRecord; links: NativeRecord["links"]; back: string } =
  $props();
const related = $derived(
  links.map((link) => {
    const outgoing = link.sourceId === record.id;
    return { link, outgoing, id: outgoing ? link.targetId : link.sourceId };
  }),
);
const reads = createQueries(() => ({
  queries: related.map(({ id }) => ({
    ...stableRead,
    queryKey: ["record", id, record.node, null, 0],
    queryFn: ({ signal }: { signal: AbortSignal }) => getRecord(id, record.node, null, 0, signal),
  })),
}));
</script>

<div class="rel">
  {#each related as item, index (JSON.stringify(item.link))}
    {@const other = reads[index]?.data}
    {@const current = other ? currentOf(other) : undefined}
    {@const relation = item.link.relation}
    <a class="card rel-c" href={recordHref(item.id, record.node, back)}>
      <span class="pill {relation === 'supports' ? 'ok' : relation === 'contradicts' ? 'bad' : ''}">{item.outgoing ? `this record ${relation}` : `${relation} this record`}</span>
      {#if current && other}
        <b>{current.statement}</b>
        <small>{label(current.details.kind)} · {scopeName(other.scope)}</small>
      {:else}
        <b class="mono">…{item.id.slice(-12)}</b>
      {/if}
    </a>
  {/each}
</div>

<style>
  .rel {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .rel-c {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 12px 14px;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
  }
  a.rel-c:hover {
    background: var(--surface-hover);
  }
  b {
    font-size: 13.5px;
    font-weight: 500;
    line-height: 1.4;
    color: var(--text-strong);
    overflow-wrap: anywhere;
  }
  small {
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
