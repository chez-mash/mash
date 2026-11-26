<script lang="ts">
import { createQueries } from "@tanstack/svelte-query";
import { currentOf } from "../lib/memoryView";
import { stableRead } from "../lib/queries";
import { scopeIcons } from "../lib/recordIcons";
import { getRecord, type NativeRecord, scopeLabel, scopeName } from "../lib/records";
import { recordHref } from "../lib/route.svelte";
import KindIcon from "./KindIcon.svelte";
import StatusBadge from "./StatusBadge.svelte";

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
      {#if relation === "supports" || relation === "contradicts"}<StatusBadge pill tone={relation === "supports" ? "ok" : "bad"} words={item.outgoing ? `this record ${relation}` : `${relation} this record`} size={12} />{:else}<span class="pill">{item.outgoing ? `this record ${relation}` : `${relation} this record`}</span>{/if}
      {#if current && other}
        {@const ScopeIcon = scopeIcons[other.scope.kind]}
        <b>{current.statement}</b>
        <small><KindIcon kind={current.details.kind} size={12} /><span class="rel-s" title={scopeLabel(other.scope)}><ScopeIcon size={12} aria-hidden="true" />{scopeName(other.scope)}</span></small>
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
    font-weight: var(--weight-label);
    line-height: 1.4;
    color: var(--text-strong);
    overflow-wrap: anywhere;
  }
  small,
  .rel-s {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .rel-s {
    gap: 4px;
  }
  small {
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
