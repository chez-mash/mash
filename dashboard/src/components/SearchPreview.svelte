<script lang="ts">
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import { createQuery } from "@tanstack/svelte-query";
import { agents } from "../lib/agents";
import { context } from "../lib/api";
import { highlightParts } from "../lib/highlight";
import { plainTitle } from "../lib/markdown";
import { saidText } from "../lib/message";
import { stableRead } from "../lib/queries";
import type { Message, SearchHit } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import Highlight from "./Highlight.svelte";

// Beside the results on a wide screen: the message a result matched, with
// what the reader asked just before it, read around that message the way a
// chat opens at a search result. It reads only; opening the chat is a link.
let { hit, query, href }: { hit: SearchHit; query: string; href: string } = $props();
const around = createQuery(() => ({
  ...stableRead,
  queryKey: ["search-preview", hit.chat.machine, hit.chat.id, hit.messageId],
  enabled: !!hit.messageId,
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    context(hit.messageId, hit.chat.machine, signal),
}));
const said = (message: Message) =>
  message.presentation === undefined || message.presentation === "dialogue";
// The matched message, and the reader's own message before it when the
// match is the agent's answer.
const shown = $derived.by(() => {
  const messages = around.data?.messages ?? [];
  const at = messages.findIndex((message) => message.id === hit.messageId);
  const matched = messages[at];
  if (!matched) return [];
  if (matched.role === "user") return [matched];
  const asked = messages
    .slice(0, at)
    .findLast((message) => message.role === "user" && said(message));
  return asked ? [asked, matched] : [matched];
});
const position = $derived.by(() => {
  const matched = around.data?.messages.find((message) => message.id === hit.messageId);
  const total = around.data?.total;
  if (matched?.ordinal === undefined) return "";
  const place = (matched.ordinal + 1).toLocaleString();
  return total ? `Message ${place} of ${total.toLocaleString()}` : `Message ${place}`;
});
/** A passage of a long message, starting a little before its first match. */
function passage(text: string, limit: number): string {
  const flat = plainTitle(text).replace(/\s+/g, " ").trim();
  if (flat.length <= limit) return flat;
  let offset = 0;
  for (const part of highlightParts(flat, query)) {
    if (part.hit) break;
    offset += part.text.length;
  }
  const start = offset > limit / 3 ? flat.lastIndexOf(" ", offset - 60) + 1 : 0;
  const end = Math.min(flat.length, start + limit);
  return `${start > 0 ? "…" : ""}${flat.slice(start, end)}${end < flat.length ? "…" : ""}`;
}
</script>

<aside class="pv" aria-label="Preview">
  <div class="pv-h">
    {#if position}<span class="pv-n">{position}</span>{/if}
    <h2><Highlight text={plainTitle(hit.chat.title)} {query} /></h2>
  </div>
  {#if around.isPending}
    <div class="pv-sk" role="status" aria-label="Loading the message"><span class="sk" style:width="70%"></span><span class="sk" style:width="92%"></span><span class="sk" style:width="55%"></span></div>
  {:else if around.error || !shown.length}
    <!-- Without the message around it, the passage the search returned stands in. -->
    <p class="pv-t"><Highlight text={hit.before} {query} /></p>
  {:else}
    {#each shown as message (message.id)}
      {#if message.role === "user"}
        <div class="pv-you" class:hit={message.id === hit.messageId}><span class="pv-who">You</span><p><Highlight text={passage(saidText(message), 360)} {query} /></p></div>
      {:else}
        <div class="pv-agent"><span class="pv-who"><AgentMark agent={hit.chat.agent} size="sm" />{agents[hit.chat.agent].name}</span><p class="pv-t"><Highlight text={passage(saidText(message), 700)} {query} /></p></div>
      {/if}
    {/each}
  {/if}
  <div class="pv-f">
    <a class="btn sm" {href}>Open chat<ArrowRight size={14} aria-hidden="true" /></a>
    <span class="pv-k">Open <kbd class="kbd">↵</kbd></span>
    <span class="pv-k">Next <kbd class="kbd">↓</kbd></span>
    <span class="pv-k">Previous <kbd class="kbd">↑</kbd></span>
  </div>
</aside>

<style>
  .pv {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 100%;
    padding: 20px 24px;
    font-size: 13.5px;
    line-height: 1.6;
    color: var(--text-strong);
  }
  .pv-h {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .pv-n,
  .pv-who {
    font-size: 12px;
    color: var(--text-faint);
  }
  .pv-h h2 {
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.015em;
    line-height: 1.3;
  }
  .pv-you {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: 0 0 0 var(--stroke-outline) var(--line-outline);
  }
  .pv-agent {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .pv-who {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .pv-who :global(.mark) {
    width: 16px;
    height: 16px;
  }
  .pv-t,
  .pv-you p {
    overflow-wrap: anywhere;
  }
  .pv :global(mark) {
    box-shadow: var(--find-glow);
    color: var(--text-strong);
  }
  .pv-sk {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .pv-sk .sk {
    height: 12px;
  }
  .pv-f {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 14px;
    margin-top: auto;
    padding-top: 12px;
    border-top: var(--border-divider);
    font-size: 12px;
    color: var(--text-muted);
  }
  .pv-k {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
</style>
