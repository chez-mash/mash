<script lang="ts">
import CircleMinus from "@lucide/svelte/icons/circle-minus";
import Copy from "@lucide/svelte/icons/copy";
import KeyRound from "@lucide/svelte/icons/key-round";
import Plus from "@lucide/svelte/icons/plus";
import Trash from "@lucide/svelte/icons/trash";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { read } from "../lib/api";
import { type Change, changeMachine, lastPublish, machineState, type NewJoin } from "../lib/shared";

// Managing the server's machines with the admin key saved on this machine.
// A join string is shown once, after it is made, and never stored.
const client = useQueryClient();
const overview = createQuery(() => ({
  queryKey: ["shared-admin"],
  queryFn: ({ signal }: { signal: AbortSignal }) =>
    read("/api/shared/admin", new URLSearchParams(), signal),
}));
const groups = $derived([
  ...new Set(overview.data?.machines.map((machine) => machine.group_id) ?? []),
]);

let name = $state("");
let busy = $state(false);
let failure = $state("");
let fresh = $state<{ node: string; join: NewJoin } | null>(null);
let copied = $state(false);
/** The machine whose deletion is being confirmed, and the name typed so far. */
let deleting = $state<{ group: string; node: string } | null>(null);
let typed = $state("");

async function act(change: Change, group: string, node: string, confirm?: string): Promise<void> {
  failure = "";
  busy = true;
  copied = false;
  try {
    const join = await changeMachine(change, group, node, confirm);
    fresh = join ? { node, join } : null;
    if (change === "add") name = "";
    deleting = null;
    typed = "";
    await client.invalidateQueries({ queryKey: ["shared-admin"] });
    await client.invalidateQueries({ queryKey: ["shared-server"] });
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  } finally {
    busy = false;
  }
}

async function copy(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
  copied = true;
}
</script>

<div class="adm" aria-label="Manage the server's machines">
  {#if overview.error}<p class="warn" role="alert">{overview.error.message}</p>{/if}
  {#if overview.data}
    {@const data = overview.data}
    {@const taken = data.machines.find((machine) => machine.group_id === groups[0] && machine.node_id === name.trim())}
    <form class="card adm-add" aria-label="Add a machine" onsubmit={(event) => { event.preventDefault(); void act("add", groups[0] ?? "", name.trim()); }}>
      <label class="adm-l" for="new-machine">Name the new machine</label>
      <div class="adm-row">
        <input id="new-machine" class="field" autocomplete="off" spellcheck="false" bind:value={name} />
        <button class="btn p" type="submit" disabled={!name.trim() || busy || groups.length !== 1 || !!taken}><Plus size={15} aria-hidden="true" />Create join string</button>
      </div>
      <p class="meta">Each machine needs its own name. It appears beside every chat in search results. Use lower-case letters, numbers and dashes.</p>
      {#if taken}
        <p class="warn" role="alert"><strong>{taken.group_id} already has a machine called {taken.node_id}.</strong> Pick another name. If this is the same {taken.node_id} after a reinstall, <button class="a" type="button" disabled={busy} onclick={() => act("replace", taken.group_id, taken.node_id)}>replace {taken.node_id}’s credential</button> instead.</p>
      {/if}
      {#if groups.length > 1}<p class="meta">This server serves several groups; add machines with <code class="i">mash-server --add-node GROUP/NODE</code>.</p>{/if}
    </form>
    {#if fresh}
      <div class="card adm-fresh" role="status" aria-label="Join string for {fresh.node}">
        <p class="adm-t">Join string for {fresh.node}</p>
        <code class="adm-join">{fresh.join.join}</code>
        <button class="btn" type="button" onclick={() => fresh && void copy(fresh.join.join)}><Copy size={15} aria-hidden="true" />{copied ? "Copied" : "Copy join string"}</button>
        <p class="meta">This string contains {fresh.node}’s credential. Send it privately: anyone who has it can publish as {fresh.node} and read every chat shared in this group. It works once, within 24 hours, and is shown only now.</p>
        <p class="meta">On {fresh.node}: open Mash, choose Join a shared index in Settings, and paste it. Or in a terminal there: <code class="i">mash init --join '…' --no-index --no-agents</code>.</p>
        {#if fresh.join.restart_needed}<p class="warn">This group is new to the server: restart mash-server so it accepts the machine.</p>{/if}
      </div>
    {/if}
    {#if failure}<p class="warn" role="alert">{failure}</p>{/if}
    <h2 class="adm-h">Machines in {groups.join(", ")}</h2>
    <ul class="card pad0 adm-list" aria-label="Machines">
      {#each data.machines as machine (`${machine.group_id}/${machine.node_id}`)}
        <li>
          <span class="adm-who"><strong>{machine.node_id}</strong>
          <span class="meta">{machine.group_id} · {machineState(machine, data.now)} · {lastPublish(machine)}{#if machine.fingerprint} · credential …{machine.fingerprint}{/if}</span></span>
          {#if machine.state !== "configured"}
            <span class="adm-acts">
            <button class="btn sm" type="button" disabled={busy} onclick={() => act("replace", machine.group_id, machine.node_id)}><KeyRound size={14} aria-hidden="true" />Replace credential</button>
            {#if machine.state === "removed"}
              <button class="btn sm" type="button" disabled={busy || machine.delete_requested} onclick={() => { deleting = { group: machine.group_id, node: machine.node_id }; typed = ""; }}><Trash size={14} aria-hidden="true" />Delete what it published</button>
            {:else}
              <button class="btn sm" type="button" disabled={busy} onclick={() => act("remove", machine.group_id, machine.node_id)}><CircleMinus size={14} aria-hidden="true" />Remove</button>
            {/if}
            </span>
          {/if}
        </li>
      {/each}
    </ul>
    {#if deleting}
      {@const target = deleting}
      <form class="card adm-del" aria-label="Delete what {target.node} published" onsubmit={(event) => { event.preventDefault(); void act("delete", target.group, target.node, typed); }}>
        <p class="adm-warn"><TriangleAlert size={15} aria-hidden="true" /><span><strong>This permanently deletes everything {target.node} published</strong>: its chats, messages, memories and attachments, from the server and from search. It can’t be undone. {target.node}’s own copy is not touched.</span></p>
        <label class="adm-l" for="confirm-delete">Type {target.node} to confirm</label>
        <input id="confirm-delete" class="field" autocomplete="off" spellcheck="false" bind:value={typed} />
        <div class="adm-acts">
          <button class="btn p" type="submit" disabled={typed !== target.node || busy}><Trash size={14} aria-hidden="true" />Delete what {target.node} published</button>
          <button class="btn" type="button" onclick={() => (deleting = null)}>Cancel</button>
        </div>
      </form>
    {/if}
    <p class="meta">Removing a machine stops its credential at once. What it published stays on the server, and searchable, until you delete it.</p>
    {#if data.actions.length}
      <h2 class="adm-h">Recent admin actions</h2>
      <ul class="card pad0 adm-log" aria-label="Recent admin actions">
        {#each data.actions.slice(0, 10) as action (`${action.at}-${action.target}-${action.action}`)}
          <li class="meta"><time>{new Date(action.at * 1000).toLocaleString()}</time> · {action.action} {action.target} · key {action.key}{action.from ? ` from ${action.from}` : ""}</li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .adm {
    display: flex;
    flex-direction: column;
    gap: 18px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .adm > :global(*),
  .adm :global(.card) > :global(*) {
    margin: 0;
  }
  .adm-add,
  .adm-fresh,
  .adm-del {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 18px 20px;
  }
  .adm-l,
  .adm-t {
    font-weight: 500;
    color: var(--text-strong);
  }
  .adm-t {
    font-weight: 600;
  }
  .adm-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .adm-row .field {
    flex: 1 1 220px;
    min-width: 0;
  }
  /* A join string just made, and the deletion being confirmed: the butter
     ring and glow of something under way (the memory roles), as the
     wireframe draws them. */
  .adm-fresh,
  .adm-del {
    border-color: var(--memory-ring);
    box-shadow: var(--memory-glow);
  }
  .adm-fresh > :global(.btn),
  .adm-del :global(.btn) {
    align-self: flex-start;
  }
  .adm-join {
    display: block;
    padding: 10px 12px;
    border-radius: var(--radius-5);
    background: var(--surface-muted);
    font-family: var(--type-mono);
    font-size: 12.5px;
    color: var(--text-strong);
    white-space: pre-wrap;
    word-break: break-all;
  }
  .adm-h {
    margin-bottom: -8px !important;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-strong);
  }
  .adm-list li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
    padding: 13px 16px;
  }
  .adm-list li + li,
  .adm-log li + li {
    border-top: var(--border-divider);
  }
  .adm-who {
    display: flex;
    flex: 1 1 300px;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .adm-who strong {
    color: var(--text-strong);
  }
  .adm-who .meta {
    font-size: 12.5px;
    overflow-wrap: anywhere;
  }
  .adm-acts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .adm-warn {
    display: flex;
    gap: 9px;
    color: var(--status-warn-text);
  }
  .adm-warn :global(svg) {
    flex: none;
    margin-top: 3px;
  }
  .adm-del .field {
    max-width: 300px;
  }
  .adm-log li {
    padding: 10px 16px;
    font-size: 12.5px;
  }
  .adm-log time {
    font-family: var(--type-mono);
  }
</style>
