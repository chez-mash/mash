<script lang="ts">
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import { agents } from "../lib/agents";
import { plainTitle } from "../lib/markdown";
import { chatHref } from "../lib/route.svelte";
import { parentOf } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";
import ChatPlace from "./ChatPlace.svelte";

// Above a subagent's transcript: the chat that started it. A parent that is
// not indexed here (say, on another machine) is named as such, unlinked.
let { chat, all }: { chat: ChatSummary; all: readonly ChatSummary[] } = $props();
const parent = $derived(parentOf(all, chat));
</script>

{#if chat.subagent}
  {#if parent}
    <a class="card tight flat cream fork-of" href={chatHref(parent.id, parent.machine)}><ChevronLeft size={15} aria-hidden="true" /><span>Subagent of <b>{plainTitle(parent.title)}</b></span><span class="mtx"><ChatPlace agent={parent.agent} machine={parent.machine} /></span></a>
  {:else if chat.subagent.parentId}
    <p class="card tight flat cream fork-of">Subagent of {#if chat.subagent.parentTitle}<b>{plainTitle(chat.subagent.parentTitle)}</b>{:else}a chat not on this machine{/if}</p>
  {:else}
    <p class="card tight flat cream fork-of">A subagent {agents[chat.agent].name} started for part of a task</p>
  {/if}
{/if}
