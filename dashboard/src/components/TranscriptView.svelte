<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import ArrowsClockwiseIcon from "phosphor-svelte/lib/ArrowsClockwiseIcon";
import CaretDoubleDownIcon from "phosphor-svelte/lib/CaretDoubleDownIcon";
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
import LightbulbIcon from "phosphor-svelte/lib/LightbulbIcon";
import TerminalIcon from "phosphor-svelte/lib/TerminalIcon";
import { tick, untrack } from "svelte";
import Icon from "$lib/components/Icon.svelte";
import { agents } from "../lib/agents";
import { transcript } from "../lib/api";
import { ChatReading, toBottom } from "../lib/chatReading.svelte";
import { collectChat, type ExportFormat, exportChat, type ReadPage } from "../lib/export";
import { countLabel, type FindMatch, findMatches } from "../lib/find";
import { clearMatches, paintMatches, revealMatch } from "../lib/findHighlight";
import { concerns, onChange } from "../lib/live";
import { messageText, saidText, shownText } from "../lib/message";
import { onThisMachine } from "../lib/openIn";
import { catalogOptions, stableRead } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { recordPage } from "../lib/records";
import { chatHref } from "../lib/route.svelte";
import { stepLabel, stepParts, stepsSummary, transcriptItems } from "../lib/steps";
import { childrenOf } from "../lib/subagents";
import { followTopBar } from "../lib/topBar";
import { startsAgentTurn } from "../lib/turns";
import type { Message } from "../lib/types";
import { updates } from "../lib/updates.svelte";
import ChatEnds from "./ChatEnds.svelte";
import "./reading.css";
import AgentMark from "./AgentMark.svelte";
import Attachments from "./Attachments.svelte";
import ChatHeader from "./ChatHeader.svelte";
import ChatSidebar from "./ChatSidebar.svelte";
import FindBar from "./FindBar.svelte";
import HitMap from "./HitMap.svelte";
import MessageActions from "./MessageActions.svelte";
import MessageBody from "./MessageBody.svelte";
import QueryError from "./QueryError.svelte";
import SavedMemory from "./SavedMemory.svelte";
import SubagentList from "./SubagentList.svelte";
import SubagentSpawns from "./SubagentSpawns.svelte";
import ToolStep from "./ToolStep.svelte";

let {
  id,
  node,
  messageId,
  shared = false,
}: { id: string; node: string; messageId: string; shared?: boolean } = $props();
const catalogue = createQuery(() => catalogOptions);
const machine = $derived(node || "");
const displayMachine = $derived(machine || catalogue.data?.localNode || "Local");
// Lists fold subagents into their parents; a subagent's own page still finds it.
const all = $derived(catalogue.data?.all ?? []);
const chat = $derived(all.find((chat) => chat.id === id && chat.machine === displayMachine));
const children = $derived(chat ? childrenOf(all, chat) : []);
// Where the screen is only moderately wide, the list beside the transcript
// starts hidden, so the transcript has a readable measure; its button shows
// the list. Crossing that width starts it over.
const middleQuery =
  typeof window === "undefined"
    ? undefined
    : window.matchMedia("(min-width: 701px) and (max-width: 1199px)");
let showSessions = $state(!(middleQuery?.matches ?? false));
// On a phone the list takes the whole screen in place of the transcript when
// the reader opens it, and closing it returns to where they were reading.
const phoneQuery =
  typeof window === "undefined" ? undefined : window.matchMedia("(max-width: 700px)");
let phone = $state(phoneQuery?.matches ?? false);
let sessionsOpen = $state(false);
$effect(() => {
  if (!phoneQuery || !middleQuery) return;
  const update = (): void => {
    phone = phoneQuery.matches;
  };
  const middle = (): void => {
    showSessions = !middleQuery.matches;
  };
  phoneQuery.addEventListener("change", update);
  middleQuery.addEventListener("change", middle);
  return () => {
    phoneQuery.removeEventListener("change", update);
    middleQuery.removeEventListener("change", middle);
  };
});
const sessionsShown = $derived(phone ? sessionsOpen : showSessions);
let readingAt = 0;
async function openSessions(): Promise<void> {
  readingAt = window.scrollY;
  sessionsOpen = true;
  await tick();
  window.scrollTo(0, 0);
}
/** Puts the transcript back; `restore` returns to where the reader was. */
async function closeSessions(restore: boolean): Promise<void> {
  if (!sessionsOpen) return;
  sessionsOpen = false;
  // The transcript has no height while the list replaces it.
  await tick();
  if (restore) window.scrollTo(0, readingAt);
}
function toggleSessions(): void {
  if (!phone) showSessions = !showSessions;
  else if (sessionsOpen) void closeSessions(true);
  else void openSessions();
}
let exportOpen = $state(false);
let currentMessage = $state("");
const memoryParams = $derived(
  new URLSearchParams({
    chat: id,
    ...(shared || (machine && machine !== catalogue.data?.localNode)
      ? { shared: "true", node: machine }
      : {}),
  }),
);
const memories = createQuery(() => ({
  ...stableRead,
  enabled: catalogue.data !== undefined,
  queryKey: ["chat-memory", id, memoryParams.toString()],
  queryFn: ({ signal }) => recordPage(memoryParams.toString(), signal),
}));
function remember(): void {
  const pane = transcriptPane;
  if (!pane || !reader.loaded) return;
  // The page scrolls under a sticky header; the first message below it is
  // where the reader is.
  const elements = Array.from(pane.querySelectorAll<HTMLElement>("[data-message]"));
  const first = elements.find((element) => element.getBoundingClientRect().bottom > 90);
  if (first?.dataset.message) {
    currentMessage = first.dataset.message;
    reading.remember({ id, node: displayMachine, message: first.dataset.message, shared });
  }
}
function jump(message: string): void {
  currentMessage = message;
  document.getElementById(`message-${message}`)?.scrollIntoView({ block: "start" });
}

let transcriptPane: HTMLElement | undefined = $state();
const reader = new ChatReading(
  () => ({ id, machine, shared }),
  untrack(() => (messageId ? { at: "message", message: messageId } : { at: "end" })),
  { pane: () => transcriptPane, covered: () => sessionsOpen },
);
// The rail's Check for updates rereads the chat in place, so the reader keeps
// their spot and new messages arrive the way the stream brings them.
updates.offer({
  check: () => reader.checkForUpdates(true),
  owns: ([kind]) => kind === "transcript",
});
const readPage: ReadPage = (id, node, offset, signal, limit) =>
  transcript(id, node, offset, signal, shared, limit);

// Find in chat reads the whole chat, every page, so matches on pages not
// shown yet still count; going to one loads its page and opens any
// collapsed step around it.
let findOpen = $state(false);
let findQuery = $state("");
let findIndex = $state(0);
let findActive = $state(false);
let findProgress = $state(0);
let findInput = $state<HTMLInputElement>();
let pendingFind: FindMatch | undefined;
const whole = createQuery(() => ({
  queryKey: ["find-chat", id, machine, shared],
  enabled: findOpen,
  staleTime: 60_000,
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    collectChat(
      id,
      machine,
      signal,
      (count) => {
        findProgress = count;
      },
      readPage,
    ),
}));
const matches = $derived(
  findOpen && whole.data
    ? findMatches(
        whole.data.map((message) => ({ id: message.id, text: shownText(message) })),
        findQuery,
      )
    : [],
);
const findStatus = $derived.by(() => {
  if (whole.isPending) return `Reading the chat… ${findProgress.toLocaleString()} messages`;
  if (whole.error) return "Couldn’t read the whole chat";
  if (!findQuery.trim()) return "";
  if (!matches.length) return "No matches";
  return findActive
    ? countLabel(findIndex, matches.length)
    : `${countLabel(0, matches.length).replace(/^1 of /, "")} matches`;
});
$effect(() => {
  void findQuery;
  findActive = false;
  findIndex = 0;
});
function openFind(): void {
  findOpen = true;
  void tick().then(() => {
    findInput?.focus();
    findInput?.select();
  });
}
function closeFind(): void {
  findOpen = false;
  findActive = false;
  clearMatches();
}
function go(step: 1 | -1): void {
  const count = matches.length;
  if (!count) return;
  findIndex = findActive ? (findIndex + step + count) % count : step > 0 ? 0 : count - 1;
  findActive = true;
  const target = matches[findIndex];
  if (!target) return;
  // A match outside what is loaded is read around its own message.
  if (reader.loaded?.messages.some((message) => message.id === target.messageId))
    void reveal(target);
  else {
    pendingFind = target;
    reader.goTo({ at: "message", message: target.messageId });
  }
}
async function reveal(target: FindMatch): Promise<void> {
  await tick();
  if (transcriptPane) await revealMatch(transcriptPane, findQuery, target);
}
$effect(() => {
  const query = findQuery;
  const open = findOpen;
  void reader.loaded;
  if (!open) return;
  void tick().then(() => {
    if (transcriptPane && !pendingFind) paintMatches(transcriptPane, query);
  });
});
$effect(() => () => clearMatches());
let notice = $state("");
let exporting = $state(false);
let exportController: AbortController | undefined;
// Keep the sidebar mounted across chat navigation. Only explicit navigation
// resets where the reader is; catalogue updates never touch it.
$effect(() => {
  const route = [id, node, messageId, shared] as const;
  reader.restart(route[2] ? { at: "message", message: route[2] } : { at: "end" });
  notice = "";
  exportOpen = false;
  sessionsOpen = false;
  window.scrollTo({ top: 0, behavior: "instant" });
  return () => exportController?.abort();
});

// Puts the reader where they asked to be once that stretch has loaded.
let arrived = -1;
$effect(() => {
  const ready = reader.loaded !== undefined;
  const wanted = reader.arrival;
  if (!ready || arrived === wanted) return;
  arrived = wanted;
  const first = untrack(() => reader.loaded?.messages[0]);
  const place = reader.place;
  const linked = place.at === "message" ? place.message : "";
  if (first && catalogue.data) {
    currentMessage = linked || first.id;
    const position = { id, node: displayMachine, message: first.id, shared };
    untrack(() => reading.remember(position));
  }
  void tick().then(async () => {
    if (pendingFind) {
      const target = pendingFind;
      pendingFind = undefined;
      await reveal(target);
    } else if (reader.place.at === "end") toBottom();
    else if (linked)
      document.getElementById(`message-${linked}`)?.scrollIntoView({ block: "center" });
    else window.scrollTo({ top: 0, behavior: "instant" });
    await tick();
    reader.placed = reader.arrival;
    reader.fill();
  });
});

// The list beside the chat stays in view under the top bar.
$effect(followTopBar);
// The same chat ID may exist on another machine; follow the copy shown.
$effect(() => {
  const chatId = id;
  const from = displayMachine;
  const local = !shared && (!machine || machine === catalogue.data?.localNode);
  return onChange((change) => {
    if (concerns(change, chatId, from, local)) void reader.checkForUpdates();
  });
});

function plainText(message: Message): string {
  return messageText(message);
}
async function startExport(format: ExportFormat): Promise<void> {
  exportOpen = false;
  if (exporting) return;
  const controller = new AbortController();
  exportController = controller;
  exporting = true;
  const source = { id, node: machine, title: chat?.title ?? id, machine: displayMachine, readPage };
  try {
    await exportChat(format, source, controller.signal, (text) => {
      notice = text;
    });
  } finally {
    exporting = false;
    exportController = undefined;
  }
}
async function copySession(): Promise<void> {
  try {
    await navigator.clipboard.writeText(id);
    notice = "Session ID copied.";
  } catch {
    notice = `Copy failed. The session ID is ${id}.`;
  }
}
// Consecutive tool calls, tool output, thinking and context read as one run
// of steps, folded so the conversation stays the main thing on the page.
const items = $derived(reader.loaded ? transcriptItems(reader.loaded) : []);
function characters(message: Message): string {
  const count = plainText(message).length;
  return `${count.toLocaleString()} character${count === 1 ? "" : "s"}`;
}
// Context is not a prompt: only what a person said is listed.
const prompts = $derived(
  (reader.loaded?.messages ?? []).filter(
    (message) => message.role === "user" && message.presentation === "dialogue",
  ),
);
</script>

<svelte:window
  onscroll={() => {
    remember();
    reader.fill();
  }}
  onkeydown={(event) => {
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      openFind();
    } else if (event.key === "Escape") {
      exportOpen = false;
      if (findOpen) closeFind();
    }
  }}
/>
<div class="chat" class:nosess={!showSessions} class:sessopen={sessionsOpen}>
  <ChatSidebar {id} machine={displayMachine} {chat} sessions={catalogue.data?.chats ?? []} {prompts} {currentMessage} records={memories.data?.records} recordsError={memories.error} {memoryParams} closable={phone && sessionsOpen} onclose={() => closeSessions(true)} onprompt={async (message) => { await closeSessions(false); jump(message); }} />
  <article class="log" bind:this={transcriptPane}>
    <ChatHeader {id} href={chatHref(id, machine, "", shared)} {chat} {all} machine={displayMachine} total={reader.total} local={chat ? onThisMachine(chat, catalogue.data?.localNode, shared) : false} {phone} {sessionsShown} {findOpen} bind:exportOpen canExport={!exporting && !!reader.loaded} ontogglesessions={toggleSessions} onfind={openFind} oncopy={copySession} onexport={startExport} onnotice={(text) => (notice = text)} />
    {#if findOpen}<FindBar bind:query={findQuery} bind:input={findInput} status={findStatus} matches={matches.length} onstep={go} onclose={closeFind} />{/if}
    {#if exporting}<p class="card tight cream" style:display="flex" style:font-size="12.5px"><button class="a sp" type="button" onclick={() => exportController?.abort()}>Cancel export</button></p>{/if}
    <QueryError title="Couldn’t load the chat list" query={catalogue} retryLabel="Retry catalogue" />
    {#each catalogue.data?.warnings ?? [] as warning (warning)}<p role="status" class="meta">{warning}</p>{/each}
    {#if reader.stretches.isPending}
      <div class="card" role="status">
        <p style:display="flex" style:align-items="center" style:gap="10px" style:font="600 13px var(--type-mono)"><Icon icon={TerminalIcon} size={16} />Loading messages…</p>
        <div class="skel" style:width="62%"></div><div class="skel" style:width="86%"></div><div class="skel" style:width="54%"></div>
      </div>
    {/if}
    <QueryError title="Couldn’t load this conversation" query={reader.stretches} />
    {#if reader.changed}<button type="button" class="btn" onclick={() => reader.showChanged()}><Icon icon={ArrowsClockwiseIcon} size={15} />This chat has changed. Show updated messages</button>{/if}
    {#if notice}<p role="status" class="meta">{notice}</p>{/if}
    <div class="newpill" aria-live="polite">{#if reader.added && !sessionsOpen}<button type="button" class="btn p" onclick={() => reader.showNew()}><Icon icon={CaretDoubleDownIcon} size={15} state="strong" />{reader.added.toLocaleString()} new message{reader.added === 1 ? "" : "s"}</button>{/if}</div>
    <div class="tv-body">
    <div class="tv-msgs">
    {#if reader.loaded}
      {#if !sessionsOpen}<ChatEnds onstart={() => reader.toStart()} onend={() => reader.toEnd()} />{/if}
      {#if !reader.loaded.messages.length}<p class="card tight cream" style:font-size="13.5px">No messages here. The chat may have been removed.</p>{/if}
      {#if reader.stretches.isFetchingPreviousPage}<p class="meta" role="status">Loading earlier messages…</p>{/if}
      {#snippet saved(message: Message)}
        <SavedMemory records={memories.data?.records ?? []} messageId={message.id} {memoryParams} />
      {/snippet}
      <!-- Thinking and context: a closed row that opens to plain text. -->
      {#snippet note(message: Message)}
        {@const context = message.presentation === "context"}
        <details class="fold" class:hit={message.id === messageId} id="message-{message.id}" data-message={message.id} open={message.id === messageId} style:scroll-margin-top="80px">
          <summary>{#if context}<Icon icon={FileTextIcon} size={15} />{:else}<Icon icon={LightbulbIcon} size={15} />{/if}{stepLabel(message)}<span class="mtx">{context ? characters(message) : message.at}</span><span class="cv"><Icon icon={CaretRightIcon} size={15} /></span></summary>
          <div><pre class="pre" style:max-height="32rem" style:overflow="auto">{plainText(message)}</pre></div>
        </details>
        {@render files(message)}
        {@render saved(message)}
      {/snippet}
      {#snippet files(message: Message)}
        {#if message.attachments?.length}<Attachments attachments={message.attachments} />{/if}
      {/snippet}
      {#snippet actions(message: Message)}
        <MessageActions {message} href={chatHref(id, machine, message.id, shared)} />
      {/snippet}
      <!-- Who speaks: the agent's name at the start of each of its turns. -->
      {#snippet who()}
        <p class="tv-who">{#if chat}<AgentMark agent={chat.agent} size="sm" />{agents[chat.agent].name}{:else}Agent{/if}</p>
      {/snippet}
      {#each items as item, index (item.kind === "message" ? item.message.id : item.messages[0]?.id)}
        {#if startsAgentTurn(items, index)}{@render who()}{/if}
        {#if item.kind === "steps"}
          {@const only = item.messages.length === 1 ? item.messages[0] : undefined}
          {#if only?.presentation === "context"}
            {@render note(only)}
          {:else}
            <details class="fold" open={item.messages.some((message) => message.id === messageId)}>
              <summary><Icon icon={TerminalIcon} size={15} />{stepsSummary(item)}<span class="cv"><Icon icon={CaretRightIcon} size={15} /></span></summary>
              <div>
                {#each stepParts(item) as part (part.kind === "call" ? part.call.id : part.message.id)}
                  {#if part.kind === "call"}
                    <ToolStep call={part.call} selected={messageId} />
                    {#each part.messages as message (message.id)}{@render files(message)}{@render saved(message)}{/each}
                  {:else}
                    {@render note(part.message)}
                  {/if}
                {/each}
              </div>
            </details>
            <SubagentSpawns messages={item.messages} {children} />
          {/if}
        {:else}
          {@const message = item.message}
          {@const selected = message.id === messageId}
          <div class="hitbox" class:hit={selected} id="message-{message.id}" data-message={message.id} style:scroll-margin-top="80px">
            {#if selected && message.role !== "user"}<span class="hitlabel">Linked message</span>{/if}
            {#if message.role === "user"}
              <span class="tv-you">{selected ? "You · linked message" : "You"}</span>
              <div class="you"><MessageBody text={saidText(message)} own />{@render actions(message)}</div>
              <div class="tv-files">{@render files(message)}</div>
              {#if message.dialogue !== undefined}<details class="fold"><summary><Icon icon={FileTextIcon} size={15} />Context added by the agent<span class="mtx">{characters(message)}</span><span class="cv"><Icon icon={CaretRightIcon} size={15} /></span></summary><div><pre class="pre" style:max-height="32rem" style:overflow="auto">{plainText(message)}</pre></div></details>{/if}
            {:else}
              <div class="msg">
                {#each message.blocks as block, i (i)}
                  {#if block.type === "text"}<MessageBody text={block.markdown} />{/if}
                {/each}
                {@render files(message)}
                {@render actions(message)}
              </div>
              {#if message.activity}<details class="fold"><summary><Icon icon={TerminalIcon} size={15} />Recorded tool activity<span class="cv"><Icon icon={CaretRightIcon} size={15} /></span></summary><div><pre class="pre" style:max-height="20rem" style:overflow="auto">{message.activity}</pre></div></details>{/if}
            {/if}
            {@render saved(message)}
          </div>
        {/if}
      {/each}
      {#if reader.stretches.isFetchingNextPage}<p class="meta" role="status">Loading later messages…</p>{/if}
    {/if}
    </div>
    <!-- Beside the messages: the subagents this chat started, and while
         finding, where the matches fall in the whole chat. -->
    {#if children.length || (findOpen && matches.length)}
      <aside class="tv-side">
        {#if findOpen && matches.length && whole.data}<HitMap count={whole.data.length} {matches} current={findActive ? findIndex : -1} />{/if}
        <SubagentList {children} />
      </aside>
    {/if}
    </div>
  </article>
</div>
