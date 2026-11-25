<script lang="ts">
import GitFork from "@lucide/svelte/icons/git-fork";
import { whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { chatHref } from "../lib/route.svelte";
import { detailLabel } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

// Beside a parent chat: the subagents it started, each linking to its page.
let { children }: { children: readonly ChatSummary[] } = $props();
const now = Date.now();
</script>

{#if children.length}
  <section class="forks" aria-label="Subagents">
    <h2 class="sl-h"><GitFork size={14} aria-hidden="true" />Subagents ({children.length})</h2>
    <div class="sl-l">
      {#each children as child (child.machine + child.id)}
        <a class="sl-r" href={chatHref(child.id, child.machine)}><AgentMark agent={child.agent} size="sm" /><span><b>{plainTitle(child.title)}</b><small>{detailLabel(child, whenLabel(child.updatedMs, now))}</small></span></a>
      {/each}
    </div>
    <p class="meta forks-n">Started by this chat. Their messages are searchable and open on their own pages.</p>
  </section>
{/if}

<style>
  .forks {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .sl-h {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .sl-l {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-outline);
  }
  .sl-r {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px;
    border-radius: var(--radius-5);
    color: var(--text-strong);
  }
  .sl-r:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .sl-r :global(.mark) {
    width: 16px;
    height: 16px;
    margin-top: 2px;
  }
  .sl-r > span {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .sl-r b {
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
  }
  .sl-r small {
    font-size: 12px;
    color: var(--text-faint);
  }
  .forks-n {
    font-size: 12px;
    line-height: 1.5;
  }
</style>
