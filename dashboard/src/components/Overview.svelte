<script lang="ts">
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import Bookmark from "@lucide/svelte/icons/bookmark";
import Bot from "@lucide/svelte/icons/bot";
import Clock from "@lucide/svelte/icons/clock";
import Laptop from "@lucide/svelte/icons/laptop";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { groupChats } from "../lib/browse";
import { stableRead } from "../lib/queries";
import { decisionOptions, decisionsByChat, scopedRecordPage } from "../lib/records";
import { agentHref, machineHref } from "../lib/route.svelte";
import { readingScope, scope, scopeText } from "../lib/scope.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { statusOptions } from "../lib/status";
import { foldedNote } from "../lib/subagents";
import AgentMark from "./AgentMark.svelte";
import ChatRow from "./ChatRow.svelte";
import MachineDot from "./MachineDot.svelte";
import MemoryRow from "./MemoryRow.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import Skeleton from "./Skeleton.svelte";

// Everything here follows the viewing scope chosen in the sidebar.
const catalogue = createScoped();
const status = createQuery(() => statusOptions);
// Memory still shows when the chat list cannot be read (see readingScope).
const reading = $derived(readingScope(catalogue.data, catalogue.isError, scope.chosen));
const memories = createQuery(() => ({
  ...stableRead,
  queryKey: ["records", "overview", reading ? scopeText(reading.scope) : ""],
  enabled: reading !== undefined,
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    reading
      ? scopedRecordPage("", reading.scope, reading.reach, signal)
      : Promise.reject(new Error("The viewing scope is not known yet.")),
}));
// Both lists follow the index by themselves (see lib/live.ts).
const chats = $derived(catalogue.data?.chats ?? []);
const recent = $derived(chats.slice(0, 8));
// Chats that produced decisions say how many; the memory page reads the same list.
const decisions = createQuery(() => decisionOptions);
const decided = $derived(decisionsByChat(decisions.data?.records ?? []));
const machines = $derived(groupChats(chats, (chat) => chat.machine));
const providers = $derived(groupChats(chats, (chat) => chat.agent));
</script>

<div><h1 class="h1">Overview</h1><p class="meta">{catalogue.data ? `${chats.length.toLocaleString()} chat${chats.length === 1 ? "" : "s"} from ${machines.length.toLocaleString()} machine${machines.length === 1 ? "" : "s"}${foldedNote(catalogue.data.folded)}` : ""}</p><ScopeNote scoped={catalogue.data} /></div>
{#if status.data && !status.data.configuration.loaded}<p class="card tight warn" style:display="flex" style:gap="10px" style:align-items="flex-start"><TriangleAlert size={17} aria-hidden="true" style="flex:none;margin-top:2px" /><span>Mash is not set up on this machine yet. <a class="a" href="#/setup">Set it up here</a>: choose which agent histories to index and connect your agents.</span></p>{/if}
<QueryError title="Couldn’t load chats" query={catalogue} />
<div class="cols">
  <section class="c-main">
    <div class="hd" style:margin-bottom="14px">
      <h2 class="h2"><Clock size={15} aria-hidden="true" />recent chats</h2>
      <!-- The count is in the title line above; this is a way to the list, not a page counter. -->
      {#if chats.length}<a class="a ic" href="#/search" style:font-size="12.5px">All chats<ArrowRight size={13} aria-hidden="true" /></a>{/if}
    </div>
    {#if catalogue.isPending}<div class="card pad0"><Skeleton rows={8} /></div>{/if}
    {#if recent.length}
      <div class="card pad0 list">
        {#each recent as chat (chat.machine + chat.id)}<ChatRow {chat} decisions={decided.get(chat.id) ?? 0} />{/each}
      </div>
    {/if}
    {#if catalogue.data && !chats.length}
      <div class="card empty">
        <img src={asset("mashy.png")} alt="" />
        <p>No indexed chats yet. <a class="a" href="https://getmash.io/synchronization/#refresh-now">Update your chat history</a>.</p>
      </div>
    {/if}
    {#each catalogue.data?.warnings ?? [] as warning (warning)}<p class="meta" role="status" style:margin-top="10px">{warning}</p>{/each}
  </section>
  <div class="c-side">
    <section>
      <h2 class="h2" style:margin-bottom="14px"><a href="#/records?"><Bookmark size={15} aria-hidden="true" />recent memories</a></h2>
      {#if memories.isPending}<div class="card pad0"><Skeleton rows={3} /></div>{/if}
      {#if memories.data?.page.records.length}
        <div class="card pad0 list">
          {#each memories.data.page.records.slice(0, 4) as record (record.node + record.id)}<MemoryRow {record} />{/each}
        </div>
      {/if}
      {#if memories.data && !memories.data.page.records.length}<p class="card tight cream">Nothing saved yet. Agents save decisions and preferences as you state them, and notes when you ask.</p>{/if}
      <QueryError title="Couldn’t load memory" query={memories} />
    </section>
    <div class="g2">
      <section>
        <h2 class="h2" style:margin-bottom="14px"><a href="#/machines"><Laptop size={15} aria-hidden="true" />machines</a></h2>
        {#if machines.length}
          <div class="card tight tally">
            {#each machines as machine (machine.key)}
              <a class="li" href={machineHref(machine.key)}><MachineDot machine={machine.key} /><b class="g">{machine.key}</b><span class="num">{machine.chats.length.toLocaleString()}</span></a>
            {/each}
          </div>
        {/if}
      </section>
      <section>
        <h2 class="h2" style:margin-bottom="14px"><a href="#/agents"><Bot size={15} aria-hidden="true" />agents</a></h2>
        {#if providers.length}
          <div class="card tight tally">
            {#each providers as provider (provider.key)}
              {@const agent = provider.chats[0]?.agent}
              {#if agent}<a class="li" href={agentHref(agent)}><AgentMark {agent} size="sm" /><b class="g">{agentInfo[agent].name}</b><span class="num">{provider.chats.length.toLocaleString()}</span></a>{/if}
            {/each}
          </div>
        {/if}
      </section>
    </div>
  </div>
</div>
