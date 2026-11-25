<script lang="ts">
import Plus from "@lucide/svelte/icons/plus";
import RefreshCw from "@lucide/svelte/icons/refresh-cw";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { isAgent } from "../lib/api";
import { asset } from "../lib/asset";
import { groupChats } from "../lib/browse";
import { catalogOptions } from "../lib/queries";
import { createScoped } from "../lib/scoped.svelte";
import { serverViewOptions } from "../lib/serverStatus";
import { serviceAccessOptions, serviceOptions } from "../lib/services.svelte";
import { draftOf, problem, saveSettings, settingsOptions } from "../lib/settings";
import { sharingLeads } from "../lib/shared";
import { statusOptions } from "../lib/status";
import type { Agent } from "../lib/types";
import AgentCard from "./AgentCard.svelte";
import AgentSetupList from "./AgentSetupList.svelte";
import MachineCard from "./MachineCard.svelte";
import MachinesServer from "./MachinesServer.svelte";
import QueryError from "./QueryError.svelte";
import ScopeNote from "./ScopeNote.svelte";
import SharedServer from "./SharedServer.svelte";
import Skeleton from "./Skeleton.svelte";

// The agents or machines that have chats, one card each.
let { field }: { field: "agent" | "machine" } = $props();
// Agents follow the viewing scope; machines are about the machines themselves,
// so every machine is listed whatever the scope.
const whole = createQuery(() => catalogOptions);
const scopedQuery = createScoped();
const catalogue = $derived(field !== "agent" ? whole : scopedQuery);
const scoped = $derived(field !== "agent" ? undefined : scopedQuery.data);
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
// This machine's settings say which agents it indexes and where it finds
// their history; an agent set up here with no chats yet still gets a card.
const settings = createQuery(() => settingsOptions);
const setting = (agent: Agent) => settings.data?.agents.find((row) => row.agent === agent);
const agentCards = $derived.by(() => {
  const seen = groups.flatMap((group) =>
    isAgent(group.key)
      ? [
          {
            agent: group.key,
            chats: group.chats.length,
            latestMs: group.latestMs as number | undefined,
            machines: group.machines.map(([machine]) => machine),
          },
        ]
      : [],
  );
  const idle = (settings.data?.agents ?? [])
    .filter((row) => !seen.some((card) => card.agent === row.agent))
    .map((row) => ({ agent: row.agent, chats: 0, latestMs: undefined, machines: [] }));
  return [...seen, ...idle];
});
// What this machine shares leads the Machines page, as in the wireframes; its
// card then names the server, so the summary strip is left out.
const serverView = createQuery(() => ({ ...serverViewOptions, enabled: field === "machine" }));
const sharing = $derived(field === "machine" && sharingLeads(serverView.data, !!settings.data));
const machineCount = $derived(new Set(chats.map((chat) => chat.machine)).size);
// The setup list is this machine's own, read from its service check.
const access = createQuery(() => ({ ...serviceAccessOptions, enabled: field === "agent" }));
const fleet = createQuery(() => serviceOptions(field === "agent" && access.data !== undefined));
const own = $derived(access.data?.localNode ?? "");
const reachable = $derived(
  fleet.data?.some((machine) => machine.id === own && machine.reachable) ?? false,
);
const now = Date.now();
// Each agent card's switch writes this machine's settings, as the Settings
// page does: the rest of the file is kept as it is.
const client = useQueryClient();
let indexing = $state(false);
let indexFailure = $state("");
async function setIndexed(agent: Agent, on: boolean): Promise<void> {
  const current = settings.data;
  if (!current || indexing) return;
  const draft = draftOf(current);
  draft.agents = { ...draft.agents, [agent]: on };
  indexFailure = problem(current, draft);
  if (indexFailure) return;
  indexing = true;
  try {
    client.setQueryData(settingsOptions.queryKey, await saveSettings(current, draft));
  } catch (error) {
    indexFailure = error instanceof Error ? error.message : "The setting was not saved.";
  } finally {
    indexing = false;
  }
}
// Reads again what the page shows: the chats, this machine's settings and
// its agents' setup.
let checking = $state(false);
async function checkAgain(): Promise<void> {
  checking = true;
  indexFailure = "";
  try {
    await Promise.all([
      client.invalidateQueries({ queryKey: catalogOptions.queryKey }),
      client.invalidateQueries({ queryKey: settingsOptions.queryKey }),
      client.invalidateQueries({ queryKey: ["services"] }),
      client.invalidateQueries({ queryKey: ["service-setup"] }),
    ]);
  } finally {
    checking = false;
  }
}
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

{#if field === "agent"}
  <header class="bt-h">
    <div>
      <h1 class="h1">Agents</h1>
      <p class="meta">{plural(chats.length, "chat")} from {plural(groups.length, "agent")}{machineCount > 1 ? ` on ${plural(machineCount, "machine")}` : ""}. Mash reads the history each agent already keeps.</p>
    </div>
    <span class="bt-a">
      <button class="btn" type="button" disabled={checking} onclick={checkAgain}><RefreshCw size={15} aria-hidden="true" />{checking ? "Checking…" : "Check again"}</button>
      {#if own && own === local && settings.data?.editable}<a class="btn p" href="#/setup"><Plus size={15} aria-hidden="true" />Connect an agent</a>{/if}
    </span>
  </header>
  {#if indexFailure}<p class="meta warn" role="alert">{indexFailure}</p>{/if}
  {#if scoped}<ScopeNote {scoped} />{/if}
{:else}
  <header class="bt-h">
    <div><h1 class="h1">Machines</h1><p class="meta">{plural(chats.length, "chat")} on {plural(groups.length, field)}</p></div>
  </header>
{/if}
<QueryError title="Couldn’t load chats" query={catalogue} />
{#if catalogue.isPending}<Skeleton rows={4} />{/if}
{#if field === "agent"}
  <div class="bt-g">
    {#each agentCards as card (card.agent)}
      <AgentCard {...card} total={chats.length} setting={setting(card.agent)} {now} busy={indexing} onindex={settings.data?.editable ? (on) => setIndexed(card.agent, on) : undefined} />
    {/each}
  </div>
  {#if own && reachable}
    <AgentSetupList machineId={own} enabled title="Agent setup on {own}" note="Connection lets an agent search Mash; the plugin and skill teach it when to." />
  {/if}
{:else}
  {#snippet machineGrid()}
    {#if groups.length}
      <div class="sec">
        <h2 class="bt-l">Machines</h2>
        <div class="bt-g">
          {#each groups as group (group.key)}<MachineCard {group} {local} />{/each}
        </div>
      </div>
    {/if}
  {/snippet}
  {#if sharing}
    <SharedServer joinable placement="lead" below={machineGrid} />
  {:else}
    <MachinesServer />
    {@render machineGrid()}
  {/if}
  {#if server && others.length}
    <p class="card tight note"><img src={asset("mashy.png")} alt="" /><span>Chats from {#each others as name, index (name)}{index ? (index === others.length - 1 ? " and " : ", ") : ""}<b>{name}</b>{/each} arrive through your shared server, <code>{host(server)}</code>.</span></p>
  {/if}
{/if}

<style>
  .bt-h {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px 20px;
  }
  .bt-a {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .bt-h .meta {
    margin-top: 6px;
  }
  .bt-g {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(240px, 100%), 1fr));
    gap: 13px;
  }
  .bt-l {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted);
  }
</style>
