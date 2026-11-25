<script lang="ts">
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import GitFork from "@lucide/svelte/icons/git-fork";
import { whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { chatHref } from "../lib/route.svelte";
import { spawnedBy, stateLabel } from "../lib/subagents";
import type { ChatSummary, Message } from "../lib/types";

// After a run of steps: the subagents its tool calls started. A call links to
// a subagent only when the agent recorded that exact call as its start.
let { messages, children }: { messages: readonly Message[]; children: readonly ChatSummary[] } =
  $props();
const now = Date.now();
const started = $derived(
  children.length
    ? messages
        .flatMap((message) => message.tools?.calls ?? [])
        .map((call) => spawnedBy(children, call.callId))
        .filter((child): child is ChatSummary => child !== undefined)
    : [],
);
</script>

{#each started as child (child.machine + child.id)}
  {@const done = stateLabel(child.subagent)}
  {@const state = [done, whenLabel(child.updatedMs, now)].filter(Boolean).join(" · ")}
  <a class="fold spawn" href={chatHref(child.id, child.machine)}><span class="spawn-h"><GitFork size={15} aria-hidden="true" /><span class="spawn-t">Started a subagent: <b>{plainTitle(child.title)}</b></span><span class="pill" class:ok={done === "done"} class:busy={done === "running"}>{#if done}<i class="dot" class:busy={done === "running"}></i>{/if}{state}</span><span class="spawn-o">Open<ChevronRight size={14} aria-hidden="true" /></span></span></a>
{/each}

<style>
  /* A raised row per subagent the step started: what it was asked, its state
     and age, and the way to its own page. */
  .spawn {
    border: 0;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-outline);
    color: var(--text-strong);
  }
  .spawn:hover {
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-strong);
    color: var(--text-strong);
  }
  .spawn-h {
    flex-wrap: nowrap;
    gap: 12px;
    min-height: 52px;
    padding: 10px 14px;
    font: 400 13.5px / 1.4 var(--type-sans);
  }
  .spawn-h > :global(svg) {
    flex: none;
    color: var(--text-muted);
  }
  .spawn-t {
    flex: 1;
    color: var(--text-muted);
  }
  .spawn-t b {
    color: var(--text-strong);
    font-weight: 600;
  }
  .spawn-h .pill {
    flex: none;
    border: 0;
    font: 500 12px / 1 var(--type-sans);
  }
  .spawn-h .pill .dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border: 0;
    border-radius: var(--radius-round);
    background: var(--status-ok-dot);
  }
  .spawn-h .pill .dot.busy {
    background: var(--status-running-dot);
  }
  .spawn-o {
    flex: none;
    margin-left: 0;
    font-weight: 500;
  }
</style>
