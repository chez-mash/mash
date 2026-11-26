<script lang="ts">
import Server from "@lucide/svelte/icons/server";
import { createQuery } from "@tanstack/svelte-query";
import { whenLabel } from "../lib/browse";
import { reported, serverReport, serverViewOptions } from "../lib/serverStatus";
import ToneIcon from "./ToneIcon.svelte";

// The shared server in one strip above the machines: its address and group,
// then three numbers from its own report: the chats its machines published,
// how long since this machine last published, and whether search over this
// machine's copy is ready. The card below the machines manages the server.
const view = createQuery(() => serverViewOptions);
const report = $derived(serverReport(view.data));
const own = $derived(view.data?.node_id ? reported(report, view.data.node_id) : undefined);
const shared = $derived(
  report.kind === "reported"
    ? report.machines.reduce((sum, machine) => sum + (machine.chats ?? 0), 0)
    : null,
);
function host(url: string): string {
  try {
    return new URL(url).host || url;
  } catch {
    return url;
  }
}
const search = $derived.by(() => {
  const state = own?.search;
  if (!state) return null;
  if (state.words !== "ready") return { value: "Catching up", note: "search on the server" };
  return state.meaning === "ready"
    ? { value: "Ready", note: "word and meaning search" }
    : { value: "Ready", note: "word search" };
});
</script>

{#if report.kind !== "none"}
  <div class="card srv">
    <span class="srv-i" aria-hidden="true"><Server size={16} /></span>
    <span class="srv-n">
      <b>Shared server<ToneIcon tone={report.kind === "reported" ? "ok" : "warn"} size={14} /></b>
      <code>{host(report.address)}{#if report.kind === "reported"} · group {report.group}{/if}</code>
    </span>
    {#if report.kind === "reported"}
      {#if shared !== null}<span class="srv-s"><b>{shared.toLocaleString()}</b><small>chats shared</small></span>{/if}
      {#if own?.publishedAt != null}<span class="srv-s"><b>{whenLabel(own.publishedAt * 1000, report.now * 1000)}</b><small>since this machine published</small></span>{/if}
      {#if search}<span class="srv-s"><b>{search.value}</b><small>{search.note}</small></span>{/if}
    {:else}
      <span class="srv-s warn"><small class="st-l"><ToneIcon tone="warn" size={13} />{report.message}</small></span>
    {/if}
  </div>
{/if}

<style>
  .srv {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px 0;
    padding: 18px 20px;
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
  }
  .srv-i {
    flex: none;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin-right: 28px;
    border-radius: var(--radius-6);
    background: var(--surface-muted);
    color: var(--text-strong);
    box-shadow: var(--shadow-pick);
  }
  .srv-n {
    display: flex;
    flex: 1 1 220px;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .srv-n b {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .srv-n code {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 12px var(--type-mono);
    color: var(--text-muted);
  }
  .srv-s {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 120px;
    padding: 0 18px;
    border-left: var(--border-divider);
  }
  .srv-s b {
    font-size: 20px;
    font-weight: var(--weight-label);
    letter-spacing: -0.02em;
    color: var(--text-strong);
  }
  .srv-s small {
    font-size: 12px;
    color: var(--text-muted);
  }
  .srv-s.warn small {
    color: var(--status-warn-text);
  }
  @container (max-width: 640px) {
    .srv-s {
      flex: 1 1 30%;
      min-width: 0;
      padding: 0 0 0 12px;
    }
    .srv-i {
      margin-right: 14px;
    }
  }
</style>
