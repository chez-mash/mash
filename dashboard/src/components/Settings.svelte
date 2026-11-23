<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import Check from "@lucide/svelte/icons/check";
import Copy from "@lucide/svelte/icons/copy";
import Database from "@lucide/svelte/icons/database";
import Eye from "@lucide/svelte/icons/eye";
import Lock from "@lucide/svelte/icons/lock";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import Save from "@lucide/svelte/icons/save";
import Search from "@lucide/svelte/icons/search";
import Server from "@lucide/svelte/icons/server";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
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
import Skeleton from "./Skeleton.svelte";
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
let reviewing = $state(false);
let saving = $state(false);
let failure = $state("");
let saved = $state(0);
let days = $state(90);

function discard(): void {
  if (file) draft = draftOf(file);
  reviewing = false;
  failure = "";
}
// Mashy is set up here by choosing a service and pasting its API key. The
// key is saved on the machine running the dashboard and never shown again.
let replacingKey = $state(false);
// Mashy links here when it is not set up: the assistant section is brought
// into view once everything above it has loaded, so it does not move after.
let assistantSection = $state<HTMLElement>();
$effect(() => {
  const route = router.current;
  if (route.page !== "settings" || route.section !== "assistant") return;
  if (status.isPending || settings.isPending) return;
  assistantSection?.scrollIntoView({ block: "start" });
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
    reviewing = false;
  } catch (error) {
    failure = error instanceof Error ? error.message : "The settings were not saved.";
  } finally {
    saving = false;
  }
}
</script>

{#snippet row(name: string, value: string)}
  <dt>{name}</dt><dd><Wrapped {value} /></dd>
{/snippet}
{#snippet copyPath()}
  {#if path}<button class="btn sm" type="button" onclick={copyConfigPath}>{#if copied}<Check size={13} aria-hidden="true" />Copied{:else}<Copy size={13} aria-hidden="true" />Copy path{/if}</button>{/if}
{/snippet}
{#snippet toggle(label: string, on: boolean, flip: () => void, description?: string)}
  <button class="sw" type="button" role="switch" aria-checked={on} aria-label={label} aria-describedby={description} onclick={() => { saved = 0; flip(); }}><i></i></button>
{/snippet}
{#snippet changedPill(key: string)}
  {#if touched.has(key)}<span class="pill warn">changed</span>{/if}
{/snippet}

<div class="hd" style:align-items="flex-start">
  <div>
    <h1 class="h1">Settings</h1>
    <p class="meta set-path" style:margin-top="6px"><span class="mono"><Wrapped value={path || "Mash configuration"} /></span> {@render copyPath()} <a class="a" href="https://getmash.io/dashboard-settings/" target="_blank" rel="noreferrer">Settings guide</a></p>
  </div>
  {#if status.data}
    <span class="pill" class:ok={status.data.ready} class:warn={!status.data.ready}>
      {#if status.data.ready}<Check size={13} aria-hidden="true" />Local search ready{:else}<TriangleAlert size={13} aria-hidden="true" />Local search needs attention{/if}
    </span>
  {/if}
</div>
{#if file && !file.editable}
  <p class="card tight">Shown from this machine's configuration file. This dashboard can be reached from other machines and was not started with a services configuration, so it does not change settings. Open the dashboard on the machine itself, or edit the file named above. <a class="a" href="https://getmash.io/dashboard-settings/#if-settings-are-read-only" target="_blank" rel="noreferrer">Why settings are read-only</a></p>
{:else if file}
  <p>These are this machine's settings. Choose <strong>Save to file</strong> to write your changes.</p>
{/if}
{#if status.isPending || settings.isPending}<Skeleton rows={5} />{/if}
<QueryError title="Couldn’t load settings" query={status} />
{#if settings.error}<p class="meta warn" role="alert">Settings can't be changed right now: {settings.error.message}</p>{/if}
{#if status.data && config}
  {#if !config.loaded}
    <p class="card tight warn">Mash has no configuration file yet. <a class="a" href="#/setup">Set up Mash here</a>{#if status.data.searchService === "reachable" && !desktop} or run <code class="i">mash init</code> in a terminal{/if}.</p>
  {/if}
  {#if status.data.searchService === "unreachable"}
    <p class="card tight warn">Mash could not reach the local search service at <code class="i">{config.searchStore}</code>. <a class="a" href="https://getmash.io/troubleshooting/#qdrant-connection-failed" target="_blank" rel="noreferrer">Start or repair the search service</a>.</p>
  {:else if status.data.searchService === "rejected"}
    <p class="card tight warn">The search service at <code class="i">{config.searchStore}</code> responded, but Mash could not use it. <a class="a" href="https://getmash.io/troubleshooting/#qdrant-connection-failed" target="_blank" rel="noreferrer">Check the search service</a>.</p>
  {:else if status.data.nextStep === "index"}
    <p class="card tight warn">Run <code class="i">{mashCommand("index")}</code> in a terminal on this machine to update local search.</p>
  {:else if status.data.nextStep === "models install"}
    <p class="card tight warn">Search by meaning needs local models. Run <code class="i">{mashCommand("models install")}</code> on this machine. <a class="a" href="https://getmash.io/indexing/#enable-semantic-search" target="_blank" rel="noreferrer">Finish setting up search by meaning</a>.</p>
  {/if}

  <section class="sec">
    <div class="hd">
      <h2 class="h2"><Bot size={15} aria-hidden="true" />agent histories</h2>
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
              <span class="set-t">{@render changedPill(`agent:${entry.agent}`)}{#if file.editable}{@render toggle(`Index ${agentInfo[entry.agent].name}`, on, () => { if (draft) draft.agents[entry.agent] = !on; }, "history-sources-help")}{:else}<span class="meta">{on ? "indexed" : "not indexed"}</span>{/if}</span>
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

  <div class="g2 set-g">
    <section class="sec">
      <h2 class="h2"><Eye size={15} aria-hidden="true" />indexing</h2>
      {#if file && draft && file.editable}
        <div class="card set-card">
          <div class="set-r" class:changed={touched.has("range")}>
            <span class="set-l">history range</span>
            <span class="set-c">
              <span class="seg" role="group" aria-label="History range" aria-describedby="history-range-help">
                <button type="button" aria-pressed={draft.sinceDays === null} onclick={() => { saved = 0; if (draft) draft.sinceDays = null; }}>all history</button>
                <button type="button" aria-pressed={draft.sinceDays !== null} onclick={() => { saved = 0; if (draft && draft.sinceDays === null) draft.sinceDays = file.sinceDays ?? days; }}>recent updates</button>
              </span>
              {#if draft.sinceDays !== null}
                <span class="meta">the last <input class="field" type="number" min="1" max="36500" aria-label="Days of history" aria-describedby="history-range-help" style:width="72px" style:margin="0 4px" value={draft.sinceDays} oninput={(event) => { saved = 0; days = event.currentTarget.valueAsNumber; if (draft) draft.sinceDays = Number.isNaN(days) ? 0 : days; }} /> days</span>
              {/if}
              {@render changedPill("range")}
              <span id="history-range-help" class="meta" style:flex-basis="100%">
                The range applies to future indexing. Chats already indexed stay searchable.
                <a class="a" href="https://getmash.io/privacy/#remove-a-chat" target="_blank" rel="noreferrer">How to remove an indexed chat</a>
              </span>
            </span>
          </div>
          <div class="set-r" class:changed={touched.has("destination")}>
            <span class="set-l">index into</span>
            <span class="set-c">
              {#if file.shared}
                <span class="seg" role="group" aria-label="Index into">
                  <button type="button" aria-pressed={draft.destination === "both"} onclick={() => { saved = 0; if (draft) draft.destination = "both"; }}>this machine and the shared server</button>
                  <button type="button" aria-pressed={draft.destination === "local"} onclick={() => { saved = 0; if (draft) draft.destination = "local"; }}>this machine only</button>
                </span>
                {@render changedPill("destination")}
                {#if draft.destination === "local" && file.destination === "both"}<span class="set-h">History already on the shared server stays there.</span>{/if}
              {:else}
                <span class="meta">this machine only · no shared server is configured</span>
              {/if}
            </span>
          </div>
          <div class="set-r inline" class:changed={touched.has("memory")}>
            <span class="set-l">memory capture<small>agents save decisions and preferences as you state them</small></span>
            <span class="set-c">{@render changedPill("memory")}{@render toggle("Memory capture", draft.memoryCapture, () => { if (draft) draft.memoryCapture = !draft.memoryCapture; })}</span>
          </div>
        </div>
      {:else}
        <dl class="card kvs wide">
          {@render row("history range", config.sinceDays === null ? "all history" : `the last ${config.sinceDays} days`)}
          {@render row("index into", config.destination === "both" ? destinations.both : destinations.local)}
          {@render row("memory capture", config.recordsEnabled ? "on" : "off")}
        </dl>
      {/if}
    </section>

    <section class="sec">
      <h2 class="h2"><Lock size={15} aria-hidden="true" />privacy</h2>
      <div class="card set-card">
        {#if file && draft && file.editable}
          <div class="set-r inline" class:changed={touched.has("results")}>
            <span class="set-l">hide credentials in results</span>
            <span class="set-c">{@render changedPill("results")}{@render toggle("Hide credentials in results", draft.hideInResults, () => { if (draft) draft.hideInResults = !draft.hideInResults; })}</span>
          </div>
          <div class="set-r inline" class:changed={touched.has("publishing")}>
            <span class="set-l">hide credentials when publishing</span>
            <span class="set-c">
              {@render changedPill("publishing")}
              {@render toggle("Hide credentials when publishing", draft.hideWhenPublishing, () => { if (draft) draft.hideWhenPublishing = !draft.hideWhenPublishing; })}
              {#if draft.hideWhenPublishing !== file.hideWhenPublishing}
                <span class="set-h">
                  The next publication replaces this machine's shared copy of its history.
                  {#if !draft.hideWhenPublishing}That copy can include credentials found in chats.{/if}
                </span>
              {/if}
            </span>
          </div>
        {:else}
          <dl class="kvs flip">
            {@render row("hide credentials in results", config.redactOutput ? "on" : "off")}
            {@render row("hide credentials when publishing", config.redactPublication ? "on" : "off")}
          </dl>
        {/if}
        <p class="meta">Credentials are never shown here. The configuration names the environment variables that hold them.</p>
      </div>
    </section>
  </div>

  <section class="sec" id="assistant" bind:this={assistantSection}>
    <div class="hd">
      <h2 class="h2"><MessagesSquare size={15} aria-hidden="true" />assistant</h2>
      <a class="a" href="https://getmash.io/configuration/#the-dashboard-assistant" target="_blank" rel="noreferrer" style:font-size="12.5px">Set up Mashy</a>
    </div>
    <div class="card mashy-card set-card">
      {#if file?.assistant && draft?.assistant && file.editable}
        <div class="set-r inline" class:changed={touched.has("mashy")}>
          <span class="set-l mashy-l"><img src={asset("mashy.png")} alt="" /><span>Mashy<small>answers questions from your history, and sends the model the chat text it reads</small></span></span>
          <span class="set-c">{@render changedPill("mashy")}{@render toggle("Mashy", draft.assistant.enabled, () => { if (draft?.assistant) draft.assistant.enabled = !draft.assistant.enabled; })}</span>
        </div>
        <div class="set-r" class:changed={touched.has("model")}>
          <span class="set-l">model<small>{models.data?.length ? `${models.data.length.toLocaleString()} offered by the configured service` : models.error ? "the service did not list its models; type a name" : "as the configured service names it"}</small></span>
          <span class="set-c">
            <input class="field" list="assistant-models" aria-label="Mashy's model" autocomplete="off" autocapitalize="off" spellcheck="false" style:width="min(100%, 340px)" value={draft.assistant.model} oninput={(event) => { saved = 0; if (draft?.assistant) draft.assistant.model = event.currentTarget.value; }} />
            {@render changedPill("model")}
            <datalist id="assistant-models">{#each models.data ?? [] as model (model)}<option value={model}></option>{/each}</datalist>
          </span>
        </div>
        {#if file.assistant.enabled && assistant.data && !assistant.data.enabled}
          <p class="meta warn">Mashy has no API key on this computer yet.</p>
          {#if file}<AssistantKeyForm settings={file} withService={false} id="settings-mashy" />{/if}
        {:else if replacingKey}
          {#if file}<AssistantKeyForm settings={file} withService={false} id="settings-mashy" oncancel={() => (replacingKey = false)} onsaved={() => (replacingKey = false)} />{/if}
        {:else}
          <p class="meta">Mashy uses your model service and API key on the computer running the dashboard. <button class="a" type="button" onclick={() => (replacingKey = true)}>Replace the API key</button></p>
        {/if}
      {:else}
        <div class="mashy-l"><img src={asset("mashy.png")} alt="" /><dl class="kvs flip">
          {@render row("Mashy", assistant.data?.enabled ? "on" : "off")}
          {#if file?.assistant}{@render row("model", file.assistant.model)}{:else if assistant.data?.model}{@render row("model", assistant.data.model)}{/if}
        </dl></div>
        {#if !file?.assistant && !assistant.data?.enabled}
          {#if file?.editable}
            <p class="meta">Mashy answers questions about your history. Choose a model service and paste its API key to turn it on.</p>
            {#if file}<AssistantKeyForm settings={file} withService={true} id="settings-mashy" />{/if}
          {:else}
            <p class="meta">Mashy is optional. To use it, set it up from the dashboard on the computer that runs it.</p>
          {/if}
        {/if}
      {/if}
    </div>
  </section>

  <div class="g3 set-g">
    <section class="sec">
      <h2 class="h2"><Search size={15} aria-hidden="true" />search</h2>
      <dl class="card kvs">
        {@render row("search mode", status.data.mode)}
        {@render row("embeddings", config.embeddings)}
        {@render row("query models", modelLabels[status.data.queryModels])}
        {@render row("vector search", config.vectorSearch.toUpperCase())}
        {@render row("ranking", config.ranking)}
      </dl>
    </section>
    <section class="sec">
      <h2 class="h2"><Database size={15} aria-hidden="true" />storage</h2>
      <dl class="card kvs">
        {@render row("database", status.data.databasePath ?? "not created yet")}
        {@render row("chats", status.data.chats.toLocaleString())}
        {@render row("messages stored", status.data.messages.toLocaleString())}
        {@render row("messages eligible for search", status.data.searchableMessages.toLocaleString())}
        {@render row("search store", config.searchStore)}
      </dl>
    </section>
    <section class="sec">
      <h2 class="h2"><Server size={15} aria-hidden="true" />shared index</h2>
      {#if config.shared}
        <dl class="card kvs">
          {@render row("this machine", config.shared.node)}
          {@render row("group", config.shared.group)}
          {@render row("server", config.shared.url)}
        </dl>
      {:else}
        <p class="card tight cream meta">Not configured. This machine's history stays local.</p>
      {/if}
    </section>
  </div>
  <p class="meta">To change the search engine or storage settings, edit the configuration file. <a class="a" href="https://getmash.io/configuration/" target="_blank" rel="noreferrer">Configuration guide</a></p>
{/if}

{#if file?.editable && (changed.length || saved || failure)}
  <div class="set-bar card butter" role="region" aria-label="Unsaved settings">
    {#if changed.length}
      <div class="set-bt">
        <strong>{changed.length} unsaved change{changed.length === 1 ? "" : "s"}</strong>
        <button class="btn" type="button" disabled={saving} onclick={discard}>Discard</button>
        <button class="btn" type="button" aria-expanded={reviewing} onclick={() => (reviewing = !reviewing)}>{reviewing ? "Hide changes" : "Review changes"}</button>
        <button class="btn p" type="button" disabled={saving || !!blocked} onclick={save}><Save size={15} aria-hidden="true" />{saving ? "Saving…" : "Save to file"}</button>
      </div>
      {#if blocked}<p class="meta warn" role="alert">{blocked}</p>{/if}
      {#if reviewing}
        <ul class="set-d">
          {#each changed as change (change.key)}<li><span>{change.label}</span><span class="was">{change.from}</span><span aria-hidden="true">→</span><span class="now">{change.to}</span></li>{/each}
        </ul>
        <p class="meta">The file is rewritten; the previous one is kept beside it as <code class="i">config.toml.previous</code>. Settings the page does not show are kept.</p>
      {/if}
    {/if}
    {#if failure}<p class="fail" role="alert">{failure}</p>{/if}
    {#if saved && !changed.length}
      <p class="ok" role="status">
        Saved {saved} change{saved === 1 ? "" : "s"} to the file. New requests use the saved
        settings. To update the index now, run <code class="i">{mashCommand("index")}</code> on this machine,
        or wait for the next pass if a watcher is running.
      </p>
    {/if}
  </div>
{/if}
