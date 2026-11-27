<script lang="ts">
// The overview's recent chats: each with its agent's mark, its title, where
// it happened, how many decisions it produced and how long ago it was.
import { createQuery } from "@tanstack/svelte-query";
import { whenLabel } from "../lib/browse";
import { chatMenu } from "../lib/chatMenu";
import { contextMenu } from "../lib/contextMenu.svelte";
import { plainTitle } from "../lib/markdown";
import { onThisMachine } from "../lib/openIn";
import { catalogOptions } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { chatHref } from "../lib/route.svelte";
import { recency } from "../lib/tone";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatPlace from "./ChatPlace.svelte";
import DecisionCount from "./DecisionCount.svelte";
import SubagentPill from "./SubagentPill.svelte";

let { chats, decided }: { chats: readonly ChatSummary[]; decided: ReadonlyMap<string, number> } =
  $props();
const now = Date.now();
// A chat opened earlier in this browser session reopens where it was left.
function hrefOf(chat: ChatSummary): string {
  const position = reading.positions.find(
    (position) => position.id === chat.id && position.node === chat.machine,
  );
  return chatHref(chat.id, chat.machine, position?.message ?? "", position?.shared ?? false);
}
// A right-click offers the chat's own menu (lib/chatMenu.ts).
const catalogue = createQuery(() => catalogOptions);
function menu(event: MouseEvent, chat: ChatSummary, href: string): void {
  const local = onThisMachine(chat, catalogue.data?.localNode, catalogue.data?.sharedOnly ?? false);
  contextMenu.show(event, chatMenu(chat, href, local));
}
</script>

<div class="oc">
  {#each chats as chat (chat.machine + chat.id)}
    {@const decisions = decided.get(chat.id) ?? 0}
    <a class="oc-row" href={hrefOf(chat)} oncontextmenu={(event) => menu(event, chat, hrefOf(chat))}>
      <AgentMark agent={chat.agent} size="sm" />
      <span class="oc-t">
        <b>{plainTitle(chat.title)}</b>
        <span class="oc-m"><ChatPlace agent={chat.agent} machine={chat.machine} project={chat.project ?? ""} led /></span>
      </span>
      <SubagentPill {chat} />
      {#if decisions}<span class="oc-badge"><DecisionCount count={decisions} /></span>{/if}
      <span class="oc-w {recency(chat.updatedMs, now)}">{whenLabel(chat.updatedMs, now)}</span>
    </a>
  {/each}
</div>

<style>
  .oc {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border-radius: var(--radius-8);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--line-outline),
      var(--shadow-panel);
  }
  .oc-row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
    padding: 10px 12px;
    border-radius: var(--radius-5);
    color: var(--text-strong);
  }
  .oc-row:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .oc-t {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .oc-t b {
    font-size: 13px;
    font-weight: var(--weight-strong);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .oc-m {
    font-size: 12px;
    color: var(--text-faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .oc-badge {
    flex: none;
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--radius-pill);
    background: var(--find-bg);
    box-shadow: var(--find-glow);
    color: var(--find-text);
    font-size: 12px;
    font-weight: var(--weight-label);
  }
  .oc-w {
    flex: none;
    min-width: 34px;
    font-size: 12px;
    color: var(--text-faint);
    text-align: right;
  }
  /* On a phone the badge moves under the title and the time to the top. */
  @media (max-width: 640px) {
    .oc-row {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr) auto;
      align-items: start;
      column-gap: 12px;
      row-gap: 6px;
    }
    .oc-row > :global(.mark) {
      grid-row: span 2;
      margin-top: 2px;
    }
    .oc-t b {
      font-size: 15px;
      white-space: normal;
    }
    .oc-badge,
    .oc-row > :global(.pill) {
      grid-column: 2;
      justify-self: start;
    }
    .oc-w {
      grid-column: 3;
      grid-row: 1;
    }
  }
</style>
