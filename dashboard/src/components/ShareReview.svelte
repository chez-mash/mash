<script lang="ts">
import Send from "@lucide/svelte/icons/send";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { catalogOptions } from "../lib/queries";
import { draftOf, saveSettings, settingsOptions } from "../lib/settings";
import type { ServerView } from "../lib/shared";
import { agentShares, shareSummary, unsharedOf } from "../lib/sharing";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";

// Choose what this machine sends to its shared server before anything goes.
// Publishing turns sharing on with the agents left out here kept local; the
// watcher then sends the rest. Saving for later keeps the index local.
let { server, done }: { server: ServerView; done: () => void } = $props();

const client = useQueryClient();
const settings = createQuery(() => settingsOptions);
const catalogue = createQuery(() => catalogOptions);

let kept = $state<Agent[] | null>(null);
let since = $state<number | null | undefined>(undefined);
let hide = $state<boolean | null>(null);
let saving = $state<"publish" | "later" | null>(null);
let failure = $state("");

const file = $derived(settings.data);
const indexed = $derived(file?.agents.filter((row) => row.indexed).map((row) => row.agent) ?? []);
const unshared = $derived(kept ?? [...(file?.unshared ?? [])]);
const rows = $derived(
  agentShares(catalogue.data?.all ?? [], catalogue.data?.localNode ?? "", indexed, unshared),
);
const summary = $derived(shareSummary(rows));
const range = $derived(since === undefined ? (file?.sinceDays ?? null) : since);
const masking = $derived(hide ?? file?.hideWhenPublishing ?? true);
const machines = $derived(
  [
    ...new Set([
      ...(server.status?.machines.map((machine) => machine.node_id) ?? []),
      server.node_id ?? "",
    ]),
  ].filter(Boolean),
);
const names = (agents: readonly Agent[]) => agents.map((agent) => agentInfo[agent].name).join(", ");

function toggle(agent: Agent): void {
  failure = "";
  kept = unshared.includes(agent)
    ? unshared.filter((name) => name !== agent)
    : [...unshared, agent];
}

async function save(publish: boolean): Promise<void> {
  if (!file) return;
  failure = "";
  saving = publish ? "publish" : "later";
  try {
    const draft = draftOf(file);
    await saveSettings(file, {
      ...draft,
      destination: publish ? "both" : "local",
      sinceDays: range,
      hideWhenPublishing: masking,
      unshared: unsharedOf(rows),
    });
    await client.invalidateQueries({ queryKey: ["settings"] });
    await client.invalidateQueries({ queryKey: ["shared-server"] });
    done();
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  } finally {
    saving = null;
  }
}
</script>

<section class="card share" aria-labelledby="share-review-title" style:margin-top="18px">
  <h2 class="h2" id="share-review-title"><Send size={15} aria-hidden="true" />Choose what {server.node_id} shares</h2>
  <p class="meta">Publishing sends these chats, and the attachments Mash kept with them, to <code class="i">{server.server}</code>. Every machine in {server.group_id} can then search and read them. Nothing has been sent yet.</p>
  {#if settings.error}
    <p class="warn" role="alert">The settings could not be read: {settings.error.message}</p>
  {:else if file && !file.editable}
    <p class="meta" role="status">What this machine shares is chosen on the machine itself, or from a dashboard started with a services configuration.</p>
  {:else if file}
    <ul class="card pad0 list share-agents" aria-label="Agent histories">
      {#each rows as row (row.agent)}
        <li class="share-row" class:kept={!row.shared}>
          <AgentMark agent={row.agent} size="sm" />
          <span class="share-name"><strong>{agentInfo[row.agent].name}</strong></span>
          <span class="meta share-count">{row.chats.toLocaleString()} chat{row.chats === 1 ? "" : "s"}</span>
          <span class="meta share-fate">{row.shared ? "shared" : `stays on ${server.node_id}`}</span>
          <button class="sw" type="button" role="switch" aria-checked={row.shared} aria-label="Share {agentInfo[row.agent].name}" disabled={saving !== null} onclick={() => toggle(row.agent)}><i></i></button>
        </li>
      {/each}
    </ul>
    <p class="meta">{server.node_id} still indexes and searches the chats you leave out. They just aren’t sent to the server.</p>

    <div class="card tight">
      <span class="set-l">History range</span>
      <span class="seg" role="group" aria-label="History range">
        <button type="button" aria-pressed={range !== null} onclick={() => (since = range ?? 90)}>recent updates</button>
        <button type="button" aria-pressed={range === null} onclick={() => (since = null)}>all history</button>
      </span>
      {#if range !== null}
        <span class="meta">the last <input class="field" type="number" min="1" max="36500" aria-label="Days of history" style:width="72px" style:margin="0 4px" value={range} oninput={(event) => { const value = event.currentTarget.valueAsNumber; since = Number.isNaN(value) ? range : value; }} /> days</span>
      {/if}
      <p class="meta">The range picks which history files Mash reads. Chats already in {server.node_id}’s index count too, and are shared too.</p>
      <label class="share-mask">
        <button class="sw" type="button" role="switch" aria-checked={masking} aria-label="Hide credentials when publishing" onclick={() => (hide = !masking)}><i></i></button>
        <span><strong>Hide credentials when publishing</strong><br /><span class="meta">API keys and passwords Mash recognises are replaced in the shared copy. {server.node_id}’s own copy is unchanged.</span></span>
      </label>
    </div>

    <dl class="kvs" aria-label="Summary">
      <dt>Will be shared</dt><dd><strong>{summary.shared.toLocaleString()}</strong> {summary.shared === 1 ? "chat" : "chats"}{summary.sharedAgents.length ? ` from ${names(summary.sharedAgents)}` : ""}</dd>
      <dt>Stays on {server.node_id}</dt><dd><strong>{summary.kept.toLocaleString()}</strong> {summary.kept === 1 ? "chat" : "chats"}{summary.keptAgents.length ? ` from ${names(summary.keptAgents)}` : ""}</dd>
      <dt>Readable by</dt><dd>{machines.join(", ")}</dd>
      <dt>After this</dt><dd>new chats are published when Mash indexes them</dd>
    </dl>
    <p class="meta">You can change this later here. Turning an agent off later removes its chats from the server at the next publish.</p>

    {#if summary.sharedAgents.length === 0}
      <p class="warn" role="status">Nothing will be shared. Turn on at least one history, or choose Save and publish later.</p>
    {/if}
    {#if failure}<p class="warn" role="alert">{failure} Nothing was sent.</p>{/if}
    <div class="share-actions">
      <button class="btn" type="button" disabled={saving !== null} onclick={() => save(false)}>{saving === "later" ? "Saving…" : "Save and publish later"}</button>
      <button class="btn p" type="button" disabled={saving !== null || summary.sharedAgents.length === 0} onclick={() => save(true)}>{saving === "publish" ? "Starting…" : `Publish ${summary.shared.toLocaleString()} ${summary.shared === 1 ? "chat" : "chats"}`}</button>
    </div>
  {:else}
    <p class="meta" role="status">Loading what this machine indexes…</p>
  {/if}
</section>

<style>
  .share > :global(* + *) {
    margin-top: 14px;
  }
  .share-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto auto;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
  }
  .share-row + .share-row {
    border-top: var(--border-divider);
  }
  .share-row.kept {
    background: var(--surface-muted);
  }
  .share-fate {
    min-width: 7.5em;
    text-align: right;
  }
  .share-mask {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    margin-top: 12px;
  }
  .share-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
  }
  @media (max-width: 560px) {
    .share-row {
      grid-template-columns: auto minmax(0, 1fr) auto;
    }
    .share-count,
    .share-fate {
      grid-column: 2;
      text-align: left;
    }
    .share-row .sw {
      grid-column: 3;
      grid-row: 1 / span 3;
    }
  }
</style>
