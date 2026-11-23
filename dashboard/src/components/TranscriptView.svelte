<script lang="ts">
import ArrowDownToLine from "@lucide/svelte/icons/arrow-down-to-line";
import ArrowUpToLine from "@lucide/svelte/icons/arrow-up-to-line";
import Bookmark from "@lucide/svelte/icons/bookmark";
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import ChevronUp from "@lucide/svelte/icons/chevron-up";
import ChevronsDown from "@lucide/svelte/icons/chevrons-down";
import Copy from "@lucide/svelte/icons/copy";
import Download from "@lucide/svelte/icons/download";
import FileCode from "@lucide/svelte/icons/file-code";
import FileText from "@lucide/svelte/icons/file-text";
import Folder from "@lucide/svelte/icons/folder";
import Lightbulb from "@lucide/svelte/icons/lightbulb";
import ListOrdered from "@lucide/svelte/icons/list-ordered";
import PanelLeft from "@lucide/svelte/icons/panel-left";
import Printer from "@lucide/svelte/icons/printer";
import RefreshCw from "@lucide/svelte/icons/refresh-cw";
import Search from "@lucide/svelte/icons/search";
import Terminal from "@lucide/svelte/icons/terminal";
import X from "@lucide/svelte/icons/x";
import { createInfiniteQuery, createQuery, useQueryClient } from "@tanstack/svelte-query";
import { tick, untrack } from "svelte";
import {
  type ChatPlace,
  type ChatStretch,
  chatStretch,
  earlierPlace,
  laterPlace,
  type MessagePage,
  PAGE_SIZE,
  transcript,
} from "../lib/api";
import { projectLabel, whenLabel } from "../lib/browse";
import {
  collectChat,
  downloadChat,
  type ExportFormat,
  htmlExport,
  markdownExport,
} from "../lib/export";
import { countLabel, type FindMatch, findMatches } from "../lib/find";
import { bringIntoView, clearMatches, paintMatches } from "../lib/findHighlight";
import { concerns, onChange, pageUpdate } from "../lib/live";
import { plainTitle } from "../lib/markdown";
import { messageText, saidText, shownText } from "../lib/message";
import { catalogOptions, stableRead } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { label, recordPage } from "../lib/records";
import { chatHref, projectHref, recordHref, recordsHref } from "../lib/route.svelte";
import { stepLabel, stepParts, stepsSummary, transcriptItems } from "../lib/steps";
import type { Message } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import MessageActions from "./MessageActions.svelte";
import MessageBody from "./MessageBody.svelte";
import MetaChips from "./MetaChips.svelte";
import QueryError from "./QueryError.svelte";
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
const sessions = $derived(catalogue.data?.chats ?? []);
const chat = $derived(sessions.find((chat) => chat.id === id && chat.machine === displayMachine));
/** Where the reader asked to start reading; scrolling reads on from there. */
let place = $state<ChatPlace>(
  untrack(() => (messageId ? { at: "message", message: messageId } : { at: "end" })),
);
/** Counts the reader's requests to go somewhere, so asking again goes again. */
let arrival = $state(0);
let showSessions = $state(true);
// On a phone the list takes the whole screen in place of the transcript when
// the reader opens it, and closing it returns to where they were reading.
const phoneQuery =
  typeof window === "undefined" ? undefined : window.matchMedia("(max-width: 700px)");
let phone = $state(phoneQuery?.matches ?? false);
let sessionsOpen = $state(false);
$effect(() => {
  if (!phoneQuery) return;
  const update = (): void => {
    phone = phoneQuery.matches;
  };
  phoneQuery.addEventListener("change", update);
  return () => phoneQuery.removeEventListener("change", update);
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
// Other chats from the same project, with this one kept in view.
const projectSessions = $derived.by(() => {
  if (!chat) return [];
  const same = sessions.filter((session) => session.project === chat.project).slice(0, 6);
  return same.includes(chat) ? same : [chat, ...same.slice(0, 5)];
});
const now = Date.now();
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
  if (!pane || !loaded) return;
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
const shortcut =
  typeof navigator !== "undefined" && /mac|iphone|ipad/i.test(navigator.platform) ? "⌘F" : "Ctrl F";
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
      (id, node, offset, signal, limit) => transcript(id, node, offset, signal, shared, limit),
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
  if (loaded?.messages.some((message) => message.id === target.messageId)) void reveal(target);
  else {
    pendingFind = target;
    goTo({ at: "message", message: target.messageId });
  }
}
async function reveal(target: FindMatch): Promise<void> {
  await tick();
  const pane = transcriptPane;
  if (!pane) return;
  const element = pane.querySelector(`[data-message="${CSS.escape(target.messageId)}"]`);
  if (!element) return;
  if (element instanceof HTMLDetailsElement) element.open = true;
  for (
    let fold = element.closest("details");
    fold;
    fold = fold.parentElement?.closest("details") ?? null
  )
    fold.open = true;
  await tick();
  const spot = paintMatches(pane, findQuery, { element, occurrence: target.occurrence });
  // The match may sit in a fold inside the message, such as its added context.
  if (spot instanceof Range) {
    for (
      let fold = spot.startContainer.parentElement?.closest("details");
      fold && element.contains(fold);
      fold = fold.parentElement?.closest("details")
    )
      fold.open = true;
    await tick();
  }
  if (spot) bringIntoView(spot);
}
$effect(() => {
  const query = findQuery;
  const open = findOpen;
  void loaded;
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
  place = route[2] ? { at: "message", message: route[2] } : { at: "end" };
  untrack(() => arrival++);
  changed = false;
  held = undefined;
  added = 0;
  notice = "";
  exportOpen = false;
  sessionsOpen = false;
  window.scrollTo({ top: 0, behavior: "instant" });
  return () => exportController?.abort();
});

// The chat is read in stretches as the reader scrolls: it opens at its end,
// or around a linked message, and the stretches before and after it load as
// they come near the screen. At most ten stay loaded; the far ones are let
// go and read again on the way back.
const client = useQueryClient();
const chatKey = $derived(["transcript", id, machine, shared, JSON.stringify(place)]);
const stretches = createInfiniteQuery(() => ({
  ...stableRead,
  queryKey: chatKey,
  initialPageParam: place,
  queryFn: ({ pageParam, signal }: { pageParam: ChatPlace; signal: AbortSignal }) =>
    chatStretch(id, machine, pageParam, signal, shared),
  getPreviousPageParam: earlierPlace,
  getNextPageParam: laterPlace,
  maxPages: 10,
}));
/** Everything loaded, read as one page. */
const loaded = $derived.by((): MessagePage | undefined => {
  const pages = stretches.data?.pages;
  const first = pages?.[0];
  const last = pages?.at(-1);
  if (!pages || !first || !last) return undefined;
  const seen = new Set<string>();
  const messages = pages
    .flatMap((stretch) => stretch.page.messages)
    .filter((message) => !seen.has(message.id) && seen.add(message.id));
  const total = last.total ?? first.total;
  return {
    messages,
    ...(total !== undefined ? { total } : {}),
    ...(first.page.before ? { before: first.page.before } : {}),
    ...(last.page.after ? { after: last.page.after } : {}),
  };
});
const total = $derived(loaded?.total);
const atStart = $derived(stretches.data?.pages[0]?.start === 0);
const atEnd = $derived(stretches.data !== undefined && !stretches.hasNextPage);

/** Read from `next`, or go there again when it is where the reader already started. */
function goTo(next: ChatPlace): void {
  place = next;
  arrival++;
}

// Puts the reader where they asked to be once that stretch has loaded.
let arrived = -1;
$effect(() => {
  const ready = loaded !== undefined;
  const wanted = arrival;
  if (!ready || arrived === wanted) return;
  arrived = wanted;
  const first = untrack(() => loaded?.messages[0]);
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
    } else if (place.at === "end") toBottom();
    else if (linked)
      document.getElementById(`message-${linked}`)?.scrollIntoView({ block: "center" });
    else window.scrollTo({ top: 0, behavior: "instant" });
    await tick();
    placed = arrival;
    fill();
  });
});

// Reading on. A stretch that loads above the screen pushes the text down, so
// the message the reader is looking at is held in place; so is one when a
// far stretch is let go.
let placed = -1;
function onScreen(): { element: HTMLElement; top: number } | undefined {
  const element = Array.from(
    transcriptPane?.querySelectorAll<HTMLElement>("[data-message]") ?? [],
  ).find((element) => element.getBoundingClientRect().bottom > 90);
  return element ? { element, top: element.getBoundingClientRect().top } : undefined;
}
async function readOn(direction: "earlier" | "later"): Promise<void> {
  const before = onScreen();
  await (direction === "earlier" ? stretches.fetchPreviousPage() : stretches.fetchNextPage());
  await tick();
  if (before?.element.isConnected)
    window.scrollBy({
      top: before.element.getBoundingClientRect().top - before.top,
      behavior: "instant",
    });
  fill();
}
/** Loads the next stretch on either side once it is within a screen or two. */
function fill(): void {
  if (placed !== arrival || sessionsOpen || !transcriptPane) return;
  if (stretches.isFetching || stretches.isFetchingPreviousPage || stretches.isFetchingNextPage)
    return;
  const reach = window.innerHeight * 1.5;
  const pane = transcriptPane.getBoundingClientRect();
  if (stretches.hasPreviousPage && pane.top > -reach) void readOn("earlier");
  else if (stretches.hasNextPage && pane.bottom < window.innerHeight + reach) void readOn("later");
}

// When Mash indexes more of this chat, the last stretch is read again. Text
// already shown never changes by itself: new messages are added at the end
// only while the reader is at the bottom of the chat, and are otherwise
// offered.
let changed = $state(false);
let held = $state<ChatStretch | undefined>();
let added = $state(0);
let checking = $state(false);
$effect(() => {
  const chatId = id;
  const from = displayMachine;
  // The same chat ID may exist on another machine; follow the copy shown.
  const local = !shared && (!machine || machine === catalogue.data?.localNode);
  return onChange((change) => {
    if (concerns(change, chatId, from, local)) void checkForUpdates();
  });
});
function atBottom(): boolean {
  const page = document.documentElement;
  return window.innerHeight + window.scrollY >= page.scrollHeight - 120;
}
function toBottom(): void {
  void tick().then(() =>
    window.scrollTo({ top: document.documentElement.scrollHeight, behavior: "instant" }),
  );
}
/** Replaces the last loaded stretch with the same stretch read again. */
function keep(next: ChatStretch): void {
  client.setQueryData<{ pages: ChatStretch[]; pageParams: ChatPlace[] }>(chatKey, (data) =>
    data
      ? {
          pages: [...data.pages.slice(0, -1), next],
          pageParams: [
            ...data.pageParams.slice(0, -1),
            { at: "offset", offset: next.start, limit: next.end - next.start },
          ],
        }
      : data,
  );
}
/**
 * Reads the end of what is loaded again, and with `everything`, the rest of
 * it too, as the Check for updates button does.
 */
async function checkForUpdates(everything = false): Promise<void> {
  const pages = stretches.data?.pages ?? [];
  const last = pages.at(-1);
  if (!last || checking) return;
  checking = true;
  const key = chatKey;
  const again = (stretch: ChatStretch, limit: number) =>
    chatStretch(
      id,
      machine,
      { at: "offset", offset: stretch.start, limit },
      new AbortController().signal,
      shared,
    );
  try {
    const earlier = everything
      ? pages.slice(0, -1).filter((stretch) => stretch.end > stretch.start)
      : [];
    const [next, ...reread] = await Promise.all([
      // Reading past the old end picks up new messages along with it.
      again(last, Math.max(last.end - last.start, PAGE_SIZE)),
      ...earlier.map((stretch) => again(stretch, stretch.end - stretch.start)),
    ]);
    if (key !== chatKey || !next) return;
    const same = (a: MessagePage, b: MessagePage): boolean =>
      JSON.stringify(a.messages) === JSON.stringify(b.messages);
    if (
      earlier.some((stretch, index) => !reread[index] || !same(stretch.page, reread[index].page))
    ) {
      changed = true;
      return;
    }
    const update = pageUpdate(
      { messages: last.page.messages, ...(last.total !== undefined ? { total: last.total } : {}) },
      { messages: next.page.messages, ...(next.total !== undefined ? { total: next.total } : {}) },
    );
    if (update.kind === "rewritten") changed = true;
    if (update.kind !== "appended") return;
    if (!sessionsOpen && atBottom()) {
      keep(next);
      if (!update.onPage) await stretches.fetchNextPage();
      toBottom();
    } else {
      // Off the bottom, only the length is kept, so text on screen stays put.
      if (!update.onPage) keep(next);
      else held = next;
      added += update.added;
    }
  } catch {
    // The stream says again when the chat next changes.
  } finally {
    checking = false;
  }
}
/** Shows the messages that arrived while the reader was elsewhere in the chat. */
function showNew(): void {
  if (held) keep(held);
  held = undefined;
  added = 0;
  toEnd();
}
function showChanged(): void {
  changed = false;
  held = undefined;
  void stretches.refetch();
}

function plainText(message: Message): string {
  return messageText(message);
}
async function exportChat(format: ExportFormat): Promise<void> {
  if (exporting) return;
  const popup = format === "pdf" ? window.open("", "_blank") : null;
  if (format === "pdf" && !popup) {
    notice = "Allow a new tab to prepare the PDF export.";
    return;
  }
  if (popup) {
    popup.opener = null;
    popup.document.body.textContent = "Preparing the entire chat for printing…";
  }
  const controller = new AbortController();
  exportController = controller;
  const title = chat?.title ?? id;
  const source = displayMachine;
  exporting = true;
  notice = "Preparing export…";
  try {
    const messages = await collectChat(
      id,
      machine,
      controller.signal,
      (count) => {
        notice = `Preparing export: ${count.toLocaleString()} messages…`;
      },
      (id, node, offset, signal, limit) => transcript(id, node, offset, signal, shared, limit),
    );
    controller.signal.throwIfAborted();
    if (format === "pdf" && popup) {
      if (popup.closed) throw new Error("The PDF preview tab was closed.");
      popup.document.open();
      popup.document.write(htmlExport(title, source, messages));
      popup.document.close();
      popup.focus();
      popup.print();
      notice = "Choose Save as PDF in the print dialog. The preview includes the entire chat.";
    } else {
      downloadChat(
        format === "markdown"
          ? markdownExport(title, source, messages)
          : htmlExport(title, source, messages),
        format === "markdown" ? "markdown" : "html",
      );
      notice = `Exported ${messages.length.toLocaleString()} messages.`;
    }
  } catch (error) {
    popup?.close();
    notice = controller.signal.aborted
      ? "Export cancelled."
      : error instanceof Error
        ? error.message
        : "Export failed.";
  } finally {
    exporting = false;
    exportController = undefined;
  }
}
function toStart(): void {
  if (atStart) window.scrollTo({ top: 0, behavior: "instant" });
  else goTo({ at: "offset", offset: 0 });
}
function toEnd(): void {
  if (atEnd) toBottom();
  else goTo({ at: "end" });
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
const items = $derived(loaded ? transcriptItems(loaded) : []);
function characters(message: Message): string {
  const count = plainText(message).length;
  return `${count.toLocaleString()} character${count === 1 ? "" : "s"}`;
}
// Context is not a prompt: only what a person said is listed.
const prompts = $derived(
  (loaded?.messages ?? []).filter(
    (message) => message.role === "user" && message.presentation === "dialogue",
  ),
);
</script>

<svelte:window
  onscroll={() => {
    remember();
    fill();
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
<!-- Scrolling reads the rest of the chat; these float beside it and jump to either end. -->
{#snippet ends()}
  <nav aria-label="Chat ends" class="ends">
    <button class="btn icon" type="button" onclick={toStart} aria-label="First message" title="First message"><ArrowUpToLine size={15} aria-hidden="true" /></button>
    <button class="btn icon" type="button" onclick={toEnd} aria-label="Latest message" title="Latest message"><ArrowDownToLine size={15} aria-hidden="true" /></button>
  </nav>
{/snippet}
<div class="chat" class:nosess={!showSessions} class:sessopen={sessionsOpen}>
  <nav class="sess" aria-label="Prompts and sessions">
    {#if phone && sessionsOpen}<button class="btn sessclose" type="button" onclick={() => closeSessions(true)}><X size={15} aria-hidden="true" />Back to chat</button>{/if}
    {#if prompts.length >= 2}
      <div class="outl">
        <h2 class="h2"><ListOrdered size={15} aria-hidden="true" />your prompts</h2>
        <div class="outl-l">
          {#each prompts as message, index (message.id)}
            <button type="button" class:on={currentMessage === message.id} aria-current={currentMessage === message.id ? "location" : undefined} onclick={async () => { await closeSessions(false); jump(message.id); }}><small>{index + 1}</small><span>{saidText(message).replace(/\s+/g, " ").slice(0, 80)}</span></button>
          {/each}
        </div>
      </div>
    {/if}
    {#if chat?.project}<h2 class="h2"><a href={projectHref(chat.project)}><Folder size={15} aria-hidden="true" />{projectLabel(chat.project)} sessions</a></h2>{/if}
    {#if projectSessions.length}
      <div class="card pad0 list sess-l">
        {#each projectSessions as session (session.machine + session.id)}
          {@const current = session.id === id && session.machine === displayMachine}
          <a class="chatrow ses" class:on={current} href={chatHref(session.id, session.machine)} aria-current={current ? "page" : undefined}><AgentMark agent={session.agent} size="sm" /><span><span class="clamp">{session.title}</span><small>{whenLabel(session.updatedMs, now)}</small></span></a>
        {/each}
      </div>
    {/if}
    <h2 class="h2"><a href={recordsHref(memoryParams)}><Bookmark size={15} aria-hidden="true" />memory from this chat</a></h2>
    {#if memories.error}<p class="meta" role="alert">{memories.error.message}</p>{/if}
    {#each memories.data?.records ?? [] as record (record.id)}
      {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
      {#if revision}<a class="card tight butter saved" href={recordHref(record.id, record.node, recordsHref(memoryParams))}><b class="clamp">{revision.statement}</b><small>{label(revision.details.kind).toLowerCase()} / {label(revision.details.state).toLowerCase()}</small></a>{/if}
    {/each}
    {#if memories.data && !memories.data.records.length}<p class="card tight cream" style:font-size="13px">Nothing saved from this chat.</p>{/if}
  </nav>
  <article class="log" bind:this={transcriptPane}>
    <header class="chat-h">
      <h1 class="h1">{chat ? plainTitle(chat.title) : `Chat ${id.slice(-8)}`}</h1>
      <div class="acts">
        <button class="btn" class:icon={!phone} type="button" aria-pressed={sessionsShown} aria-label="{sessionsShown ? 'Hide' : 'Show'} session list" title="{sessionsShown ? 'Hide' : 'Show'} session list" onclick={toggleSessions}><PanelLeft size={15} aria-hidden="true" />{#if phone}Sessions{/if}</button>
        <button class="btn" type="button" aria-expanded={findOpen} onclick={openFind}><Search size={15} aria-hidden="true" />Find in chat<kbd class="kbd">{shortcut}</kbd></button>
        <button class="btn" type="button" onclick={copySession}><Copy size={15} aria-hidden="true" />Copy session ID</button>
        <span style:position="relative">
        <button class="btn" type="button" aria-expanded={exportOpen} onclick={() => (exportOpen = !exportOpen)}><Download size={15} aria-hidden="true" />Export{#if exportOpen}<ChevronUp size={14} aria-hidden="true" />{:else}<ChevronDown size={14} aria-hidden="true" />{/if}</button>
        {#if exportOpen}
          <div class="menu">
            <button type="button" disabled={exporting || !loaded} onclick={() => { exportOpen = false; void exportChat("pdf"); }}><Printer size={15} aria-hidden="true" />Export to PDF</button>
            <button type="button" disabled={exporting || !loaded} onclick={() => { exportOpen = false; void exportChat("html"); }}><FileCode size={15} aria-hidden="true" />Export to HTML</button>
            <button type="button" disabled={exporting || !loaded} onclick={() => { exportOpen = false; void exportChat("markdown"); }}><FileText size={15} aria-hidden="true" />Export to Markdown</button>
          </div>
        {/if}
        </span>
        <button class="btn icon" type="button" aria-label="Check for updates" title="Check for updates" disabled={checking} onclick={() => checkForUpdates(true)}><RefreshCw size={15} /></button>
      </div>
      <div class="mts chat-m">
        {#if chat}<MetaChips {chat} agentMark />{:else}<span class="mtx">{displayMachine}</span>{/if}
        {#if total}<span class="mtx">{total.toLocaleString()} message{total === 1 ? "" : "s"}</span>{/if}
      </div>
    </header>
    {#if findOpen}
      <div class="findbar" role="search">
        <Search size={16} aria-hidden="true" />
        <label class="sr-only" for="find-in-chat">Find in this chat</label>
        <input
          id="find-in-chat"
          type="text"
          enterkeyhint="search"
          placeholder="Find in this chat"
          autocomplete="off"
          bind:this={findInput}
          bind:value={findQuery}
          onkeydown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              go(event.shiftKey ? -1 : 1);
            }
          }}
        />
        <span class="findn" role="status">{findStatus}</span>
        <button class="btn sm icon" type="button" aria-label="Previous match" title="Previous match" disabled={!matches.length} onclick={() => go(-1)}><ChevronUp size={15} /></button>
        <button class="btn sm icon" type="button" aria-label="Next match" title="Next match" disabled={!matches.length} onclick={() => go(1)}><ChevronDown size={15} /></button>
        <button class="btn sm icon ghost" type="button" aria-label="Close find" title="Close find" onclick={closeFind}><X size={15} /></button>
      </div>
    {/if}
    {#if exporting}<p class="card tight cream" style:display="flex" style:font-size="12.5px"><button class="a sp" type="button" onclick={() => exportController?.abort()}>Cancel export</button></p>{/if}
    <QueryError title="Couldn’t load the chat list" query={catalogue} retryLabel="Retry catalogue" />
    {#each catalogue.data?.warnings ?? [] as warning (warning)}<p role="status" class="meta">{warning}</p>{/each}
    {#if stretches.isPending}
      <div class="card" role="status">
        <p style:display="flex" style:align-items="center" style:gap="10px" style:font="600 13px var(--type-mono)"><Terminal size={16} aria-hidden="true" />Loading messages…</p>
        <div class="skel" style:width="62%"></div><div class="skel" style:width="86%"></div><div class="skel" style:width="54%"></div>
      </div>
    {/if}
    <QueryError title="Couldn’t load this conversation" query={stretches} />
    {#if changed}<button type="button" class="btn" onclick={showChanged}><RefreshCw size={15} aria-hidden="true" />This chat has changed. Show updated messages</button>{/if}
    {#if notice}<p role="status" class="meta">{notice}</p>{/if}
    <div class="newpill" aria-live="polite">{#if added && !sessionsOpen}<button type="button" class="btn p" onclick={showNew}><ChevronsDown size={15} aria-hidden="true" />{added.toLocaleString()} new message{added === 1 ? "" : "s"}</button>{/if}</div>
    {#if loaded}
      {#if !sessionsOpen}{@render ends()}{/if}
      {#if !loaded.messages.length}<p class="card tight cream" style:font-size="13.5px">No messages here. The chat may have been removed.</p>{/if}
      {#if stretches.isFetchingPreviousPage}<p class="meta" role="status">Loading earlier messages…</p>{/if}
      {#snippet saved(message: Message)}
        {#each memories.data?.records ?? [] as record (record.id)}
          {@const revision = record.revisions.find((revision) => revision.revision === record.currentRevision)}
          {#if revision?.evidence.some((source) => source.messageId === message.id)}
            <a class="a saved-as" href={recordHref(record.id, record.node, recordsHref(memoryParams))} style:align-self={message.role === "user" ? "flex-end" : "flex-start"}>Saved as {label(revision.details.kind).toLowerCase()}: {revision.statement}</a>
          {/if}
        {/each}
      {/snippet}
      <!-- Thinking and context: a closed row that opens to plain text. -->
      {#snippet note(message: Message)}
        {@const context = message.presentation === "context"}
        <details class="fold" class:hit={message.id === messageId} id="message-{message.id}" data-message={message.id} open={message.id === messageId} style:scroll-margin-top="80px">
          <summary>{#if context}<FileText size={15} aria-hidden="true" />{:else}<Lightbulb size={15} aria-hidden="true" />{/if}{stepLabel(message)}<span class="mtx">{context ? characters(message) : message.at}</span><span class="cv"><ChevronRight size={15} aria-hidden="true" /></span></summary>
          <div><pre class="pre" style:max-height="32rem" style:overflow="auto">{plainText(message)}</pre></div>
        </details>
        {@render saved(message)}
      {/snippet}
      {#snippet actions(message: Message)}
        <MessageActions {message} href={chatHref(id, machine, message.id, shared)} />
      {/snippet}
      {#each items as item (item.kind === "message" ? item.message.id : item.messages[0]?.id)}
        {#if item.kind === "steps"}
          {@const only = item.messages.length === 1 ? item.messages[0] : undefined}
          {#if only?.presentation === "context"}
            {@render note(only)}
          {:else}
            <details class="fold" open={item.messages.some((message) => message.id === messageId)}>
              <summary><Terminal size={15} aria-hidden="true" />{stepsSummary(item)}<span class="cv"><ChevronRight size={15} aria-hidden="true" /></span></summary>
              <div>
                {#each stepParts(item) as part (part.kind === "call" ? part.call.id : part.message.id)}
                  {#if part.kind === "call"}
                    <ToolStep call={part.call} selected={messageId} />
                    {#each part.messages as message (message.id)}{@render saved(message)}{/each}
                  {:else}
                    {@render note(part.message)}
                  {/if}
                {/each}
              </div>
            </details>
          {/if}
        {:else}
          {@const message = item.message}
          {@const selected = message.id === messageId}
          <div class="hitbox" class:hit={selected} id="message-{message.id}" data-message={message.id} style:scroll-margin-top="80px">
            {#if selected}<span class="hitlabel">Linked message</span>{/if}
            {#if message.role === "user"}
              <div class="you"><MessageBody text={saidText(message)} own />{@render actions(message)}</div>
              {#if message.dialogue !== undefined}<details class="fold"><summary><FileText size={15} aria-hidden="true" />Context added by the agent<span class="mtx">{characters(message)}</span><span class="cv"><ChevronRight size={15} aria-hidden="true" /></span></summary><div><pre class="pre" style:max-height="32rem" style:overflow="auto">{plainText(message)}</pre></div></details>{/if}
            {:else}
              <div class="msg">
                {#each message.blocks as block, i (i)}
                  {#if block.type === "text"}<MessageBody text={block.markdown} />{/if}
                {/each}
                {@render actions(message)}
              </div>
              {#if message.activity}<details class="fold"><summary><Terminal size={15} aria-hidden="true" />Recorded tool activity<span class="cv"><ChevronRight size={15} aria-hidden="true" /></span></summary><div><pre class="pre" style:max-height="20rem" style:overflow="auto">{message.activity}</pre></div></details>{/if}
            {/if}
            {@render saved(message)}
          </div>
        {/if}
      {/each}
      {#if stretches.isFetchingNextPage}<p class="meta" role="status">Loading later messages…</p>{/if}
    {/if}
  </article>
</div>
