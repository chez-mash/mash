<script lang="ts">
import type { Snippet } from "svelte";
import { plainTitle } from "../lib/markdown";
import { reading } from "../lib/reading.svelte";
import { chatHref } from "../lib/route.svelte";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import MetaChips from "./MetaChips.svelte";

// One chat in a card list (.list > .chatrow): its agent and title, how many
// decisions it produced, then where and when it happened.
let {
  chat,
  decisions = 0,
  icons = true,
  title,
  snippet,
  to,
}: {
  chat: ChatSummary;
  /** Decisions saved from this chat; a butter pill says how many. */
  decisions?: number;
  /** Whether the agent, machine and project carry small icons. */
  icons?: boolean;
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
</script>

<a class="chatrow" {href}>
  <span class="chatrow-t"><AgentMark agent={chat.agent} size="sm" /><b>{#if title}{@render title()}{:else}{plainTitle(chat.title)}{/if}</b>{#if decisions}<span class="pill decision">{decisions} decision{decisions === 1 ? "" : "s"}</span>{/if}</span>
  {#if snippet}<span class="snip">{@render snippet()}</span>{/if}
  <MetaChips {chat} {icons} />
</a>
