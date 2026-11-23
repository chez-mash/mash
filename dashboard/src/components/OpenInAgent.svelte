<script lang="ts">
import SquareArrowOutUpRight from "@lucide/svelte/icons/square-arrow-out-up-right";
import Terminal from "@lucide/svelte/icons/terminal";
import { createQuery } from "@tanstack/svelte-query";
import { appLink, mayOpenInApp, resumeCommand } from "../lib/openIn";
import { stableRead } from "../lib/queries";
import type { ChatSummary } from "../lib/types";

// Back to the session in its agent: its desktop app when this machine has one
// that opens it, else the command that resumes it in a terminal. Only for a
// chat of this machine, since both act on the computer showing the page.
let {
  chat,
  local,
  onnotice,
}: { chat: ChatSummary; local: boolean; onnotice: (text: string) => void } = $props();
const asked = $derived(local && mayOpenInApp(chat.agent, chat.id));
const link = createQuery(() => ({
  ...stableRead,
  enabled: asked,
  queryKey: ["open-in", chat.agent, chat.id, chat.project],
  queryFn: ({ signal }) => appLink(chat, signal),
}));
const command = $derived(local ? resumeCommand(chat.agent, chat.id) : undefined);
// Wait for the answer, so the button does not change under the pointer.
const settled = $derived(!asked || !link.isPending);

async function copyCommand(text: string): Promise<void> {
  const where = chat.agent === "claude-code" ? " Run it in the chat’s project folder." : "";
  try {
    await navigator.clipboard.writeText(text);
    onnotice(`Resume command copied.${where}`);
  } catch {
    onnotice(`Copy failed. The command is ${text}`);
  }
}
</script>

{#if asked && link.data}
  <!-- A session the app does not have yet is added to it as a new one, so
       the button says so before the click. -->
  {#if link.data.addsSession}
    <a class="btn" href={link.data.url} title="Adds this session to the {link.data.app} app as a new session, then opens it"><SquareArrowOutUpRight size={15} aria-hidden="true" />Add to {link.data.app}</a>
  {:else}
    <a class="btn" href={link.data.url} title="Open this session in the {link.data.app} app"><SquareArrowOutUpRight size={15} aria-hidden="true" />Open in {link.data.app}</a>
  {/if}
{:else if settled && command}
  <button class="btn" type="button" title={command} onclick={() => copyCommand(command)}><Terminal size={15} aria-hidden="true" />Copy resume command</button>
{/if}
