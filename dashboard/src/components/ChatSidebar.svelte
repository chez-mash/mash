<script lang="ts">
// Beside a chat: an outline of the reader's own prompts, the other chats from
// the same project, and the memories saved from this chat. On a phone it
// takes the place of the transcript while open, so it offers a way back.
import Bookmark from "@lucide/svelte/icons/bookmark";
import Folder from "@lucide/svelte/icons/folder";
import ListOrdered from "@lucide/svelte/icons/list-ordered";
import X from "@lucide/svelte/icons/x";
import { projectLabel, whenLabel } from "../lib/browse";
import { saidText } from "../lib/message";
import { label, type NativeRecord } from "../lib/records";
import { chatHref, projectHref, recordHref, recordsHref } from "../lib/route.svelte";
import type { ChatSummary, Message } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import SubagentList from "./SubagentList.svelte";

let {
  id,
  machine,
  chat,
  sessions,
  children,
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
  /** The subagents this chat started. */
  children: readonly ChatSummary[];
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
  {#if closable}<button class="btn sessclose" type="button" onclick={onclose}><X size={15} aria-hidden="true" />Back to chat</button>{/if}
  {#if prompts.length >= 2}
    <div class="outl">
      <h2 class="h2"><ListOrdered size={15} aria-hidden="true" />Your prompts</h2>
      <div class="outl-l">
        {#each prompts as message, index (message.id)}
          <button type="button" class:on={currentMessage === message.id} aria-current={currentMessage === message.id ? "location" : undefined} onclick={() => onprompt(message.id)}><small>{index + 1}</small><span>{saidText(message).replace(/\s+/g, " ").slice(0, 80)}</span></button>
        {/each}
      </div>
    </div>
  {/if}
  <SubagentList {children} />
  {#if chat?.project}<h2 class="h2"><a href={projectHref(chat.project)}><Folder size={15} aria-hidden="true" />{projectLabel(chat.project)} sessions</a></h2>{/if}
  {#if projectSessions.length}
    <div class="card pad0 list sess-l">
      {#each projectSessions as session (session.machine + session.id)}
        {@const current = session.id === id && session.machine === machine}
        <a class="chatrow ses" class:on={current} href={chatHref(session.id, session.machine)} aria-current={current ? "page" : undefined}><AgentMark agent={session.agent} size="sm" /><span><span class="clamp">{session.title}</span><small>{whenLabel(session.updatedMs, now)}</small></span></a>
      {/each}
    </div>
  {/if}
  <h2 class="h2"><a href={recordsHref(memoryParams)}><Bookmark size={15} aria-hidden="true" />Memory from this chat</a></h2>
  {#if recordsError}<p class="meta" role="alert">{recordsError.message}</p>{/if}
  {#each records ?? [] as record (record.id)}
    {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
    {#if revision}<a class="card tight butter saved" href={recordHref(record.id, record.node, recordsHref(memoryParams))}><b class="clamp">{revision.statement}</b><small>{label(revision.details.kind).toLowerCase()} / {label(revision.details.state).toLowerCase()}</small></a>{/if}
  {/each}
  {#if records && !records.length}<p class="card tight cream" style:font-size="13px">Nothing saved from this chat.</p>{/if}
</nav>
