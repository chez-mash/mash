<script lang="ts">
import RefreshCw from "@lucide/svelte/icons/refresh-cw";
import Send from "@lucide/svelte/icons/send";
import { useQueryClient } from "@tanstack/svelte-query";
import { servicesHref } from "../lib/services.svelte";
import type { ServerView } from "../lib/shared";
import { publishProgress } from "../lib/sharing";

// How far the shared server has this machine's chats, by the server's own
// report. Mash publishes as it indexes; when nothing arrives, the likely cause
// is that automatic indexing is not running here, so that is what it says.
let { server, change }: { server: ServerView; change: () => void } = $props();

const client = useQueryClient();
const machine = $derived(server.status?.machines.find((row) => row.node_id === server.node_id));
const progress = $derived(publishProgress(machine));
let checking = $state(false);

async function check(): Promise<void> {
  checking = true;
  try {
    await client.invalidateQueries({ queryKey: ["shared-server"] });
  } finally {
    checking = false;
  }
}
</script>

<section class="card share" aria-labelledby="share-done-title" style:margin-top="18px">
  <h2 class="h2" id="share-done-title"><Send size={15} aria-hidden="true" />Publishing to {server.group_id}</h2>
  {#if server.problem}
    <p class="warn" role="alert">{server.problem.message}</p>
    <p class="meta">Nothing is lost: {server.node_id} keeps everything and publishes again when the server answers.</p>
  {:else if progress.state === "receiving"}
    <p role="status"><strong>{progress.sent.toLocaleString()}</strong> of {progress.total.toLocaleString()} chats received by the server.</p>
    <span class="bar" role="progressbar" aria-label="Chats received" aria-valuemin={0} aria-valuemax={progress.total} aria-valuenow={progress.sent}><i style:width="{Math.round((progress.sent / Math.max(1, progress.total)) * 100)}%"></i></span>
  {:else if progress.state === "published"}
    <p role="status"><strong>{progress.chats.toLocaleString()}</strong> chats from {server.node_id} are on the server{progress.at ? ` · last published ${new Date(progress.at).toLocaleString()}` : ""}.</p>
    <p class="meta">New chats are published when Mash indexes them.</p>
  {:else}
    <p role="status">The server has nothing from {server.node_id} yet.</p>
    <p class="meta">Mash publishes when it indexes. If automatic indexing is not running on this machine, start it on the <a class="a" href={servicesHref()}>Services</a> page, or run <code class="i">mash index</code> in a terminal.</p>
  {/if}
  <div class="share-actions">
    <button class="btn" type="button" onclick={change}>Change what’s shared</button>
    <button class="btn" type="button" disabled={checking} onclick={check}><RefreshCw size={14} aria-hidden="true" />{checking ? "Checking…" : "Check again"}</button>
  </div>
</section>

<style>
  .share > :global(* + *) {
    margin-top: 14px;
  }
  .share-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 12px;
  }
</style>
