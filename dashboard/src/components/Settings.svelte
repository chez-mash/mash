<script lang="ts">
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import CopyIcon from "phosphor-svelte/lib/CopyIcon";
import PencilSimpleIcon from "phosphor-svelte/lib/PencilSimpleIcon";
import WarningIcon from "phosphor-svelte/lib/WarningIcon";
import Icon from "$lib/components/Icon.svelte";
import { Switch } from "$lib/components/ui/switch";
import * as ToggleGroup from "$lib/components/ui/toggle-group";
import { agents as agentInfo } from "../lib/agents";
import { appearance, appearances } from "../lib/appearance.svelte";
import { assistantModelsOptions, assistantOptions } from "../lib/assistant";
import { mashCommand } from "../lib/command";
import { router } from "../lib/route.svelte";
import {
  type Draft,
  destinations,
  differences,
  draftOf,
  problem,
  saveSettings,
  settingsOptions,
} from "../lib/settings";
import { inDesktopApp } from "../lib/shareLink";
import { statusOptions } from "../lib/status";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import AssistantKeyForm from "./AssistantKeyForm.svelte";
import QueryError from "./QueryError.svelte";
import SettingsNav from "./SettingsNav.svelte";
import SettingsSaveBar from "./SettingsSaveBar.svelte";
import SharedServer from "./SharedServer.svelte";
import Skeleton from "./Skeleton.svelte";
import ToneIcon from "./ToneIcon.svelte";
import Wrapped from "./Wrapped.svelte";

// Settings a person has a reason to change are controls; the rest of the
// configuration is shown as it is. Nothing is written until the reader
// saves, and history locations are never typed here: the machine
// finds them itself.
const queryClient = useQueryClient();
// Inside the desktop app the setup page, not a terminal, is the way to set up.
const desktop = inDesktopApp();
const status = createQuery(() => statusOptions);
const assistant = createQuery(() => assistantOptions);
const settings = createQuery(() => settingsOptions);
const config = $derived(status.data?.configuration);
const file = $derived(settings.data);
// The file to edit for anything the page does not change, with a way to copy it.
const path = $derived(file?.path ?? config?.path ?? "");
let copied = $state(false);
async function copyConfigPath(): Promise<void> {
  try {
    await navigator.clipboard.writeText(path);
    copied = true;
  } catch {
    copied = false;
  }
}
const modelLabels = {
  "not-required": "not needed for word search",
  available: "available",
  unavailable: "unavailable",
};
// The service is asked for its models only where one could be chosen.
const models = createQuery(() => ({
  ...assistantModelsOptions,
  enabled: !!settings.data?.assistant && settings.data.editable,
}));
const names = Object.fromEntries(
  Object.entries(agentInfo).map(([agent, info]) => [agent, info.name]),
) as Record<Agent, string>;

let draft = $state<Draft | null>(null);
let based = $state("");
// A fresh copy of the file's settings whenever the file itself changes.
$effect(() => {
  if (file && file.revision !== based) {
    draft = draftOf(file);
    based = file.revision;
  }
});
const changed = $derived(file && draft ? differences(file, draft, names) : []);
const touched = $derived(new Set(changed.map((change) => change.key)));
const blocked = $derived(file && draft ? problem(file, draft) : "");
let saving = $state(false);
let failure = $state("");
let saved = $state(0);
let days = $state(90);

function discard(): void {
  if (file) draft = draftOf(file);
  failure = "";
}
// Mashy is set up here by choosing a service and pasting its API key. The
// key is saved on the machine running the dashboard and never shown again.
let replacingKey = $state(false);
// The sections in page order, for the list beside them. A route naming one
// (Mashy links to "assistant" when it is not set up) brings it into view
// once everything above it has loaded, so it does not move after.
const sections = [
  { id: "agents", label: "Agent histories" },
  { id: "indexing", label: "Indexing" },
  { id: "search", label: "Search" },
  { id: "privacy", label: "Privacy" },
  { id: "assistant", label: "Models and Mashy" },
  { id: "sync", label: "Machines and sync" },
  { id: "storage", label: "Storage" },
  { id: "appearance", label: "Appearance" },
] as const;
$effect(() => {
  const route = router.current;
  if (route.page !== "settings" || !route.section) return;
  if (status.isPending || settings.isPending) return;
  document.getElementById(`set-${route.section}`)?.scrollIntoView({ block: "start" });
});
async function save(): Promise<void> {
  if (!file || !draft || saving || blocked || !changed.length) return;
  const count = changed.length;
  saving = true;
  failure = "";
  try {
    const next = await saveSettings(file, draft);
    queryClient.setQueryData(settingsOptions.queryKey, next);
    await queryClient.invalidateQueries({ queryKey: statusOptions.queryKey });
    await queryClient.invalidateQueries({ queryKey: assistantOptions.queryKey, exact: true });
    saved = count;
  } catch (error) {
    failure = error instanceof Error ? error.message : "The settings were not saved.";
  } finally {
    saving = false;
  }
}
</script>

{#snippet title(name: string, description?: string)}
  <span class="set-t"><span>{name}</span>{#if description}<small>{description}</small>{/if}</span>
{/snippet}
{#snippet value(name: string, shown: string, description?: string)}
  <div class="set-row"><dt class="set-t"><span>{name}</span>{#if description}<small>{description}</small>{/if}</dt><dd class="set-v"><Wrapped value={shown} /></dd></div>
{/snippet}
{#snippet copyPath()}
  {#if path}<button class="btn sm" type="button" onclick={copyConfigPath}>{#if copied}<Icon icon={CheckIcon} size={13} />Copied{:else}<Icon icon={CopyIcon} size={13} />Copy path{/if}</button>{/if}
{/snippet}
{#snippet toggle(label: string, on: boolean, flip: () => void, description?: string)}
  <Switch aria-label={label} aria-describedby={description} bind:checked={() => on, () => { saved = 0; flip(); }} />
{/snippet}
{#snippet changedPill(key: string)}
  {#if touched.has(key)}<span class="pill warn"><Icon icon={PencilSimpleIcon} size={11} />changed</span>{/if}
{/snippet}
{#snippet heading(name: string, note?: string)}
  <div class="hd set-hd"><h2 class="h2">{name}</h2>{#if note}<span class="meta">{note}</span>{/if}</div>
{/snippet}

<div class="hd set-top">
  <div>
    <h1 class="h1">Settings</h1>
    <p class="meta set-path"><span class="mono"><Wrapped value={path || "Mash configuration"} /></span> {@render copyPath()} <a class="a" href="https://getmash.io/dashboard-settings/" target="_blank" rel="noreferrer">Settings guide</a></p>
  </div>
  {#if status.data}
    <span class="pill" class:ok={status.data.ready} class:warn={!status.data.ready}>
      {#if status.data.ready}<Icon icon={CheckIcon} size={13} />Local search ready{:else}<Icon icon={WarningIcon} size={13} state="strong" />Local search needs attention{/if}
    </span>
  {/if}
</div>

<div class="set-layout">
<SettingsNav {sections} />
<div class="set-main">
{#if file && !file.editable}
  <p class="card tight">Shown from this machine's configuration file. This dashboard can be reached from other machines and was not started with a services configuration, so it does not change settings. Open the dashboard on the machine itself, or edit the file named above. <a class="a" href="https://getmash.io/dashboard-settings/#if-settings-are-read-only" target="_blank" rel="noreferrer">Why settings are read-only</a></p>
{:else if file}
  <p class="meta">These are this machine's settings. Choose <strong>Save to file</strong> to write your changes.</p>
{/if}

{#if status.isPending || settings.isPending}<Skeleton rows={5} />{/if}
<QueryError title="Couldn’t load settings" query={status} />
{#if settings.error}<p class="meta warn st" role="alert"><ToneIcon tone="warn" size={13} /><span>Settings can't be changed right now: {settings.error.message}</span></p>{/if}
{#if status.data && config}
  {#if !config.loaded}
    <p class="card tight warn"><ToneIcon tone="warn" size={15} /><span>Mash has no configuration file yet. <a class="a" href="#/setup">Set up Mash here</a>{#if status.data.searchService === "reachable" && !desktop} or run <code class="i">mash init</code> in a terminal{/if}.</span></p>
  {/if}
  {#if status.data.searchService === "unreachable"}
    <p class="card tight warn"><ToneIcon tone="warn" size={15} /><span>Mash could not reach the local search service at <code class="i">{config.searchStore}</code>. <a class="a" href="https://getmash.io/troubleshooting/#qdrant-connection-failed" target="_blank" rel="noreferrer">Start or repair the search service</a>.</span></p>
  {:else if status.data.searchService === "rejected"}
    <p class="card tight warn"><ToneIcon tone="warn" size={15} /><span>The search service at <code class="i">{config.searchStore}</code> responded, but Mash could not use it. <a class="a" href="https://getmash.io/troubleshooting/#qdrant-connection-failed" target="_blank" rel="noreferrer">Check the search service</a>.</span></p>
  {:else if status.data.nextStep === "index"}
    <p class="card tight warn"><ToneIcon tone="warn" size={15} /><span>Run <code class="i">{mashCommand("index")}</code> in a terminal on this machine to update local search.</span></p>
  {:else if status.data.nextStep === "models install"}
    <p class="card tight warn"><ToneIcon tone="warn" size={15} /><span>Search by meaning needs local models. Run <code class="i">{mashCommand("models install")}</code> on this machine. <a class="a" href="https://getmash.io/indexing/#enable-semantic-search" target="_blank" rel="noreferrer">Finish setting up search by meaning</a>.</span></p>
  {/if}

  <section class="sec" id="set-agents">
    <div class="hd set-hd">
      <h2 class="h2">Agent histories</h2>
      {#if config.loaded && file?.editable}<a class="a" href="#/setup" style:font-size="12.5px">Connect an agent</a>{/if}
    </div>
    {#if file && draft}
      {#if file.agents.length}
        <div class="card pad0 list">
          {#each file.agents as entry (entry.agent)}
            {@const on = draft.agents[entry.agent] ?? entry.indexed}
            <div class="set-a" class:off={!on} class:changed={touched.has(`agent:${entry.agent}`)}>
              <AgentMark agent={entry.agent} />
              <span class="set-n">{agentInfo[entry.agent].name}</span>
              <span class="set-p"><Wrapped value={entry.roots.join(", ")} /></span>
              <span class="set-s">{@render changedPill(`agent:${entry.agent}`)}<span class="meta set-st" class:set-on={on}><ToneIcon tone={on ? "ok" : "idle"} size={13} />{on ? "indexed" : "not indexed"}</span>{#if file.editable}{@render toggle(`Index ${agentInfo[entry.agent].name}`, on, () => { if (draft) draft.agents[entry.agent] = !on; }, "history-sources-help")}{/if}</span>
            </div>
          {/each}
        </div>
      {/if}
      {#if file.editable && file.agents.length}
        <p id="history-sources-help" class="meta">These switches control which histories Mash reads from now on. Connected agents can still look up chats already indexed. <a class="a" href="https://getmash.io/agents/#disconnect-an-agent" target="_blank" rel="noreferrer">Remove an agent connection</a></p>
      {:else if !file.agents.length}
        <p class="meta">Mash found no agent histories on this machine.</p>
      {/if}
    {:else}
      <div class="card pad0 list">
        {#each config.providers as provider (provider.agent)}
          <div class="set-a"><AgentMark agent={provider.agent} /><span class="set-n">{agentInfo[provider.agent].name}</span><span class="set-p">{provider.roots.join(", ")}</span></div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="sec" id="set-indexing">
    {@render heading("Indexing", "Applies to this machine")}
    {#if file && draft && file.editable}
      <div class="card set-list">
        <div class="set-row stack" class:changed={touched.has("range")}>
          {@render title("History range", "How far back Mash reads.")}
          <span class="set-c">
            <ToggleGroup.Root type="single" aria-label="History range" aria-describedby="history-range-help" bind:value={() => (draft?.sinceDays === null ? "all" : "recent"), (range) => { if (!draft || !range) return; saved = 0; if (range === "all") draft.sinceDays = null; else if (draft.sinceDays === null) draft.sinceDays = file.sinceDays ?? days; }}>
              <ToggleGroup.Item value="all">all history</ToggleGroup.Item>
              <ToggleGroup.Item value="recent">recent updates</ToggleGroup.Item>
            </ToggleGroup.Root>
            {#if draft.sinceDays !== null}
              <span class="meta">the last <input class="field" type="number" min="1" max="36500" aria-label="Days of history" aria-describedby="history-range-help" style:width="72px" style:margin="0 4px" value={draft.sinceDays} oninput={(event) => { saved = 0; days = event.currentTarget.valueAsNumber; if (draft) draft.sinceDays = Number.isNaN(days) ? 0 : days; }} /> days</span>
            {/if}
            {@render changedPill("range")}
            <span id="history-range-help" class="meta set-help">
              The range applies to future indexing. Chats already indexed stay searchable.
              <a class="a" href="https://getmash.io/privacy/#remove-a-chat" target="_blank" rel="noreferrer">How to remove an indexed chat</a>
            </span>
          </span>
        </div>
        <div class="set-row stack" class:changed={touched.has("destination")}>
          {@render title("Index into", "Where this machine writes its index.")}
          <span class="set-c">
            {#if file.shared}
              <ToggleGroup.Root type="single" aria-label="Index into" bind:value={() => draft?.destination ?? "both", (destination) => { if (!draft || (destination !== "both" && destination !== "local")) return; saved = 0; draft.destination = destination; }}>
                <ToggleGroup.Item value="both">this machine and the shared server</ToggleGroup.Item>
                <ToggleGroup.Item value="local">this machine only</ToggleGroup.Item>
              </ToggleGroup.Root>
              {@render changedPill("destination")}
              {#if draft.destination === "local" && file.destination === "both"}<span class="set-h">History already on the shared server stays there.</span>{/if}
            {:else}
              <span class="meta">this machine only · no shared server is configured</span>
            {/if}
          </span>
        </div>
        <div class="set-row" class:changed={touched.has("memory")}>
          {@render title("Memory capture", "Agents save decisions and preferences as you state them.")}
          <span class="set-c">{@render changedPill("memory")}{@render toggle("Memory capture", draft.memoryCapture, () => { if (draft) draft.memoryCapture = !draft.memoryCapture; })}</span>
        </div>
      </div>
    {:else}
      <dl class="card set-list">
        {@render value("History range", config.sinceDays === null ? "all history" : `the last ${config.sinceDays} days`, "How far back Mash reads.")}
        {@render value("Index into", config.destination === "both" ? destinations.both : destinations.local, "Where this machine writes its index.")}
        {@render value("Memory capture", config.recordsEnabled ? "on" : "off", "Agents save decisions and preferences as you state them.")}
      </dl>
    {/if}
  </section>

  <section class="sec" id="set-search">
    {@render heading("Search", "Set in the configuration file")}
    <dl class="card set-list">
      {@render value("Search mode", status.data.mode, "What a search runs: words, meaning, or both.")}
      {@render value("Embeddings", config.embeddings, "Where search by meaning computes its vectors.")}
      {@render value("Vector search", config.vectorSearch.toUpperCase())}
      {@render value("Ranking", config.ranking, "How word and meaning results are combined.")}
    </dl>
  </section>

  <section class="sec" id="set-privacy">
    {@render heading("Privacy")}
    {#if file && draft && file.editable}
      <div class="card set-list">
        <div class="set-row" class:changed={touched.has("results")}>
          {@render title("Hide credentials in results", "Mask API keys and tokens that agents printed in chats.")}
          <span class="set-c">{@render changedPill("results")}{@render toggle("Hide credentials in results", draft.hideInResults, () => { if (draft) draft.hideInResults = !draft.hideInResults; })}</span>
        </div>
        <div class="set-row" class:changed={touched.has("publishing")}>
          {@render title("Hide credentials when publishing", "Mask them in the copy this machine sends to a shared server.")}
          <span class="set-c">
            {@render changedPill("publishing")}
            {@render toggle("Hide credentials when publishing", draft.hideWhenPublishing, () => { if (draft) draft.hideWhenPublishing = !draft.hideWhenPublishing; })}
          </span>
          {#if draft.hideWhenPublishing !== file.hideWhenPublishing}
            <span class="set-h">
              The next publication replaces this machine's shared copy of its history.
              {#if !draft.hideWhenPublishing}That copy can include credentials found in chats.{/if}
            </span>
          {/if}
        </div>
        <p class="meta set-foot">Credentials are never shown here. The configuration names the environment variables that hold them.</p>
      </div>
    {:else}
      <dl class="card set-list">
        {@render value("Hide credentials in results", config.redactOutput ? "on" : "off")}
        {@render value("Hide credentials when publishing", config.redactPublication ? "on" : "off")}
        <p class="meta set-foot">Credentials are never shown here. The configuration names the environment variables that hold them.</p>
      </dl>
    {/if}
  </section>

  <section class="sec" id="set-assistant">
    <div class="hd set-hd">
      <h2 class="h2">Models and Mashy</h2>
      <a class="a" href="https://getmash.io/configuration/#the-dashboard-assistant" target="_blank" rel="noreferrer" style:font-size="12.5px">Set up Mashy</a>
    </div>
    <div class="card set-list">
      <dl class="set-dl">{@render value("Search by meaning", modelLabels[status.data.queryModels], "Local models that rank results by what you meant.")}</dl>
      {#if file?.assistant && draft?.assistant && file.editable}
        <div class="set-row" class:changed={touched.has("mashy")}>
          {@render title("Mashy", "Answers questions from your history, and sends the model the chat text it reads.")}
          <span class="set-c">{@render changedPill("mashy")}{@render toggle("Mashy", draft.assistant.enabled, () => { if (draft?.assistant) draft.assistant.enabled = !draft.assistant.enabled; })}</span>
        </div>
        <div class="set-row" class:changed={touched.has("model")}>
          {@render title("Mashy's model", models.data?.length ? `${models.data.length.toLocaleString()} offered by the configured service.` : models.error ? "The service did not list its models; type a name." : "As the configured service names it.")}
          <span class="set-c">
            {@render changedPill("model")}
            <input class="field" list="assistant-models" aria-label="Mashy's model" autocomplete="off" autocapitalize="off" spellcheck="false" style:width="min(100%, 280px)" value={draft.assistant.model} oninput={(event) => { saved = 0; if (draft?.assistant) draft.assistant.model = event.currentTarget.value; }} />
            <datalist id="assistant-models">{#each models.data ?? [] as model (model)}<option value={model}></option>{/each}</datalist>
          </span>
        </div>
        <div class="set-foot">
          {#if file.assistant.enabled && assistant.data && !assistant.data.enabled}
            <p class="meta warn">Mashy has no API key on this computer yet.</p>
            <AssistantKeyForm settings={file} withService={false} id="settings-mashy" />
          {:else if replacingKey}
            <AssistantKeyForm settings={file} withService={false} id="settings-mashy" oncancel={() => (replacingKey = false)} onsaved={() => (replacingKey = false)} />
          {:else}
            <p class="meta">Mashy uses your model service and API key on the computer running the dashboard. <button class="a" type="button" onclick={() => (replacingKey = true)}>Replace the API key</button></p>
          {/if}
        </div>
      {:else}
        <dl class="set-dl set-mashy">
          {@render value("Mashy", assistant.data?.enabled ? "on" : "off", "Answers questions about your history.")}
          {#if file?.assistant}{@render value("Mashy's model", file.assistant.model, "Used only when you ask Mashy something.")}{:else if assistant.data?.model}{@render value("Mashy's model", assistant.data.model, "Used only when you ask Mashy something.")}{/if}
        </dl>
        {#if !file?.assistant && !assistant.data?.enabled}
          <div class="set-foot">
            {#if file?.editable}
              <p class="meta">Mashy answers questions about your history. Choose a model service and paste its API key to turn it on.</p>
              <AssistantKeyForm settings={file} withService={true} id="settings-mashy" />
            {:else}
              <p class="meta">Mashy is optional. To use it, set it up from the dashboard on the computer that runs it.</p>
            {/if}
          </div>
        {/if}
      {/if}
    </div>
  </section>

  <section class="sec" id="set-sync">
    {@render heading("Machines and sync")}
    <dl class="card set-list">
      {#if config.shared}
        {@render value("This machine", config.shared.node, "Shown as the machine name in shared results.")}
        {@render value("Shared server", config.shared.url, `Publishes to the ${config.shared.group} group.`)}
      {:else}
        {@render value("Shared server", "not connected", "History stays on each machine.")}
      {/if}
      {@render value("Watching for new chats", status.data.watching ? "running" : "not running", "Indexes chats as your agents write them.")}
    </dl>
    <SharedServer joinable />
  </section>

  <section class="sec" id="set-storage">
    {@render heading("Storage")}
    <dl class="card set-list">
      {@render value("Database", status.data.databasePath ?? "not created yet")}
      {@render value("Chats", status.data.chats.toLocaleString())}
      {@render value("Messages stored", status.data.messages.toLocaleString())}
      {@render value("Messages eligible for search", status.data.searchableMessages.toLocaleString(), "Dialogue only; tool calls stay out of search.")}
      {@render value("Search store", config.searchStore)}
    </dl>
    <p class="meta">To change the search engine or storage settings, edit the configuration file. <a class="a" href="https://getmash.io/configuration/" target="_blank" rel="noreferrer">Configuration guide</a></p>
  </section>
{/if}

<section class="sec" id="set-appearance">
  {@render heading("Appearance", "This browser only")}
  <div class="card set-list">
    <div class="set-row">
      {@render title("Theme", "Kept in this browser only; other browsers and the desktop app keep their own choice.")}
      <span class="set-c">
        <ToggleGroup.Root type="single" aria-label="Theme" bind:value={() => appearance.current, (choice) => { const next = appearances.find((known) => known === choice); if (next) appearance.set(next); }}>
          {#each appearances as choice (choice)}
            <ToggleGroup.Item value={choice}>{choice === "system" ? "match the system" : choice}</ToggleGroup.Item>
          {/each}
        </ToggleGroup.Root>
      </span>
    </div>
  </div>
</section>

{#if file?.editable && (changed.length || saved || failure)}
  <SettingsSaveBar {changed} {blocked} {saving} {failure} {saved} {discard} {save} />
{/if}
</div>
</div>

<style>
  /* An agent history's state: the status dot the agent cards use, with its word. */
  .set-st {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-weight: var(--weight-label);
  }
  .set-st.set-on {
    color: var(--status-ok-text);
  }
</style>
