<script lang="ts">
import ArrowRight from "@lucide/svelte/icons/arrow-right";
import Check from "@lucide/svelte/icons/check";
import Server from "@lucide/svelte/icons/server";
import ShieldCheck from "@lucide/svelte/icons/shield-check";
import { type JoinView, joinShared } from "../lib/shared";

// Join a shared index from a join string made on the machine that runs the
// server. Checking saves nothing; Save and continue writes the settings and
// the token and starts nothing else.
let { joined }: { joined: () => void } = $props();

let text = $state("");
let checking = $state(false);
let saving = $state(false);
let preview = $state<JoinView | null>(null);
let failure = $state("");

const refusal: Record<string, string> = {
  expired:
    "This join string has expired: each one works for 24 hours. Ask for a new one on the machine that runs the server.",
  used: "This join string was already used: each one works once, on one computer. Ask for a new one.",
  revoked:
    "The server’s operator replaced this join string or removed its machine. Ask for a new one.",
  unknown:
    "The server did not make this join string, or the machine it was for was deleted. Ask for a new one.",
};

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

<form class="card" aria-label="Join a shared index" onsubmit={(event) => { event.preventDefault(); void check(); }}>
  <h2 class="h2"><Server size={15} aria-hidden="true" />Join a shared index</h2>
  <p class="meta">Paste the join string made for this computer on the machine that runs your shared server. Mash checks it and shows where it connects. Nothing is saved until you choose Save and continue.</p>
  <label class="meta" style:display="block" for="join-string">Join string</label>
  <textarea id="join-string" class="field" style:width="100%" style:word-break="break-all" rows="3" spellcheck="false" autocomplete="off" bind:value={text} oninput={() => { preview = null; failure = ""; }}></textarea>
  <p class="meta">It starts with <code class="i">mash-join.</code> and is one long line.</p>
  <button class="btn" type="submit" disabled={!text.trim() || checking}><ShieldCheck size={14} aria-hidden="true" />{checking ? "Checking…" : "Check"}</button>
  {#if failure}<p class="warn" role="alert">{failure} Nothing was saved.</p>{/if}
  {#if preview}
    {#if preview.check.status === "ready"}
      <div class="card tight cream" role="status">
        <p><Check size={14} aria-hidden="true" /> The server accepted this join string.</p>
        <dl class="kvs">
          <dt>This computer joins as</dt><dd><strong>{preview.node_id}</strong></dd>
          <dt>Shared server</dt><dd><code class="i">{preview.server}</code></dd>
          <dt>Group</dt><dd>{preview.group_id}</dd>
          <dt>Already in {preview.group_id}</dt><dd>{preview.check.machines?.length ? preview.check.machines.join(", ") : "no other machines yet"}</dd>
        </dl>
        <p class="meta">This string was made for {preview.node_id}. Each computer gets its own; if this computer is not {preview.node_id}, ask for a string made for it.</p>
        <p class="meta">Save and continue saves the server, group and name in Mash’s configuration and the credential in <code class="i">~/.mash/node.env</code>, readable only by you. It does not index, publish or change your agents.</p>
        <button class="btn p" type="button" disabled={saving} onclick={save}><ArrowRight size={14} aria-hidden="true" />{saving ? "Saving…" : "Save and continue"}</button>
      </div>
    {:else}
      <p class="warn" role="alert">{refusal[preview.check.status] ?? "The server refused this join string."} Nothing was saved.</p>
    {/if}
  {/if}
</form>
