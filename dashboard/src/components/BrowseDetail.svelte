<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import ChevronLeft from "@lucide/svelte/icons/chevron-left";
import Clock from "@lucide/svelte/icons/clock";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import { createQuery } from "@tanstack/svelte-query";
import type { Snippet } from "svelte";
import { agents as agentInfo } from "../lib/agents";
import { groupChats, projectLabel, whenLabel } from "../lib/browse";
import { plainTitle } from "../lib/markdown";
import { createProjectChoices } from "../lib/projectChoices.svelte";
import { catalogOptions } from "../lib/queries";
import { reading } from "../lib/reading.svelte";
import { decisionOptions, decisionsByChat } from "../lib/records";
import { agentHref, chatHref, machineHref, projectHref, searchHref } from "../lib/route.svelte";
import { scope } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { foldedNote } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatPlace from "./ChatPlace.svelte";
import ChatRow from "./ChatRow.svelte";
import DecisionCount from "./DecisionCount.svelte";
import EmptyState from "./EmptyState.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";
import SubagentPill from "./SubagentPill.svelte";

// One agent, machine or project: its chats and how they break down by the
// other two. `extra` adds page-specific sections above the recent chats.
let {
  field,
  value,
  title,
  icon,
  back,
  extra,
}: {
  field: "agent" | "machine" | "project";
  value: string;
  title: string;
  icon: Snippet;
  back?: { href: string; label: string };
  extra?: Snippet;
} = $props();
// An agent's or project's chats follow the viewing scope; a machine's do not.
const whole = createQuery(() => catalogOptions);
const scopedQuery = createScoped();
const catalogue = $derived(field === "machine" ? whole : scopedQuery);
const scoped = $derived(field === "machine" ? undefined : scopedQuery.data);
const key = (chat: ChatSummary) =>
  field === "agent" ? chat.agent : field === "machine" ? chat.machine : chat.project;
const chats = $derived((catalogue.data?.chats ?? []).filter((chat) => key(chat) === value));
const folded = $derived(
  (catalogue.data?.all ?? []).filter((chat) => key(chat) === value).length - chats.length,
);
const group = $derived(groupChats(chats, key)[0]);
const named = $derived(group ? group.projects.filter(([name]) => name) : []);
// The list of this page's projects leaves out the ones the reader hid; the
// count above it still counts them.
const choices = createProjectChoices();
const listed = $derived(named.filter(([name]) => !choices.isHidden(name)));
const latest = $derived(
  chats.reduce<ChatSummary | undefined>(
    (best, chat) => (best && (best.updatedMs ?? 0) >= (chat.updatedMs ?? 0) ? best : chat),
    undefined,
  ),
);
// A project exists only through its chats, so one with none is a link to
// nothing. An agent or machine can be real with no chats yet.
const unknown = $derived(field === "project" && catalogue.data !== undefined && !group);
// Chats that produced decisions say how many, as search results do.
const decisions = createQuery(() => decisionOptions);
const decided = $derived(decisionsByChat(decisions.data?.records ?? []));
const now = Date.now();
// A chat opened earlier in this browser session reopens where it was left, as in ChatRow.
function rowHref(chat: ChatSummary): string {
  const at = reading.positions.find((item) => item.id === chat.id && item.node === chat.machine);
  return chatHref(chat.id, chat.machine, at?.message ?? "", at?.shared ?? false);
}
</script>

{#if back}<a class="back" href={back.href} aria-label="Back to {back.label}"><ChevronLeft size={15} aria-hidden="true" />{back.label.replace(/^./, (letter) => letter.toUpperCase())}</a>{/if}
{#if unknown}
  <EmptyState card title="Project not found" heading="h1">
    No indexed chat belongs to <code style:overflow-wrap="anywhere">{value}</code>.
    {#snippet action()}<a class="btn p" href="#/projects"><Folder size={15} aria-hidden="true" />All projects</a>{/snippet}
  </EmptyState>
{:else}
<div class="dh">
  {@render icon()}
  <div><h1 class="h1">{title}</h1><p class="meta">{chats.length.toLocaleString()} chat{chats.length === 1 ? "" : "s"}{foldedNote(folded)}{#if field === "project"}{" · "}<code class="bd-p">{value}</code>{/if}</p>{#if scoped}<ScopeNote {scoped} />{/if}</div>
</div>
{/if}
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if catalogue.isPending}<Skeleton rows={5} />{/if}
{#if catalogue.data && !group && !unknown}<div class="state-c"><p>No indexed chats here.</p></div>{/if}
{#if group}
  <nav class="bd-stats" aria-label="Summary">
    <a href={searchHref({ [field]: value })}><span class="bd-e"><MessagesSquare size={12} aria-hidden="true" />Chats</span><b>{chats.length.toLocaleString()}</b></a>
    {#if field !== "agent"}<a href={group.agents.length === 1 ? agentHref(group.agents[0]?.[0] ?? "") : "#/agents"}><span class="bd-e"><Bot size={12} aria-hidden="true" />Agents</span><b>{group.agents.length}</b></a>{/if}
    {#if field !== "project"}<a href={named.length === 1 ? projectHref(named[0]?.[0] ?? "") : "#/projects"}><span class="bd-e"><Folder size={12} aria-hidden="true" />Projects</span><b>{named.length}</b></a>{/if}
    {#if field !== "machine"}<a href={group.machines.length === 1 ? machineHref(group.machines[0]?.[0] ?? "") : "#/machines"}><span class="bd-e"><Laptop size={12} aria-hidden="true" />Machines</span><b>{group.machines.length}</b></a>{/if}
    {#if latest}<a href={chatHref(latest.id, latest.machine)} title={plainTitle(latest.title)}><span class="bd-e"><Clock size={12} aria-hidden="true" />Last active</span><b>{whenLabel(group.latestMs, now)}</b></a>{/if}
  </nav>
  <div class="cols c32">
    <div class="c-main stack">
      {#if extra}{@render extra()}{/if}
      <section class="sec">
        <div class="hd"><h2 class="h2"><Clock size={15} aria-hidden="true" />Recent chats</h2><a class="a" href={searchHref({ [field]: value })} style:font-size="12.5px">All {chats.length.toLocaleString()}</a></div>
        <div class="card pad0 list bd-card">
          {#each chats.slice(0, 6) as chat (chat.machine + chat.id)}
            {#if field === "machine"}
              <!-- On a machine's page every chat is from that machine, so a row is one line: the title, then its project and when. -->
              {@const count = decided.get(chat.id) ?? 0}
              <a class="bd-r" href={rowHref(chat)}><AgentMark agent={chat.agent} size="sm" /><b>{plainTitle(chat.title)}</b><SubagentPill {chat} />{#if count}<span class="pill decision"><DecisionCount {count} /></span>{/if}<span class="bd-w">{#if chat.project}<ChatPlace project={chat.project} />{/if}<span>{whenLabel(chat.updatedMs, now)}</span></span></a>
            {:else}
              <ChatRow {chat} decisions={decided.get(chat.id) ?? 0} />
            {/if}
          {/each}
        </div>
      </section>
    </div>
    <div class="c-side">
      {#if field !== "agent"}
        <section class="sec"><h2 class="h2"><Bot size={15} aria-hidden="true" />Agents</h2>
          <div class="card tight tally bd-card">
            {#each group.agents.slice(0, 6) as [agent, count] (agent)}<a class="li" href={agentHref(agent)}><AgentMark {agent} size="sm" /><b class="g">{agentInfo[agent].name}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
      {#if field !== "machine"}
        <section class="sec"><h2 class="h2"><Laptop size={15} aria-hidden="true" />Machines</h2>
          <div class="card tight tally bd-card">
            {#each group.machines.slice(0, 6) as [machine, count] (machine)}<a class="li" href={machineHref(machine)}><Laptop size={15} aria-hidden="true" /><b class="g">{machine}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
      {#if field !== "project" && listed.length}
        <section class="sec"><h2 class="h2"><Folder size={15} aria-hidden="true" />Projects</h2>
          <div class="card tight tally bd-card">
            {#each listed.slice(0, 8) as [project, count] (project)}<a class="li" href={projectHref(project)} title={project}><Folder size={15} aria-hidden="true" /><b class="g">{projectLabel(project)}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
    </div>
  </div>
{/if}

<style>
  /* The numbers as a row of tiles: a small label with its icon over the value. */
  .bd-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(150px, 100%), 1fr));
    gap: 11px;
  }
  .bd-stats > a {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    padding: 14px 16px;
    border: var(--border-outline);
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    box-shadow: var(--shadow-panel);
    color: var(--text-body);
  }
  .bd-stats > a:hover {
    background: var(--surface-hover);
    color: var(--text-body);
  }
  .bd-e {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .bd-stats b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.02em;
    line-height: 1.15;
    color: var(--text-strong);
  }
  .bd-p {
    font: 12px var(--type-mono);
    overflow-wrap: anywhere;
  }
  .bd-card {
    border-color: var(--line-outline);
    box-shadow: var(--shadow-panel);
  }
  /* Names in the side lists and chat titles at a medium weight, as drawn. */
  .bd-card .g,
  .bd-card :global(.chatrow-t > b) {
    font-weight: 500;
  }
  /* A decision badge sits right after the title, in a soft butter tint. */
  .bd-card :global(.chatrow-t > .pill.decision),
  .bd-r .pill.decision {
    margin-left: 0;
    border-color: transparent;
    background: var(--decision-bg);
    color: var(--decision-text);
  }
  .bd-r {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 44px;
    padding: 8px 16px;
    color: var(--text-body);
  }
  .bd-r + .bd-r {
    border-top: var(--border-divider);
  }
  .bd-r:hover {
    background: var(--surface-hover);
    color: var(--text-body);
  }
  .bd-r > b {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-strong);
  }
  .bd-w {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    margin-left: auto;
    font-size: 12px;
    color: var(--text-muted);
  }
  .bd-r > :global(.pill) {
    flex: none;
  }
  /* On a phone the project and time go under the title. */
  @media (max-width: 640px) {
    .bd-r {
      flex-wrap: wrap;
      row-gap: 2px;
    }
    .bd-w {
      flex-basis: 100%;
      margin-left: 31px;
    }
  }
</style>
