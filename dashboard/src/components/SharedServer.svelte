<script lang="ts">
import KeyRound from "@lucide/svelte/icons/key-round";
import Server from "@lucide/svelte/icons/server";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { serverViewOptions } from "../lib/serverStatus";
import { settingsOptions } from "../lib/settings";
import { machineLine, setAdminKey } from "../lib/shared";
import JoinShared from "./JoinShared.svelte";
import MachineDot from "./MachineDot.svelte";
import ServerAdmin from "./ServerAdmin.svelte";
import ShareDone from "./ShareDone.svelte";
import ShareReview from "./ShareReview.svelte";

// The shared server this machine publishes to, as the server reports this
// machine's group, read with this machine's own token. With an admin key
// saved here, its machines can be managed too.
let { joinable = false }: { joinable?: boolean } = $props();

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

{#if view.data}
  {@const data = view.data}
  {#if data.server}
    <section class="card ss" aria-label="Shared server" style:margin-top="18px">
      <h2 class="h2"><Server size={15} aria-hidden="true" />Shared server</h2>
      <dl class="kvs">
        <dt>Address</dt><dd><code class="i">{data.server}</code></dd>
        <dt>Group</dt><dd>{data.group_id}</dd>
        <dt>This machine</dt><dd>{data.node_id}</dd>
        {#if data.status}<dt>Server build</dt><dd><code class="i">{data.status.build.slice(0, 8)}</code></dd>{/if}
      </dl>
      {#if data.problem}
        <p class="warn" role="alert">{data.problem.message}</p>
      {/if}
      {#if data.status && !(data.admin_key && data.may_change)}
        {#if data.status.restart_needed}<p class="warn" role="status">The server needs a restart before it accepts machines of this group.</p>{/if}
        <ul class="card pad0 list" aria-label="Machines in {data.group_id}">
          {#each data.status.machines as machine (machine.node_id)}
            {@const line = machineLine(machine, now)}
            <li class="chatrow">
              <span class="ss-name"><MachineDot machine={machine.node_id} /><strong>{machine.node_id}</strong>{#if machine.node_id === data.node_id}<span class="pill">this machine</span>{/if}</span>
              <span class="meta" title={line.title}>{line.text}</span>
            </li>
          {/each}
        </ul>
      {/if}
      {#if data.admin_key && data.may_change}
        <ServerAdmin />
        <button class="a" type="button" onclick={forgetKey}>Forget the admin key on this machine</button>
      {:else if data.may_change}
        {#if adding}
          <form class="card tight cream" aria-label="Add an admin key" onsubmit={(event) => { event.preventDefault(); void saveKey(); }}>
            <label class="meta" style:display="block" for="admin-key">Admin key</label>
            <input id="admin-key" class="field" type="password" autocomplete="off" spellcheck="false" bind:value={key} />
            <p class="meta">An admin key starts with <code class="i">mash-admin.</code>. Make one on the server machine with <code class="i">mash-server --server-config … --add-admin NAME</code>. Mash checks it with the server before saving it, readable only by you.</p>
            <button class="btn p" type="submit" disabled={!key.trim() || savingKey}>{savingKey ? "Checking…" : "Save admin key"}</button>
            <button class="btn" type="button" onclick={() => { adding = false; key = ""; keyFailure = ""; }}>Cancel</button>
            {#if keyFailure}<p class="warn" role="alert">{keyFailure}</p>{/if}
          </form>
        {:else}
          <button class="btn" type="button" onclick={() => (adding = true)}><KeyRound size={15} aria-hidden="true" />Manage this server</button>
        {/if}
      {:else}
        <p class="meta">Machines are managed from Mash on the machine itself, or from a dashboard that every visitor signs in to.</p>
      {/if}
    </section>
    {#if data.may_change && settings.data}
      {#if reviewing || held}
        <ShareReview server={data} done={() => (reviewing = false)} />
      {:else}
        <ShareDone server={data} change={() => (reviewing = true)} />
      {/if}
    {/if}
  {:else if joinable && data.may_change}
    <JoinShared joined={() => void client.invalidateQueries({ queryKey: ["shared-server"] })} />
  {/if}
{:else if view.error}
  <p class="meta" role="status">The shared server’s state could not be read: {view.error.message}</p>
{/if}

<style>
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
  .ss-name {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
</style>
