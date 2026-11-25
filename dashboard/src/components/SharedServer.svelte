<script lang="ts">
import KeyRound from "@lucide/svelte/icons/key-round";
import Server from "@lucide/svelte/icons/server";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { serverViewOptions } from "../lib/serverStatus";
import { settingsOptions } from "../lib/settings";
import { machineLine, type ServerView, setAdminKey } from "../lib/shared";
import JoinShared from "./JoinShared.svelte";
import MachineDot from "./MachineDot.svelte";
import ServerAdmin from "./ServerAdmin.svelte";
import ShareDone from "./ShareDone.svelte";
import SharedHelp from "./SharedHelp.svelte";
import ShareReview from "./ShareReview.svelte";

// The shared server this machine publishes to, as the server reports this
// machine's group, read with this machine's own token. With an admin key
// saved here, its machines can be managed too.
// `notes` adds the cards of terminal commands and what each step keeps beside
// the main card; first-run setup, already a long page, leaves them out.
let { joinable = false, notes = true }: { joinable?: boolean; notes?: boolean } = $props();

const client = useQueryClient();
const view = createQuery(() => serverViewOptions);
const now = $derived(view.data?.status?.now ?? Math.floor(Date.now() / 1000));
// A joined machine whose index is still local has not chosen what to share.
const settings = createQuery(() => settingsOptions);
let reviewing = $state(false);
const held = $derived(settings.data?.shared === true && settings.data.destination === "local");

let adding = $state(false);
let key = $state("");
let keyFailure = $state("");
let savingKey = $state(false);

async function saveKey(): Promise<void> {
  keyFailure = "";
  savingKey = true;
  try {
    await setAdminKey(key);
    key = "";
    adding = false;
    await client.invalidateQueries({ queryKey: ["shared-server"] });
  } catch (error) {
    keyFailure = error instanceof Error ? error.message : String(error);
  } finally {
    savingKey = false;
  }
}

async function forgetKey(): Promise<void> {
  await setAdminKey(null);
  await client.invalidateQueries({ queryKey: ["shared-server"] });
}
</script>

{#snippet serverCard(data: ServerView)}
  {@const admin = data.admin_key && data.may_change}
  <section class="card ss" aria-label="Shared server">
    <h2 class="ss-h"><Server size={16} aria-hidden="true" />Shared server</h2>
    <dl class="kvs ss-kv">
      <dt>Address</dt><dd><code class="i">{data.server}</code></dd>
      <dt>Group</dt><dd>{data.group_id}</dd>
      <dt>This machine</dt><dd>{data.node_id}</dd>
      {#if data.status}<dt>Server build</dt><dd><code class="i">{data.status.build.slice(0, 8)}</code></dd>{/if}
    </dl>
    {#if data.problem}
      <p class="warn" role="alert">{data.problem.message}</p>
    {/if}
    {#if data.status && !admin}
      {#if data.status.restart_needed}<p class="warn" role="status">The server needs a restart before it accepts machines of this group.</p>{/if}
      <ul class="ss-list" aria-label="Machines in {data.group_id}">
        {#each data.status.machines as machine (machine.node_id)}
          {@const line = machineLine(machine, now)}
          <li class:own={machine.node_id === data.node_id}>
            <span class="ss-name"><MachineDot machine={machine.node_id} /><strong>{machine.node_id}</strong>{#if machine.node_id === data.node_id}<span class="pill">this machine</span>{/if}</span>
            <span class="meta" title={line.title}>{line.text}</span>
          </li>
        {/each}
      </ul>
    {/if}
    {#if admin}
      <p class="meta">Machines are managed with the admin key saved on this machine, in <code class="i">~/.mash/admin.env</code>.</p>
      <button class="a" type="button" onclick={forgetKey}>Forget the admin key on this machine</button>
    {:else if data.may_change}
      {#if adding}
        <form class="ss-key" aria-label="Add an admin key" onsubmit={(event) => { event.preventDefault(); void saveKey(); }}>
          <label class="set-l" for="admin-key">Admin key</label>
          <input id="admin-key" class="field" type="password" autocomplete="off" spellcheck="false" bind:value={key} />
          <p class="meta">An admin key starts with <code class="i">mash-admin.</code>. Make one on the server machine with <code class="i">mash-server --server-config … --add-admin NAME</code>. Mash checks it with the server before saving it, readable only by you.</p>
          <div class="ss-acts">
            <button class="btn p" type="submit" disabled={!key.trim() || savingKey}><KeyRound size={14} aria-hidden="true" />{savingKey ? "Checking…" : "Save admin key"}</button>
            <button class="btn" type="button" onclick={() => { adding = false; key = ""; keyFailure = ""; }}>Cancel</button>
          </div>
          {#if keyFailure}<p class="warn" role="alert">{keyFailure}</p>{/if}
        </form>
      {:else}
        <button class="btn" type="button" onclick={() => (adding = true)}><KeyRound size={15} aria-hidden="true" />Manage this server</button>
      {/if}
    {:else}
      <p class="meta">Machines are managed from Mash on the machine itself, or from a dashboard that every visitor signs in to.</p>
    {/if}
  </section>
{/snippet}

{#if view.data}
  {@const data = view.data}
  {#if data.server}
    {@const admin = data.admin_key && data.may_change}
    {@const sharing = data.may_change && !!settings.data}
    {@const review = reviewing || held}
    <div class="ss-wrap">
      <div class="ss-grid" class:solo={!sharing && !admin}>
        {#if sharing || admin}
          <div class="ss-main">
            {#if sharing}
              {#if review}
                <ShareReview server={data} done={() => (reviewing = false)} />
              {:else}
                <ShareDone server={data} change={() => (reviewing = true)} />
              {/if}
            {/if}
            {#if admin}<ServerAdmin />{/if}
          </div>
        {/if}
        <div class="ss-side">
          {@render serverCard(data)}
          {#if !notes}
            <!-- First-run setup shows the server card alone. -->
          {:else if admin}
            <SharedHelp mode="admin" node={data.node_id ?? ""} />
          {:else if sharing}
            <SharedHelp mode={review ? "review" : "done"} node={data.node_id ?? ""} />
          {/if}
        </div>
      </div>
    </div>
  {:else if joinable && data.may_change}
    <div class="ss-wrap">
      <div class="ss-grid">
        <div class="ss-main">
          <JoinShared joined={() => void client.invalidateQueries({ queryKey: ["shared-server"] })} />
        </div>
        {#if notes}<div class="ss-side"><SharedHelp mode="join" /></div>{/if}
      </div>
    </div>
  {/if}
{:else if view.error}
  <p class="meta" role="status">The shared server’s state could not be read: {view.error.message}</p>
{/if}

<style>
  /* The main card and its notes side by side once the space they are given
     is wide enough, whichever page holds them; one column otherwise. */
  .ss-wrap {
    container-type: inline-size;
    margin-top: 18px;
  }
  .ss-grid {
    display: grid;
    gap: 18px;
  }
  @container (min-width: 860px) {
    .ss-grid {
      grid-template-columns: minmax(0, 1fr) minmax(300px, 370px);
      align-items: start;
    }
    .ss-grid.solo {
      grid-template-columns: minmax(0, 560px);
    }
  }
  .ss-main,
  .ss-side {
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-width: 0;
  }
  .ss-main > :global(*),
  .ss-side > :global(*) {
    margin: 0 !important;
  }
  /* The card's parts sit apart evenly; its buttons keep their own width. */
  .ss {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .ss > :global(*) {
    margin: 0;
  }
  .ss > :global(button) {
    align-self: flex-start;
  }
  .ss-h {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 15px;
    font-weight: 700;
    color: var(--text-strong);
  }
  .ss-kv {
    grid-template-columns: minmax(88px, max-content) minmax(0, 1fr);
    gap: 9px 14px;
  }
  .ss-kv dd {
    justify-self: start;
  }
  /* The group's machines: a quiet list in an outlined box, this machine first
     in butter. */
  .ss-list {
    display: grid;
    margin: 0;
    padding: 4px;
    border: var(--border-outline);
    border-radius: var(--radius-6);
    list-style: none;
  }
  .ss-list li {
    display: grid;
    gap: 2px;
    padding: 9px 10px;
    border-radius: var(--radius-4);
  }
  .ss-list li + li {
    border-top: var(--border-divider);
    border-radius: 0;
  }
  .ss-list li.own {
    background: var(--selected-soft-bg);
  }
  .ss-name {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .ss-key {
    display: grid;
    gap: 8px;
    padding-top: 14px;
    border-top: var(--border-divider);
  }
  .ss-key > :global(*) {
    margin: 0;
  }
  .ss-acts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
