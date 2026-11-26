<script lang="ts">
import GitForkIcon from "phosphor-svelte/lib/GitForkIcon";
import Icon from "$lib/components/Icon.svelte";
import { plainTitle } from "../lib/markdown";
import { originLabel } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";

// The pills a chat row carries about where it came from: how many subagents a
// parent started, which chat started a subagent, and whether the chat is a
// copy of a session that ran over SSH on another machine.
let { chat, parentTitle }: { chat: ChatSummary; parentTitle?: string | undefined } = $props();
</script>

{#if chat.sshCopy}
  <span class="pill" title="A copy the Claude desktop app keeps of a session that ran over SSH on another machine">ran over SSH</span>
{/if}
{#if chat.subagent}
  <span class="pill fork"><Icon icon={GitForkIcon} size={12} /><span class="fork-t">{originLabel(chat, parentTitle === undefined ? undefined : plainTitle(parentTitle))}</span></span>
{:else if chat.subagents}
  <span class="pill fork"><Icon icon={GitForkIcon} size={12} />{chat.subagents} subagent{chat.subagents === 1 ? "" : "s"}</span>
{/if}
