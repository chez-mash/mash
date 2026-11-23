<script lang="ts">
import { stateLabel } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";

// In a subagent's header: its role and, when the agent recorded it, its state.
let { chat }: { chat: ChatSummary } = $props();
const state = $derived(stateLabel(chat.subagent));
const kind = $derived(["subagent", chat.subagent?.role].filter(Boolean).join(" · "));
</script>

{#if chat.subagent}
  <span class="pill fork">{kind}</span>
  {#if state}<span class="pill" class:ok={state === "done"} class:busy={state === "running"}>{state}</span>{/if}
{/if}
