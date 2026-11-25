<script lang="ts">
import {
  label,
  type NativeRecord,
  type Revision,
  scopeLabel,
  scopeName,
  standing,
} from "../lib/records";

// What is known about a memory, as names and values in one card: its kind,
// where it stands, what it applies to, where it lives and its id.
let { record, revision, machine }: { record: NativeRecord; revision: Revision; machine: string } =
  $props();
</script>

<dl class="card facts">
  <div><dt>Kind</dt><dd>{label(revision.details.kind)}</dd></div>
  <div><dt>Status</dt><dd>{label(revision.details.state)}</dd></div>
  <div><dt>Confidence</dt><dd>{label(standing(revision))}</dd></div>
  <div><dt>Applies to</dt><dd title={scopeLabel(record.scope)}>{scopeName(record.scope)}</dd></div>
  {#if revision.confirmations}<div><dt>Confirmed</dt><dd>{revision.confirmations.toLocaleString()} time{revision.confirmations === 1 ? "" : "s"}</dd></div>{/if}
  <div><dt>Machine</dt><dd>{machine}</dd></div>
  <div><dt>Owner</dt><dd>{record.owner}</dd></div>
  <div><dt>ID</dt><dd class="mono" title={record.id}>{record.id}</dd></div>
</dl>

<style>
  .facts {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin: 0;
    padding: 6px 16px;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
  }
  .facts > div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 7px 0;
    font-size: 13px;
  }
  dt {
    flex: none;
    color: var(--text-muted);
  }
  dd {
    min-width: 0;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-strong);
    text-align: right;
  }
</style>
