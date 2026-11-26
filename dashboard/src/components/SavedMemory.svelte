<script lang="ts">
import Brain from "@lucide/svelte/icons/brain";
import { label, type NativeRecord } from "../lib/records";
import { recordHref, recordsHref } from "../lib/route.svelte";

// Under a message: each memory saved from it, as a raised card in a butter
// ring with its statement and the way to it on the memory page.
let {
  records,
  messageId,
  memoryParams,
}: { records: readonly NativeRecord[]; messageId: string; memoryParams: URLSearchParams } =
  $props();
/** "a decision", "an observation": a kind of memory with its article. */
function article(kind: string): string {
  const word = kind.toLowerCase();
  return `${/^[aeiou]/.test(word) ? "an" : "a"} ${word}`;
}
</script>

{#each records as record (record.id)}
  {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
  {#if revision?.evidence.some((source) => source.messageId === messageId)}
    <a class="tv-mem" href={recordHref(record.id, record.node, recordsHref(memoryParams))}>
      <span class="tv-mem-i" aria-hidden="true"><Brain size={14} /></span>
      <span class="tv-mem-t"><b>{revision.statement}</b><small>Saved as {article(label(revision.details.kind))} from this message</small></span>
      <span class="tv-mem-o">Open in Memory</span>
    </a>
  {/if}
{/each}
