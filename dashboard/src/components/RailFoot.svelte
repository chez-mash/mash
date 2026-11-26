<script lang="ts">
// The rail's foot: which machine this dashboard runs on and whether Mash is
// keeping its index up to date, with the dashboard's one "Check for updates"
// control beside it.
import { createQuery, useQueryClient } from "@tanstack/svelte-query";
import { catalogOptions } from "../lib/queries";
import { statusOptions } from "../lib/status";
import type { Tone } from "../lib/tone";
import { updates } from "../lib/updates.svelte";
import CheckUpdates from "./CheckUpdates.svelte";
import MachineIcon from "./MachineIcon.svelte";
import StatusBadge from "./StatusBadge.svelte";

let { unavailable }: { unavailable: boolean } = $props();
const client = useQueryClient();
const catalogue = createQuery(() => catalogOptions);
const status = createQuery(() => statusOptions);
const machine = $derived(
  catalogue.data && !catalogue.data.sharedOnly ? catalogue.data.localNode : "",
);
// Only what the status says: a watcher running for this machine's index.
const watching = $derived(status.data?.watching);
// The state reads at a glance: a green check while watching, a dashed ring
// when not (not a fault: indexing can be run by hand), amber when history
// cannot be read at all.
const state = $derived.by((): { tone: Tone; words: string } | undefined => {
  if (unavailable) return { tone: "warn", words: "history unavailable" };
  if (watching === undefined) return undefined;
  return watching ? { tone: "ok", words: "watching" } : { tone: "idle", words: "not watching" };
});
</script>

<div class="status">
  <span class="status-t">{#if machine && !unavailable}<span class="rf-m" title="This machine: {machine}"><MachineIcon {machine} size={12} />{machine}</span>{/if}{#if state}<StatusBadge tone={state.tone} words={state.words} size={12} />{/if}</span>
  <CheckUpdates compact checking={updates.checking} check={() => updates.check(client)} />
</div>

<style>
  .rf-m {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-muted);
    font-weight: var(--weight-label);
  }
</style>
