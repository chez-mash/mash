<script lang="ts">
import ArrowRightIcon from "phosphor-svelte/lib/ArrowRightIcon";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import ShieldCheckIcon from "phosphor-svelte/lib/ShieldCheckIcon";
import Icon from "$lib/components/Icon.svelte";
import { type JoinView, joinRefusals, joinShared } from "../lib/shared";

// Join a shared index from a join string made on the machine that runs the
// server. Checking saves nothing; Save and continue writes the settings and
// the token and starts nothing else.
let { joined }: { joined: () => void } = $props();

let text = $state("");
let checking = $state(false);
let saving = $state(false);
let preview = $state<JoinView | null>(null);
let failure = $state("");

async function check(): Promise<void> {
  failure = "";
  preview = null;
  checking = true;
  try {
    preview = await joinShared(text, false);
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  } finally {
    checking = false;
  }
}

async function save(): Promise<void> {
  failure = "";
  saving = true;
  try {
    const answer = await joinShared(text, true);
    if (answer.joined) {
      text = "";
      joined();
    } else {
      preview = answer;
    }
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  } finally {
    saving = false;
  }
}
</script>

<form class="card join" aria-label="Join a shared index" onsubmit={(event) => { event.preventDefault(); void check(); }}>
  <h2 class="join-h"><Icon icon={HardDrivesIcon} size={16} />Join a shared index</h2>
  <p class="meta">Paste the join string made for this computer on the machine that runs your shared server. Mash checks it and shows where it connects. Nothing is saved until you choose Save and continue.</p>
  <label class="join-l" for="join-string">Join string</label>
  <textarea id="join-string" class="field join-t" rows="3" spellcheck="false" autocomplete="off" bind:value={text} oninput={() => { preview = null; failure = ""; }}></textarea>
  <p class="meta join-hint">It starts with <code class="i">mash-join.</code> and is one long line.</p>
  <button class="btn join-check" type="submit" disabled={!text.trim() || checking}><Icon icon={ShieldCheckIcon} size={14} />{checking ? "Checking…" : "Check"}</button>
  {#if failure}<p class="warn" role="alert">{failure} Nothing was saved.</p>{/if}
  {#if preview}
    {#if preview.check.status === "ready"}
      <div class="card cream join-ok" role="status">
        <p class="join-yes"><Icon icon={CheckIcon} size={15} />The server accepted this join string.</p>
        <dl class="kvs">
          <dt>This computer joins as</dt><dd><strong>{preview.node_id}</strong></dd>
          <dt>Shared server</dt><dd><code class="i">{preview.server}</code></dd>
          <dt>Group</dt><dd>{preview.group_id}</dd>
          <dt>Already in {preview.group_id}</dt><dd>{preview.check.machines?.length ? preview.check.machines.join(", ") : "no other machines yet"}</dd>
        </dl>
        <p class="meta">This string was made for {preview.node_id}. Each computer gets its own; if this computer is not {preview.node_id}, ask for a string made for it.</p>
        <p class="meta">Save and continue saves the server, group and name in Mash’s configuration and the credential in <code class="i">~/.mash/node.env</code>, readable only by you. It does not index, publish or change your agents.</p>
        <button class="btn p" type="button" disabled={saving} onclick={save}><Icon icon={ArrowRightIcon} size={14} state="strong" />{saving ? "Saving…" : "Save and continue"}</button>
      </div>
    {:else}
      <p class="warn" role="alert">{joinRefusals[preview.check.status] ?? "The server refused this join string."} Nothing was saved.</p>
    {/if}
  {/if}
</form>

<style>
  .join {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 22px 22px 22px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .join > :global(*),
  .join-ok > :global(*) {
    margin: 0;
  }
  .join-h {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 15px;
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .join-l {
    margin-top: 4px !important;
    font-weight: var(--weight-label);
    color: var(--text-strong);
  }
  .join-t {
    width: 100%;
    min-height: 76px;
    font-family: var(--type-mono);
    font-size: 12.5px;
    line-height: 1.55;
    word-break: break-all;
    resize: vertical;
  }
  .join-hint {
    margin-top: -4px !important;
    font-size: 12.5px;
  }
  .join-check {
    align-self: flex-start;
  }
  .join-ok {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px;
  }
  .join-ok > :global(.btn) {
    align-self: flex-start;
  }
  .join-yes {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: var(--weight-strong);
    color: var(--text-strong);
  }
  .join-yes :global(svg) {
    color: var(--status-ok);
  }
  .join-ok :global(.kvs) {
    grid-template-columns: 150px minmax(0, 1fr);
    gap: 9px 14px;
  }
</style>
