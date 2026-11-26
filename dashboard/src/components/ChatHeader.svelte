<script lang="ts">
import ChevronDown from "@lucide/svelte/icons/chevron-down";
import ChevronUp from "@lucide/svelte/icons/chevron-up";
import Copy from "@lucide/svelte/icons/copy";
import Download from "@lucide/svelte/icons/download";
import FileCode from "@lucide/svelte/icons/file-code";
import FileText from "@lucide/svelte/icons/file-text";
import Link2 from "@lucide/svelte/icons/link-2";
import PanelLeft from "@lucide/svelte/icons/panel-left";
import Printer from "@lucide/svelte/icons/printer";
import Search from "@lucide/svelte/icons/search";
import { agents } from "../lib/agents";
import { whenLabel } from "../lib/browse";
import type { ExportFormat } from "../lib/export";
import { plainTitle } from "../lib/markdown";
import { agentHref, machineHref, projectHref } from "../lib/route.svelte";
import { shareLink } from "../lib/shareLink";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatPhoneBar from "./ChatPhoneBar.svelte";
import OpenInAgent from "./OpenInAgent.svelte";
import SubagentOrigin from "./SubagentOrigin.svelte";
import SubagentState from "./SubagentState.svelte";

// A chat's header: where it came from when it is a subagent, its title, a
// quiet line naming its agent, machine, project folder, length and age, and
// the actions on the whole chat. On a phone a bar of its own leads it, with
// the way back and a menu, and two wide buttons follow the title.
let {
  id,
  href,
  chat,
  all,
  machine,
  total,
  local,
  phone,
  sessionsShown,
  findOpen,
  exportOpen = $bindable(),
  canExport,
  ontogglesessions,
  onfind,
  oncopy,
  onexport,
  onnotice,
}: {
  id: string;
  /** The chat's own address in the dashboard, which Copy link shares. */
  href: string;
  chat: ChatSummary | undefined;
  all: readonly ChatSummary[];
  /** The machine the chat is read from, named when the chat is not listed. */
  machine: string;
  total: number | undefined;
  /** Whether the chat ran on this machine, so its agent can open it here. */
  local: boolean;
  phone: boolean;
  sessionsShown: boolean;
  findOpen: boolean;
  exportOpen: boolean;
  canExport: boolean;
  ontogglesessions: () => void;
  onfind: () => void;
  oncopy: () => void;
  onexport: (format: ExportFormat) => void;
  onnotice: (text: string) => void;
} = $props();
const shortcut =
  typeof navigator !== "undefined" && /mac|iphone|ipad/i.test(navigator.platform) ? "⌘F" : "Ctrl F";
const now = Date.now();
// A project folder in a home directory reads as ~/…, as a shell prints it.
const folder = $derived(
  chat?.project.replace(
    /^(?:\/home|\/Users)\/[^/]+(?=\/|$)|^[A-Za-z]:\\Users\\[^\\]+(?=\\|$)/,
    "~",
  ),
);
async function copyLink(): Promise<void> {
  try {
    await navigator.clipboard.writeText(shareLink(href, `${location.origin}${location.pathname}`));
    onnotice("Copied the link to this chat.");
  } catch {
    onnotice("Couldn’t copy the link to this chat.");
  }
}
const age = $derived.by(() => {
  const label = whenLabel(chat?.updatedMs, now);
  return /^\d+[mhd]$/.test(label) ? `${label} ago` : label;
});
</script>

<header class="chat-h">
  {#if phone}<ChatPhoneBar {sessionsShown} {findOpen} {canExport} {ontogglesessions} {onfind} {oncopy} {onexport} />{/if}
  {#if chat}<SubagentOrigin {chat} {all} />{/if}
  <div class="ch-top">
    <div class="ch-id">
      <h1 class="h1">{chat ? plainTitle(chat.title) : `Chat ${id.slice(-8)}`}</h1>
      <div class="mts chat-m">
        {#if chat}
          <a class="ch-a" href={agentHref(chat.agent)}><AgentMark agent={chat.agent} size="sm" />{agents[chat.agent].name}</a>
          <a class="ch-a" href={machineHref(chat.machine)}>{chat.machine}</a>
          {#if chat.project}<a class="ch-a ch-p" href={projectHref(chat.project)} title={chat.project}>{folder}</a>{/if}
          <SubagentState {chat} />
        {:else}<span class="mtx">{machine}</span>{/if}
        {#if total}<span class="mtx">{total.toLocaleString()} message{total === 1 ? "" : "s"}</span>{/if}
        {#if chat?.updatedMs !== undefined}<span class="mtx" class:ch-age={!!total}>{age}</span>{/if}
      </div>
    </div>
    {#if phone}
      <div class="ch-two">
        <button class="btn" type="button" onclick={copyLink}><Link2 size={15} aria-hidden="true" />Copy link</button>
        {#if chat}<OpenInAgent {chat} {local} {onnotice} />{/if}
      </div>
    {:else}
    <div class="acts ch-acts">
      <button class="btn" class:icon={!phone} type="button" aria-pressed={sessionsShown} aria-label="{sessionsShown ? 'Hide' : 'Show'} session list" title="{sessionsShown ? 'Hide' : 'Show'} session list" onclick={ontogglesessions}><PanelLeft size={15} aria-hidden="true" />{#if phone}Sessions{/if}</button>
      <button class="btn" type="button" aria-expanded={findOpen} onclick={onfind}><Search size={15} aria-hidden="true" />Find in chat<kbd class="kbd">{shortcut}</kbd></button>
      <button class="btn" type="button" onclick={oncopy}><Copy size={15} aria-hidden="true" />Copy session ID</button>
      {#if chat}<OpenInAgent {chat} {local} {onnotice} />{/if}
      <span class="ch-x">
        <button class="btn" type="button" aria-expanded={exportOpen} onclick={() => (exportOpen = !exportOpen)}><Download size={15} aria-hidden="true" />Export{#if exportOpen}<ChevronUp size={14} aria-hidden="true" />{:else}<ChevronDown size={14} aria-hidden="true" />{/if}</button>
        {#if exportOpen}
          <div class="menu">
            <button type="button" disabled={!canExport} onclick={() => onexport("pdf")}><Printer size={15} aria-hidden="true" />Export to PDF</button>
            <button type="button" disabled={!canExport} onclick={() => onexport("html")}><FileCode size={15} aria-hidden="true" />Export to HTML</button>
            <button type="button" disabled={!canExport} onclick={() => onexport("markdown")}><FileText size={15} aria-hidden="true" />Export to Markdown</button>
          </div>
        {/if}
      </span>
    </div>
    {/if}
  </div>
</header>

<style>
  .chat-h {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px 32px 18px;
    border-bottom: var(--border-divider);
  }
  .ch-top {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px 16px;
  }
  .ch-id {
    flex: 1 1 320px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ch-id .h1 {
    font-size: 22px;
    font-weight: var(--weight-strong);
    letter-spacing: -0.02em;
    line-height: 1.25;
  }
  .chat-m {
    min-height: 0;
    gap: 4px 12px;
    font-size: 12.5px;
  }
  .ch-a {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-muted);
  }
  .ch-a:hover {
    color: var(--text-strong);
  }
  .ch-a :global(.mark) {
    width: 18px;
    height: 18px;
  }
  .ch-p {
    font-family: var(--type-mono);
    font-size: 12px;
  }
  .ch-age::before {
    content: "· ";
    margin-left: -8px;
  }
  .ch-acts {
    flex: 0 1 auto;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
  }
  .ch-x {
    position: relative;
  }
  /* Two buttons side by side across the width; one alone takes all of it. */
  .ch-two {
    flex: 1 1 100%;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
    gap: 8px;
  }
  .ch-two > :global(.btn) {
    justify-content: center;
    min-height: 42px;
    border-radius: var(--radius-pill);
  }
  .ch-two > :global(.btn:last-child:not(:first-child)) {
    background: var(--action-bg);
    color: var(--action-text);
    border-color: transparent;
  }
  @media (max-width: 700px) {
    .chat-h {
      padding: 14px 16px;
    }
    .ch-acts {
      justify-content: flex-start;
    }
  }
</style>
