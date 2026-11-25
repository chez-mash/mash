<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Folder from "@lucide/svelte/icons/folder";
import { agents } from "../lib/agents";
import { projectLabel, whenLabel } from "../lib/browse";
import { agentHref, machineHref, projectHref } from "../lib/route.svelte";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import MachineIcon from "./MachineIcon.svelte";

// The agent, machine and project of a chat, as quiet words or as chips, then its time.
let {
  chat,
  extra = "",
  agentMark = false,
}: {
  chat: ChatSummary;
  extra?: string;
  /** Outlined chips, as a chat's header shows them: the agent with its mark,
   * its machine and its project, each a link to its page. */
  agentMark?: boolean;
} = $props();
const now = Date.now();
</script>

<span class="mts">
  {#if agentMark}
    <a class="mt chip agent" href={agentHref(chat.agent)}><AgentMark agent={chat.agent} size="sm" />{agents[chat.agent].name}</a>
    <a class="mt chip" href={machineHref(chat.machine)}><MachineIcon machine={chat.machine} size={14} />{chat.machine}</a>
    {#if chat.project}<a class="mt chip" href={projectHref(chat.project)}><Folder size={14} aria-hidden="true" />{projectLabel(chat.project)}</a>{/if}
  {:else}
    <span class="mt"><Bot size={12} aria-hidden="true" />{agents[chat.agent].name}</span>
    <span class="mt"><MachineIcon machine={chat.machine} size={12} />{chat.machine}</span>
    {#if chat.project}<span class="mt"><Folder size={12} aria-hidden="true" />{projectLabel(chat.project)}</span>{/if}
  {/if}
  <span class="mtx">{whenLabel(chat.updatedMs, now)}</span>
  {#if extra}<span class="mtx">{extra}</span>{/if}
</span>
