<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import SparkleIcon from "phosphor-svelte/lib/SparkleIcon";
import Icon from "$lib/components/Icon.svelte";
import { whenLabel } from "../lib/browse";
import { chatMenu } from "../lib/chatMenu";
import { contextMenu } from "../lib/contextMenu.svelte";
import { plainTitle } from "../lib/markdown";
import { onThisMachine } from "../lib/openIn";
import { catalogOptions } from "../lib/queries";
import { recency } from "../lib/tone";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatPlace from "./ChatPlace.svelte";
import DecisionCount from "./DecisionCount.svelte";
import Highlight from "./Highlight.svelte";
import SubagentPill from "./SubagentPill.svelte";

// One chat in the search page's list: its agent and title with its age, the
// passage that matched with the searched words marked, then where it ran.
// The row the preview shows is the current one.
let {
  chat,
  href,
  query = "",
  snippet = "",
  decisions = 0,
  parentTitle,
  byMeaning = false,
  selected = false,
  onselect,
}: {
  chat: ChatSummary;
  href: string;
  query?: string;
  snippet?: string;
  decisions?: number;
  parentTitle?: string | undefined;
  /** Found by meaning rather than by its words. */
  byMeaning?: boolean;
  selected?: boolean;
  /** The reader moved to this row, with the pointer or the keyboard. */
  onselect?: () => void;
} = $props();
const now = Date.now();
// A right-click offers the chat's own menu (lib/chatMenu.ts).
const catalogue = createQuery(() => catalogOptions);
function menu(event: MouseEvent, chat: ChatSummary, href: string): void {
  const local = onThisMachine(chat, catalogue.data?.localNode, catalogue.data?.sharedOnly ?? false);
  contextMenu.show(event, chatMenu(chat, href, local));
}
</script>

<a class="sr chatrow" class:on={selected} {href} oncontextmenu={(event) => menu(event, chat, href)} aria-current={selected ? "true" : undefined} data-row onmouseenter={onselect} onfocus={onselect}>
  <span class="sr-t">
    <AgentMark agent={chat.agent} size="sm" />
    <b><Highlight text={plainTitle(chat.title)} {query} /></b>
    {#if byMeaning}<span class="pill info"><Icon icon={SparkleIcon} size={11} />by meaning</span>{/if}
    <SubagentPill {chat} {parentTitle} />
    {#if decisions}<span class="pill decision"><DecisionCount count={decisions} /></span>{/if}
    <span class="sr-when {recency(chat.updatedMs, now)}">{whenLabel(chat.updatedMs, now)}</span>
  </span>
  {#if snippet}<span class="sr-s"><Highlight text={plainTitle(snippet)} {query} /></span>{/if}
  <span class="sr-m"><ChatPlace agent={chat.agent} machine={chat.machine} project={chat.project ?? ""} led /></span>
</a>

<style>
  .sr {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    border: 0;
    border-radius: var(--radius-7);
    color: var(--text-strong);
    text-decoration: none;
  }
  .sr:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .sr.on {
    background: var(--selected-surface);
  }
  .sr-t {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .sr-t > b {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13.5px;
    font-weight: var(--weight-strong);
  }
  .sr-t > :global(.pill) {
    flex: none;
  }
  .sr-when {
    flex: none;
    margin-left: auto;
    padding-left: 8px;
    font-size: 12px;
    color: var(--text-faint);
  }
  .sr-s,
  .sr-m {
    padding-left: 30px;
  }
  .sr-s {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    overflow: hidden;
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-muted);
    overflow-wrap: anywhere;
  }
  .sr :global(mark) {
    box-shadow: var(--find-glow);
    color: var(--text-strong);
  }
  .sr-m {
    font-size: 12px;
    color: var(--text-faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
