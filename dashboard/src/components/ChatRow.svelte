<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import type { Snippet } from "svelte";
import { chatMenu } from "../lib/chatMenu";
import { contextMenu } from "../lib/contextMenu.svelte";
import { plainTitle } from "../lib/markdown";
import { onThisMachine } from "../lib/openIn";
import { catalogOptions } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { chatHref } from "../lib/route.svelte";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import DecisionCount from "./DecisionCount.svelte";
import MetaChips from "./MetaChips.svelte";
import SubagentPill from "./SubagentPill.svelte";

// One chat in a card list (.list > .chatrow): its agent and title, how many
// decisions it produced, then where and when it happened.
let {
  chat,
  decisions = 0,
  title,
  snippet,
  to,
}: {
  chat: ChatSummary;
  /** Decisions saved from this chat; a butter pill says how many. */
  decisions?: number;
  /** The title as drawn, for instance with the searched words marked. */
  title?: Snippet;
  /** A line of the chat's text under the title. */
  snippet?: Snippet;
  /** Where the row leads, when not to where the reader left the chat. */
  to?: string;
} = $props();
// A chat opened earlier in this browser session reopens where it was left.
const href = $derived.by(() => {
  if (to !== undefined) return to;
  const position = reading.positions.find(
    (position) => position.id === chat.id && position.node === chat.machine,
  );
  return chatHref(chat.id, chat.machine, position?.message ?? "", position?.shared ?? false);
});
// A right-click offers the chat's own menu (lib/chatMenu.ts).
const catalogue = createQuery(() => catalogOptions);
function menu(event: MouseEvent, chat: ChatSummary, href: string): void {
  const local = onThisMachine(chat, catalogue.data?.localNode, catalogue.data?.sharedOnly ?? false);
  contextMenu.show(event, chatMenu(chat, href, local));
}
</script>

<a class="chatrow" {href} oncontextmenu={(event) => menu(event, chat, href)}>
  <span class="chatrow-t"><AgentMark agent={chat.agent} size="sm" /><b>{#if title}{@render title()}{:else}{plainTitle(chat.title)}{/if}</b><SubagentPill {chat} />{#if decisions}<span class="pill decision"><DecisionCount count={decisions} /></span>{/if}</span>
  {#if snippet}<span class="snip">{@render snippet()}</span>{/if}
  <MetaChips {chat} />
</a>
