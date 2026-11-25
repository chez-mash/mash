<script lang="ts">
// The rail's foot: which machine this dashboard runs on and whether Mash is
// keeping its index up to date, with the dashboard's one "Check for updates"
// control beside it.
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { catalogOptions } from "../lib/queries";
import { statusOptions } from "../lib/status";
import { updates } from "../lib/updates.svelte";
import CheckUpdates from "./CheckUpdates.svelte";

let { unavailable }: { unavailable: boolean } = $props();
const client = useQueryClient();
const catalogue = createQuery(() => catalogOptions);
const status = createQuery(() => statusOptions);
const machine = $derived(
  catalogue.data && !catalogue.data.sharedOnly ? catalogue.data.localNode : "",
);
// Only what the status says: a watcher running for this machine's index.
const watching = $derived(status.data?.watching);
const words = $derived.by(() => {
  if (unavailable) return "history unavailable";
  if (watching === undefined) return machine;
  const state = watching ? "watching" : "not watching";
  return machine ? `${machine} · ${state}` : state;
});
</script>

<div class="status">
  <span class="status-t"><i class="status-dot" class:on={watching === true} aria-hidden="true"></i><span>{words}</span></span>
  <CheckUpdates compact checking={updates.checking} check={() => updates.check(client)} />
</div>
