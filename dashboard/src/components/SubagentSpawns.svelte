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
  {@const state = [stateLabel(child.subagent), whenLabel(child.updatedMs, now)].filter(Boolean).join(" · ")}
  <a class="fold spawn" href={chatHref(child.id, child.machine)}><span class="spawn-h"><GitFork size={15} aria-hidden="true" /><span class="spawn-t">Started a subagent: <b>{plainTitle(child.title)}</b></span><span class="pill">{state}</span><span class="spawn-o">Open<ChevronRight size={14} aria-hidden="true" /></span></span></a>
{/each}
