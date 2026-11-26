<script lang="ts">
// Beside a chat: the other chats from the same project, as a list to move
// between, then an outline of the reader's own prompts and the memories saved
// from this chat. On a phone it takes the place of the transcript while open,
// so it offers a way back. The subagents a chat started are listed beside
// the transcript instead (see TranscriptView).
import BrainIcon from "phosphor-svelte/lib/BrainIcon";
import ChatIcon from "phosphor-svelte/lib/ChatIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import ListNumbersIcon from "phosphor-svelte/lib/ListNumbersIcon";
import XIcon from "phosphor-svelte/lib/XIcon";
import Icon from "$lib/components/Icon.svelte";
import { projectLabel, whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { saidText } from "../lib/message";
import { label, type NativeRecord } from "../lib/records";
import { chatHref, projectHref, recordHref, recordsHref } from "../lib/route.svelte";
import { recency } from "../lib/tone";
import type { ChatSummary, Message } from "../lib/types";
import ChatPlace from "./ChatPlace.svelte";
import KindIcon from "./KindIcon.svelte";
import SubagentPill from "./SubagentPill.svelte";

let {
  id,
  machine,
  chat,
  sessions,
  prompts,
  currentMessage,
  records,
  recordsError,
  memoryParams,
  closable,
  onclose,
  onprompt,
}: {
  id: string;
  /** The machine the chat shown is read from. */
  machine: string;
  chat: ChatSummary | undefined;
  sessions: readonly ChatSummary[];
  /** What a person said in the loaded part of the chat, in order. */
  prompts: readonly Message[];
  currentMessage: string;
  records: readonly NativeRecord[] | undefined;
  recordsError: Error | null;
  memoryParams: URLSearchParams;
  /** Whether the list covers the transcript, as on a phone, and so offers a way back. */
  closable: boolean;
  onclose: () => void;
  onprompt: (message: string) => void;
} = $props();
const now = Date.now();
// How many chats the project holds, for the list's heading.
const projectCount = $derived(
  chat ? sessions.filter((session) => session.project === chat.project).length : 0,
);
// Other chats from the same project, with this one kept in view.
const projectSessions = $derived.by(() => {
  if (!chat) return [];
  const same = sessions.filter((session) => session.project === chat.project).slice(0, 6);
  // Compared by identity, not by object: the catalogue's two lists are copied apart.
  const listed = same.some((session) => session.id === chat.id && session.machine === chat.machine);
  return listed ? same : [chat, ...same.slice(0, 5)];
});
</script>

<nav class="sess" aria-label="Prompts and sessions">
  {#if closable}<button class="btn sessclose" type="button" onclick={onclose}><Icon icon={XIcon} size={15} />Back to chat</button>{/if}
  {#if chat?.project}
    <h2 class="cs-h"><a href={projectHref(chat.project)}><Icon icon={FolderIcon} size={14} /><span>{projectLabel(chat.project)}</span></a><span class="cs-n" title="{projectCount.toLocaleString()} chat{projectCount === 1 ? '' : 's'}"><Icon icon={ChatIcon} size={12} />{projectCount.toLocaleString()}<span class="vh">{projectCount === 1 ? " chat" : " chats"}</span></span></h2>
  {/if}
  {#if projectSessions.length}
    <div class="cs-l">
      {#each projectSessions as session (session.machine + session.id)}
        {@const current = session.id === id && session.machine === machine}
        <a class="cs-r" class:on={current} href={chatHref(session.id, session.machine)} aria-current={current ? "page" : undefined}>
          <span class="cs-t"><b>{plainTitle(session.title)}</b><small class={recency(session.updatedMs, now)}>{whenLabel(session.updatedMs, now)}</small></span>
          <span class="cs-m"><ChatPlace agent={session.agent} machine={session.machine} /><SubagentPill chat={session} /></span>
        </a>
      {/each}
    </div>
  {/if}
  {#if prompts.length >= 2}
    <div class="outl">
      <h2 class="cs-h"><span><Icon icon={ListNumbersIcon} size={14} />Your prompts</span></h2>
      <div class="outl-l">
        {#each prompts as message, index (message.id)}
          <button type="button" class:on={currentMessage === message.id} aria-current={currentMessage === message.id ? "location" : undefined} onclick={() => onprompt(message.id)}><small>{index + 1}</small><span>{saidText(message).replace(/\s+/g, " ").slice(0, 80)}</span></button>
        {/each}
      </div>
    </div>
  {/if}
  <h2 class="cs-h"><a href={recordsHref(memoryParams)}><Icon icon={BrainIcon} size={14} /><span>Memory from this chat</span></a></h2>
  {#if recordsError}<p class="meta" role="alert">{recordsError.message}</p>{/if}
  {#each records ?? [] as record (record.id)}
    {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
    {#if revision}<a class="cs-mem" href={recordHref(record.id, record.node, recordsHref(memoryParams))}><b class="clamp">{revision.statement}</b><small><KindIcon kind={revision.details.kind} size={12} />{label(revision.details.state).toLowerCase()}</small></a>{/if}
  {/each}
  {#if records && !records.length}<p class="cs-none">Nothing saved from this chat.</p>{/if}
</nav>

<style>
  .sess {
    gap: 6px;
    padding: 12px;
  }
  .cs-h {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 32px;
    padding: 6px 8px 4px;
    font-size: 13.5px;
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .cs-h:not(.sess > :first-child) {
    margin-top: 14px;
    font-size: 12px;
    font-weight: var(--weight-label);
    color: var(--text-faint);
  }
  .cs-h > a,
  .cs-h > span:first-child {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    color: inherit;
  }
  .cs-h > a > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sess > .cs-h:first-child > a :global(svg) {
    display: none;
  }
  /* The project's name heads the column in a band of its own. */
  .sess > .cs-h:first-child {
    min-height: 46px;
    margin: -12px -12px 6px;
    padding: 0 20px;
    border-bottom: var(--border-divider);
  }
  .sess:has(> .sessclose) > .cs-h:nth-child(2) {
    min-height: 46px;
    margin: 0 -12px 6px;
    padding: 0 20px;
    border-bottom: var(--border-divider);
  }
  .cs-n {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: var(--weight-regular);
    color: var(--text-faint);
  }
  .cs-l {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cs-r {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px;
    border-radius: var(--radius-6);
    color: var(--text-strong);
  }
  .cs-r:hover {
    background: var(--surface-hover);
    color: var(--text-strong);
  }
  .cs-r.on {
    background: var(--selected-surface);
  }
  .cs-t {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .cs-t b {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    font-weight: var(--weight-strong);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
  }
  .cs-t small {
    flex: none;
    font-size: 12px;
    color: var(--text-faint);
  }
  .cs-m {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-faint);
  }
  .cs-m :global(.mark) {
    width: 16px;
    height: 16px;
  }
  /* In the narrow list a subagent's pill takes a line of its own, whole. */
  .cs-m :global(.pill.fork) {
    max-width: 100%;
  }
  .cs-mem {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0 2px;
    padding: 10px 12px;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow:
      0 0 0 var(--stroke-outline) var(--memory-ring),
      var(--memory-glow);
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text-strong);
  }
  .cs-mem small {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    color: var(--text-faint);
  }
  .vh {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .cs-none {
    padding: 0 8px;
    font-size: 12.5px;
    color: var(--text-faint);
  }
</style>
