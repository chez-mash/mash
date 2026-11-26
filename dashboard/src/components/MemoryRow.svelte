<script lang="ts">
import { whenLabel } from "../lib/browse";
import { type NativeRecord, standing } from "../lib/records";
import { recordHref } from "../lib/route.svelte";
import { standingTone } from "../lib/tone";
import KindPill from "./KindPill.svelte";
import StatusBadge from "./StatusBadge.svelte";

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
      <KindPill kind={revision.details.kind} lower />
      {#if settled !== "established"}<StatusBadge pill tone={standingTone(settled)} words={settled} size={12} />{/if}
      <span class="mtx">{whenLabel(revision.recordedAt, Date.now())}</span>
    </span>
  </a>
{/if}
