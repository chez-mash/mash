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
import { agentHref, chatHref, machineHref, projectHref, searchHref } from "../lib/route.svelte";
import { scope } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { foldedNote } from "../lib/subagents";
import type { ChatSummary } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import ChatRow from "./ChatRow.svelte";
import EmptyState from "./EmptyState.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";

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
const now = Date.now();
</script>

{#if back}<a class="back" href={back.href} aria-label="Back to {back.label}"><ChevronLeft size={15} aria-hidden="true" />{back.label}</a>{/if}
{#if unknown}
  <EmptyState card title="Project not found" heading="h1">
    No indexed chat belongs to <code style:overflow-wrap="anywhere">{value}</code>.
    {#snippet action()}<a class="btn p" href="#/projects"><Folder size={15} aria-hidden="true" />All projects</a>{/snippet}
  </EmptyState>
{:else}
<div class="dh">
  {@render icon()}
  <div><h1 class="h1">{title}</h1><p class="meta">{chats.length.toLocaleString()} chat{chats.length === 1 ? "" : "s"}{foldedNote(folded)}</p>{#if scoped}<ScopeNote {scoped} />{/if}</div>
</div>
{/if}
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if catalogue.isPending}<Skeleton rows={5} />{/if}
{#if catalogue.data && !group && !unknown}<div class="state-c"><p>No indexed chats here.</p></div>{/if}
{#if group}
  <nav class="stats" aria-label="Summary">
    <a href={searchHref({ [field]: value })}><span class="eyebrow"><MessagesSquare size={12} aria-hidden="true" />chats</span><b>{chats.length.toLocaleString()}</b></a>
    {#if field !== "agent"}<a href={group.agents.length === 1 ? agentHref(group.agents[0]?.[0] ?? "") : "#/agents"}><span class="eyebrow"><Bot size={12} aria-hidden="true" />agents</span><b>{group.agents.length}</b></a>{/if}
    {#if field !== "project"}<a href={named.length === 1 ? projectHref(named[0]?.[0] ?? "") : "#/projects"}><span class="eyebrow"><Folder size={12} aria-hidden="true" />projects</span><b>{named.length}</b></a>{/if}
    {#if field !== "machine"}<a href={group.machines.length === 1 ? machineHref(group.machines[0]?.[0] ?? "") : "#/machines"}><span class="eyebrow"><Laptop size={12} aria-hidden="true" />machines</span><b>{group.machines.length}</b></a>{/if}
    {#if latest}<a href={chatHref(latest.id, latest.machine)} title={plainTitle(latest.title)}><span class="eyebrow"><Clock size={12} aria-hidden="true" />last active</span><b>{whenLabel(group.latestMs, now)}</b></a>{/if}
  </nav>
  <div class="cols c32">
    <div class="c-main stack">
      {#if extra}{@render extra()}{/if}
      <section class="sec">
        <div class="hd"><h2 class="h2"><Clock size={15} aria-hidden="true" />Recent chats</h2><a class="a" href={searchHref({ [field]: value })} style:font-size="12.5px">all {chats.length.toLocaleString()}</a></div>
        <div class="card pad0 list">
          {#each chats.slice(0, 6) as chat (chat.machine + chat.id)}<ChatRow {chat} />{/each}
        </div>
      </section>
    </div>
    <div class="c-side">
      {#if field !== "agent"}
        <section class="sec"><h2 class="h2"><Bot size={15} aria-hidden="true" />Agents</h2>
          <div class="card tight tally">
            {#each group.agents.slice(0, 6) as [agent, count] (agent)}<a class="li" href={agentHref(agent)}><AgentMark {agent} size="sm" /><b class="g">{agentInfo[agent].name}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
      {#if field !== "machine"}
        <section class="sec"><h2 class="h2"><Laptop size={15} aria-hidden="true" />Machines</h2>
          <div class="card tight tally">
            {#each group.machines.slice(0, 6) as [machine, count] (machine)}<a class="li" href={machineHref(machine)}><Laptop size={15} aria-hidden="true" /><b class="g">{machine}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
      {#if field !== "project" && listed.length}
        <section class="sec"><h2 class="h2"><Folder size={15} aria-hidden="true" />Projects</h2>
          <div class="card tight tally">
            {#each listed.slice(0, 8) as [project, count] (project)}<a class="li" href={projectHref(project)} title={project}><Folder size={15} aria-hidden="true" /><b class="g">{projectLabel(project)}</b><span class="num">{count.toLocaleString()}</span></a>{/each}
          </div>
        </section>
      {/if}
    </div>
  </div>
{/if}
