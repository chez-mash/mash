<script lang="ts">
import GitFork from "@lucide/svelte/icons/git-fork";
import { plainTitle } from "../lib/markdown";
import { originLabel } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";

// The pill a chat row carries about subagents: how many a parent started, or,
// on a subagent's own row, which chat started it.
let { chat, parentTitle }: { chat: ChatSummary; parentTitle?: string | undefined } = $props();
</script>

{#if chat.subagent}
  <span class="pill fork"><GitFork size={12} aria-hidden="true" /><span class="fork-t">{originLabel(chat, parentTitle === undefined ? undefined : plainTitle(parentTitle))}</span></span>
{:else if chat.subagents}
  <span class="pill fork"><GitFork size={12} aria-hidden="true" />{chat.subagents} subagent{chat.subagents === 1 ? "" : "s"}</span>
{/if}
