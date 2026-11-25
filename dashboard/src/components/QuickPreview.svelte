<script lang="ts">
import Bookmark from "@lucide/svelte/icons/bookmark";
import { createQuery } from "@tanstack/svelte-query";
import { agents } from "../lib/agents";
import { context } from "../lib/api";
import { agoLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { saidText } from "../lib/message";
import { passageAround, plainPassage } from "../lib/passage";
import { stableRead } from "../lib/queries";
import { chatMemoryParams, pageTarget, type QuickRow } from "../lib/quick";
import { label, recordPage } from "../lib/records";
import { recordHref, recordsHref } from "../lib/route.svelte";
import ChatPlace from "./ChatPlace.svelte";
import Highlight from "./Highlight.svelte";

// Beside the quick panel's list when the panel is wide enough: the selected
// chat, and for a search result the message it matched, read around that
// message as the search page's preview does, and the latest memory saved
// from the chat when Memory is on. It only reads; Enter opens the chat.
let {
  row,
  query,
  now,
  local,
  memory,
  desktop,
}: {
  row: QuickRow;
  query: string;
  now: number;
  /** The chat ran on this machine. */
  local: boolean;
  /** Memory is turned on. */
  memory: boolean;
  desktop: boolean;
} = $props();
const chat = $derived(row.chat);
const around = createQuery(() => ({
  ...stableRead,
  queryKey: ["search-preview", row.chat.machine, row.chat.id, row.messageId],
  enabled: !!row.messageId,
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    context(row.messageId, row.chat.machine, signal),
}));
// The chat page's own query for what was saved from the chat, so the two share it.
const memoryParams = $derived(chatMemoryParams(row.chat, local));
const saved = createQuery(() => ({
  ...stableRead,
  enabled: memory,
  queryKey: ["chat-memory", row.chat.id, memoryParams.toString()],
  queryFn: ({ signal }: { signal: AbortSignal }) => recordPage(memoryParams.toString(), signal),
}));
const card = $derived.by(() => {
  const record = saved.data?.records[0];
  const revision = record?.revisions.find((each) => each.revision === record.currentRevision);
  if (!record || !revision) return undefined;
  const back = recordsHref(memoryParams);
  return {
    statement: revision.statement,
    kind: label(revision.details.kind),
    href: pageTarget(recordHref(record.id, record.node, back), desktop),
  };
});
const matched = $derived(around.data?.messages.find((message) => message.id === row.messageId));
const position = $derived.by(() => {
  const total = around.data?.total;
  if (matched?.ordinal === undefined) return "";
  const place = (matched.ordinal + 1).toLocaleString();
  return total ? `Message ${place} of ${total.toLocaleString()}` : `Message ${place}`;
});
const when = $derived(agoLabel(chat.updatedMs, now));
</script>

<aside class="qp" aria-label="Preview">
  <div class="qp-h">
    <h2><Highlight text={plainTitle(chat.title)} {query} /></h2>
    <p class="qp-m"><ChatPlace agent={chat.agent} machine={chat.machine} project={chat.project ?? ""} mark />{#if when}<span>{when}</span>{/if}</p>
  </div>
  {#if row.messageId}
    {#if position}<span class="qp-n">{position}</span>{/if}
    <div class="qp-said">
      {#if matched}
        <span class="qp-who">{matched.role === "user" ? "You" : agents[chat.agent].name}</span>
        <p><Highlight text={passageAround(saidText(matched), query, 320)} {query} /></p>
      {:else if !around.isPending}
        <!-- Without the message around it, the passage the search returned stands in. -->
        <p><Highlight text={plainPassage(row.passage)} {query} /></p>
      {/if}
    </div>
  {:else if chat.messageCount}
    <span class="qp-n">{chat.messageCount.toLocaleString()} message{chat.messageCount === 1 ? "" : "s"}</span>
  {/if}
  {#if memory && card}
    <a class="qp-mem" href={card.href}>
      <Bookmark size={14} aria-hidden="true" />
      <span class="qp-mem-t">{card.statement}</span>
      <span class="qp-mem-k">{card.kind}</span>
    </a>
  {/if}
</aside>

<style>
  .qp {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 20px 22px;
    overflow: hidden;
  }
  .qp-h {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .qp-h h2 {
    margin: 0;
    color: var(--text-strong);
    font-size: 16px;
    font-weight: 600;
    line-height: 1.3;
    letter-spacing: -0.015em;
  }
  .qp-m {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
  }
  .qp-m :global(.mark.sm) {
    width: 16px;
    height: 16px;
  }
  .qp-m :global(.mark.sm svg) {
    width: 10px;
    height: 10px;
  }
  .qp-m span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .qp-n {
    color: var(--text-faint);
    font: 400 11px / 1 var(--type-mono);
  }
  .qp-said {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
  }
  .qp-who {
    color: var(--text-faint);
    font-size: 12px;
  }
  .qp-said p {
    display: -webkit-box;
    margin: 0;
    overflow: hidden;
    color: var(--text-body);
    font-size: 14px;
    line-height: 1.6;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 6;
    line-clamp: 6;
  }
  .qp-mem {
    display: flex;
    flex: none;
    align-items: center;
    gap: 10px;
    margin-top: auto;
    padding: 10px 12px;
    border-radius: var(--radius-6);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-outline),
      var(--shadow-pick);
    color: var(--text-strong);
    text-decoration: none;
  }
  .qp-mem-t {
    flex: 1;
    min-width: 0;
    display: -webkit-box;
    overflow: hidden;
    font-size: 13px;
    font-weight: 500;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
  .qp-mem-k {
    flex: none;
    color: var(--text-faint);
    font-size: 12px;
  }
  .qp-said :global(mark) {
    box-shadow: var(--find-glow);
  }
</style>
