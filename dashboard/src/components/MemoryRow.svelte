<script lang="ts">
import { whenLabel } from "../lib/browse";
import { kindPill, label, type NativeRecord, standing, standingPill } from "../lib/records";
import { recordHref } from "../lib/route.svelte";

// One memory in a card list: what it says, then its kind and standing as
// pills and when it was saved.
let { record, back = "#/records" }: { record: NativeRecord; back?: string } = $props();
const revision = $derived(
  record.revisions.find((revision) => revision.revision === record.currentRevision) ??
    record.revisions[0],
);
</script>

{#if revision}
  {@const settled = standing(revision)}
  <a class="chatrow" href={recordHref(record.id, record.node, back)}>
    <span class="clamp clamp3">{revision.statement}</span>
    <span class="pills">
      <span class="pill {kindPill(revision.details.kind)}">{label(revision.details.kind).toLowerCase()}</span>
      {#if settled !== "established"}<span class="pill {standingPill(settled)}">{settled}</span>{/if}
      <span class="mtx">{whenLabel(revision.recordedAt, Date.now())}</span>
    </span>
  </a>
{/if}
