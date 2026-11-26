<script lang="ts">
import Bookmark from "@lucide/svelte/icons/bookmark";
import { createQuery } from "@tanstack/svelte-query";
import { type Scoped, stableRead } from "../lib/queries";
import { scopeIcons } from "../lib/recordIcons";
import { label, scopedRecordPage, scopeLabel, scopeName } from "../lib/records";
import { recordHref, recordsHref } from "../lib/route.svelte";
import { scopeText } from "../lib/scope.svelte";
import Highlight from "./Highlight.svelte";
import KindIcon from "./KindIcon.svelte";

// Above the chats a search found: the saved memories whose statements match
// the same words, as the memory page's own search finds them. Only a few are
// shown; the heading's link opens the memory page with the same search.
let { query, scoped }: { query: string; scoped: Scoped } = $props();
const shown = 2;
const params = $derived(new URLSearchParams({ q: query }));
const found = createQuery(() => ({
  ...stableRead,
  queryKey: ["search-memory", query, scopeText(scoped.scope)],
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    scopedRecordPage(params.toString(), scoped.scope, scoped.reach, signal),
}));
const records = $derived(found.data?.page.records ?? []);
</script>

<!-- A failed or empty memory search says nothing: the chats are the page. -->
{#if records.length}
  <section class="mm" aria-label="From memory">
    <h2 class="sx-label"><a href={recordsHref(params)}>From memory</a></h2>
    {#each records.slice(0, shown) as record (record.node + record.id)}
      {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
      {#if revision}
        {@const ScopeIcon = scopeIcons[record.scope.kind]}
        <a class="mm-r" href={recordHref(record.id, record.node, recordsHref(params))}>
          <span class="mm-i" aria-hidden="true"><Bookmark size={13} /></span>
          <span class="mm-t">
            <b><Highlight text={revision.statement} {query} /></b>
            <small><KindIcon kind={revision.details.kind} size={12} /><span>{label(revision.details.state).toLowerCase()}</span><span class="mm-s" title={scopeLabel(record.scope)}><ScopeIcon size={12} aria-hidden="true" />{scopeName(record.scope)}</span></small>
          </span>
        </a>
      {/if}
    {/each}
  </section>
{/if}

<style>
  .mm {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 8px;
  }
  .mm-r {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--memory-ring),
      var(--memory-glow);
    color: var(--text-strong);
  }
  .mm-r:hover {
    color: var(--text-strong);
  }
  .mm-i {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-5);
    background: var(--memory-tile);
    color: var(--memory-tile-text);
  }
  .mm-t {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .mm-t b {
    font-size: 13.5px;
    font-weight: var(--weight-strong);
    line-height: 1.4;
  }
  .mm-t small,
  .mm-s {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
  }
  .mm-s {
    display: inline-flex;
    gap: 4px;
  }
  .mm-t small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .mm :global(mark) {
    box-shadow: var(--find-glow);
    color: inherit;
  }
</style>
