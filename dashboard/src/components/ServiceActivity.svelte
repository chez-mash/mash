<script lang="ts">
import { checkedTime, type Operation } from "../lib/services.svelte";

// The service actions recorded on one machine, newest as the server lists
// them: when each was asked, what it was and how it ended.
let {
  title,
  items,
  error,
  pending,
}: {
  title: string;
  items: readonly Operation[] | undefined;
  error: Error | null;
  pending: boolean;
} = $props();

const action = {
  start: "Start",
  stop: "Stop",
  restart: "Restart",
  reinstall_plugin: "Reinstall plugin",
} as const;
const clock = (seconds: number): string =>
  new Date(seconds * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
</script>

<section class="sec svc-activity">
  <h2 class="h2">{title}</h2>
  {#if error}<p class="fail" role="alert">{error.message}</p>{/if}
  {#if pending}<p class="meta" role="status">Loading activity…</p>{/if}
  {#if items?.length}
    <ol class="card pad0 act-list">
      {#each items as item (item.request.request_id)}
        <li>
          <time class="mono" title={checkedTime(item.requested_at)}>{clock(item.requested_at)}</time>
          <span><b class:fail={item.outcome === "failed"}>{action[item.request.action]} {item.request.service}: {item.outcome}</b>{#if item.message}<span class="msg">{item.message}</span>{/if}</span>
        </li>
      {/each}
    </ol>
  {:else if items}<p class="card tight meta">No recorded service actions.</p>{/if}
</section>

<style>
  .act-list {
    margin: 0;
    padding: 6px 0;
    list-style: none;
  }
  .act-list li {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr);
    gap: 10px;
    padding: 9px 16px;
    font-size: 13px;
    line-height: 1.45;
  }
  time {
    color: var(--text-faint);
    font-size: 12px;
    line-height: 1.6;
  }
  b {
    display: block;
    font-weight: var(--weight-label);
  }
  b:not(.fail) {
    color: var(--text-strong);
  }
  .msg {
    display: block;
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }
</style>
