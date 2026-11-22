<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import { agents } from "../lib/agents";
import { projectLabel, whenLabel } from "../lib/browse";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

// The agent, machine and project of a chat as quiet words, then its time.
let {
  chat,
  extra = "",
  icons = true,
  agentMark = false,
}: {
  chat: ChatSummary;
  extra?: string;
  icons?: boolean;
  /** Name the agent on an outlined chip with its mark, as a chat's header does. */
  agentMark?: boolean;
} = $props();
const now = Date.now();
</script>

<span class="mts">
  {#if agentMark}
    <span class="mt agent"><AgentMark agent={chat.agent} size="sm" />{agents[chat.agent].name}</span>
  {:else}
    <span class="mt">{#if icons}<Bot size={12} aria-hidden="true" />{/if}{agents[chat.agent].name}</span>
  {/if}
  <span class="mt">{#if icons}<Laptop size={12} aria-hidden="true" />{/if}{chat.machine}</span>
  {#if chat.project}<span class="mt">{#if icons}<Folder size={12} aria-hidden="true" />{/if}{projectLabel(chat.project)}</span>{/if}
  <span class="mtx">{whenLabel(chat.updatedMs, now)}</span>
  {#if extra}<span class="mtx">{extra}</span>{/if}
</span>
