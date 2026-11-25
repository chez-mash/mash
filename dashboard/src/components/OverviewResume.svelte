<script lang="ts">
// The overview's "Pick up where you left off" card: the most recent chat,
// with a way into it and, when this machine can, back into its agent. The
// agent's app opens it where the server finds one (see lib/openIn.ts);
// otherwise Claude Code and Codex sessions offer their resume command. Any
// other chat offers only the chat itself.
import MessageSquare from "@lucide/svelte/icons/message-square";
import SquareArrowOutUpRight from "@lucide/svelte/icons/square-arrow-out-up-right";
import Terminal from "@lucide/svelte/icons/terminal";
import { createQuery } from "@tanstack/svelte-query";
import { agents } from "../lib/agents";
import { agoLabel, projectLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { appLink, mayOpenInApp, resumeCommand } from "../lib/openIn";
import { stableRead } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { chatHref } from "../lib/route.svelte";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

let { chat, local }: { chat: ChatSummary; local: boolean } = $props();
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
let notice = $state("");
// A chat opened earlier in this browser session reopens where it was left.
const href = $derived.by(() => {
  const position = reading.positions.find(
    (position) => position.id === chat.id && position.node === chat.machine,
  );
  return chatHref(chat.id, chat.machine, position?.message ?? "", position?.shared ?? false);
});
const where = $derived(
  [agents[chat.agent].name, chat.machine, chat.project ? projectLabel(chat.project) : ""]
    .filter(Boolean)
    .join(" · "),
);

async function copy(text: string): Promise<void> {
  const folder = chat.agent === "claude-code" ? " Run it in the chat’s project folder." : "";
  try {
    await navigator.clipboard.writeText(text);
    notice = `Resume command copied.${folder}`;
  } catch {
    notice = `Copy failed. The command is ${text}`;
  }
}
</script>

<section class="resume" aria-labelledby="resume-title">
  <AgentMark agent={chat.agent} size="lg" />
  <div class="resume-t">
    <span class="resume-k">Pick up where you left off</span>
    <h2 id="resume-title" class="resume-h">{plainTitle(chat.title)}</h2>
    <span class="resume-m">{where}{chat.updatedMs !== undefined ? ` · last message ${agoLabel(chat.updatedMs, Date.now())}` : ""}</span>
    {#if notice}<span class="resume-m" role="status">{notice}</span>{/if}
  </div>
  <div class="resume-a">
    <a class="btn" {href}><MessageSquare size={15} aria-hidden="true" />Open chat</a>
    {#if asked && link.data}
      <a class="btn p" href={link.data.url} title={link.data.addsSession ? `Adds this session to the ${link.data.app} app as a new session, then opens it` : `Open this session in the ${link.data.app} app`}><SquareArrowOutUpRight size={15} aria-hidden="true" />Pick up in {link.data.app}</a>
    {:else if settled && command}
      <button class="btn p" type="button" title={command} onclick={() => copy(command)}><Terminal size={15} aria-hidden="true" />Copy resume command</button>
    {/if}
  </div>
</section>

<style>
  .resume {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px 20px;
    padding: 18px 20px;
    border-radius: var(--radius-9);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--memory-ring),
      var(--memory-glow);
  }
  .resume-t {
    flex: 1 1 300px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .resume-k {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-faint);
  }
  .resume-h {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-strong);
    overflow-wrap: anywhere;
  }
  .resume-m {
    font-size: 13px;
    color: var(--text-muted);
  }
  .resume-a {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .resume-a :global(.btn) {
    min-height: 32px;
    font-size: 13px;
    font-weight: 500;
  }
  @media (max-width: 640px) {
    .resume {
      align-items: flex-start;
      padding: 16px;
    }
    .resume-t {
      flex: 1 1 0;
    }
    .resume-a {
      flex: 1 0 100%;
    }
    .resume-a :global(.btn) {
      flex: 1;
      justify-content: center;
      min-height: 44px;
      border-radius: var(--radius-pill);
    }
  }
</style>
