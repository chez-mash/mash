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
    <h2 class="h2"><GitFork size={15} aria-hidden="true" />Subagents ({children.length})</h2>
    <div class="card pad0 list sess-l">
      {#each children as child (child.machine + child.id)}
        <a class="chatrow ses" href={chatHref(child.id, child.machine)}><AgentMark agent={child.agent} size="sm" /><span><span class="clamp">{plainTitle(child.title)}</span><small>{detailLabel(child, whenLabel(child.updatedMs, now))}</small></span></a>
      {/each}
    </div>
    <p class="meta forks-n">Started by this chat. Their messages are searchable and open on their own pages.</p>
  </section>
{/if}
