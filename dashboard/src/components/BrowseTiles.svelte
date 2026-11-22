<script lang="ts">
import Laptop from "@lucide/svelte/icons/laptop";
import { createQuery } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { groupChats, whenLabel } from "../lib/browse";
import { catalogOptions } from "../lib/queries";
import { agentHref, machineHref } from "../lib/route.svelte";
import { statusOptions } from "../lib/status";
import AgentMark from "./AgentMark.svelte";
import QueryError from "./QueryError.svelte";
import Skeleton from "./Skeleton.svelte";

// The agents or machines that have chats, one card each.
let { field }: { field: "agent" | "machine" } = $props();
const catalogue = createQuery(() => catalogOptions);
const chats = $derived(catalogue.data?.chats ?? []);
const groups = $derived(
  groupChats(chats, (chat) => (field === "agent" ? chat.agent : chat.machine)),
);
const local = $derived(catalogue.data?.localNode ?? "");
// Machines other than this one reach the page through the shared server, so
// the note names the server only when this machine is configured to use one.
const status = createQuery(() => ({ ...statusOptions, enabled: field === "machine" }));
const server = $derived(status.data?.configuration.shared?.url ?? "");
const others = $derived(groups.map((group) => group.key).filter((key) => key !== local));
const now = Date.now();
const MARKS = 4;
function plural(count: number, word: string): string {
  return `${count.toLocaleString()} ${word}${count === 1 ? "" : "s"}`;
}
function host(url: string): string {
  try {
    return new URL(url).host || url;
  } catch {
    return url;
  }
}
</script>

<div>
  <h1 class="h1">{field === "agent" ? "Agents" : "Machines"}</h1>
  <p class="meta">{chats.length.toLocaleString()} chats {field === "agent" ? "from" : "on"} {plural(groups.length, field)}</p>
</div>
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if catalogue.isPending}<Skeleton rows={4} />{/if}
<div class="sums">
  {#each groups as group (group.key)}
    {@const agent = group.chats[0]?.agent}
    {@const named = group.projects.filter(([name]) => name).length}
    <a class="card sum" href={field === "agent" ? agentHref(group.key) : machineHref(group.key)}>
      <span class="sum-h">
        {#if field === "agent" && agent}<AgentMark {agent} size="lg" /><b>{agentInfo[agent].name}</b>
        {:else}<span class="ico lg"><Laptop size={20} aria-hidden="true" /></span><b>{group.key}</b>{#if group.key === local}<span class="pill ok"><i class="dot" aria-hidden="true"></i>this machine</span>{/if}{/if}
      </span>
      <span class="big"><b>{group.chats.length.toLocaleString()}</b><span>chat{group.chats.length === 1 ? "" : "s"}</span></span>
      <span class="bar"><i style:width="{Math.max(2, Math.round((group.chats.length / Math.max(1, chats.length)) * 100))}%"></i></span>
      {#if field === "agent"}
        <span class="sum-f">
          {#each group.machines as [machine] (machine)}<span class="mchip"><Laptop size={13} aria-hidden="true" />{machine}</span>{/each}
          <span class="sp">{whenLabel(group.latestMs, now)}</span>
        </span>
      {:else}
        <span class="marks">
          {#each group.agents.slice(0, MARKS) as [name] (name)}<AgentMark agent={name} size="sm" />{/each}
          {#if group.agents.length > MARKS}<small aria-label="{group.agents.length - MARKS} more agents">+{group.agents.length - MARKS}</small>{/if}
        </span>
        <hr class="dash" />
        <span class="sum-f"><span>{plural(named, "project")}</span><span class="sp">{whenLabel(group.latestMs, now)}</span></span>
      {/if}
    </a>
  {/each}
</div>
{#if field === "machine" && server && others.length}
  <p class="card tight note"><img src={asset("mashy.png")} alt="" /><span>Chats from {#each others as name, index (name)}{index ? (index === others.length - 1 ? " and " : ", ") : ""}<b>{name}</b>{/each} arrive through your shared server, <code>{host(server)}</code>.</span></p>
{/if}
