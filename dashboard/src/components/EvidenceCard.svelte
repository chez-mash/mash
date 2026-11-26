<script lang="ts">
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import Icon from "$lib/components/Icon.svelte";
import { plainTitle } from "../lib/markdown";
import { speaker } from "../lib/memoryView";
import { type Evidence, label } from "../lib/records";
import { chatHref } from "../lib/route.svelte";
import type { ChatSummary } from "../lib/types";
import ChatPlace from "./ChatPlace.svelte";

// A quoted message behind a memory: the words, who said them and in which
// chat, and the way to the message. The reader's own words carry a butter
// edge, since they are what settled the memory. `brief` is the short form
// the memory page's preview uses.
let {
  source,
  node,
  chat,
  brief = false,
}: { source: Evidence; node: string; chat?: ChatSummary | undefined; brief?: boolean } = $props();
const href = $derived(chatHref(source.chatId, node, source.messageId, Boolean(node)));
const who = $derived(speaker(source.sourceRole));
const mine = $derived(source.sourceRole === "user");
</script>

<figure class="ev" class:mine class:brief>
  <blockquote>“{source.quote}”</blockquote>
  {#if brief}
    <figcaption>
      <span class="ev-who"><span>{who}</span><ChatPlace agent={chat?.agent} machine={chat?.machine || node} /></span>
      <a class="ev-go" {href} aria-label="Open the quoted message">Open<Icon icon={CaretRightIcon} size={14} /></a>
    </figcaption>
  {:else}
    <figcaption>
      <span class="ev-who"><span><b>{mine ? "You said this" : `${who} said this`}</b>{#if source.purpose}{` · ${label(source.purpose).toLowerCase()}`}{/if}</span>{#if chat}<span class="ev-chat" title={plainTitle(chat.title)}>{plainTitle(chat.title)}</span>{/if}</span>
      <a class="ev-go" {href}>Open the message<Icon icon={CaretRightIcon} size={14} /></a>
    </figcaption>
  {/if}
</figure>

<style>
  .ev {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 16px 18px;
    border: var(--border-outline);
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: var(--shadow-panel);
  }
  .ev.mine:not(.brief) {
    border-color: var(--linked-line);
    box-shadow: 0 0 0 1px var(--find-soft-bg), var(--shadow-panel);
  }
  .ev.brief {
    padding: 14px 15px;
  }
  blockquote {
    margin: 0;
    padding: 0;
    border: 0;
    font-size: 15px;
    font-weight: var(--weight-label);
    line-height: 1.5;
    color: var(--text-strong);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .brief blockquote {
    font-size: 14px;
    font-weight: var(--weight-regular);
  }
  figcaption {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    font-size: 12.5px;
    color: var(--text-muted);
  }
  .brief figcaption {
    gap: 6px;
    font-size: 12px;
  }
  .ev-who {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 10px;
    min-width: 0;
    flex: 1;
  }
  .ev-who b {
    font-weight: var(--weight-label);
    color: var(--text-body);
  }
  .ev-chat {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }
  .brief .ev-who {
    display: flex;
    flex-wrap: nowrap;
    align-items: center;
    gap: 10px;
    overflow: hidden;
    white-space: nowrap;
  }
  .ev-go {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
    font-size: 13px;
    font-weight: var(--weight-label);
    color: var(--text-strong);
    white-space: nowrap;
  }
  .ev-go:hover {
    color: var(--accent);
  }
  /* On a phone the way to the message gets a line of its own under a rule. */
  @container (max-width: 520px) {
    .ev:not(.brief) figcaption {
      flex-direction: column;
      align-items: stretch;
    }
    .ev:not(.brief) .ev-who {
      flex-direction: column;
      gap: 2px;
    }
    .ev:not(.brief) .ev-go {
      margin-left: 0;
      padding-top: 10px;
      border-top: var(--border-divider);
    }
  }
</style>
